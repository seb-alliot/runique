# Tera filters & functions

## Asset filters

| Filter | Description | Example |
|--------|-------------|---------|
| `static` | App static URL prefix | `{{ "css/main.css" \| static }}` |
| `media` | App media URL prefix | `{{ "photo.jpg" \| media }}` |

---

## Markdown filter

| Filter     | Description                                        | Example                          |
|------------|----------------------------------------------------|----------------------------------|
| `markdown` | Converts Markdown to HTML (automatically safe)     | `{{ page.content \| markdown }}` |

> Runique's preprocessor automatically injects `\| safe` — no need to add it manually.

---

## Form filter

| Filter | Description | Example |
|--------|-------------|---------|
| `form` | Full form rendering | `{{ form.my_form \| form \| safe }}` |
| `form(field='xxx')` | Single field rendering | `{{ form.my_form \| form(field='email') \| safe }}` |
| `csrf_field` | Generates a hidden CSRF input | `{{ csrf_token \| csrf_field \| safe }}` |

---

## Tera functions

| Function | Description | Example |
|----------|-------------|---------|
| `link(link='...')` | Named URL resolution | `{{ link(link='index') }}` |

## Auto-injected context variables

| Variable | Description |
|----------|-------------|
| `csrf_token` | Masked CSRF token (used by `{% csrf %}` and `\| csrf_field`) |
| `csp_nonce` | CSP nonce value for the header (used by `{% csp %}`) |
| `messages` | Request flash messages |
| `user` | Currently authenticated user (if logged in) |

---

## See also

| Section | Description |
| --- | --- |
| [Django-like tags](/docs/en/template/tags) | Syntactic sugar |
| [Tera syntax](/docs/en/template/syntax) | Inheritance, loops, conditions |

## Back to summary

- [Templates](/docs/en/template)
