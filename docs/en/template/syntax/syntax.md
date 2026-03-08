# Tera syntax

## Template inheritance

### Parent template (base.html)

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

### Child template (index.html)

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

## Loops

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

---

## Conditions

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
    <p>{{ posts | length }} posts found.</p>
{% endif %}
```

---

## Tera macros (inside templates)

```html
{% macro render_user(user) %}
    <div class="user-card">
        <h3>{{ user.name }}</h3>
        <p>{{ user.email }}</p>
    </div>
{% endmacro %}

<!-- Usage: -->
{% for u in users %}
    {{ self::render_user(user=u) }}
{% endfor %}
```

---

## `context_update!` macro

```rust
context_update!(request => {
    "title" => "My page",
    "user" => &form,
    "items" => &vec!["a", "b", "c"],
});

request.render("my_page.html")
```

---

## See also

| Section | Description |
| --- | --- |
| [Django-like tags](https://github.com/seb-alliot/runique/blob/main/docs/en/template/tags/tags.md) | Syntactic sugar |
| [Filters & functions](https://github.com/seb-alliot/runique/blob/main/docs/en/template/filters/filters.md) | Low-level filters |

## Back to summary

- [Templates](https://github.com/seb-alliot/runique/blob/main/docs/en/template/06-templates.md)
