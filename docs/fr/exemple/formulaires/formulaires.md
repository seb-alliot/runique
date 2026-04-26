# CRUD avec formulaires

## Formulaire d'inscription

### Formulaire manuel (sans modèle)

```rust
// src/forms.rs
use runique::prelude::*;

pub struct RegisterForm {
    pub form: Forms,
}

#[async_trait]
impl RuniqueForm for RegisterForm {
    impl_form_access!();

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

    // Validation métier — appelée automatiquement par is_valid()
    async fn clean(&mut self) -> Result<(), StrMap> {
        let mut errors = StrMap::new();
        if !self.get_string("email").contains('@') {
            errors.insert("email".to_string(), "Email invalide".to_string());
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}
```

### Formulaire basé sur un modèle

`#[form(...)]` génère la struct et `impl ModelForm`.
Le dev écrit `impl RuniqueForm` avec `impl_form_access!(model)` :

```rust
use runique::prelude::*;

#[form(schema = users_schema, fields = [username, email, password])]
pub struct RegisterForm;

#[async_trait]
impl RuniqueForm for RegisterForm {
    impl_form_access!(model);

    async fn clean(&mut self) -> Result<(), StrMap> {
        let mut errors = StrMap::new();
        if self.get_string("username").len() < 3 {
            errors.insert("username".to_string(), "Minimum 3 caractères".to_string());
        }
        if !self.get_string("email").contains('@') {
            errors.insert("email".to_string(), "Email invalide".to_string());
        }
        if self.get_string("password").len() < 10 {
            errors.insert("password".to_string(), "Minimum 10 caractères".to_string());
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}
```

> `#[async_trait]` est requis uniquement quand on override `clean` ou `clean_field`.
> Sans override async, `impl RuniqueForm { impl_form_access!(model); }` suffit.

---

## Handler inscription

```rust
pub async fn inscription(mut request: Request) -> AppResult<Response> {
    let mut form: RegisterForm = request.form();
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

{% block content %}
    <h1>{{ title }}</h1>
    {% messages %}

    <form method="post" action='{% link "inscription" %}'>
        {% form.inscription_form %}
        <button type="submit">S'inscrire</button>
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
pub async fn info_user(mut request: Request) -> AppResult<Response> {
    let mut form: UsernameForm = request.form();
    let template = "profile/view_user.html";

    if request.is_get() && form.is_valid().await {
        let username = form.cleaned_string("username").unwrap_or_default();
        let db = request.engine.db.clone();

        let user_opt = UserEntity::find()
            .filter(user::Column::Username.eq(&username))
            .one(&*db)
            .await
            .unwrap_or(None);

        match user_opt {
            Some(user) => {
                context_update!(request => {
                    "title" => "Vue utilisateur",
                    "found_user" => &user,  // ⚠️ NE PAS nommer "user" → collision avec le form
                    "user" => &form,
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

    context_update!(request => { "title" => "Rechercher un utilisateur", "user" => &form });
    request.render(template)
}
```

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Application minimale](/docs/fr/exemple/minimal) | Point de départ simple |
| [Upload](/docs/fr/exemple/upload) | Upload de fichier |

## Retour au sommaire

- [Exemples](/docs/fr/exemple)
