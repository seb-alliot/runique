# Tera filters & functions

## Asset filters

| Filter | Description | Example |
|--------|-------------|---------|
| `static` | App static URL prefix | `{{ "css/main.css" \| static }}` |
| `media` | App media URL prefix | `{{ "photo.jpg" \| media }}` |
| `runique_static` | Framework internal static assets | `{{ "css/error.css" \| runique_static }}` |
| `runique_media` | Framework internal media | `{{ "logo.png" \| runique_media }}` |

---

## Form filter

| Filter | Description | Example |
|--------|-------------|---------|
| `form` | Full form rendering | `{{ my_form \| form \| safe }}` |
| `form(field='xxx')` | Single field rendering | `{{ my_form \| form(field='email') \| safe }}` |
| `csrf_field` | Generates a hidden CSRF input | `{{ csrf_token \| csrf_field \| safe }}` |

---

## Tera functions

| Function | Description | Example |
|----------|-------------|---------|
| `csrf()` | Generates a CSRF field from context | `{{ csrf() }}` |
| `nonce()` | Returns the CSP nonce | `{{ nonce() }}` |
| `link(link='...')` | Named URL resolution | `{{ link(link='index') }}` |

---

## See also

| Section | Description |
| --- | --- |
| [Django-like tags](https://github.com/seb-alliot/runique/blob/main/docs/en/template/tags/tags.md) | Syntactic sugar |
| [Tera syntax](https://github.com/seb-alliot/runique/blob/main/docs/en/template/syntax/syntax.md) | Inheritance, loops, conditions |

## Back to summary

- [Templates](https://github.com/seb-alliot/runique/blob/main/docs/en/template/06-templates.md)
