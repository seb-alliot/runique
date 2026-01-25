# 💬 Flash Messages

## Message System

Runique provides a message system for user notifications. Messages are automatically injected into `TemplateContext` under the `messages` key.

```rust
use runique::prelude::*;
use runique::flash::Message;
```

---

## Available Macros

### success! - Success message

```rust
success!(message => "Record created successfully!");
```

### error! - Error message

```rust
error!(message => "An error occurred");
error!(message => format!("Error: {}", e));
```

### info! - Informational message

```rust
info!(message => "Please check your email");
```

### warning! - Warning message

```rust
warning!(message => "This action cannot be undone");
```

---

## Using in Handlers

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
        error!(message => "Invalid form");
        template.context.insert("form", &form);
        return template.render("post/form.html").unwrap();
    }

    match form.save(&*template.engine.db.clone()).await {
        Ok(post) => {
            success!(message => format!(
                "Article '{}' created!",
                post.title
            ));
            Redirect::to(&format!("/posts/{}", post.id)).into_response()
        }
        Err(e) => {
            error!(message => format!("Error: {}", e));
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
            success!(message => "User deleted");
            Redirect::to("/users").into_response()
        }
        Err(_) => {
            error!(message => "Unable to delete user");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
```

---

## Display in Templates

**Note:** Messages are automatically injected into `TemplateContext` under the `messages` variable. No need to pass them manually to the template.

### Automatic tag

```html
<!-- The {% messages %} tag automatically displays all messages -->
{% messages %}
```

**Built-in template used:**
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

**Customization:** To customize the display, you can create your own `message.html` template in your templates folder or manually loop over `messages` with your own styles.

---

## Consumption (Flash Effect)

Messages are automatically consumed when `TemplateContext` is created (flash effect - single read):

```rust
async fn page(template: TemplateContext) -> Response {
    // Messages are already in template.messages
    // They display ONCE then disappear
    template.render("page.html").unwrap()
}

async fn other_page(template: TemplateContext) -> Response {
    // After redirect, old messages have disappeared
    // (already consumed during first render)
    template.render("other.html").unwrap()
}
```

---

## Complete Pattern

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

    // Find user
    let user = match users::Entity::find_by_id(id)
        .one(&*db)
        .await
    {
        Ok(Some(u)) => u,
        Ok(None) => {
            error!(message => "User not found");
            return StatusCode::NOT_FOUND.into_response();
        }
        Err(e) => {
            error!(message => format!("DB Error: {}", e));
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // Update
    let mut active_user = user.into_active_model();
    active_user.username = Set(payload.username.clone());
    active_user.email = Set(payload.email.clone());

    match active_user.update(&*db).await {
        Ok(updated) => {
            success!(message => format!(
                "{}'s profile updated!",
                updated.username
            ));

            template.context.insert("user", &updated);
            template.render("users/profile.html").unwrap()
        }
        Err(e) => {
            error!(message => "Error updating profile");
            warning!(message => format!("Details: {}", e));

            template.context.insert("user", &user);
            template.render("users/profile.html").unwrap()
        }
    }
}

async fn list_posts(
    message: Message,
    template: TemplateContext,
) -> Response {
    info!(message => "Welcome to the articles list");
    // Messages are already automatically in template.messages
    template.render("posts/list.html").unwrap()
}
```

---

## Next Steps

← [**Middleware & Security**](https://github.com/seb-alliot/runique/blob/main/docs/en/08-middleware.md) | [**Practical Examples**](https://github.com/seb-alliot/runique/blob/main/docs/en/10-examples.md) →
