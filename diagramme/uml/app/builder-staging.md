# UML — App : builder, staging, pipeline middleware

[`app/builder/mod.rs`](../../../runique/src/app/builder/mod.rs),
[`app/runique_app.rs`](../../../runique/src/app/runique_app.rs),
[`app/staging/middleware_staging/applicator.rs`](../../../runique/src/app/staging/middleware_staging/applicator.rs)

## Builder & staging (pattern Builder)

```mermaid
classDiagram
    class RuniqueAppBuilder {
        -RuniqueConfig config
        -CoreStaging core
        -MiddlewareStaging middleware
        -StaticStaging statics
        -Option~Router~ router
        -AdminStaging admin
        -Option~PasswordResetStaging~ password_reset
        +new(config) Self
        +core(f) / middleware(f) / with_admin(f)
        +with_database(db) / with_password_reset(f)
        +routes(router) / build() RuniqueApp
    }
    class RuniqueApp {
        +AEngine engine
        +Router router
        +Vec~WorkerGuard~ _log_guards
        +run() / run_http() / run_with_acme()
    }
    class CoreStaging
    class MiddlewareStaging {
        -bool dev
        +Vec~CustomMiddleware~ custom_middlewares
        +MiddlewareConfig features
        +new(dev) / from_config(config)
        +apply_to_router(router) Router
    }
    class StaticStaging
    class AdminStaging
    RuniqueAppBuilder *-- CoreStaging
    RuniqueAppBuilder *-- MiddlewareStaging
    RuniqueAppBuilder *-- StaticStaging
    RuniqueAppBuilder *-- AdminStaging
    RuniqueAppBuilder ..> RuniqueApp : build()
```

Collecte différée : chaque `.core()/.middleware()/.with_admin()` **stocke** sans exécuter ;
`build()` assemble (ordre indépendant de l'ordre d'appel du dev).

Mode : `RUNIQUE_ENV` (`dev` ou `prod`, `prod` si absente ou inconnue) donne `config.dev`,
qui choisit le profil (`MiddlewareConfig::development()` / `production()`), la validation
des hôtes par défaut, l'URL publique, le SRI. `DEBUG` ne règle plus que les pages d'erreur
détaillées.

## `build()` — étapes

[`app/builder/build.rs`](../../../runique/src/app/builder/build.rs)

```mermaid
flowchart TB
    L[0 log_init + subscriber] --> V[1 validate : staging + cross_validate + all_ready]
    V --> D[2 connexion DB]
    D --> X[3 destructuration ; dev → public_url = http://127.0.0.1:PORT]
    X --> A1{admin activé ?}
    A1 -- oui --> A2[check_replaced_views puis dispatch → block_dispatch]
    A1 -- non --> T
    A2 --> T[TemplateLoader::init_with_integrity]
    T --> A3{admin activé ?}
    A3 -- oui --> A4[missing_pages puis unknown_variables sur un Tera vierge]
    A3 -- non --> P
    A4 --> P[password_init config.password]
    P --> E[engine + URLs + middlewares]
    E --> AD[admin : prune droits orphelins, PageSheets, sans integrity en dev, routes]
    AD --> R[RuniqueApp]
```

Les deux contrôles de l'admin (page absente, variable inconnue) refusent le démarrage
**dans tous les environnements** : aucune variable d'env ne les désactive. Détail :
[../admin/admin-complements.md](../admin/admin-complements.md#contrôles-des-surcharges-au-boot).

## ACME — renouvellement à chaud

[`app/runique_app.rs`](../../../runique/src/app/runique_app.rs)

```mermaid
sequenceDiagram
    participant R as run_with_acme
    participant O as obtain_and_store
    participant LE as Let's Encrypt
    participant TLS as RustlsConfig

    R->>R: key.pem présent et cert_needs_renewal(cert.pem, domaine) faux ?
    alt absent, illisible, autre domaine ou moins d'un tiers de vie
        R->>O: obtain_and_store(domain, email, store, certs_dir)
        O->>LE: HTTP-01
        O->>O: écrit cert.pem, key.pem en 0600 par private_file
    end
    R->>TLS: from_pem
    loop toutes les 12 h
        R->>R: renew_if_due(cert.pem, domaine)
        opt à renouveler
            R->>O: obtain_and_store
            O-->>TLS: reload_from_pem (sans redémarrage)
            Note over R: échec → warn, certificat courant gardé
        end
    end
```

## Pipeline middleware par slots (ordre réel exécuté)

`apply_to_router` trie par slot. Ordre **extérieur → intérieur** (un slot bas = plus externe) :

```mermaid
flowchart TB
    R[Requête] --> S0[0 Extensions: engine/Tera/config]
    S0 --> S2[2 TrustedProxies: vraie IP client]
    S2 --> S5[5 Compression]
    S5 --> S8[8 CORS: preflight OPTIONS hors CSRF]
    S8 --> S10[10 ErrorHandler: enveloppe toute la stack]
    S10 --> S15[15 HostValidation: rejet avant coût session/auth/CSRF, encore couvert par ErrorHandler]
    S15 --> S17[17 HttpsRedirect: redirection construite sur un Host vérifié]
    S17 --> S20[20+ Custom dev]
    S20 --> S25[25 OpenRedirect]
    S25 --> S30[30 SecurityHeaders + CSP + nonce]
    S30 --> S40[40 Cache]
    S40 --> S50[50 Session]
    S50 --> S55[55 SessionUpgrade TTL]
    S55 --> S57[57 Auth: charge CurrentUser]
    S57 --> S60[60 CSRF]
    S60 --> S65[65 AntiBot: honeypot]
    S65 --> H[Handler]
```

## Anomalies / flux suspects

### ✅ AP1 — `RuniqueEngine::attach_middlewares` était du code mort (confirmait E2) — SUPPRIMÉ (2026-09-21)
Le pipeline réel est `MiddlewareStaging::apply_to_router` (slots ci-dessus). Trouvé en creusant
un bug réel du chemin vivant : slot 30 (`security_headers_middleware`, toujours actif) écrasé
par slot 31 (`csp_middleware`, ajouté dès `.with_csp()`) — le nonce CSP disparaît du header final.
Voir E3 dans `anomalies.md` pour ce bug encore ouvert.

### 🟡 AP2 — ErrorHandler (slot 10) gaté par `enable_debug_errors` (confirme E1, rétrogradé)
[`applicator.rs:348`](../../../runique/src/app/staging/middleware_staging/applicator.rs#L348)
Monté car le flag vaut `true` par défaut partout. Risque uniquement si désactivé
explicitement (nom trompeur). Pas un bug par défaut.

### 🟢 AP3 — Ordre des slots : cohérent et justifié (pas d'anomalie)
CORS (8) **hors** ErrorHandler (10) pour que le preflight OPTIONS n'atteigne jamais CSRF ;
Session (50) avant CSRF (60) car CSRF en dépend ; Auth (57) après Session. Ordre sain,
documenté slot par slot. À conserver tel quel.

**HostValidation déplacé du slot 70 au slot 15** : auparavant dernier middleware avant le
handler, il payait le coût Session/SessionUpgrade/Auth/CSRF/AntiBot sur une requête au Host
invalide. Repositionné juste après ErrorHandler (10) — toujours couvert par son filet
anti-panic, mais rejette désormais un Host invalide avant ce coût.
