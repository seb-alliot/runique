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

← [**ORM & Database**](/docs/fr/orm) | [**Flash Messages**](/docs/fr/flash) →
