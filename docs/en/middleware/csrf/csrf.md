# CSRF Protection

## How it works

- Token generated **automatically** for each session
- **Session-side synchronizer token** pattern: the token is stored server-side in the session, exposed only through a hidden field or the `X-CSRF-Token` header — no separate JS-readable cookie (the session cookie itself is `HttpOnly`)
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
    m.csrf_exempt(vec!["/webhook/stripe", "/api/callback"])
})
```

CSRF is always on — there is no `.with_csrf()` method to call, only the exemption exists.

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
