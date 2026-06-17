# Tera Tags and Filters

## Django-like Tags (syntactic sugar)

| Tag | Transformed into | Description |
| --- | ------------- | ----------- |
| `{% static "..." %}` | `{{ "..." \| static }}` | Static file URL |
| `{% media "..." %}` | `{{ "..." \| media }}` | Media file URL |
| `{% csrf %}` | `{% include "csrf" %}` | Hidden CSRF field |
| `{% messages %}` | `{% include "message" %}` | Display flash messages |
| `{% csp %}` | `{% include "csp" %}` | CSP nonce attribute |
| `{% link "name" %}` | `{{ link(link='name') }}` | Named route URL |
| `{% form.xxx %}` | `{{ xxx \| form \| safe }}` | Full form rendering |
| `{% form.xxx.field %}` | `{{ xxx \| form(field='field') \| safe }}` | Single field rendering |
| `{% form.xxx.js %}` | `{{ xxx \| form(field='js') \| safe }}` | Form JS scripts (field-by-field rendering) |

---

## Tera Filters

| Filter | Description |
| ------ | ----------- |
| `static` | App static URL prefix |
| `media` | App media URL prefix |
| `form` | Render full form or specific field |
| `csrf_field` | Generate a hidden CSRF input |

---

## Tera Functions

| Function | Description |
| -------- | ----------- |
| `link(link='...')` | Named URL resolution |

---

## In context

The tags combine in a real template:

```html
{% extends "base.html" %}

{% block content %}
  <link rel="stylesheet" href="{% static "css/contact.css" %}">

  {% messages %}

  <form method="post" action="{% link "contact" %}">
    {% form.contact_form %}
    <button type="submit">Send</button>
  </form>

  <img src="{% media avatar %}" alt="avatar">
{% endblock %}
```

> `{% static %}` / `{% media %}` accept a string literal or a Tera variable (`avatar` above).
> See [Django-like tags](/docs/en/template/tags) for the details of each tag.

---

## See also

| Section | Description |
| --- | --- |
| [Key concepts](/docs/en/architecture/concepts) | `RuniqueEngine`, `Request`, `request.form()` |
| [Macros](/docs/en/architecture/macros) | Context, flash, routing, error macros |
| [Middleware stack](/docs/en/architecture/middleware) | Slot order, dependency injection |
| [Request lifecycle](/docs/en/architecture/lifecycle) | Lifecycle, best practices |

## Back to summary

- [Architecture](/docs/en/architecture)
