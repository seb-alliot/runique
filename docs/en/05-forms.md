# 📝 Forms

## Define a Form

Use `#[derive(RuniqueForm)]` to automatically create a form with validation:

```rust
use runique::derive_form::RuniqueForm;
use serde::{Deserialize, Serialize};

#[derive(RuniqueForm, Debug, Clone, Serialize, Deserialize)]
pub struct LoginForm {
    #[field(label = "Email", required, input_type = "email")]
    pub email: String,

    #[field(label = "Password", required, input_type = "password")]
    pub password: String,
}

#[derive(RuniqueForm, Debug, Clone, Serialize, Deserialize)]
pub struct RegisterForm {
    #[field(label = "Username", required, min_length = 3, max_length = 50)]
    pub username: String,

    #[field(label = "Email", required, input_type = "email")]
    pub email: String,

    #[field(label = "Password", required, min_length = 8, input_type = "password")]
    pub password: String,

    #[field(label = "Confirm", required, input_type = "password")]
    pub confirm_password: String,
}
```

---

## Use in a Handler

### Display an empty form

```rust
use demo_app::forms::LoginForm;
use runique::context::request::TemplateContext;

async fn login_form(mut template: TemplateContext) -> Response {
    template.context.insert("form", LoginForm::new());
    template.render("login.html")
}
```

### Process a submission (Prisme)

```rust
use runique::forms::Prisme;
use runique::flash::Message;

async fn login_submit(
    mut template: TemplateContext,
    Message(mut messages): Message,
    Prisme(mut form): Prisme<LoginForm>,
) -> Response {
    // Automatic validation
    if !form.is_valid().await {
        template.context.insert("form", form);
        template.context.insert("has_errors", true);
        return template.render("login.html");
    }

    // Authenticate user
    if let Some(user) = authenticate(&form.email, &form.password).await {
        messages.success("Welcome!");
        return Redirect::to("/dashboard").into_response();
    }

    messages.error("Email or password incorrect");
    template.context.insert("form", form);
    template.render("login.html")
}
```

---

## Form Rendering

The CSRF token is **always automatically included** in all forms.

### Display flash messages

```html
{% block messages %}
    {% messages %}
{% endblock %}
```

### Complete form rendering

```html
<form method="post" action="/login">
    {% csrf %}
    {% form.login_form %}
    <button type="submit">Login</button>
</form>
```

### Field-by-field rendering

```html
<form method="post" action="/login">
    {% csrf %}

    <div class="form-group">
        <label for="email">Email</label>
        {% form.login_form.email %}
    </div>

    <div class="form-group">
        <label for="password">Password</label>
        {% form.login_form.password %}
    </div>

    <button type="submit">Login</button>
</form>
```

---

## Custom Validation (Hook `clean`)

Override `is_valid()` to add business logic validations:

```rust
#[derive(RuniqueForm, Debug, Clone, Serialize, Deserialize)]
pub struct RegisterForm {
    #[field(label = "Username", required, min_length = 3)]
    pub username: String,

    #[field(label = "Email", required, input_type = "email")]
    pub email: String,

    #[field(label = "Password", required, min_length = 8)]
    pub password: String,

    #[field(label = "Confirm", required, input_type = "password")]
    pub confirm_password: String,
}

impl RegisterForm {
    pub async fn is_valid(&mut self) -> bool {
        let mut valid = true;

        // Field validation (min_length, required, etc.)
        // is automatic via the RuniqueForm trait

        // Custom business logic
        if self.password != self.confirm_password {
            self.add_error("confirm_password", "Passwords do not match");
            valid = false;
        }

        // Check email uniqueness
        if let Ok(Some(_)) = User::find_by_email(&self.email).await {
            self.add_error("email", "Email already used");
            valid = false;
        }

        valid
    }
}
```

---

## Complete Example: Registration Form

```html
{% extends 'index.html' %}

{% block content %}
    <h1>Register</h1>

    <!-- Flash messages -->
    {% block messages %}
        {% messages %}
    {% endblock %}

    <!-- Form -->
    <form method="post" action="/register">
        {% csrf %}

        <div class="form-group">
            <label for="username">Username:</label>
            {% form.register_form.username %}
        </div>

        <div class="form-group">
            <label for="email">Email:</label>
            {% form.register_form.email %}
        </div>

        <div class="form-group">
            <label for="password">Password:</label>
            {% form.register_form.password %}
        </div>

        <div class="form-group">
            <label for="confirm_password">Confirm:</label>
            {% form.register_form.confirm_password %}
        </div>

        <button type="submit">Register</button>
    </form>
{% endblock %}
```

---

## Search and Filters

```rust
#[derive(RuniqueForm, Debug, Clone, Serialize, Deserialize)]
pub struct SearchForm {
    #[field(label = "Search", required)]
    pub query: String,

    #[field(label = "Category")]
    pub category: Option<String>,
}

async fn search(
    Prisme(form): Prisme<SearchForm>,
) -> Json<serde_json::Value> {
    let results = search_items(&form.query, form.category).await;
    Json(json!({ "results" => results }))
}
```

---

## Next Steps

← [**Routing**](./04-routing.md) | [**Templates**](./06-templates.md) →
