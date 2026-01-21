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
template.render("index.html", &context! {
    "title" => "Home",
    "items" => vec!["A", "B", "C"],
    "user" => user_data
})
```

---

## Available Filters

### static - Static Assets

```html
<link rel="stylesheet" href="{{ 'css/main.css' | static }}">
<script src="{{ 'js/app.js' | static }}"></script>
```

Generates: `/static/css/main.css`

### media - Media Files

```html
<img src="{{ user.avatar | media }}" alt="Avatar">
```

Generates: `/media/avatars/profile.jpg`

### csrf_field - CSRF Token

```html
<form method="post">
    {{ '' | csrf_field }}
    <!-- Automatically generates: -->
    <!-- <input type="hidden" name="csrf_token" value="..."> -->
</form>
```

### form - Form Fields

```html
{{ form | form }}
<!-- Or render specific field: -->
{{ form.email | form_field }}
```

### link - URL Links

```html
<a href="{{ 'index' | link }}">Home</a>
<a href="{{ 'profile' | link('user_id=5') }}">Profile</a>
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
    <p>Please login</p>
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
    <link rel="stylesheet" href="{{ 'css/main.css' | static }}">
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
{% if form.has_error('email') %}
    <span class="error">{{ form.get_error('email') }}</span>
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

← [**Forms**](./05-forms.md) | [**ORM & Database**](./07-orm.md) →
