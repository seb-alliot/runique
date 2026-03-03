# 🎨 Templates

## Tera Engine

Runique uses **Tera** as its template engine, with a Django-inspired syntax layer. Templates are written in standard HTML enriched with Tera tags and **Django-like tags** that Runique automatically transforms.

```rust
use runique::prelude::*;

pub async fn index(mut request: Request) -> AppResult<Response> {
    context_update!(request => {
        "title" => "Welcome to Runique",
        "description" => "A modern web framework",
    });

    request.render("index.html")
}
```

---

## The Request Object

Each handler receives a `Request` containing everything required:

```rust
pub struct Request {
    pub engine: AEngine,       // Arc<RuniqueEngine> (DB, Tera, Config)
    pub session: Session,      // tower-sessions session
    pub notices: Message,      // Flash messages
    pub csrf_token: CsrfToken, // CSRF token
    pub context: Context,      // Tera context
    pub method: Method,        // HTTP method
}
```

**Main methods:**

| Method                        | Description                          |
| ----------------------------- | ------------------------------------ |
| `request.render("page.html")` | Render template with current context |
| `request.is_get()`            | Checks whether the method is GET     |
| `request.is_post()`           | Checks whether the method is POST    |
| `request.is_put()`            | Checks whether the method is PUT     |
| `request.is_delete()`         | Checks whether the method is DELETE  |

> 💡 **Hot reload**: When `DEBUG=true`, templates are reloaded on every request (no Tera cache).

---

## `context_update!` Macro

A simplified syntax for injecting variables into the template context:

```rust
context_update!(request => {
    "title" => "My page",
    "user" => &form,
    "items" => &vec!["a", "b", "c"],
});

request.render("my_page.html")
```

Each `"key" => value` pair calls `request.context.insert("key", &value)`.

---

## Django-like Tags (Syntactic Sugar)

Runique pre-processes templates to transform Django-like syntax into standard Tera syntax. You can use both forms, but **Django-like tags** are recommended for readability.

### `{% static %}` — Static Assets

```html
<link rel="stylesheet" href='{% static "css/main.css" %}'>
<script src='{% static "js/app.js" %}'></script>
<img src='{% static "images/logo.png" %}' alt="Logo">
```

**Transformed into:** `{{ "css/main.css" | static }}`
**Result:** `/static/css/main.css`

---

### `{% media %}` — Media Files (Uploads)

```html
<img src='{% media "avatars/photo.jpg" %}' alt="Profile photo">
<a href='{% media "documents/cv.pdf" %}'>Download CV</a>
```

**Transformed into:** `{{ "avatars/photo.jpg" | media }}`
**Result:** `/media/avatars/photo.jpg`

---

### `{% csrf %}` — CSRF Protection

```html
<form method="post" action="/signup">
    {% csrf %}
    <!-- Automatically generates:
         <input type="hidden" name="csrf_token" value="...">
         + validation JS script -->

    <button type="submit">Submit</button>
</form>
```

**Transformed into:** `{% include "csrf/csrf_field.html" %}`

> ⚠️ **Not required** inside Runique forms (`{% form.xxx %}`) — the CSRF token is injected automatically. Use `{% csrf %}` only for manually written HTML forms.

---

### `{% messages %}` — Flash Messages

```html
{% messages %}
```

**Transformed into:** `{% include "message/message_include.html" %}`

Automatically displays all flash messages (success, error, info, warning) with corresponding CSS classes. See the [Flash Messages guide](09-flash-messages.md) for more details.

---

### `{% csp_nonce %}` — CSP Nonce

```html
<script {% csp_nonce %}>
    console.log("Secured script with CSP nonce");
</script>
```

**Transformed into:** `{% include "csp/csp_nonce.html" %}`

Injects the `nonce="..."` attribute on `<script>` or `<style>` tags for Content Security Policy.

---

### `{% link %}` — Named Route Links

```html
<a href='{% link "index" %}'>Home</a>
<a href='{% link "user_detail" id="42" %}'>User profile</a>
```

**Transformed into:** `{{ link(link='index') }}`

Resolves the name of a route registered in the URL registry (see Routing and the `urlpatterns!` macro).

---

### `{% form.xxx %}` — Full Form Rendering

```html
<form method="post" action="/signup">
    {% form.signup_form %}
    <button type="submit">Sign up</button>
</form>
```

**Transformed into:** `{{ signup_form | form | safe }}`

Renders the entire form: all HTML fields, validation errors, CSRF token, and required JS scripts.

---

### `{% form.xxx.field %}` — Single Field Rendering

```html
<form method="post" action="/signup">
    <div class="row">
        <div class="col">{% form.signup_form.username %}</div>
        <div class="col">{% form.signup_form.email %}</div>
    </div>
    <div class="row">
        {% form.signup_form.password %}
    </div>
    <button type="submit">Sign up</button>
</form>
```

**Transformed into:** `{{ signup_form | form(field='username') | safe }}`

Renders a single form field. JS scripts are automatically injected after the **last rendered field**.

---

## Tera Filters

Filters are the lower-level equivalent of Django-like tags. You can use them directly in standard Tera syntax if preferred.

### Asset Filters

| Filter           | Description                      | Example                                   |
| ---------------- | -------------------------------- | ----------------------------------------- |
| `static`         | App static URL prefix            | `{{ "css/main.css" \| static }}`          |
| `media`          | App media URL prefix             | `{{ "photo.jpg" \| media }}`              |
| `runique_static` | Framework internal static assets | `{{ "css/error.css" \| runique_static }}` |
| `runique_media`  | Framework internal media         | `{{ "logo.png" \| runique_media }}`       |

### Form Filter

| Filter              | Description                 | Example                                        |
| ------------------- | --------------------------- | ---------------------------------------------- |
| `form`              | Full form rendering         | `{{ my_form \| form \| safe }}`                |
| `form(field='xxx')` | Single field rendering      | `{{ my_form \| form(field='email') \| safe }}` |
| `csrf_field`        | Generates hidden CSRF input | `{{ csrf_token \| csrf_field \| safe }}`       |

### Tera Functions

| Function           | Description                       | Example                    |
| ------------------ | --------------------------------- | -------------------------- |
| `csrf()`           | Generates CSRF field from context | `{{ csrf() }}`             |
| `nonce()`          | Returns CSP nonce                 | `{{ nonce() }}`            |
| `link(link='...')` | Named URL resolution              | `{{ link(link='index') }}` |

---

## Template Inheritance

### Parent Template (base.html)

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>{% block title %}My Site{% endblock %}</title>
    <link rel="stylesheet" href='{% static "css/main.css" %}'>
</head>
<body>
    <header>
        <nav>
            <a href='{% link "index" %}'>Home</a>
            <a href='{% link "about" %}'>About</a>
        </nav>
    </header>

    {% messages %}

    <main>
        {% block content %}{% endblock %}
    </main>

    <footer>
        <p>&copy; 2026 My App</p>
    </footer>
</body>
</html>
```

### Child Template (index.html)

```html
{% extends "base.html" %}

{% block title %}Home{% endblock %}

{% block content %}
    <h2>{{ title }}</h2>
    <p>{{ description }}</p>

    {% if items %}
        <ul>
            {% for item in items %}
                <li>{{ item }}</li>
            {% endfor %}
        </ul>
    {% endif %}
{% endblock %}
```

---

## Loops and Conditions

### Loops

```html
<!-- Simple loop -->
<ul>
{% for item in items %}
    <li>{{ item.name }} - {{ item.price }}€</li>
{% endfor %}
</ul>

<!-- With index -->
{% for item in items %}
    <div class="item-{{ loop.index }}">{{ item }}</div>
{% endfor %}

<!-- With first/last -->
{% for item in items %}
    {% if loop.first %}<ul>{% endif %}
    <li>{{ item }}</li>
    {% if loop.last %}</ul>{% endif %}
{% endfor %}
```

### Conditions

```html
{% if user %}
    <p>Welcome, {{ user.name }}!</p>
{% elif guest %}
    <p>Welcome, visitor!</p>
{% else %}
    <p>Please log in.</p>
{% endif %}

{% if user and user.is_active %}
    <span class="badge">Active</span>
{% endif %}

{% if posts | length > 0 %}
    <p>{{ posts | length }} posts found.</p>
{% endif %}
```

---

## Tera Macros (Inside Templates)

Tera macros allow reusable components:

```html
{% macro render_user(user) %}
    <div class="user-card">
        <h3>{{ user.name }}</h3>
        <p>{{ user.email }}</p>
    </div>
{% endmacro %}

{% for u in users %}
    {{ self::render_user(user=u) }}
{% endfor %}
```

---

## Form Error Handling

### Automatic Display (via `{% form.xxx %}`)

When using `{% form.signup_form %}`, validation errors are **automatically rendered** under each relevant field.

### Manual Global Error Display

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

## Complete Example: Page with Form

```rust
pub async fn signup(
    mut request: Request,
    Prisme(mut form): Prisme<RegisterForm>,
) -> AppResult<Response> {
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

## Auto-Injected Variables

These variables are automatically available in all templates:

| Variable     | Type                | Description                          |
| ------------ | ------------------- | ------------------------------------ |
| `csrf_token` | `String`            | Session CSRF token                   |
| `csp_nonce`  | `String`            | CSP nonce for inline scripts/styles  |
| `messages`   | `Vec<FlashMessage>` | Flash messages from previous session |
| `debug`      | `bool`              | Debug mode status                    |

---

## ⚠️ Common Pitfall: Variable Name Collision

When using `{% form.user %}` in a template, Tera applies the `form` filter to the `user` variable in the context. **This variable must be a Prisme form**, not an arbitrary object.

```rust
// ❌ ERROR: "user" is a SeaORM Model, not a form
context_update!(request => {
    "user" => &db_user,
});

// ✅ CORRECT: separate form and DB entity
context_update!(request => {
    "user" => &form,
    "found_user" => &db_user,
});
```

---

## Next Steps

← [**Forms**](https://github.com/seb-alliot/runique/blob/main/docs/en/05-forms.md) | [**ORM & Database**](https://github.com/seb-alliot/runique/blob/main/docs/en/07-orm.md) →
