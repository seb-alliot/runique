# Other examples

## Flash messages — all types

```rust
pub async fn demo_messages(mut request: Request) -> AppResult<Response> {
    success!(request.notices => "This is a success message.");
    info!(request.notices => "This is an informational message.");
    warning!(request.notices => "This is a warning message.");
    error!(request.notices => "This is an error message.");

    context_update!(request => {
        "title" => "Messages demo",
    });
    request.render("demo.html")
}
```

```html
{% extends "base.html" %}

{% block content %}
    <h1>{{ title }}</h1>
    {% messages %}
    <p>The messages above come from the flash session.</p>
{% endblock %}
```

---

## REST API

### API routes

```rust
pub fn routes() -> Router {
    urlpatterns! {
        "/api/users" => view!{ api_list_users }, name = "api_users",
    }
}
```

### JSON API handler

```rust
use axum::Json;
use serde_json::json;

pub async fn api_list_users(request: Request) -> AppResult<Response> {
    let users = users::Entity::find()
        .all(&*request.engine.db)
        .await?;

    Ok(Json(json!({
        "status": "success",
        "count": users.len(),
        "data": users
    })).into_response())
}
```

---

## Complete base template

```html
<!-- templates/base.html -->
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{% block title %}My App{% endblock %}</title>
    <link rel="stylesheet" href='{% static "css/main.css" %}'>
    {% block extra_css %}{% endblock %}
</head>
<body>
    <header>
        <nav>
            <a href='{% link "index" %}'>Home</a>
            <a href='{% link "about" %}'>About</a>
            <a href='{% link "signup" %}'>Sign up</a>
        </nav>
    </header>

    {% messages %}

    <main>
        {% block content %}{% endblock %}
    </main>

    <footer>
        <p>&copy; 2026 — Powered by Runique</p>
    </footer>

    {% block extra_js %}{% endblock %}
</body>
</html>
```

---

## Pattern summary

| Pattern | When to use |
|---------|-------------|
| `request.render("template.html")` | Standard HTML rendering |
| `Redirect::to("/").into_response()` | After a successful action (POST) |
| `context_update!(request => {...})` | Inject variables into the template |
| `success!(request.notices => "...")` | Flash message before redirect |
| `flash_now!(error => "...")` | Immediate message (no redirect) |
| `form.is_valid().await` | Validate a Prisme form |
| `form.save(&db).await` | Persist to the database |
| `form.get_form_mut().database_error(&err)` | Display a DB error inside the form |

---

## See also

| Section | Description |
| --- | --- |
| [Minimal application](https://github.com/seb-alliot/runique/blob/main/docs/en/exemple/minimal/minimal.md) | Simple starting point |
| [Forms](https://github.com/seb-alliot/runique/blob/main/docs/en/exemple/forms/forms.md) | CRUD with forms |
| [Upload](https://github.com/seb-alliot/runique/blob/main/docs/en/exemple/upload/upload.md) | File upload |

## Back to summary

- [Examples](https://github.com/seb-alliot/runique/blob/main/docs/en/exemple/10-examples.md)
