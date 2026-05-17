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

## Exempt paths (webhooks, APIs)

Some endpoints receive POST requests without a CSRF token — Stripe webhooks, third-party callbacks, JSON APIs called by other servers.
Use `.csrf_exempt()` to bypass CSRF validation on specific paths:

```rust
.middleware(|m| {
    m.with_csrf()
     .csrf_exempt(vec!["/webhook/stripe", "/api/callback"])
})
```

Matching is **exact** — `/webhook/stripe` does not exempt `/webhook/stripe/sub`.

> After exempting a path, verify the request authenticity by other means in your handler
> (e.g. `Stripe-Signature` HMAC-SHA256 for Stripe webhooks).

---

## See also

| Section | Description |
| --- | --- |
| [CSP & headers](/docs/en/middleware/csp) | Content Security Policy |
| [Builder](/docs/en/middleware/builder) | Builder configuration |

## Back to summary

- [Middleware & Security](/docs/en/middleware)
