# Other examples

## Flash messages — every type

The `success!`, `info!`, `warning!`, `error!` macros store the message in the session: it shows on the **next** page, so after a redirect.

```rust
pub async fn demo_messages(request: Request) -> AppResult<Response> {
    success!(request.notices => "This is a success message.");
    info!(request.notices => "This is an information message.");
    warning!(request.notices => "This is a warning message.");
    error!(request.notices => "This is an error message.");
    Ok(Redirect::to("/").into_response())
}
```

For a message on the page the same request renders (a form shown again after an error), use `flash_now!`:

```rust
context_update!(request => {
    "messages" => flash_now!(warning => "Check the fields."),
});
request.render("page.html")
```

`{% messages %}` shows both kinds.

---

## REST API

### API routes

```rust
pub fn routes() -> Routes {
    urlpatterns! {
        "/api/articles" => view!{ api_list_articles }, name = "api_articles",
    }
}
```

### JSON API handler

```rust
use axum::Json;
use serde_json::json;

pub async fn api_list_articles(request: Request) -> AppResult<Response> {
    let articles = article::Entity::find()
        .all(&*request.engine.db)
        .await?;

    Ok(Json(json!({
        "count": articles.len(),
        "data": articles
    })).into_response())
}
```

`data` serializes every column of the model. Never do it on a table holding a secret (the accounts: password hash): select the columns to expose, or go through a dedicated struct.

An API POST without a Runique form must check `request.prisme.csrf_valid` itself, like sign-out (see [Forms](/docs/en/exemple/forms)).

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
| `ValidationForm::try_form(form, &request, "name").await` | Validate a form (only when it was sent) |
| `validated.save(&db).await` | Persist to the database (not for an account: see [Forms](/docs/en/exemple/forms)) |
| `form.forms_mut().database_error(&err)` | Display a DB error inside the form |

---

## See also

| Section | Description |
| --- | --- |
| [Minimal application](/docs/en/exemple/minimal) | Simple starting point |
| [Forms](/docs/en/exemple/forms) | CRUD with forms |
| [Upload](/docs/en/exemple/upload) | File upload |

## Back to summary

- [Examples](/docs/en/exemple)
