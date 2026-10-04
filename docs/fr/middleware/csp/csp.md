# Content Security Policy (CSP)

Runique applique une politique CSP **par défaut, sans configuration** — le middleware de sécurité (headers + CSP) est actif sur toutes les réponses, même sans jamais appeler `.with_csp(...)`. Un nonce unique est généré par requête et injecté dans les templates Tera.

---

## Table des matières

| Section | Description |
| --- | --- |
| [Profils CSP](/docs/fr/middleware/csp-profils) | `default()`, `strict()`, `permissive()` — comparaison et cas d'usage |
| [Directives](/docs/fr/middleware/csp-directives) | Toutes les directives configurables |
| [Nonce CSP](/docs/fr/middleware/csp-nonce) | Fonctionnement du nonce, usage dans les templates |
| [Headers de sécurité](/docs/fr/middleware/csp-headers) | Tous les headers injectés automatiquement |

---

## Démarrage rapide

Sans rien configurer, la CSP par défaut (`SecurityPolicy::default()`) et tous les headers de sécurité (X-Frame-Options, X-Content-Type-Options, Referrer-Policy, Permissions-Policy, COEP/COOP/CORP, HSTS si HTTPS réel) sont déjà envoyés sur chaque réponse. `.with_csp(...)` ne les **active** pas — il remplace la politique par défaut par la vôtre :

```rust
RuniqueApp::builder(config)
    .middleware(|m| {
        m.with_csp(|c| c.policy(SecurityPolicy::strict()))
    })
    .build()
    .await?;
```

Pour personnaliser :

```rust
.middleware(|m| {
    m.with_csp(|c| {
        c.scripts(vec!["'self'", "https://cdn.example.com"])
         .images(vec!["'self'", "data:"])
    })
})
```

Dans vos templates :

```html
<script {% csp %}>
    // Ce script est autorisé par le nonce CSP
    console.log("OK");
</script>
```

---

## HTTPS forcé (`enforce_https`)

`ENFORCE_HTTPS=true` sert au déploiement **derrière un reverse proxy qui termine le TLS** (nginx, Caddy, Cloudflare…) : Runique tourne en HTTP derrière lui et ne voit pas lui-même si le client est arrivé en HTTP ou en HTTPS. Il le lit dans l'en-tête `X-Forwarded-Proto` posé par le proxy.

| `X-Forwarded-Proto` reçu | Effet |
| --- | --- |
| `http` | redirection **308** vers la même URL en `https://` (même hôte, même chemin, même query) |
| `https` | la requête passe |
| absent | la requête passe — sans l'en-tête, rien ne distingue une requête HTTP d'une requête que le proxy a reçue en HTTPS sans le dire ; la rediriger bouclerait à l'infini |

Avec plusieurs proxys en chaîne (`http, https`), seule la première valeur compte : c'est celle vue par le client.

L'hôte de l'URL de redirection est lu dans l'en-tête `Host`. La redirection s'exécute juste après la validation du Host (slot 17, après le slot 15) : avec `ALLOWED_HOSTS` renseigné, un `Host` forgé est refusé avant d'être utilisé.

**Avec ACME** (`ACME_ENABLED=true`), la redirection n'est **pas montée** : Runique sert alors le TLS lui-même, aucune requête ne porte `X-Forwarded-Proto`, et son écouteur du port 80 redirige déjà vers HTTPS (en conservant le chemin et la query). `ENFORCE_HTTPS` n'a donc aucun effet sur la redirection dans ce mode.

Dans les deux cas, `ENFORCE_HTTPS` ou ACME active aussi l'en-tête HSTS (voir [En-têtes de sécurité](/docs/fr/middleware/csp-headers)).

> **⚠️ Configuration du proxy exigée :**
> - le proxy doit **poser lui-même** `X-Forwarded-Proto` selon la connexion réelle, en écrasant toute valeur envoyée par le client ;
> - il doit transmettre l'hôte d'origine dans `Host`, sinon la redirection pointerait vers l'adresse interne de Runique (ex. `https://127.0.0.1:3000/...`) ;
> - s'il n'envoie pas `X-Forwarded-Proto`, aucune redirection n'a lieu (jamais de boucle).
>
> Un client qui forge `X-Forwarded-Proto: https` en HTTP direct n'échappe la redirection que pour sa propre connexion : aucun impact sur les autres utilisateurs.

```env
# .env
ENFORCE_HTTPS=true
ALLOWED_HOSTS=monsite.fr
```

```nginx
# nginx — en-têtes à transmettre à Runique
proxy_set_header Host $host;
proxy_set_header X-Forwarded-Proto $scheme;
```

Si le proxy redirige déjà lui-même HTTP vers HTTPS, la redirection de Runique ne se déclenche jamais (toutes les requêtes lui arrivent avec `X-Forwarded-Proto: https`) : pas de double redirection, et `ENFORCE_HTTPS=true` reste utile pour HSTS.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [CSRF](/docs/fr/middleware/csrf) | Protection CSRF |
| [Builder & configuration](/docs/fr/middleware/builder) | Configuration du builder |

## Retour au sommaire

- [Middleware & Sécurité](/docs/fr/middleware)
