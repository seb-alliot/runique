
# 🎨 Templates

## Tera Engine

Runique uses **Tera** as its template engine, with a syntax layer inspired by Django. Templates are written in standard HTML enriched with Tera tags and **Django-like tags** that Runique automatically transforms.

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

Each handler receives a `Request` containing everything needed:

```rust
pub struct Request {
    pub engine: AEngine,       // Arc<RuniqueEngine> (DB, Tera, Config)
    pub session: Session,      // Tower sessions
    pub notices: Message,      // Flash messages
    pub csrf_token: CsrfToken, // CSRF token
    pub context: Context,      // Tera context
    pub method: Method,        // HTTP method
}
```

**Main methods:**

| Method                        | Description                                  |
| ----------------------------- | -------------------------------------------- |
| `request.render("page.html")` | Render the template with the current context |
| `request.is_get()`            | Checks if the method is GET                  |
| `request.is_post()`           | Checks if the method is POST                 |
| `request.is_put()`            | Checks if the method is PUT                  |
| `request.is_delete()`         | Checks if the method is DELETE               |

> 💡 **Hot-reload**: In `DEBUG=true` mode, templates are reloaded on every request (no Tera caching).

---

## Macro `context_update!`

Simplified syntax for injecting variables into the template context:

```rust
context_update!(request => {
    "title" => "My Page",
    "user" => &form,
    "items" => &vec!["a", "b", "c"],
});

request.render("my_page.html")
```

Each `"key" => value` pair calls `request.context.insert("key", &value)`.

---

## Django-like Tags (Sugar Syntax)

Runique pre-processes templates to transform Django-like syntax into standard Tera syntax. Both forms work, but **Django-like tags** are recommended for readability.

### `{% static %}` — Static assets

```html
<link rel="stylesheet" href='{% static "css/main.css" %}'>
<script src='{% static "js/app.js" %}'></script>
<img src='{% static "images/logo.png" %}' alt="Logo">
```

**Transformed to:** `{{ "css/main.css" | static }}`
**Result:** `/static/css/main.css`

---

### `{% media %}` — Media files (uploads)

```html
<img src='{% media "avatars/photo.jpg" %}' alt="Profile Photo">
<a href='{% media "documents/cv.pdf" %}'>Download CV</a>
```

**Transformed to:** `{{ "avatars/photo.jpg" | media }}`
**Result:** `/media/avatars/photo.jpg`

---

### `{% csrf %}` — CSRF protection

```html
<form method="post" action="/register">
    {% csrf %}
    <!-- Automatically generates <input type="hidden" name="csrf_token" value="..."> -->
    <!-- + JS validation script -->

    <button type="submit">Submit</button>
</form>
```

**Transformed to:** `{% include "csrf/csrf_field.html" %}`

> ⚠️ **Not needed** in Runique forms (`{% form.xxx %}`) — CSRF token is automatically injected. Use `{% csrf %}` only for manually written HTML forms.

---

### `{% messages %}` — Flash messages

```html
{% messages %}
```

**Transformed to:** `{% include "message/message_include.html" %}`

Automatically displays all flash messages (success, error, info, warning) with the proper CSS classes. See the [Flash Messages guide](09-flash-messages.md) for details.

---

### `{% csp_nonce %}` — CSP Nonce

```html
<script {% csp_nonce %}>
    console.log("Secure script with CSP nonce");
</script>
```

**Transformed to:** `{% include "csp/csp_nonce.html" %}`

Injects the `nonce="..."` attribute on a `<script>` or `<style>` tag for Content Security Policy (CSP).

---

### `{% link %}` — Named route links

```html
<a href='{% link "index" %}'>Home</a>
<a href='{% link "user_detail" id="42" %}'>User Profile</a>
```

**Transformed to:** `{{ link(link='index') }}`

Resolves the name of a registered route (see [Routing](04-routing.md) and the `urlpatterns!` macro).

---

### `{% form.xxx %}` — Full form rendering

```html
<form method="post" action="/register">
    {% form.register_form %}
    <button type="submit">Register</button>
</form>
```

**Transformed to:** `{{ register_form | form | safe }}`

Renders the entire form: all HTML fields, validation errors, CSRF token, and necessary JS scripts.

---

### `{% form.xxx.field %}` — Single field rendering

```html
<form method="post" action="/register">
    <div class="row">
        <div class="col">{% form.register_form.username %}</div>
        <div class="col">{% form.register_form.email %}</div>
    </div>
    <div class="row">
        {% form.register_form.password %}
    </div>
    <button type="submit">Register</button>
</form>
```

**Transformed to:** `{{ register_form | form(field='username') | safe }}`

Renders a single field. JS scripts are automatically injected after the **last field** rendered.

---

## Tera Filters

Filters are the "low-level" form of Django-like tags. You can use them directly in standard Tera syntax if preferred.

### Asset Filters

| Filter           | Description                      | Example             |                    |
| ---------------- | -------------------------------- | ------------------- | ------------------ |
| `static`         | Adds app static URL prefix       | `{{ "css/main.css"  | static }}`         |
| `media`          | Adds media URL prefix            | `{{ "photo.jpg"     | media }}`          |
| `runique_static` | Internal framework static assets | `{{ "css/error.css" | runique_static }}` |
| `runique_media`  | Internal framework media         | `{{ "logo.png"      | runique_media }}`  |

### Form Filter

| Filter              | Description                   | Example        |                     |          |
| ------------------- | ----------------------------- | -------------- | ------------------- | -------- |
| `form`              | Renders the complete form     | `{{ my_form    | form                | safe }}` |
| `form(field='xxx')` | Renders a single field        | `{{ my_form    | form(field='email') | safe }}` |
| `csrf_field`        | Generates a hidden CSRF input | `{{ csrf_token | csrf_field          | safe }}` |

### Tera Functions

| Function           | Description                             | Example                    |
| ------------------ | --------------------------------------- | -------------------------- |
| `csrf()`           | Generates a CSRF field from the context | `{{ csrf() }}`             |
| `nonce()`          | Returns the CSP nonce                   | `{{ nonce() }}`            |
| `link(link='...')` | Resolves a named route                  | `{{ link(link='index') }}` |

---

## Template Inheritance

### Parent template (`base.html`)

```html
<!DOCTYPE html>
<html lang="fr">
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

### Child template (`index.html`)

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

<!-- Using first/last -->
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

<!-- Combined tests -->
{% if user and user.is_active %}
    <span class="badge">Active</span>
{% endif %}

{% if posts | length > 0 %}
    <p>{{ posts | length }} articles found.</p>
{% endif %}
```

---

## Tera Macros (in templates)

Macros allow reusable components:

```html
{% macro render_user(user) %}
    <div class="user-card">
        <h3>{{ user.name }}</h3>
        <p>{{ user.email }}</p>
    </div>
{% endmacro %}

<!-- Usage -->
{% for u in users %}
    {{ self::render_user(user=u) }}
{% endfor %}
```

---

## Form Error Handling

### Automatic display (`{% form.xxx %}`)

When using `{% form.register_form %}`, validation errors are **automatically rendered** under the relevant fields.

### Manual display of global errors

```html
{% if register_form.errors %}
    <div class="alert alert-warning">
        <ul>
            {% for field_name, error_msg in register_form.errors %}
                <li><strong>{{ field_name }}:</strong> {{ error_msg }}</li>
            {% endfor %}
        </ul>
    </div>
{% endif %}
```

---

## Complete Example: Page with Form

```rust
// Rust handler
pub async fn register(
    mut request: Request,
    Prisme(mut form): Prisme<RegisterForm>,
) -> AppResult<Response> {
    if request.is_get() {
        context_update!(request => {
            "title" => "Register",
            "register_form" => &form,
        });
        return request.render("register.html");
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
            "register_form" => &form,
            "messages" => flash_now!(error => "Please fix the errors"),
        });
        return request.render("register.html");
    }

    request.render("register.html")
}
```

```html
<!-- Template register.html -->
{% extends "base.html" %}

{% block title %}{{ title }}{% endblock %}

{% block content %}
    <h1>{{ title }}</h1>

    {% messages %}

    <form method="post" action="/register">
        {% form.register_form %}
        <button type="submit" class="btn btn-primary">Register</button>
    </form>
{% endblock %}
```

---

## Auto-injected Variables

These variables are automatically available in all templates:

| Variable     | Type                | Description                              |
| ------------ | ------------------- | ---------------------------------------- |
| `csrf_token` | `String`            | Session CSRF token                       |
| `csp_nonce`  | `String`            | CSP nonce for inline scripts/styles      |
| `messages`   | `Vec<FlashMessage>` | Flash messages from the previous session |
| `debug`      | `bool`              | Whether debug mode is active             |

---

## ⚠️ Common Pitfall: Variable Name Collision

When using `{% form.user %}` in a template, Tera applies the `form` filter on the context variable `user`. **This variable must be a Prisme form**, not an arbitrary object.

```rust
// ❌ ERROR: "user" is a SeaORM Model, not a form
context_update!(request => {
    "user" => &db_user,  // users::Model → the form filter will crash!
});

// ✅ CORRECT: separate form and DB entity
context_update!(request => {
    "user" => &form,           // UsernameForm (Prisme) → {% form.user %} works
    "found_user" => &db_user,  // users::Model → {{ found_user.email }}
});
```

---

## Next Steps

← [**Forms**](https://github.com/seb-alliot/runique/blob/main/docs/en/05-forms.md) | [**ORM & Database**](https://github.com/seb-alliot/runique/blob/main/docs/en/07-orm.md) →
