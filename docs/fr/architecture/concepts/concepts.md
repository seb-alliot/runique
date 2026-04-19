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

## 3. `request.form()` — Extraction intégrée de formulaire

```rust
pub async fn handler(mut request: Request) -> AppResult<Response> {
    let mut form: RegisterForm = request.form();
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

`request.form()` orchestre automatiquement :

1. Parse le body de la requête
2. Crée une instance du formulaire
3. Vérifie le token CSRF
4. Remplit les données soumises

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Macros](/docs/fr/architecture/macros) | Macros de contexte, flash, routage, erreur |
| [Tags & filtres Tera](/docs/fr/architecture/tera) | Tags Django-like, filtres, fonctions |
| [Stack middleware](/docs/fr/architecture/middleware) | Ordre des slots, injection de dépendances |
| [Lifecycle d'une requête](/docs/fr/architecture/lifecycle) | Cycle de vie, bonnes pratiques |

## Retour au sommaire

- [Architecture](/docs/fr/architecture)
