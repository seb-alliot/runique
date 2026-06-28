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
        +Vec~CustomMiddleware~ custom_middlewares
        +MiddlewareConfig features
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

## Pipeline middleware par slots (ordre réel exécuté)

`apply_to_router` trie par slot. Ordre **extérieur → intérieur** (un slot bas = plus externe) :

```mermaid
flowchart TB
    R[Requête] --> S0[0 Extensions: engine/Tera/config]
    S0 --> S2[2 TrustedProxies: vraie IP client]
    S2 --> S5[5 Compression]
    S5 --> S8[8 CORS: preflight OPTIONS hors CSRF]
    S8 --> S10[10 ErrorHandler: enveloppe toute la stack]
    S10 --> S20[20+ Custom dev]
    S20 --> S25[25 OpenRedirect]
    S25 --> S30[30 SecurityHeaders]
    S30 --> S31[31 CSP]
    S31 --> S40[40 Cache]
    S40 --> S50[50 Session]
    S50 --> S55[55 SessionUpgrade TTL]
    S55 --> S57[57 Auth: charge CurrentUser]
    S57 --> S60[60 CSRF]
    S60 --> S65[65 AntiBot: honeypot]
    S65 --> S70[70 HostValidation: dernière défense]
    S70 --> H[Handler]
```

## Anomalies / flux suspects

### 🟡 AP1 — `RuniqueEngine::attach_middlewares` est du code mort (confirme E2)
Le pipeline réel est `MiddlewareStaging::apply_to_router` (slots ci-dessus).
[`engine/core.rs:110`](../../../runique/src/engine/core.rs#L110) `attach_middlewares` n'a aucun
appelant → à supprimer pour éviter la confusion (deux ordres de middleware « apparents »).

### 🟡 AP2 — ErrorHandler (slot 10) gaté par `enable_debug_errors` (confirme E1, rétrogradé)
[`applicator.rs:348`](../../../runique/src/app/staging/middleware_staging/applicator.rs#L348)
Monté car le flag vaut `true` par défaut partout. Risque uniquement si désactivé
explicitement (nom trompeur). Pas un bug par défaut.

### 🟢 AP3 — Ordre des slots : cohérent et justifié (pas d'anomalie)
CORS (8) **hors** ErrorHandler (10) pour que le preflight OPTIONS n'atteigne jamais CSRF ;
Session (50) avant CSRF (60) car CSRF en dépend ; Auth (57) après Session. Ordre sain,
documenté slot par slot. À conserver tel quel.
