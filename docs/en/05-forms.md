# 📝 Forms

## Define a Form

Using `RuniqueForm` derivation:

```rust
use runique::derive_form::RuniqueForm;
use serde::{Deserialize, Serialize};

#[derive(RuniqueForm, Debug, Clone, Serialize, Deserialize)]
pub struct LoginForm {
    #[field(label = "Email", required, input_type = "email")]
    pub email: String,

    #[field(label = "Password", required, input_type = "password")]
    pub password: String,

    #[field(label = "Remember me?", input_type = "checkbox")]
    pub remember: Option<bool>,
}

#[derive(RuniqueForm, Debug, Clone, Serialize, Deserialize)]
pub struct RegisterForm {
    #[field(label = "Username", required, min_length = 3, max_length = 50)]
    pub username: String,

    #[field(label = "Email", required, input_type = "email")]
    pub email: String,

    #[field(label = "Password", required, min_length = 8)]
    pub password: String,

    #[field(label = "Confirm password", required)]
    pub confirm_password: String,

    #[field(label = "I accept terms", required, input_type = "checkbox")]
    pub accept_terms: bool,
}
```

---

## Using in Handlers

### Automatic Extraction (Prisme)

```rust
use demo_app::forms::LoginForm;
use runique::forms::Prisme;

async fn login_form(
    template: TemplateContext,
) -> Response {
    let form = template.form::<LoginForm>();
    template.render("login.html", &context! {
        "form" => form
    })
}

async fn login_submit(
    mut ctx: RuniqueContext,
    template: TemplateContext,
    Prisme(mut form): Prisme<LoginForm>,
) -> Response {
    // Automatic validation
    if !form.is_valid().await {
        return template.render("login.html", &context! {
            "form" => form,
            "has_errors" => true
        });
    }

    // Authenticate user
    if let Ok(Some(user)) = authenticate(&form.email, &form.password).await {
        success!(ctx.flash => "Welcome!");
        ctx.session.insert("user_id", user.id).unwrap();
        return Redirect::to("/dashboard").into_response();
    }

    error!(ctx.flash => "Invalid email or password");
    template.render("login.html", &context! {
        "form" => form
    })
}
```

### Guards (Sentinel) and roles

`Prisme` runs three steps: `Sentinel` (access rules), CSRF check, then data extraction (`Aegis`). If no rule is provided, only CSRF+extraction run. To enforce login/role:

```rust
use runique::forms::utils::prisme::{GuardContext, GuardRules};

pub async fn with_rules<B>(mut req: Request<B>, next: Next<B>) -> impl IntoResponse {
    // Require login and either Admin or Editor
    req.extensions_mut()
        .insert(GuardRules::login_and_roles(["Admin", "Editor"]));

    // Inject user context from your auth layer
    req.extensions_mut().insert(GuardContext {
        user_id: Some("123".into()),
        roles: vec!["Admin".into()],
    });

    next.run(req).await
}
```

If rules are missing, `Sentinel` is a no-op. CSRF remains enforced, and form extraction proceeds normally.

---

## Form Rendering

The CSRF token is **automatically included** in all forms created with the `#[derive(RuniqueForm)]` macro.

### Full Form Rendering

Use the `form` Tera filter to display the complete form with all fields and CSRF token:

```html
{% form.inscription_form %}
```

### Field-by-Field Rendering

For full control over the layout, access fields individually using the `form` filter:

```html
{{ csrf(csrf_token=form.fields.csrf_token.value) | safe }}

<input type="text" name="username" value="{{ form.inscription_form.username }}" />
<input type="email" name="email" value="{{ form.inscription_form.email }}" />
## Custom Validation

```rust
#[derive(RuniqueForm, Debug, Clone)]
pub struct UserForm {
    pub username: String,
    pub email: String,
}

impl UserForm {
    pub async fn is_valid(&mut self) -> bool {
        let mut is_valid = true;

        // Length validation
        if self.username.len() < 3 {
            self.add_error("username", "Min 3 characters");
            is_valid = false;
        }

        // Uniqueness validation
        if let Ok(Some(_)) = User::find_by_email(&self.email).await {
            self.add_error("email", "Email already used");
            is_valid = false;
        }

        is_valid
    }

    pub async fn save(&self, db: &DatabaseConnection) -> Result<User> {
        User::create(self.username.clone(), self.email.clone()).save(db).await
    }
}
```

---

## Render in Templates

```html
<form method="post" action="/login">
    {% csrf_field %}

    <div class="form-group">
        <label for="email">Email:</label>
        <input
            type="email"
            name="email"
            id="email"
            value="{{ form.email }}"
            {% if form.has_error('email') %}class="error"{% endif %}
        >
        {% if form.has_error('email') %}
            <span class="error">{{ form.get_error('email') }}</span>
        {% endif %}
    </div>

    <div class="form-group">
        <label for="password">Password:</label>
        <input
            type="password"
            name="password"
            id="password"
        >
        {% if form.has_error('password') %}
            <span class="error">{{ form.get_error('password') }}</span>
        {% endif %}
    </div>

    <button type="submit">Login</button>
</form>
```

---

## Form Extraction

```rust
#[derive(Deserialize, RuniqueForm)]
pub struct SearchForm {
    pub query: String,
    pub category: Option<String>,
}

async fn search(
    ExtractForm(form): ExtractForm<SearchForm>,
) -> Response {
    let results = search_items(&form.query).await;
    Json(json!({ "results" => results }))
}
```

---

## Next Steps

← [**Routing**](./04-routing.md) | [**Templates**](./06-templates.md) →
