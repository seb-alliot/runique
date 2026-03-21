# Tera Tags and Filters

## Django-like Tags (syntactic sugar)

| Tag | Transformed into | Description |
| --- | ------------- | ----------- |
| `{% static "..." %}` | `{{ "..." \| static }}` | Static file URL |
| `{% media "..." %}` | `{{ "..." \| media }}` | Media file URL |
| `{% csrf %}` | `{% include "csrf/..." %}` | Hidden CSRF field |
| `{% messages %}` | `{% include "message/..." %}` | Display flash messages |
| `{% csp_nonce %}` | `{% include "csp/..." %}` | CSP nonce attribute |
| `{% link "name" %}` | `{{ link(link='name') }}` | Named route URL |
| `{% form.xxx %}` | `{{ xxx \| form \| safe }}` | Full form rendering |
| `{% form.xxx.field %}` | `{{ xxx \| form(field='field') \| safe }}` | Single field rendering |

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
| `csrf()` | Generate a CSRF field from context |
| `nonce()` | Return the CSP nonce |
| `link(link='...')` | Named URL resolution |

---

## See also

| Section | Description |
| --- | --- |
| [Key concepts](/docs/en/architecture/concepts) | `RuniqueEngine`, `Request`, `Prisme<T>` |
| [Macros](/docs/en/architecture/macros) | Context, flash, routing, error macros |
| [Middleware stack](/docs/en/architecture/middleware) | Slot order, dependency injection |
| [Request lifecycle](/docs/en/architecture/lifecycle) | Lifecycle, best practices |

## Back to summary

- [Architecture](/docs/en/architecture)
