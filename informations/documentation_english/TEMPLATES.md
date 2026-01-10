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

### 1. Tag `{% static %}`

Generates the URL for a static file.

**Syntax:**

```html
{% static 'path/to/file' %}
```

**Examples:**

```html
<!-- CSS -->
<link rel="stylesheet" href="{% static 'css/style.css' %}">

<!-- JavaScript -->
<script src="{% static 'js/app.js' %}"></script>

<!-- Images -->
<img src="{% static 'images/logo.png' %}" alt="Logo">

<!-- Fonts -->
<link rel="stylesheet" href="{% static 'fonts/custom-font.woff2' %}">
```

**Configuration (.env):**

```env
STATIC_URL=/static/
STATIC_ROOT=static/
```

**Result after preprocessing:**

```html
<link rel="stylesheet" href="/static/css/style.css">
```

### 2. Tag `{% media %}`

Generates the URL for a media file (uploaded by users).

**Syntax:**

```html
{% media 'path/to/file' %}
{% media variable %}
```

**Examples:**

```html
<!-- Uploaded image -->
<img src="{% media user.avatar %}" alt="Avatar">

<!-- Uploaded document -->
<a href="{% media document.file %}">Download document</a>

<!-- Video -->
<video src="{% media video.path %}" controls></video>
```

**Configuration (.env):**

```env
MEDIA_URL=/media/
MEDIA_ROOT=media/
```

### 3. Tag `{% csrf %}`

Generates the hidden CSRF token field.

**Syntax:**

```html
{% csrf %}
```

**Example:**

```html
<form method="post" action="/submit/">
    {% csrf %}

    <input type="text" name="username">
    <input type="password" name="password">
    <button type="submit">Log in</button>
</form>
```

**Result after preprocessing:**

```html
<input type="hidden" name="csrftoken" value="abc123...xyz789">
```

**⚠️ Important:** The `CsrfMiddleware` must be enabled for this tag to work.

### 4. Tag `{% messages %}`

Displays flash messages (success, error, warning, info).

**Syntax:**

```html
{% messages %}
```

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

### 5. Tag `{% link %}`

Generates URLs using **reverse routing** (route names).

**Syntax:**

```html
{% link 'route_name' %}
{% link 'route_name' param1=value1 param2=value2 %}
```

**Examples:**

```html
<!-- Route without parameter -->
<a href="{% link 'index' %}">Home</a>

<!-- Route with parameter -->
<a href="{% link 'post_detail' id=post.id %}">View article</a>

<!-- Multiple parameters -->
<a href="{% link 'user_post' user_id=user.id post_id=post.id %}">
    User's article
</a>
```

**Route configuration:**

```rust
use runique::prelude::*;

fn routes() -> Router {
    urlpatterns![
        path!("", index, "index"),
        path!("posts/<id>/", post_detail, "post_detail"),
        path!("users/<user_id>/posts/<post_id>/", user_post, "user_post"),
    ]
}
```

**Result after preprocessing:**

```html
<a href="/">Home</a>
<a href="/posts/42/">View article</a>
<a href="/users/10/posts/42/">User's article</a>
```

### 6. Tag `{{ csp }}`

**⚠️ IMPORTANT: Generates CSP nonce ONLY if `use_nonce: true` in CSP configuration.**

**Syntax:**

```html
{{ csp }}
```

**Example:**

```html
<script nonce="{{ csp }}">
    // Inline JavaScript code
    console.log("Script allowed with CSP nonce");
</script>

<style nonce="{{ csp }}">
    /* Inline CSS */
    body { background: #f0f0f0; }
</style>
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

**Result after preprocessing:**

```html
<script nonce="abc123xyz789">
    console.log("Script allowed");
</script>
```

**If `use_nonce: false`:**

The `{{ csp }}` tag generates an **empty string**:

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
3. **Never hardcode nonces** - always use `{{ csp }}`
4. **Add nonce** on each inline `<script>` and `<style>` tag

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
<link rel="stylesheet" href="{% static 'css/style.css' %}">
<img src="{% media user.avatar %}" alt="Avatar">
<form method="post">
    {% csrf %}
    <button type="submit">Send</button>
</form>
```

**After preprocessing (before Tera):**

```html
<link rel="stylesheet" href="{{ settings.static_url }}css/style.css">
<img src="{{ settings.media_url }}{{ user.avatar }}" alt="Avatar">
<form method="post">
    {{ csrf_input() }}
    <button type="submit">Send</button>
</form>
```

**After Tera (final HTML):**

```html
<link rel="stylesheet" href="/static/css/style.css">
<img src="/media/avatars/user123.jpg" alt="Avatar">
<form method="post">
    <input type="hidden" name="csrftoken" value="abc123...xyz">
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
| `settings.static_url` | Base URL for static files |
| `settings.media_url` | Base URL for media files |
| `csrf_token()` | Function to generate CSRF token |
| `csrf_input()` | Function to generate CSRF hidden field |
| `get_messages()` | Function to retrieve flash messages |
| `csp_nonce()` | Function to retrieve CSP nonce |

**Example:**

```html
<!-- Access settings -->
<p>Static URL: {{ settings.static_url }}</p>
<p>Media URL: {{ settings.media_url }}</p>

<!-- CSRF token -->
{{ csrf_input() }}

<!-- Flash messages -->
{% for msg in get_messages() %}
    <div class="alert">{{ msg.message }}</div>
{% endfor %}

<!-- CSP nonce (if use_nonce: true) -->
<script nonce="{{ csp_nonce() }}">
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

    <link rel="stylesheet" href="{% static 'css/style.css' %}">
    {% block extra_css %}{% endblock %}
</head>
<body>
    <header>
        <h1>My Application</h1>
        <nav>
            <a href="{% link 'index' %}">Home</a>
            <a href="{% link 'post_list' %}">Articles</a>
            <a href="{% link 'contact' %}">Contact</a>
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

    <script src="{% static 'js/app.js' %}"></script>
    {% block extra_js %}{% endblock %}
</body>
</html>
```

### Child Template (templates/posts/list.html)

```html
{% extends "base.html" %}

{% block title %}Articles{% endblock %}

{% block extra_css %}
<link rel="stylesheet" href="{% static 'css/posts.css' %}">
{% endblock %}

{% block content %}
<h2>All Articles</h2>

{% for post in posts %}
<article>
    <h3>{{ post.title }}</h3>
    <p>{{ post.content|truncate(length=200) }}</p>
    <a href="{% link 'post_detail' id=post.id %}">Read more</a>
</article>
{% endfor %}

{% if posts|length == 0 %}
<p>No articles yet.</p>
{% endif %}
{% endblock %}

{% block extra_js %}
<script src="{% static 'js/posts.js' %}"></script>
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
    {% csrf %}

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
    <a href="{% link 'register' %}">Create account</a> |
    <a href="{% link 'password_reset' %}">Forgot password?</a>
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
            <img src="{% media user.avatar %}" alt="{{ user.username }}'s avatar">
        {% else %}
            <img src="{% static 'images/default-avatar.png' %}" alt="Default avatar">
        {% endif %}

        <h2>{{ user.username }}</h2>
        <p>{{ user.bio|default(value="No biography") }}</p>
    </div>

    <div class="profile-stats">
        <p>Member since: {{ user.created_at|date(format="%m/%d/%Y") }}</p>
        <p>Published articles: {{ user.posts|length }}</p>
    </div>

    {% if user.id == current_user.id %}
        <a href="{% link 'profile_edit' %}" class="btn">Edit my profile</a>
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
        <a href="{% link 'post_detail' id=post.id %}">{{ post.title }}</a>
    </h3>

    <div class="post-meta">
        By {{ post.author.username }} on {{ post.created_at|date(format="%m/%d/%Y") }}
    </div>

    <p>{{ post.content|truncate(length=300) }}</p>

    <a href="{% link 'post_detail' id=post.id %}" class="read-more">
        Read more →
    </a>
</article>
{% endfor %}

<!-- Pagination -->
<div class="pagination">
    {% if has_previous %}
        <a href="{% link 'post_list' %}?page={{ page - 1 }}">← Previous</a>
    {% endif %}

    <span>Page {{ page }} of {{ total_pages }}</span>

    {% if has_next %}
        <a href="{% link 'post_list' %}?page={{ page + 1 }}">Next →</a>
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
    {% csrf %}

    <!-- Automatic form rendering -->
    {{ form }}

    <!-- Or manual field-by-field rendering -->
    <!--
    <div class="form-group">
        <label>{{ form.title.label }}</label>
        {{ form.title }}
        {% if form.title.errors %}
            <div class="errors">
                {% for error in form.title.errors %}
                    <span class="error">{{ error }}</span>
                {% endfor %}
            </div>
        {% endif %}
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
<script src="{% static 'js/chart.min.js' %}"></script>

<!-- Inline script (requires nonce if strict CSP) -->
<script nonce="{{ csp }}">
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

In addition to standard Tera filters, Runique can add custom filters:

### Filter `json_encode`

```html
<script nonce="{{ csp }}">
    const config = {{ config|json_encode|safe }};
</script>
```

### Filter `slugify`

```html
<a href="/posts/{{ post.title|slugify }}/">{{ post.title }}</a>
```

### Filter `markdown`

```html
<div class="content">
    {{ post.content|markdown|safe }}
</div>
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
path!("posts/<id>/", detail_post, "post_detail")

// Less good (no name)
path!("posts/<id>/", detail_post)
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

**Version:** 1.0.86 (Corrected - January 2, 2026)
**License:** MIT

*Documentation created with ❤️ by Claude for Itsuki*
