# Middleware & Sécurité

Runique intègre des middlewares de sécurité configurables appliqués automatiquement dans l'ordre optimal via le système de slots.

## Table des matières

| Module | Description |
| --- | --- |
| [Protection CSRF](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/csrf/csrf.md) | Token, Double Submit Cookie, AJAX |
| [Content Security Policy](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/csp/csp.md) | Nonce, profils, headers |
| [Sessions](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/sessions/sessions.md) | Store, durées, accès dans les handlers |
| [Hosts & Cache](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/hosts-cache/hosts-cache.md) | Allowed Hosts, Cache-Control, headers de sécurité |
| [Builder & configuration](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/builder/builder.md) | Builder classique, Builder Intelligent, variables d'env |
| [Rate Limiting](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/rate-limit/rate-limit.md) | Limitation de débit par IP, par route, configurable |

## Stack d'exécution

```text
Requête entrante
    ↓
1. Extensions (slot 0)     → Injection Engine, Tera, Config
2. ErrorHandler (slot 10)  → Capture et rendu des erreurs
3. Custom (slot 20+)       → Vos middlewares personnalisés
4. CSP (slot 30)           → Content Security Policy & headers
5. Cache (slot 40)         → No-cache en développement
6. Session (slot 50)       → Gestion des sessions
7. CSRF (slot 60)          → Protection Cross-Site Request Forgery
8. Host (slot 70)          → Validation des hosts autorisés
    ↓
Handler (votre code)
```

## Prochaines étapes

← [**ORM & Database**](https://github.com/seb-alliot/runique/blob/main/docs/fr/orm/07-orm.md) | [**Flash Messages**](https://github.com/seb-alliot/runique/blob/main/docs/fr/flash/09-flash-messages.md) →
