# Stack Middleware

## Ordre des slots

Runique applique les middlewares dans un **ordre optimal** via le système de slots :

```text
Requête entrante
    ↓
1. Extensions (slot 0)     → Injection Tera, Config, Engine
2. ErrorHandler (slot 10)  → Capture et rendu des erreurs
3. Custom (slot 20+)       → Middlewares personnalisés
4. CSP (slot 30)           → Content Security Policy & headers
5. Cache (slot 40)         → No-cache en développement
6. Session (slot 50)       → Gestion des sessions
7. CSRF (slot 60)          → Protection CSRF
8. Host (slot 70)          → Validation Allowed Hosts
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
