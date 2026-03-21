# CSRF in the admin

## What `admin_template.html` guarantees automatically

The `admin_template.html` file writes several elements **outside blocks**, meaning they are present on every admin page regardless of any applied override.

| Element | Location in HTML | Role |
|---|---|---|
| `<meta name="csrf-token" content="{{ csrf_token }}">` | in `<head>` | Exposes the CSRF token for JavaScript |
| `<script src="…/csrf.js" defer></script>` | before `</body>` | Automatic AJAX interceptor |
| `{% block messages %}` area | in the body | Flash message display (CSRF errors included) |

These elements **cannot be removed** by overriding a block. They are active on every page that inherits from the contract.

## The CSRF token

### Properties

- **Stable per session**: the token does not change between requests within the same session (synchronizer token pattern). This avoids issues with multiple tabs and the back button.
- **Masked in responses**: each response returns the token in a different masked form (XOR + random encoding), protecting against the BREACH attack (compression + TLS encryption).
- **Validated only on mutating methods**: `POST`, `PUT`, `PATCH`, `DELETE`. `GET` and `HEAD` requests are not checked.

### Validation flow

```
POST request
  └─ CSRF middleware
       ├─ reads X-CSRF-Token (header) or _csrf_token (form field)
       ├─ unmasks the received value
       ├─ compares with the stable session token
       └─ match? → continue | no → 403 Forbidden
```

## What `csrf.js` does

The `csrf.js` script is loaded automatically on every admin page. It intercepts native `fetch()` calls and adds the `X-CSRF-Token` header:

```javascript
// Internal behaviour of csrf.js (simplified)
const original = window.fetch;
window.fetch = function(url, options = {}) {
    const token = document.querySelector('meta[name="csrf-token"]')?.content;
    if (token && ["POST", "PUT", "PATCH", "DELETE"].includes((options.method || "GET").toUpperCase())) {
        options.headers = { ...options.headers, "X-CSRF-Token": token };
    }
    return original(url, options);
};
```

This covers all AJAX calls with no manual changes in frontend code.

## The `{% csrf %}` tag for HTML forms

For classic HTML forms (not managed via `form_fields`), include the Tera tag `{% csrf %}` inside the `<form>`:

```html
<form method="POST" action="/admin/login">
    {% csrf %}
    <input type="text" name="username">
    <input type="password" name="password">
    <button type="submit">Login</button>
</form>
```

`{% csrf %}` generates a hidden field:

```html
<input type="hidden" name="_csrf_token" value="…masked token…">
```

Forms rendered via `{{ form_fields.html }}` include this field automatically — the `{% csrf %}` tag is only needed for manually written forms.

## Checklist for a custom login template

If the login template is customised (outside `admin_template.html`), the following three elements are required:

- [ ] `<meta name="csrf-token" content="{{ csrf_token }}">` in `<head>`
- [ ] `{% csrf %}` inside the `<form method="POST">`
- [ ] `<script src="{{ "js/csrf.js" | runique_static }}" defer></script>` before `</body>`

### Full example: `templates/auth/login.html`

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="csrf-token" content="{{ csrf_token }}">
    <title>Login — Admin</title>
</head>
<body>
    <form method="POST" action="/admin/login">
        {% csrf %}
        <div>
            <label for="username">Username</label>
            <input type="text" id="username" name="username" required>
        </div>
        <div>
            <label for="password">Password</label>
            <input type="password" id="password" name="password" required>
        </div>
        <button type="submit">Log in</button>
    </form>

    <script src="{{ "js/csrf.js" | runique_static }}" defer></script>
</body>
</html>
```

## What is NOT protected automatically

| Situation | Risk | Solution |
|---|---|---|
| Override of `{% block content %}` with a manual `<form>` without `{% csrf %}` | `_csrf_token` field is absent → 403 on submit | Add `{% csrf %}` inside the `<form>` |
| Template that does not extend `admin_template.html` | Neither the meta nor the `csrf.js` script are present | Add both elements manually (see checklist) |
| `fetch()` call in a script loaded before `csrf.js` | Interceptor is not yet active | Load the custom script after `csrf.js` or use `{% block extra_js %}` |

## Sub-sections

| Section | Description |
| --- | --- |
| [Context keys](/docs/en/admin/template) | Variables injected by the backend into each template |
| [Override](/docs/en/admin/template) | Replace the layout or a CRUD component |

## Back to summary

| Section | Description |
| --- | --- |
| [Template summary](/docs/en/admin/template) | Admin templates |
| [Admin summary](/docs/en/admin) | Admin |
