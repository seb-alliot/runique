# 💬 Flash Messages

## Système de Messages

Runique fournit un système de messages pour les notifications utilisateur. Les messages sont automatiquement injectés dans `TemplateContext` sous la clé `messages`.

```rust
use runique::prelude::*;
use runique::flash::Message;
```

---

## Macros Disponibles

### success! - Message de succès

```rust
success!(message => "Enregistrement créé avec succès!");
```

### error! - Message d'erreur

```rust
error!(message => "Une erreur s'est produite");
error!(message => format!("Erreur: {}", e));
```

### info! - Message informatif

```rust
info!(message => "Veuillez vérifier votre email");
```

### warning! - Avertissement

```rust
warning!(message => "Cette action ne peut pas être annulée");
```

---

## Utilisation dans les Handlers

```rust
use runique::prelude::*;
use runique::flash::Message;
use axum::response::Redirect;

async fn create_post(
    message: Message,
    mut template: TemplateContext,
    Prisme(form): Prisme<PostForm>,
) -> Response {
    if !form.is_valid().await {
        error!(message => "Formulaire invalide");
        template.context.insert("form", &form);
        return template.render("post/form.html").unwrap();
    }

    match form.save(&*template.engine.db.clone()).await {
        Ok(post) => {
            success!(message => format!(
                "Article '{}' créé!",
                post.title
            ));
            Redirect::to(&format!("/posts/{}", post.id)).into_response()
        }
        Err(e) => {
            error!(message => format!("Erreur: {}", e));
            template.context.insert("form", &form);
            template.render("post/form.html").unwrap()
        }
    }
}

async fn delete_user(
    Path(id): Path<i32>,
    message: Message,
    template: TemplateContext,
) -> Response {
    let db = template.engine.db.clone();

    match users::Entity::delete_by_id(id).exec(&*db).await {
        Ok(_) => {
            success!(message => "Utilisateur supprimé");
            Redirect::to("/users").into_response()
        }
        Err(_) => {
            error!(message => "Impossible de supprimer l'utilisateur");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
```

---

## Affichage dans les Templates

**Note:** Les messages sont automatiquement injectés dans `TemplateContext` sous la variable `messages`. Pas besoin de les passer manuellement au template.

### Balise automatique

```html
<!-- La balise {% messages %} affiche automatiquement tous les messages -->
{% messages %}
```

**Template interne utilisé:**
```html
{% if messages %}
    <div class="flash-messages">
        {% for message in messages %}
        <div class="message message-{{ message.level }}">
            {{ message.content }}
        </div>
        {% endfor %}
    </div>
{% endif %}
```

**Personnalisation:** Pour personnaliser l'affichage, vous pouvez créer votre propre template `message.html` dans votre dossier templates ou boucler manuellement sur `messages` avec vos propres styles.

---

## Dissociation (Consuming)

Les messages sont automatiquement consommés lors de la création de `TemplateContext` (effet flash - une seule lecture):

```rust
async fn page(template: TemplateContext) -> Response {
    // Les messages sont déjà dans template.messages
    // Ils s'affichent UNE FOIS puis disparaissent
    template.render("page.html").unwrap()
}

async fn autre_page(template: TemplateContext) -> Response {
    // Après redirect, les anciens messages ont disparu
    // (déjà consommés lors du premier rendu)
    template.render("autre.html").unwrap()
}
```

---

## Pattern Complet

```rust
use runique::prelude::*;
use runique::flash::Message;
use axum::extract::{Path, Json};
use axum::response::Redirect;
use sea_orm::Set;

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub username: String,
    pub email: String,
}

async fn update_user(
    Path(id): Path<i32>,
    message: Message,
    mut template: TemplateContext,
    Json(payload): Json<UpdateUserRequest>,
) -> Response {
    let db = template.engine.db.clone();

    // Trouver l'utilisateur
    let user = match users::Entity::find_by_id(id)
        .one(&*db)
        .await
    {
        Ok(Some(u)) => u,
        Ok(None) => {
            error!(message => "Utilisateur non trouvé");
            return StatusCode::NOT_FOUND.into_response();
        }
        Err(e) => {
            error!(message => format!("Erreur DB: {}", e));
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // Mettre à jour
    let mut active_user = user.into_active_model();
    active_user.username = Set(payload.username.clone());
    active_user.email = Set(payload.email.clone());

    match active_user.update(&*db).await {
        Ok(updated) => {
            success!(message => format!(
                "Profil de {} mis à jour!",
                updated.username
            ));

            template.context.insert("user", &updated);
            template.render("users/profile.html").unwrap()
        }
        Err(e) => {
            error!(message => "Erreur lors de la mise à jour");
            warning!(message => format!("Détails: {}", e));

            template.context.insert("user", &user);
            template.render("users/profile.html").unwrap()
        }
    }
}

async fn list_posts(
    message: Message,
    template: TemplateContext,
) -> Response {
    info!(message => "Bienvenue sur la liste des articles");
    // Les messages sont déjà automatiquement dans template.messages
    template.render("posts/list.html").unwrap()
}
```

---

## Prochaines étapes

← [**Middleware & Sécurité**](https://github.com/seb-alliot/runique/blob/main/docs/fr/08-middleware.md) | [**Exemples Pratiques**](https://github.com/seb-alliot/runique/blob/main/docs/fr/10-examples.md) →
