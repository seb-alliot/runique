# Middleware & Sécurité

Runique intègre des middlewares de sécurité configurables appliqués automatiquement dans l'ordre optimal via le système de slots.

## Table des matières

| Module | Description |
| --- | --- |
| [Protection CSRF](/docs/fr/middleware/csrf) | Token, Double Submit Cookie, AJAX |
| [Content Security Policy](/docs/fr/middleware/csp) | Nonce, profils, headers |
| [Sessions](/docs/fr/middleware/sessions) | Store, durées, accès dans les handlers |
| [Hosts & Cache](/docs/fr/middleware/hosts-cache) | Allowed Hosts, Cache-Control, headers de sécurité |
| [Builder & configuration](/docs/fr/middleware/builder) | Builder classique, Builder Intelligent, variables d'env |
| [Rate Limiting](/docs/fr/middleware/rate-limit) | Limitation de débit par IP, par route, configurable |
| [Login Required](/docs/fr/middleware/login-required) | Protection de routes — redirige si non authentifié |
| [CORS](/docs/fr/middleware/cors) | Cross-Origin Resource Sharing — origines, credentials, preflight |
| [Proxies de confiance](/docs/fr/middleware/trusted-proxies) | IP client réelle, RFC 1918, CIDR, `ClientIp` |
| [Permissions-Policy](/docs/fr/middleware/permissions-policy) | Restrictions d'API navigateur par header HTTP |
| [Open Redirect](/docs/fr/middleware/open-redirect) | Blocage automatique des redirections vers des origines externes |
| [Anti-Bot Honeypot](/docs/fr/middleware/anti-bot) | Champ piège invisible — rejet automatique des bots |

## Stack d'exécution

```text
Requête entrante
    ↓
slot  0  Extensions          → Injection Engine, Tera, Config (toujours actif)
slot  2  TrustedProxies      → IP client réelle depuis X-Forwarded-For (toujours actif)
slot  5  Compression         → Compression des réponses (toujours actif)
slot  8  CORS                → Cross-Origin Resource Sharing (si with_cors() configuré)
slot 10  ErrorHandler        → Capture et rendu des erreurs (toujours actif)
slot 20+ Custom              → Vos middlewares personnalisés
slot 25  OpenRedirect        → Blocage redirections externes (toujours actif)
slot 30  SecurityHeaders     → X-Frame-Options, HSTS, Permissions-Policy… (toujours actif)
slot 31  CSP                 → Content Security Policy (toujours actif)
slot 40  Cache               → No-cache en développement (toujours actif)
slot 50  Session             → Gestion des sessions (toujours actif)
slot 55  SessionUpgrade      → Upgrade session anonyme → authentifiée (toujours actif)
slot 57  Auth                → Chargement CurrentUser depuis la session (toujours actif)
slot 60  CSRF                → Protection Cross-Site Request Forgery (toujours actif)
slot 65  AntiBotHoneypot     → Champ piège invisible, force_invalid si rempli (si with_anti_bot() configuré)
slot 70  HostValidation      → Validation des hosts autorisés (si with_allowed_hosts() configuré)
    ↓
Handler (votre code)
```

> **Slots "toujours actif"** : ces middlewares s'appliquent à toutes les requêtes sans configuration supplémentaire. Les autres ne s'insèrent dans la stack que si leur méthode builder correspondante est appelée.

## Prochaines étapes

← [**ORM & Database**](/docs/fr/orm) | [**Flash Messages**](/docs/fr/flash) →
