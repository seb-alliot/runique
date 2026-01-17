# Templates Guide - Runique Framework

Runique uses **Tera** as its template engine with a **custom preprocessing system** to add Django-like tags.

## Table of Contents

1. [Basic Tera Syntax](#basic-tera-syntax)
2. [Runique Custom Tags](#runique-custom-tags)
3. [Preprocessing](#preprocessing)
4. [Context and Variables](#context-and-variables)
5. [Template Inheritance](#template-inheritance)
6. [Complete Examples](#complete-examples)

---

## Basic Tera Syntax

Tera is a template engine inspired by Jinja2/Django.

### Variables

```html
{{ variable }}
{{ user.name }}
{{ user.age }}
```

### Filters

```html
{{ text|upper }}
{{ text|lower }}
{{ text|truncate(length=100) }}
{{ date|date(format="%m/%d/%Y") }}
{{ price|floatformat(decimal_places=2) }}
```

### Conditions

```html
{% if user.is_authenticated %}
    <p>Welcome {{ user.username }}!</p>
{% else %}
    <p>Please log in.</p>
{% endif %}
```

### Loops

```html
{% for post in posts %}
    <article>
        <h2>{{ post.title }}</h2>
        <p>{{ post.content }}</p>
    </article>
{% endfor %}
```

### Blocks and Inheritance

```html
{% extends "base.html" %}

{% block title %}My title{% endblock %}

{% block content %}
    <p>My page content</p>
{% endblock %}
```

---

## Runique Custom Tags

Runique adds **custom tags** via a preprocessing system that transforms tags before Tera processes them.

### 1. Filters `static` and `media`

Generate URLs for static and media files.

**Syntax:**

```html
{{ "path/to/file" | static }}
{{ "path/to/file" | media }}
{{ variable | static }}
{{ variable | media }}
```

**Examples:**

```html
<!-- CSS -->
<link rel="stylesheet" href="{{ 'css/style.css' | static }}">

<!-- JavaScript -->
<script src="{{ 'js/app.js' | static }}"></script>

<!-- Static images -->
<img src="{{ 'images/logo.png' | static }}" alt="Logo">

<!-- User uploaded image -->
<img src="{{ user.avatar | media }}" alt="Avatar">

<!-- Uploaded document -->
<a href="{{ document.file | media }}">Download document</a>

<!-- Video -->
<video src="{{ video.path | media }}" controls></video>
```

**Configuration (.env):**

```env
STATIC_URL=/static/
STATIC_ROOT=static/
MEDIA_URL=/media/
MEDIA_ROOT=media/
```

**Result after rendering:**

```html
<link rel="stylesheet" href="/static/css/style.css">
<img src="/media/avatars/user123.jpg" alt="Avatar">
```

**Filters for internal Runique files:**

```html
<!-- Internal Runique static files -->
{{ "theme.css" | runique_static }}

<!-- Internal Runique media files -->
{{ file | runique_media }}
```

### 2. Function `csrf_token()`

Generates the hidden CSRF token field.

**Syntax:**

```html
{{ csrf_token(token=csrf_token) }}
```

**Example:**

```html
<form method="post" action="/submit/">
    {{ csrf_token(token=csrf_token) }}

    <input type="text" name="username">
    <input type="password" name="password">
    <button type="submit">Log in</button>
</form>
```

**Result after rendering:**

```html
<input type="hidden" name="csrf_token" value="abc123...xyz789">
```

**⚠️ Important:** The `CsrfMiddleware` must be enabled for this function to work.

**Alternative with preprocessed tag `{% csrf %}` :**

```html
<form method="post">
    {% csrf %}
    <!-- Automatically transformed to {% include "csrf" %} -->
</form>
```

### 3. Tag `{% messages %}`

Displays flash messages (success, error, warning, info).

**Syntax:**

```html
{% messages %}
```

**Note:** This tag is automatically transformed to `{% include "message" %}` during preprocessing.

**Example:**

```html
<body>
    <header>
        <h1>My App</h1>
    </header>

    <!-- Automatic message display -->
    {% messages %}

    <main>
        {% block content %}{% endblock %}
    </main>
</body>
```

**Generated result:**

```html
<div class="messages">
    <div class="alert alert-success">Operation successful!</div>
    <div class="alert alert-error">An error occurred.</div>
</div>
```

**Usage in handlers:**

```rust
use runique::prelude::*;

async fn create_user(
    Form(form): Form<UserForm>,
    mut message: Message,
) -> Response {
    if !form.is_valid() {
        let _ = message.error("Invalid data").await;
        return redirect("/register");
    }

    // Create user...

    let _ = message.success("Account created successfully!").await;
    redirect("/dashboard")
}
```

### 4. Function `link()`

Generates URLs using **reverse routing** (route names).

**Syntax:**

```html
{{ link(link='route_name') }}
{{ link(link='route_name', param1=value1, param2=value2) }}
```

**Note:** The tag `{{ link(link='route_name') }}` is automatically transformed to `{{ link(link='route_name') }}` during preprocessing.

**Examples:**

```html
<!-- Route without parameter -->
<a href="{{ link(link='index') }}">Home</a>

<!-- Route with parameter -->
<a href="{{ link(link='post_detail', id=post.id) }}">View article</a>

<!-- Multiple parameters -->
<a href="{{ link(link='user_post', user_id=user.id, post_id=post.id) }}">
    User's article
</a>

<!-- Alternative with preprocessed tag -->
<a href="{{ link(link='index') }}">Home</a>
<a href="{{ link(link='post_detail' id=post.id %}">View article</a>
```

**Route configuration:**

```rust
use runique::prelude::*;

fn routes() -> Router {
    urlpatterns![
        path!("", index, "index"),
        path!("posts/{id}/", post_detail, "post_detail"),
        path!("users/{user_id}/posts/{post_id}/", user_post, "user_post"),
    ]
}
```

**Result after rendering:**

```html
<a href="/">Home</a>
<a href="/posts/42/">View article</a>
<a href="/users/10/posts/42/">User's article</a>
```

**Automatic validation:**

The `link()` function automatically validates:
- Missing parameters (error if a required parameter is not provided)
- Extra parameters (error if an unexpected parameter is provided)
- Parameter types (String, Number, Bool accepted)

### 5. Function `nonce()`

**⚠️ IMPORTANT: Generates CSP nonce ONLY if `use_nonce: true` in CSP configuration.**

**Syntax:**

```html
{{ nonce() }}
```

**Note:** The tag `{{ csp }}` is automatically transformed to `{% include "csp" %}` during preprocessing.

**Example:**

```html
<script {{ nonce() }}>
    // Inline JavaScript code
    console.log("Script allowed with CSP nonce");
</script>

<style {{ nonce() }}>
    /* Inline CSS */
    body { background: #f0f0f0; }
</style>

<!-- Alternative with preprocessed tag -->
<script {{ csp }}>
    console.log("Script with nonce");
</script>
```

**CSP Configuration with nonce:**

```rust
use runique::prelude::*;
use runique::middleware::CspConfig;

let csp_config = CspConfig {
    default_src: vec!["'self'".to_string()],
    script_src: vec!["'self'".to_string()],
    style_src: vec!["'self'".to_string()],
    use_nonce: true,  // ✅ Enables nonce generation
    ..Default::default()
};

RuniqueApp::new(settings).await?
    .middleware(CspMiddleware::new(csp_config))
    .routes(routes())
    .run()
    .await?;
```

**Result after rendering:**

```html
<script nonce="abc123xyz789">
    console.log("Script allowed");
</script>
```

**If `use_nonce: false`:**

The `nonce()` function generates an **empty string**:

```html
<script nonce="">
    console.log("No nonce generated");
</script>
```

**When to use CSP nonce:**

| Use Case | Use nonce? |
|----------|------------|
| Necessary inline scripts | ✅ Yes |
| Necessary inline styles | ✅ Yes |
| External scripts only | ❌ No (use `script-src 'self'`) |
| Strict mode without inline | ❌ No |

**Best practices:**

1. **Prefer external files** over inline scripts
2. **Enable `use_nonce: true`** only if you have inline code
3. **Never hardcode nonces** - always use `{{ nonce() }}`
4. **Add nonce** on each inline `<script>` and `<style>` tag

### 6. Filter `form`

Automatically generates the HTML for a form or a specific field.

**Syntax:**

```html
{{ form | form }}
{{ form | form(field='field_name' | static }}
```

**Examples:**

```html
<!-- Auto-generated complete form -->
<form method="post">
    {{ csrf_token(token=csrf_token) }}
    {{ form | form }}
    <button type="submit">Submit</button>
</form>

<!-- Specific field -->
<form method="post">
    {{ csrf_token(token=csrf_token) }}
    
    <div class="form-group">
        {{ form | form(field='username' | static }}
    </div>
    
    <div class="form-group">
        {{ form | form(field='password' | static }}
    </div>
    
    <button type="submit">Log in</button>
</form>
```

**Usage in handlers:**

```rust
use runique::prelude::*;

#[derive(Form)]
struct LoginForm {
    username: String,
    password: String,
}

async fn login_page(template: Template) -> Response {
    let form = LoginForm::default();
    
    template.render("login.html", context! {
        form: form,
        csrf_token: csrf_token,
    })
}
```

---

## Django-like tags transformed

Runique preprocesses certain tags for more concise syntax:

| Original tag | Transformed to | Description |
|--------------|----------------|-------------|
| `{% csrf %}` | `{% include "csrf" %}` | Hidden CSRF field |
| `{% messages %}` | `{% include "message" %}` | Flash messages |
| `{{ csp }}` | `{% include "csp" %}` | CSP nonce |
| `{{ "file" %}` | `{{ "file" \| static }}` | Static file |
| `{{ "file" %}` | `{{ "file" \| media }}` | Media file |
| `{% link "name" %}` | `{{ link(link='name') }}` | Reverse routing |

**Complete transformation example:**

```html
<!-- Before preprocessing -->
<link rel="stylesheet" href="{{ 'css/style.css' | static }}">
<img src="{{ user.avatar %}" alt="Avatar">
<a href="{{ link(link='home') }}">Home</a>
<form method="post">
    {% csrf %}
    {% messages %}
    <button>Submit</button>
</form>
<script {{ csp }}>console.log('Hello');</script>
```

```html
<!-- After preprocessing -->
<link rel="stylesheet" href="{{ 'css/style.css' | static }}">
<img src="{{ user.avatar | media }}" alt="Avatar">
<a href="{{ link(link='home') }}">Home</a>
<form method="post">
    {% include "csrf" %}
    {% include "message" %}
    <button>Submit</button>
</form>
<script {% include "csp" %}>console.log('Hello');</script>
```

---

## Preprocessing

Runique uses a **preprocessing system** that transforms custom tags into standard Tera tags **before** Tera processes them.

### Processing Order

1. **Runique Preprocessing** → Transforms custom tags
2. **Tera** → Processes standard Tera tags
3. **HTML Rendering** → Final result sent to client

### Transformation Example

**Original template:**

```html
<link rel="stylesheet" href="{{ 'css/style.css' | static }}">
<img src="{{ user.avatar %}" alt="Avatar">
<form method="post">
    {% csrf %}
    <button type="submit">Send</button>
</form>
```

**After preprocessing (before Tera):**

```html
<link rel="stylesheet" href="{{ 'css/style.css' | static }}">
<img src="{{ user.avatar | media }}" alt="Avatar">
<form method="post">
    {% include "csrf" %}
    <button type="submit">Send</button>
</form>
```

**After Tera (final HTML):**

```html
<link rel="stylesheet" href="/static/css/style.css">
<img src="/media/avatars/user123.jpg" alt="Avatar">
<form method="post">
    <input type="hidden" name="csrf_token" value="abc123...xyz">
    <button type="submit">Send</button>
</form>
```

---

## Context and Variables

### Passing Variables

```rust
use runique::prelude::*;

async fn index(template: Template) -> Response {
    template.render("index.html", context! {
        title: "Welcome",
        username: "Alice",
        posts: vec![
            Post { id: 1, title: "First article".to_string() },
            Post { id: 2, title: "Second article".to_string() },
        ],
    })
}
```

### Global Variables

Some variables are **automatically available** in all templates:

| Variable | Description |
|----------|-------------|
| `csrf_token` | CSRF token of the current session |
| `csp_nonce` | CSP nonce (if `use_nonce: true`) |

**Example:**

```html
<!-- CSRF token -->
{{ csrf_token(token=csrf_token) }}

<!-- CSP nonce (if use_nonce: true) -->
<script {{ nonce() }}>
    console.log("Script with nonce");
</script>
```

---

## Template Inheritance

### Base Template (templates/base.html)

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{% block title %}My App{% endblock %}</title>

    <link rel="stylesheet" href="{{ 'css/style.css' | static }}">
    {% block extra_css %}{% endblock %}
</head>
<body>
    <header>
        <h1>My Application</h1>
        <nav>
            <a href="{{ link(link='index') }}">Home</a>
            <a href="{{ link(link='post_list') }}">Articles</a>
            <a href="{{ link(link='contact') }}">Contact</a>
        </nav>
    </header>

    <!-- Flash messages -->
    {% messages %}

    <main>
        {% block content %}{% endblock %}
    </main>

    <footer>
        <p>© 2026 My App</p>
    </footer>

    <script src="{{ 'js/app.js' | static }}"></script>
    {% block extra_js %}{% endblock %}
</body>
</html>
```

### Child Template (templates/posts/list.html)

```html
{% extends "base.html" %}

{% block title %}Articles{% endblock %}

{% block extra_css %}
<link rel="stylesheet" href="{{ 'css/posts.css' | static }}">
{% endblock %}

{% block content %}
<h2>All Articles</h2>

{% for post in posts %}
<article>
    <h3>{{ post.title }}</h3>
    <p>{{ post.content|truncate(length=200) }}</p>
    <a href="{{ link(link='post_detail' id=post.id %}">Read more</a>
</article>
{% endfor %}

{% if posts|length == 0 %}
<p>No articles yet.</p>
{% endif %}
{% endblock %}

{% block extra_js %}
<script src="{{ 'js/posts.js' | static }}"></script>
{% endblock %}
```

---

## Complete Examples

### Example 1: Login Form

```html
{% extends "base.html" %}

{% block title %}Login{% endblock %}

{% block content %}
<h2>Login</h2>

<form method="post" action="/login/">
    {{ csrf_token(token=csrf_token) }}

    <div class="form-group">
        <label for="username">Username</label>
        <input type="text" id="username" name="username" required>
    </div>

    <div class="form-group">
        <label for="password">Password</label>
        <input type="password" id="password" name="password" required>
    </div>

    <button type="submit">Log in</button>
</form>

<p>
    <a href="{{ link(link='register') }}">Create account</a> |
    <a href="{{ link(link='password_reset') }}">Forgot password?</a>
</p>
{% endblock %}
```

### Example 2: User Profile with Avatar

```html
{% extends "base.html" %}

{% block title %}{{ user.username }}'s Profile{% endblock %}

{% block content %}
<div class="profile">
    <div class="profile-header">
        {% if user.avatar %}
            <img src="{{ user.avatar %}" alt="{{ user.username }}'s avatar">
        {% else %}
            <img src="{{ 'images/default-avatar.png' | static }}" alt="Default avatar">
        {% endif %}

        <h2>{{ user.username }}</h2>
        <p>{{ user.bio|default(value="No biography") }}</p>
    </div>

    <div class="profile-stats">
        <p>Member since: {{ user.created_at|date(format="%m/%d/%Y") }}</p>
        <p>Published articles: {{ user.posts|length }}</p>
    </div>

    {% if user.id == current_user.id %}
        <a href="{{ link(link='profile_edit') }}" class="btn">Edit my profile</a>
    {% endif %}
</div>
{% endblock %}
```

### Example 3: Post List with Pagination

```html
{% extends "base.html" %}

{% block title %}Articles - Page {{ page }}{% endblock %}

{% block content %}
<h2>Recent Articles</h2>

{% for post in posts %}
<article class="post">
    <h3>
        <a href="{{ link(link='post_detail' id=post.id %}">{{ post.title }}</a>
    </h3>

    <div class="post-meta">
        By {{ post.author.username }} on {{ post.created_at|date(format="%m/%d/%Y") }}
    </div>

    <p>{{ post.content|truncate(length=300) }}</p>

    <a href="{{ link(link='post_detail' id=post.id %}" class="read-more">
        Read more →
    </a>
</article>
{% endfor %}

<!-- Pagination -->
<div class="pagination">
    {% if has_previous %}
        <a href="{{ link(link='post_list') }}?page={{ page - 1 }}">← Previous</a>
    {% endif %}

    <span>Page {{ page }} of {{ total_pages }}</span>

    {% if has_next %}
        <a href="{{ link(link='post_list') }}?page={{ page + 1 }}">Next →</a>
    {% endif %}
</div>
{% endblock %}
```

### Example 4: Auto-generated Form

```html
{% extends "base.html" %}

{% block title %}Create Article{% endblock %}

{% block content %}
<h2>New Article</h2>

<form method="post">
    {{ csrf_token(token=csrf_token) }}

    <!-- Automatic form rendering -->
    {{ form | form }}

    <!-- Or manual field-by-field rendering -->
    <!--
    <div class="form-group">
        {{ form | form(field='title') }}
    </div>
    
    <div class="form-group">
        {{ form | form(field='content') }}
    </div>
    -->

    <button type="submit">Publish</button>
</form>
{% endblock %}
```

### Example 5: Inline Scripts with CSP Nonce

```html
{% extends "base.html" %}

{% block title %}Dashboard{% endblock %}

{% block content %}
<h2>Dashboard</h2>

<div id="chart"></div>

{% endblock %}

{% block extra_js %}
<!-- External script (no nonce needed) -->
<script src="{{ 'js/chart.min.js' | static }}"></script>

<!-- Inline script (requires nonce if strict CSP) -->
<script {{ nonce() }}>
    // Inline JavaScript code
    const data = {{ chart_data|json_encode|safe }};

    new Chart(document.getElementById('chart'), {
        type: 'bar',
        data: data,
    });
</script>
{% endblock %}
```

---

## Runique Custom Filters

In addition to standard Tera filters, Runique provides custom filters:

### Filter `static`

```html
<link rel="stylesheet" href="{{ 'css/style.css' | static }}">
```

### Filter `media`

```html
<img src="{{ user.avatar | media }}" alt="Avatar">
```

### Filter `runique_static`

```html
<link rel="stylesheet" href="{{ 'theme.css' | runique_static }}">
```

### Filter `runique_media`

```html
<img src="{{ file | runique_media }}" alt="Runique file">
```

### Filter `form`

```html
<!-- Complete form -->
{{ form | form }}

<!-- Specific field -->
{{ form | form(field='username') }}
```

### Filter `csrf_token` (deprecated)

```html
<!-- Use csrf_token() function instead -->
{{ csrf_token(token=csrf_token) }}
```

---

## Best Practices

### 1. Use Template Inheritance

```html
<!-- Good -->
{% extends "base.html" %}
{% block content %}...{% endblock %}

<!-- Bad: code duplication -->
<!DOCTYPE html>
<html>
<head>...</head>
<body>...</body>
</html>
```

### 2. Name Your Routes for Reverse Routing

```rust
// Good
path!("posts/{id}/", detail_post, "post_detail")

// Less good (no name)
path!("posts/{id}/", detail_post)
```

### 3. Escape User Variables

```html
<!-- Good (automatic escaping) -->
<p>{{ user.bio }}</p>

<!-- Dangerous (disables escaping) -->
<p>{{ user.bio|safe }}</p>
```

### 4. Organize Your Templates

```
templates/
├── base.html
├── components/
│   ├── header.html
│   ├── footer.html
│   └── pagination.html
├── posts/
│   ├── list.html
│   ├── detail.html
│   └── create.html
└── users/
    ├── profile.html
    └── settings.html
```

### 5. Use Includes for Reusable Components

```html
<!-- templates/components/pagination.html -->
<div class="pagination">
    {% if has_previous %}
        <a href="?page={{ page - 1 }}">← Previous</a>
    {% endif %}
    <span>Page {{ page }}</span>
    {% if has_next %}
        <a href="?page={{ page + 1 }}">Next →</a>
    {% endif %}
</div>

<!-- Usage -->
{% include "components/pagination.html" %}
```

---

## See Also

- [Getting Started](informations/documentation_english/GETTING_STARTED.md)
- [Security (CSP)](informations/documentation_english/CSP.md)
- [Forms](informations/documentation_english/FORMULAIRE.md)
- [Tera Documentation](https://keats.github.io/tera/)

Create powerful and secure templates with Runique!

---

**Version:** 1.0.87 (January 17, 2026)
**License:** MIT

*Documentation created with ❤️ by Claude for Itsuki*

