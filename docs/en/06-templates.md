# 🎨 Templates

## Tera Engine

Runique uses **Tera** for HTML rendering:

```rust
use runique::request_context::TemplateContext;
use tera::Context;

async fn index(template: TemplateContext) -> Response {
    let mut context = Context::new();
    context.insert("title", "Home");
    context.insert("items", &vec!["Product 1", "Product 2"]);

    template.render("index.html", &context)
}
```

---

## context! Macro

Simplified syntax:

```rust
context_update!(template => {
        "title" => "Database error",
        "inscription_form" => &form,
    });

    return template.render("inscription_form.html");

```

---

## Available Filters

### static - Static Assets

```html
<link rel="stylesheet" href='{% static "css/main.css" %}'>

<script src="{% static "js/main.js" %}"></script>

```

Generates: `/static/css/main.css`

### media - Media Files

```html
<img src='{% media "media.avif" %}' alt="Logo">
```

Generates: `/media/avatars/profile.jpg`

### csrf_field - CSRF Token

```html
<form method="post">
    {% csrf %}
    <!-- Automatically generates: -->
    <!-- <input type="hidden" name="csrf_token" value="..."> -->
</form>
Not normally required because it is natively handled by forms

```

**Note:** `{% csrf %}` is rewritten by the loader to include the CSRF fragment, so templates can keep this tag without extra setup.

### form - Form Fields

```html
{% form.signup_form %}
<!-- Or render a specific field: -->
{% form.signup_form.email %}

```

### link - URL Links

```html
<a href={% link "index" %}>Home</a>
<a href={% link "index", id="{{ id }}", name="{{ name }}"  %}>Home</a>

```

---

## Loops and Conditions

### Loops

```html
<ul>
    {% for item in items %}
        <li>{{ item.name }} - ${{ item.price }}</li>
    {% endfor %}
</ul>

<!-- With index -->
{% for i, item in items %}
    <div class="item-{{ i }}">{{ item }}</div>
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
    <p>Please log in</p>
{% endif %}

<!-- Tests -->
{% if user and user.is_active %}
    ...
{% endif %}

{% if posts | length > 0 %}
    ...
{% endif %}

```

---

## Template Inheritance

### Parent Template (base.html)

```html
<!DOCTYPE html>
<html>
<head>
    <title>{% block title %}My Site{% endblock %}</title>
    <link rel="stylesheet" href='{% static "css/main.css" %}'>
</head>
<body>
    <header>
        <h1>My App</h1>
    </header>

    {% block content %}{% endblock %}

    <footer>
        <p>&copy; 2026</p>
    </footer>
</body>
</html>
```

### Child Template (index.html)

```html
{% extends "base.html" %}

{% block title %}Home{% endblock %}

{% block content %}
    <h2>Welcome!</h2>
    <p>{{ message }}</p>
{% endblock %}
```

---

## Macros

```html
{% macro render_user(user) %}
    <div class="user-card">
        <h3>{{ user.name }}</h3>
        <p>{{ user.email }}</p>
    </div>
{% endmacro %}

<!-- Usage: -->
{% for user in users %}
    {{ render_user(user) }}
{% endfor %}
```

---

## Error Handling

```html
<!-- Display global errors -->
{% if errors %}
    <div class="alert alert-danger">
        {% for error in errors %}
            <p>{{ error }}</p>
        {% endfor %}
    </div>
{% endif %}

<!-- Form errors -->
<form method="post" action="/signup">
    {% form.signup_form %}
    <button type="submit">Sign up</button>
</form>
=> errors are already natively rendered by the fields

Otherwise

<!-- Display global errors before the form -->
{% if signup_form.errors %}
    <div class="alert alert-warning mt-3">
        <div class="alert-message">
            <ul>
                {% for field_name, error_msg in signup_form.errors %}
                    <li><strong>{{ field_name }}:</strong> {{ error_msg }}</li>
                {% endfor %}
            </ul>
        </div>
    </div>
{% endif %}

```

---

## Complete Structure

```
templates/
├── base.html              # Parent template
├── index.html             # Home
├── errors/
│   ├── 404.html
│   └── 500.html
├── auth/
│   ├── login.html
│   └── register.html
├── blog/
│   ├── list.html
│   ├── detail.html
│   └── form.html
└── includes/
    ├── header.html
    ├── footer.html
    └── navigation.html
```

---

## Next Steps

← [**Forms**](https://github.com/seb-alliot/runique/blob/main/docs/en/05-forms.md) | [**ORM & Database**](https://github.com/seb-alliot/runique/blob/main/docs/en/07-orm.md) →
