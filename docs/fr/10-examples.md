# 📚 Exemples Pratiques

## 1️⃣ Application minimale

### Arborescence

```
mon_app/
├── Cargo.toml
├── .env
├── src/
│   ├── main.rs
│   ├── url.rs
│   └── views.rs
├── templates/
│   └── index.html
└── static/
    └── css/
        └── main.css
```

### main.rs

```rust
#[macro_use]
extern crate runique;

mod url;
mod views;

use runique::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = RuniqueConfig::from_env();

    let db_config = DatabaseConfig::from_env()?.build();
    let db = db_config.connect().await?;

    RuniqueApp::builder(config)
        .routes(url::routes())
        .with_database(db)
        .with_static_files()
        .build()
        .await?
        .run()
        .await?;

    Ok(())
}
```

### url.rs

```rust
use crate::views;
use runique::prelude::*;
use runique::{urlpatterns, view};

pub fn routes() -> Router {
    urlpatterns! {
        "/" => view!{ GET => views::index }, name = "index",
        "/about" => view!{ GET => views::about }, name = "about",
    }
}
```

### views.rs

```rust
use runique::prelude::*;

pub async fn index(mut request: Request) -> AppResult<Response> {
    context_update!(request => {
        "title" => "Accueil",
        "message" => "Bienvenue sur mon app Runique !",
    });
    request.render("index.html")
}

pub async fn about(mut request: Request) -> AppResult<Response> {
    context_update!(request => {
        "title" => "À propos",
    });
    request.render("about.html")
}
```

### templates/index.html

```html
<!DOCTYPE html>
<html lang="fr">
<head>
    <meta charset="UTF-8">
    <title>{{ title }}</title>
    <link rel="stylesheet" href='{% static "css/main.css" %}'>
</head>
<body>
    {% messages %}
    <h1>{{ title }}</h1>
    <p>{{ message }}</p>
    <a href='{% link "about" %}'>À propos</a>
</body>
</html>
```

---

## 2️⃣ CRUD avec formulaires

### Formulaire d'inscription

```rust
// src/forms.rs
use runique::prelude::*;

#[runique_form]
pub struct RegisterForm {
    pub form: Forms,
}

impl FormTrait for RegisterForm {
    fn new() -> Self {
        let mut form = Forms::new();
        form.add_field("username", FieldBuilder::text()
            .label("Nom d'utilisateur")
            .required()
            .min_length(3)
            .max_length(50)
            .build());
        form.add_field("email", FieldBuilder::email()
            .label("Email")
            .required()
            .build());
        form.add_field("password", FieldBuilder::password()
            .label("Mot de passe")
            .required()
            .min_length(8)
            .build());
        Self { form }
    }

    fn get_form(&self) -> &Forms { &self.form }
    fn get_form_mut(&mut self) -> &mut Forms { &mut self.form }
    fn get_name(&self) -> &str { "register_form" }
}

impl RegisterForm {
    pub async fn save(&self, db: &DatabaseConnection) -> Result<users::Model, DbErr> {
        use sea_orm::Set;
        let model = users::ActiveModel {
            username: Set(self.form.get_value("username").unwrap_or_default()),
            email: Set(self.form.get_value("email").unwrap_or_default()),
            password: Set(self.form.get_value("password").unwrap_or_default()),
            ..Default::default()
        };
        model.insert(db).await
    }
}
```

### Handler inscription

```rust
// src/views.rs
pub async fn inscription(
    mut request: Request,
    Prisme(mut form): Prisme<RegisterForm>,
) -> AppResult<Response> {
    let template = "inscription_form.html";

    if request.is_get() {
        context_update!(request => {
            "title" => "Inscription",
            "inscription_form" => &form,
        });
        return request.render(template);
    }

    if request.is_post() {
        if form.is_valid().await {
            let user = form.save(&request.engine.db).await.map_err(|err| {
                form.get_form_mut().database_error(&err);
                AppError::from(err)
            })?;

            success!(request.notices => format!("Bienvenue {} !", user.username));
            return Ok(Redirect::to("/").into_response());
        }

        context_update!(request => {
            "title" => "Erreur de validation",
            "inscription_form" => &form,
            "messages" => flash_now!(error => "Veuillez corriger les erreurs"),
        });
        return request.render(template);
    }

    request.render(template)
}
```

### Template inscription

```html
{% extends "base.html" %}

{% block title %}{{ title }}{% endblock %}

{% block content %}
    <h1>{{ title }}</h1>
    {% messages %}

    <form method="post" action='{% link "inscription" %}'>
        {% form.inscription_form %}
        <button type="submit" class="btn btn-primary">S'inscrire</button>
    </form>
{% endblock %}
```

---

## 3️⃣ Recherche et affichage d'entité

### Formulaire de recherche

```rust
#[runique_form]
pub struct UsernameForm {
    pub form: Forms,
}

impl FormTrait for UsernameForm {
    fn new() -> Self {
        let mut form = Forms::new();
        form.add_field("username", FieldBuilder::text()
            .label("Nom d'utilisateur")
            .required()
            .placeholder("Rechercher un utilisateur")
            .build());
        Self { form }
    }

    fn get_form(&self) -> &Forms { &self.form }
    fn get_form_mut(&mut self) -> &mut Forms { &mut self.form }
    fn get_name(&self) -> &str { "username_form" }
}
```

### Handler de recherche

```rust
pub async fn info_user(
    mut request: Request,
    Prisme(mut form): Prisme<UsernameForm>,
) -> AppResult<Response> {
    let template = "profile/view_user.html";

    if request.is_get() {
        context_update!(request => {
            "title" => "Rechercher un utilisateur",
            "user" => &form,
        });
        return request.render(template);
    }

    if request.is_post() {
        if !form.is_valid().await {
            context_update!(request => {
                "title" => "Rechercher un utilisateur",
                "user" => &form,
                "messages" => flash_now!(error => "Erreur de validation"),
            });
            return request.render(template);
        }

        let username = form.get_form().get_value("username").unwrap_or_default();
        let db = request.engine.db.clone();

        let user_opt = UserEntity::objects
            .filter(users::Column::Username.eq(&username))
            .first(&db)
            .await?;

        match user_opt {
            Some(user) => {
                context_update!(request => {
                    "title" => "Vue utilisateur",
                    "username" => &user.username,
                    "email" => &user.email,
                    "found_user" => &user,  // ⚠️ NE PAS nommer "user" → collision avec le form
                    "user" => &form,         // Le form doit garder le nom "user" pour {% form.user %}
                    "messages" => flash_now!(success => "Utilisateur trouvé !"),
                });
            }
            None => {
                context_update!(request => {
                    "title" => "Vue utilisateur",
                    "user" => &form,
                    "messages" => flash_now!(warning => "Utilisateur introuvable"),
                });
            }
        }

        return request.render(template);
    }

    request.render(template)
}
```

---

## 4️⃣ Upload de fichier

### Formulaire d'upload

```rust
#[runique_form]
pub struct ImageForm {
    pub form: Forms,
}

impl FormTrait for ImageForm {
    fn new() -> Self {
        let mut form = Forms::new();
        form.add_field("image", FieldBuilder::image()
            .label("Image")
            .required()
            .max_size_mb(5)
            .max_files(1)
            .max_dimensions(1920, 1080)
            .allowed_extensions(vec!["jpg", "png", "webp", "avif"])
            .build());
        Self { form }
    }

    fn get_form(&self) -> &Forms { &self.form }
    fn get_form_mut(&mut self) -> &mut Forms { &mut self.form }
    fn get_name(&self) -> &str { "image_form" }
}
```

### Handler d'upload

```rust
pub async fn upload_image(
    mut request: Request,
    Prisme(mut form): Prisme<ImageForm>,
) -> AppResult<Response> {
    let template = "forms/upload_image.html";

    if request.is_get() {
        context_update!(request => {
            "title" => "Uploader un fichier",
            "image_form" => &form,
        });
        return request.render(template);
    }

    if request.is_post() {
        if form.is_valid().await {
            success!(request.notices => "Fichier uploadé avec succès !");
            return Ok(Redirect::to("/").into_response());
        }

        context_update!(request => {
            "title" => "Erreur",
            "image_form" => &form,
            "messages" => flash_now!(error => "Veuillez corriger les erreurs"),
        });
        return request.render(template);
    }

    request.render(template)
}
```

### Template d'upload

```html
{% extends "base.html" %}

{% block content %}
    <h1>{{ title }}</h1>
    {% messages %}

    <form method="post" enctype="multipart/form-data">
        {% form.image_form %}
        <button type="submit">Uploader</button>
    </form>
{% endblock %}
```

---

## 5️⃣ Page avec tous les types de messages

```rust
pub async fn demo_messages(mut request: Request) -> AppResult<Response> {
    // Messages flash (affichés après redirect)
    success!(request.notices => "Ceci est un message de succès.");
    info!(request.notices => "Ceci est un message d'information.");
    warning!(request.notices => "Ceci est un message d'avertissement.");
    error!(request.notices => "Ceci est un message d'erreur.");

    context_update!(request => {
        "title" => "Démo messages",
    });
    request.render("demo.html")
}
```

```html
{% extends "base.html" %}

{% block content %}
    <h1>{{ title }}</h1>
    {% messages %}
    <p>Les messages ci-dessus viennent de la session flash.</p>
{% endblock %}
```

---

## 6️⃣ API REST

### Routes API

```rust
pub fn routes() -> Router {
    urlpatterns! {
        "/api/users" => view!{
            GET => api_list_users,
            POST => api_create_user
        }, name = "api_users",
    }
}
```

### Handler API JSON

```rust
use axum::Json;
use serde_json::json;

pub async fn api_list_users(request: Request) -> AppResult<Response> {
    let db = request.engine.db.clone();

    let users = users::Entity::find()
        .all(&*db)
        .await?;

    Ok(Json(json!({
        "status": "success",
        "count": users.len(),
        "data": users
    })).into_response())
}
```

---

## 7️⃣ Template de base complet

```html
<!-- templates/base.html -->
<!DOCTYPE html>
<html lang="fr">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{% block title %}Mon App{% endblock %}</title>

    <!-- CSS de l'application -->
    <link rel="stylesheet" href='{% static "css/main.css" %}'>
    {% block extra_css %}{% endblock %}
</head>
<body>
    <header>
        <nav>
            <a href='{% link "index" %}'>🏠 Accueil</a>
            <a href='{% link "about" %}'>ℹ️ À propos</a>
            <a href='{% link "inscription" %}'>📝 Inscription</a>
            <a href='{% link "blog" %}'>📰 Blog</a>
        </nav>
    </header>

    <!-- Messages flash automatiques -->
    {% messages %}

    <main>
        {% block content %}{% endblock %}
    </main>

    <footer>
        <p>&copy; 2026 — Propulsé par Runique 🦀</p>
    </footer>

    <!-- Scripts avec nonce CSP -->
    {% block extra_js %}{% endblock %}
</body>
</html>
```

---

## Résumé des patterns

| Pattern | Quand l'utiliser |
|---------|-----------------|
| `request.render("template.html")` | Rendu HTML standard |
| `Redirect::to("/").into_response()` | Après une action réussie (POST) |
| `context_update!(request => {...})` | Injecter des variables dans le template |
| `success!(request.notices => "...")` | Message flash avant redirect |
| `flash_now!(error => "...")` | Message immédiat (pas de redirect) |
| `form.is_valid().await` | Valider un formulaire Prisme |
| `form.save(&db).await` | Sauvegarder en base de données |
| `form.get_form_mut().database_error(&err)` | Afficher une erreur DB dans le formulaire |

---

## Pour aller plus loin

- [Installation](01-installation.md)
- [Architecture](02-architecture.md)
- [Configuration](03-configuration.md)
- [Routage](04-routing.md)
- [Formulaires](05-forms.md)
- [Templates](06-templates.md)
- [ORM](07-orm.md)
- [Middleware](08-middleware.md)
- [Flash Messages](09-flash-messages.md)

← [**Flash Messages**](09-flash-messages.md) | [**Retour au README**](README.md) →
