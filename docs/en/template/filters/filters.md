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
| `markdown` | Converts Markdown to HTML, sanitized (XSS-safe)    | `{{ page.content \| markdown }}` |

> Runique's preprocessor automatically injects `\| safe` — no need to add it manually.
>
> The output is **sanitized via ammonia**: dangerous raw HTML (`<script>`, `on*` handlers) and `javascript:` / `data:` URLs in links and images are stripped. Legitimate Markdown (headings, tables, lists, links, images, code) is preserved — user-authored Markdown can therefore be rendered safely.

---

## Sanitize & plaintext filters

| Filter      | Description                                              | Example                            |
|-------------|---------------------------------------------------------|------------------------------------|
| `sanitize`  | Re-sanitizes stored rich HTML and renders it as HTML    | `{{ entry.description \| sanitize }}` |
| `plaintext` | Strips all tags + decodes entities → plain-text preview | `{{ entry.description \| plaintext }}` |

> `sanitize` runs **ammonia at render time**; the preprocessor injects `\| safe` automatically (like `markdown`), so the emitted HTML is always freshly cleaned — sanitization happens on **output**, never trusting what is stored. Use it to display a rich-text field as rendered HTML.
>
> `plaintext` projects a value to plain text via the strict sanitizer (tags removed, entities decoded). It stays **auto-escaped** (no `\| safe`), so a stored `&gt;` is shown as `>`. Use it for previews — e.g. list cells — where rendered block HTML would break the layout.
>
> The admin detail/list views use these automatically for columns classified as rich content; you rarely call them by hand.

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
| `current_path` | Current URL path, without query string (useful for `rel="canonical"`, `og:url`, active navigation) |

---

## See also

| Section | Description |
| --- | --- |
| [Django-like tags](/docs/en/template/tags) | Syntactic sugar |
| [Tera syntax](/docs/en/template/syntax) | Inheritance, loops, conditions |

## Back to summary

- [Templates](/docs/en/template)
