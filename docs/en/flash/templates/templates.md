# Displaying Messages in Templates

## Automatic {% messages %} Tag

The `{% messages %}` tag automatically renders all messages:

```html
{% messages %}
```

It includes the internal template `message/message_include.html`, which generates:

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

```html
{% if messages %}
    {% for msg in messages %}
        <div class="alert alert-{{ msg.level }}" role="alert">
            <strong>
                {% if msg.level == "success" %}✅
                {% elif msg.level == "error" %}❌
                {% elif msg.level == "warning" %}⚠️
                {% elif msg.level == "info" %}ℹ️
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
| [Macros](https://github.com/seb-alliot/runique/blob/main/docs/en/flash/macros/macros.md) | All flash macros |
| [Handlers](https://github.com/seb-alliot/runique/blob/main/docs/en/flash/handlers/handlers.md) | Usage in handlers |

## Back to summary

- [Flash Messages](https://github.com/seb-alliot/runique/blob/main/docs/en/flash/09-flash-messages.md)
