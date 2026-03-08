# CRUD avec formulaires

## Formulaire d'inscription

```rust
// src/forms.rs
use runique::prelude::*;

pub struct RegisterForm {
    pub form: Forms,
}

impl RuniqueForm for RegisterForm {
    fn register_fields(form: &mut Forms) {
        form.field(
            &TextField::text("username")
                .label("Nom d'utilisateur")
                .required()
                .min_length(3, "Minimum 3 caractères")
                .max_length(50, "Maximum 50 caractères")
        );
        form.field(
            &TextField::email("email")
                .label("Email")
                .required()
        );
        form.field(
            &TextField::password("password")
                .label("Mot de passe")
                .required()
                .min_length(8, "Minimum 8 caractères")
        );
    }

    impl_form_access!();
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

---

## Handler inscription

```rust
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

---

## Template inscription

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

## Recherche et affichage d'entité

### Formulaire de recherche

```rust
pub struct UsernameForm {
    pub form: Forms,
}

impl RuniqueForm for UsernameForm {
    fn register_fields(form: &mut Forms) {
        form.field(
            &TextField::text("username")
                .label("Username")
                .required()
                .placeholder("Search a user")
        );
    }
    impl_form_access!();
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

        let user_opt = users::Entity::objects
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

## Voir aussi

| Section | Description |
| --- | --- |
| [Application minimale](https://github.com/seb-alliot/runique/blob/main/docs/fr/exemple/minimal/minimal.md) | Point de départ simple |
| [Upload](https://github.com/seb-alliot/runique/blob/main/docs/fr/exemple/upload/upload.md) | Upload de fichier |

## Retour au sommaire

- [Exemples](https://github.com/seb-alliot/runique/blob/main/docs/fr/exemple/10-examples.md)
