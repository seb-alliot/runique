# Stack Middleware

## Ordre des slots

Runique applique les middlewares dans un **ordre optimal** via le système de slots :

```text
Requête entrante
    ↓
1.  Extensions (slot 0)       → Injection Tera, Config, Engine
2.  TrustedProxies (slot 2)   → IP cliente réelle
3.  Compression (slot 5)      → Compression externe
4.  CORS (slot 8)             → Avant ErrorHandler (preflight OPTIONS)
5.  ErrorHandler (slot 10)    → Capture et rendu des erreurs
6.  HostValidation (slot 15)  → Validation Allowed Hosts
7.  Custom (slot 20+)         → Middlewares personnalisés
8.  OpenRedirect (slot 25)    → Inspection des réponses 3xx
9.  Security Headers (slot 30) → HSTS, X-Frame-Options, etc.
10. CSP (slot 31)             → Content Security Policy
11. Cache (slot 40)           → No-cache en développement
12. Session (slot 50)         → Gestion des sessions
13. SessionUpgrade (slot 55)  → Lecture/écriture en session
14. Auth (slot 57)            → Charge CurrentUser depuis la session
15. CSRF (slot 60)            → Protection CSRF
16. AntiBot (slot 65)         → Honeypot
    ↓
Handler (votre code)
    ↓
Réponse sortante (middlewares en sens inverse)
```

> **Important** : Avec Axum, le dernier `.layer()` appliqué est le premier exécuté. Le Builder Intelligent gère cet ordre automatiquement.

---

## Injection de dépendances

Via les **Extensions Axum**, injectées automatiquement par le middleware Extensions :

```rust
// Enregistré automatiquement par le builder :
// Extension(engine)  → Arc<RuniqueEngine>
// Extension(tera)    → Arc<Tera>
// Extension(config)  → Arc<RuniqueConfig>

// Accessible dans les handlers via Request :
pub async fn handler(request: Request) -> AppResult<Response> {
    let db = request.engine.db.clone();
    let config = &request.engine.config;
    // ...
}
```

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Concepts clés](/docs/fr/architecture/concepts) | `RuniqueEngine`, `Request`, `request.form()` |
| [Macros](/docs/fr/architecture/macros) | Macros de contexte, flash, routage, erreur |
| [Tags & filtres Tera](/docs/fr/architecture/tera) | Tags Django-like, filtres, fonctions |
| [Lifecycle d'une requête](/docs/fr/architecture/lifecycle) | Cycle de vie, bonnes pratiques |

## Retour au sommaire

- [Architecture](/docs/fr/architecture)
