# CSRF Protection

## How it works

- Token generated **automatically** for each session
- **Double Submit Cookie** pattern (cookie + hidden field)
- Verified on POST, PUT, PATCH, DELETE requests
- Ignored on GET, HEAD, OPTIONS requests

---

## In Runique forms

When you use `{% form.xxx %}`, CSRF is **included automatically**. No need to add it manually.

---

## In manual HTML forms

```html
<form method="post" action="/submit">
    {% csrf %}
    <input type="text" name="data">
    <button type="submit">Send</button>
</form>
```

---

## For AJAX requests

```javascript
const csrfToken = document.querySelector('[name="csrf_token"]').value;

fetch('/api/endpoint', {
    method: 'POST',
    headers: {
        'Content-Type': 'application/json',
        'X-CSRF-Token': csrfToken
    },
    body: JSON.stringify(data)
});
```

---

## See also

| Section | Description |
| --- | --- |
| [CSP & headers](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/csp/csp.md) | Content Security Policy |
| [Builder](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/builder/builder.md) | Builder configuration |

## Back to summary

- [Middleware & Security](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/08-middleware.md)
