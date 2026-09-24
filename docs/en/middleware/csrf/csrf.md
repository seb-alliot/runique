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

## Token expiry & automatic refresh

The CSRF token lives in the session. **Anonymous** sessions (visitor not logged in) expire
by default after **5 minutes of inactivity** (`with_anonymous_session_duration`, see
[Sessions](/docs/en/middleware/sessions)) — a tab left open longer on a public form
(`/login`, `/register`, etc.) then carries a stale token by the time it's submitted.

Runique handles this at two levels:

- **Explicit message**: a CSRF failure sets a global error on the form (i18n key
  `csrf.invalid_or_missing`), rendered via `{% messages %}` — never a silent failure.
- **Refresh before submission**: `js/csrf.js` intercepts every submission of a form
  carrying a `csrf_token` field, does a lightweight GET to the current page to fetch a
  fresh token (every response carries an `X-CSRF-Token` header), updates the field, then
  submits — works even if the session fully expired (the server just issues a new one).
  The user never loses what they typed, and only sees the CSRF failure if the network is
  down at the moment of submission.

This script is **not loaded automatically** in an application's templates — include it
explicitly in your base layout:

```html
<script {% csp %} src="{{ "js/csrf.js" | runique_static }}" defer></script>
```

(the admin panel already loads it itself, via `runique_static`).

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

`csrf_exempt()` only skips *validation* — the CSRF token is still generated and available, so a handler on an exempt path can still use `Request`/`RuniqueContext` normally (session, template context, …) alongside its own verification logic.

---

## See also

| Section | Description |
| --- | --- |
| [CSP & headers](/docs/en/middleware/csp) | Content Security Policy |
| [Builder](/docs/en/middleware/builder) | Builder configuration |

## Back to summary

- [Middleware & Security](/docs/en/middleware)
