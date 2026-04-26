# Forms, context & pitfalls

## Form error handling

### Automatic display (via `{% form.xxx %}`)

When using `{% form.signup_form %}`, validation errors are **automatically rendered** under each relevant field.

### Manual display of global errors

```html
{% if signup_form.errors %}
    <div class="alert alert-warning">
        <ul>
            {% for field_name, error_msg in signup_form.errors %}
                <li><strong>{{ field_name }}:</strong> {{ error_msg }}</li>
            {% endfor %}
        </ul>
    </div>
{% endif %}
```

---

## Complete example: page with a form

```rust
// Rust handler
pub async fn signup(mut request: Request) -> AppResult<Response> {
    let mut form: RegisterForm = request.form();
    if request.is_get() {
        context_update!(request => {
            "title" => "Sign Up",
            "signup_form" => &form,
        });
        return request.render("signup.html");
    }

    if request.is_post() {
        if form.is_valid().await {
            let user = form.save(&request.engine.db).await.map_err(|err| {
                form.get_form_mut().database_error(&err);
                AppError::from(err)
            })?;
            success!(request.notices => format!("Welcome {}!", user.username));
            return Ok(Redirect::to("/").into_response());
        }

        context_update!(request => {
            "title" => "Validation Error",
            "signup_form" => &form,
            "messages" => flash_now!(error => "Please fix the errors"),
        });
        return request.render("signup.html");
    }

    request.render("signup.html")
}
```

```html
{% extends "base.html" %}

{% block title %}{{ title }}{% endblock %}

{% block content %}
    <h1>{{ title }}</h1>
    {% messages %}
    <form method="post" action="/signup">
        {% form.signup_form %}
        <button type="submit" class="btn btn-primary">Sign up</button>
    </form>
{% endblock %}
```

---

## Auto-injected variables

These variables are automatically available in all templates:

| Variable | Type | Description |
|----------|------|-------------|
| `csrf_token` | `String` | Session CSRF token |
| `csp_nonce` | `String` | CSP nonce for inline scripts/styles |
| `messages` | `Vec<FlashMessage>` | Flash messages from the previous session |
| `debug` | `bool` | Debug mode status |

---

## Common pitfall: variable name collision

When using `{% form.user %}`, the variable `user` **must be a form** (extracted via `request.form()`):

```rust
// ❌ ERROR: "user" is a SeaORM Model, not a form
context_update!(request => {
    "user" => &db_user,  // the form filter will crash!
});

// ✅ CORRECT: separate form and DB entity
context_update!(request => {
    "user" => &form,           // form → {% form.user %} works
    "found_user" => &db_user,  // Model → {{ found_user.email }}
});
```

---

## See also

| Section | Description |
| --- | --- |
| [Django-like tags](/docs/en/template/tags) | `{% form.xxx %}`, `{% csrf %}` |
| [Filters & functions](/docs/en/template/filters) | Low-level filters |

## Back to summary

- [Templates](/docs/en/template)
