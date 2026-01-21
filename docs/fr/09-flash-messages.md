# 💬 Flash Messages

## Système de Messages

Runique fournit un système de messages pour les notifications utilisateur:

```rust
use runique::macro_runique::flash_message::*;
```

---

## Macros Disponibles

### success! - Message de succès

```rust
success!(ctx.flash => "Enregistrement créé avec succès!");
```

### error! - Message d'erreur

```rust
error!(ctx.flash => "Une erreur s'est produite");
error!(ctx.flash => format!("Erreur: {}", e));
```

### info! - Message informatif

```rust
info!(ctx.flash => "Veuillez vérifier votre email");
```

### warning! - Avertissement

```rust
warning!(ctx.flash => "Cette action ne peut pas être annulée");
```

---

## Utilisation dans les Handlers

```rust
use runique::prelude::*;
use axum::response::Redirect;

async fn create_post(
    mut ctx: RuniqueContext,
    ExtractForm(form): ExtractForm<PostForm>,
) -> Response {
    if !form.is_valid().await {
        error!(ctx.flash => "Formulaire invalide");
        return template.render("post/form.html", &context!{
            "form" => form
        });
    }

    match form.save(&*ctx.engine.db.clone()).await {
        Ok(post) => {
            success!(ctx.flash => format!(
                "Article '{}' créé!",
                post.title
            ));
            Redirect::to(&format!("/posts/{}", post.id)).into_response()
        }
        Err(e) => {
            error!(ctx.flash => format!("Erreur: {}", e));
            template.render("post/form.html", &context!{
                "form" => form
            })
        }
    }
}

async fn delete_user(
    Path(id): Path<i32>,
    mut ctx: RuniqueContext,
) -> Response {
    let db = ctx.engine.db.clone();

    match users::Entity::delete_by_id(id).exec(&*db).await {
        Ok(_) => {
            success!(ctx.flash => "Utilisateur supprimé");
            Redirect::to("/users").into_response()
        }
        Err(_) => {
            error!(ctx.flash => "Impossible de supprimer l'utilisateur");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
```

---

## Affichage dans les Templates

### Boucle sur tous les messages

```html
<div class="messages-container">
    {% for message in messages %}
        <div class="alert alert-{{ message.type }}">
            {{ message.content }}
            <button class="close" data-dismiss="alert">&times;</button>
        </div>
    {% endfor %}
</div>
```

### Par type

```html
{% if messages %}
    {% for msg in messages %}
        {% if msg.type == 'success' %}
            <div class="alert alert-success">{{ msg.content }}</div>
        {% elif msg.type == 'error' %}
            <div class="alert alert-danger">{{ msg.content }}</div>
        {% elif msg.type == 'warning' %}
            <div class="alert alert-warning">{{ msg.content }}</div>
        {% elif msg.type == 'info' %}
            <div class="alert alert-info">{{ msg.content }}</div>
        {% endif %}
    {% endfor %}
{% endif %}
```

### Component Réutilisable

```html
{% macro render_messages(messages) %}
    {% if messages %}
        <div class="messages">
            {% for msg in messages %}
                <div class="alert alert-{{ msg.type }} alert-dismissible fade show">
                    {{ msg.content }}
                    <button type="button" class="btn-close" data-bs-dismiss="alert"></button>
                </div>
            {% endfor %}
        </div>
    {% endif %}
{% endmacro %}

<!-- Dans base.html: -->
{{ render_messages(messages) }}
```

---

## Dissociation (Consuming)

Les messages sont consommés automatiquement lors du rendu:

```rust
async fn page(ctx: RuniqueContext) -> Response {
    // Les messages s'affichent UNE FOIS
    template.render("page.html", &context! {
        "messages" => ctx.processor.get_all()
    })
}

async fn autre_page() -> Response {
    // Après redirect, messages disparus
    // (sauf s'ils sont re-créés)
}
```

---

## Pattern Complet

```rust
use runique::prelude::*;
use axum::extract::Path;
use axum::response::Redirect;

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub username: String,
    pub email: String,
}

async fn update_user(
    Path(id): Path<i32>,
    mut ctx: RuniqueContext,
    Json(payload): Json<UpdateUserRequest>,
    template: TemplateContext,
) -> Response {
    let db = ctx.engine.db.clone();

    // Trouver l'utilisateur
    let user = match users::Entity::find_by_id(id)
        .one(&*db)
        .await
    {
        Ok(Some(u)) => u,
        Ok(None) => {
            error!(ctx.flash => "Utilisateur non trouvé");
            return StatusCode::NOT_FOUND.into_response();
        }
        Err(e) => {
            error!(ctx.flash => format!("Erreur DB: {}", e));
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // Mettre à jour
    let mut active_user = user.into_active_model();
    active_user.username = Set(payload.username.clone());
    active_user.email = Set(payload.email.clone());

    match active_user.update(&*db).await {
        Ok(updated) => {
            success!(ctx.flash => format!(
                "Profil de {} mis à jour!",
                updated.username
            ));
            
            template.render("users/profile.html", &context! {
                "user" => updated,
                "messages" => ctx.processor.get_all()
            })
        }
        Err(e) => {
            error!(ctx.flash => "Erreur lors de la mise à jour");
            warning!(ctx.flash => format!("Détails: {}", e));
            
            template.render("users/profile.html", &context! {
                "user" => user,
                "messages" => ctx.processor.get_all()
            })
        }
    }
}

async fn list_posts(
    mut ctx: RuniqueContext,
    template: TemplateContext,
) -> Response {
    info!(ctx.flash => "Bienvenue sur la liste des articles");
    
    template.render("posts/list.html", &context! {
        "messages" => ctx.processor.get_all()
    })
}
```

---

## Prochaines étapes

← [**Middleware & Sécurité**](./08-middleware.md) | [**Exemples Pratiques**](./10-examples.md) →
