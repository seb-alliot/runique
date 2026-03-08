# Concepts clés

## 1. RuniqueEngine

**État principal** partagé de l'application :

```rust
pub struct RuniqueEngine {
    pub db: Arc<DatabaseConnection>,
    pub tera: Arc<Tera>,
    pub config: Arc<RuniqueConfig>,
}
```

Injecté comme Extension Axum, accessible dans chaque handler via `request.engine`.

---

## 2. Request — L'extracteur principal

`Request` est l'extracteur central de Runique. Il remplace l'ancien `TemplateContext` et contient tout le nécessaire :

```rust
pub struct Request {
    pub engine: AEngine,       // Arc<RuniqueEngine>
    pub session: Session,      // Session tower-sessions
    pub notices: Message,      // Flash messages
    pub csrf_token: CsrfToken, // Token CSRF
    pub context: Context,      // Contexte Tera
    pub method: Method,        // Méthode HTTP
}
```

**Usage dans un handler :**

```rust
pub async fn index(mut request: Request) -> AppResult<Response> {
    context_update!(request => {
        "title" => "Accueil",
    });
    request.render("index.html")
}
```

**Méthodes :**

- `request.render("template.html")` — Rendu avec le contexte courant
- `request.is_get()` / `request.is_post()` — Vérification de la méthode HTTP

---

## 3. `Prisme<T>` — Extracteur de formulaire

```rust
pub async fn handler(
    mut request: Request,
    Prisme(mut form): Prisme<RegisterForm>,
) -> AppResult<Response> {
    if request.is_post() && form.is_valid().await {
        let user = form.save(&request.engine.db).await?;
        success!(request.notices => "Utilisateur créé !");
        return Ok(Redirect::to("/").into_response());
    }

    context_update!(request => {
        "form" => &form,
    });
    request.render("form.html")
}
```

Automatiquement :

1. Parse le body de la requête
2. Crée une instance du formulaire
3. Injecte le token CSRF
4. Remplit les données soumises

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Macros](https://github.com/seb-alliot/runique/blob/main/docs/fr/architecture/macros/macros.md) | Macros de contexte, flash, routage, erreur |
| [Tags & filtres Tera](https://github.com/seb-alliot/runique/blob/main/docs/fr/architecture/tera/tera.md) | Tags Django-like, filtres, fonctions |
| [Stack middleware](https://github.com/seb-alliot/runique/blob/main/docs/fr/architecture/middleware/middleware.md) | Ordre des slots, injection de dépendances |
| [Lifecycle d'une requête](https://github.com/seb-alliot/runique/blob/main/docs/fr/architecture/lifecycle/lifecycle.md) | Cycle de vie, bonnes pratiques |

## Retour au sommaire

- [Architecture](https://github.com/seb-alliot/runique/blob/main/docs/fr/architecture/02-architecture.md)
