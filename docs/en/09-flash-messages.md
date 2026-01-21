# 💬 Flash Messages

## Message System

Runique provides a message system for user notifications:

```rust
use runique::macro_runique::flash_message::*;
```

---

## Available Macros

### success! - Success Message

```rust
success!(ctx.flash => "Record created successfully!");
```

### error! - Error Message

```rust
error!(ctx.flash => "An error occurred");
error!(ctx.flash => format!("Error: {}", e));
```

### info! - Informational Message

```rust
info!(ctx.flash => "Please check your email");
```

### warning! - Warning

```rust
warning!(ctx.flash => "This action cannot be undone");
```

---

## Using in Handlers

```rust
use runique::prelude::*;
use axum::response::Redirect;

async fn create_post(
    mut ctx: RuniqueContext,
    ExtractForm(form): ExtractForm<PostForm>,
) -> Response {
    if !form.is_valid().await {
        error!(ctx.flash => "Invalid form");
        return template.render("post/form.html", &context!{
            "form" => form
        });
    }

    match form.save(&*ctx.engine.db.clone()).await {
        Ok(post) => {
            success!(ctx.flash => format!(
                "Post '{}' created!",
                post.title
            ));
            Redirect::to(&format!("/posts/{}", post.id)).into_response()
        }
        Err(e) => {
            error!(ctx.flash => format!("Error: {}", e));
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
            success!(ctx.flash => "User deleted");
            Redirect::to("/users").into_response()
        }
        Err(_) => {
            error!(ctx.flash => "Cannot delete user");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
```

---

## Display in Templates

### Loop Through All Messages

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

### By Type

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

### Reusable Component

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

<!-- In base.html: -->
{{ render_messages(messages) }}
```

---

## Consumption (Dissociation)

Messages are automatically consumed on render:

```rust
async fn page(ctx: RuniqueContext) -> Response {
    // Messages appear ONCE
    template.render("page.html", &context! {
        "messages" => ctx.processor.get_all()
    })
}

async fn another_page() -> Response {
    // After redirect, messages gone
    // (unless re-created)
}
```

---

## Complete Pattern

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

    // Find user
    let user = match users::Entity::find_by_id(id)
        .one(&*db)
        .await
    {
        Ok(Some(u)) => u,
        Ok(None) => {
            error!(ctx.flash => "User not found");
            return StatusCode::NOT_FOUND.into_response();
        }
        Err(e) => {
            error!(ctx.flash => format!("DB Error: {}", e));
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // Update
    let mut active_user = user.into_active_model();
    active_user.username = Set(payload.username.clone());
    active_user.email = Set(payload.email.clone());

    match active_user.update(&*db).await {
        Ok(updated) => {
            success!(ctx.flash => format!(
                "{}'s profile updated!",
                updated.username
            ));
            
            template.render("users/profile.html", &context! {
                "user" => updated,
                "messages" => ctx.processor.get_all()
            })
        }
        Err(e) => {
            error!(ctx.flash => "Update error");
            warning!(ctx.flash => format!("Details: {}", e));
            
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
    info!(ctx.flash => "Welcome to posts list");
    
    template.render("posts/list.html", &context! {
        "messages" => ctx.processor.get_all()
    })
}
```

---

## Next Steps

← [**Middleware & Security**](./08-middleware.md) | [**Practical Examples**](./10-examples.md) →
