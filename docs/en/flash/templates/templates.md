# Displaying Messages in Templates

## Automatic {% messages %} Tag

The `{% messages %}` tag automatically renders all messages:

```html
{% messages %}
```

It includes the internal template `message/message.html`, which generates:

```html
{% if messages %}
    <div class="flash-messages">
        {% for message in messages %}
        <div class="message message-{{ message.level }}">
            {{ message.content }}
        </div>
        {% endfor %}
    </div>
{% endif %}
```

---

## Recommended Placement

Place `{% messages %}` in your base template, just before the main content:

```html
<!-- base.html -->
<body>
    <header>...</header>

    {% messages %}

    <main>
        {% block content %}{% endblock %}
    </main>

    <footer>...</footer>
</body>
```

---

## Custom Display

To fully customize rendering, manually loop over `messages`:

`MessageLevel` serializes as the Rust variant name (`"Success"`, `"Error"`, `"Warning"`, `"Info"`, capitalized) — compare against that exact casing, and pipe through `| lower` for a lowercase CSS class:

```html
{% if messages %}
    {% for msg in messages %}
        <div class="alert alert-{{ msg.level | lower }}" role="alert">
            <strong>
                {% if msg.level == "Success" %}✅
                {% elif msg.level == "Error" %}❌
                {% elif msg.level == "Warning" %}⚠️
                {% elif msg.level == "Info" %}ℹ️
                {% endif %}
            </strong>
            {{ msg.content }}
        </div>
    {% endfor %}
{% endif %}
```

---

## See also

| Section | Description |
| --- | --- |
| [Macros](/docs/en/flash/macros) | All flash macros |
| [Handlers](/docs/en/flash/handlers) | Usage in handlers |

## Back to summary

- [Flash Messages](/docs/en/flash)
