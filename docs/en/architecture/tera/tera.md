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
| `runique_static` | Internal framework static assets |
| `runique_media` | Internal framework media assets |
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
| [Key concepts](https://github.com/seb-alliot/runique/blob/main/docs/en/architecture/concepts/concepts.md) | `RuniqueEngine`, `Request`, `Prisme<T>` |
| [Macros](https://github.com/seb-alliot/runique/blob/main/docs/en/architecture/macros/macros.md) | Context, flash, routing, error macros |
| [Middleware stack](https://github.com/seb-alliot/runique/blob/main/docs/en/architecture/middleware/middleware.md) | Slot order, dependency injection |
| [Request lifecycle](https://github.com/seb-alliot/runique/blob/main/docs/en/architecture/lifecycle/lifecycle.md) | Lifecycle, best practices |

## Back to summary

- [Architecture](https://github.com/seb-alliot/runique/blob/main/docs/en/architecture/02-architecture.md)
