# CSRF Protection

## How it works

- Token generated **automatically** for each session
- **Session-side synchronizer token** pattern: the token is stored server-side in the session, exposed only through a hidden field or the `X-CSRF-Token` header — no separate JS-readable cookie (the session cookie itself is `HttpOnly`)
- Verified on POST, PUT, PATCH, DELETE requests
- Ignored on GET, HEAD, OPTIONS requests

---

## In Runique forms

A form built with `RuniqueForm` already carries the token. `{% form.xxx %}` places it on the page. There is no need to add `{% csrf %}`.

## Where the token is checked

`{% csrf %}` only places the token on the form, in a hidden `csrf_token` field. It checks nothing.

The check happens in `request.form()`. Without a valid token, the form is invalid and `save()` does not run. That is the path of every `RuniqueForm`.

The middleware reads the `X-CSRF-Token` header. A JSON body without that header is refused (403). For `application/x-www-form-urlencoded` and `multipart/form-data`, it lets the body through: the field is read by `request.form()`.

A handler that reads the body and writes without `request.form()` does not get that check, even when `{% csrf %}` is on the page. The session cookie is `SameSite=Strict`.

---

## In manual HTML forms

The tag places the token. The handler has to call `request.form()` to check it.

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

Runique **adds this script itself**: `request.render()` inserts a single tag before `</body>`, with the request's CSP nonce, into every page that carries a CSRF field, whatever wrote it (a Runique form, `{% csrf %}`, the `csrf_field` filter or the `csrf_token` function). A page with several forms gets one tag. Nothing is added to a page without a CSRF field, to a page already loading `js/csrf.js`, or to a fragment without `</body>` (HTMX response).

Only a page **without a form** that sends `fetch` POSTs, or an HTML response built without `request.render()`, has to include it itself:

```html
<script {% csp %} src="{% runique_static "js/csrf.js" %}" defer></script>
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

**A POST action with no form** (sign-out, a "delete" button) checks the token itself, computed when `Request` is extracted:

```rust
pub async fn logout_view(request: Request) -> AppResult<Response> {
    if !request.prisme.csrf_valid {
        return Ok(StatusCode::FORBIDDEN.into_response());
    }
    logout(&request.session, None).await?;
    Ok(Redirect::to("/").into_response())
}
```

A native Axum extractor (`Form<T>`, `Json<T>` without the header, `Multipart`) doesn't go through that check: a route using one to write must check the token itself, or rely on refusing forms from another site (“Forms sent from another site”, below).

Matching is **exact** — `/webhook/stripe` does not exempt `/webhook/stripe/sub`.

> After exempting a path, verify the request authenticity by other means in your handler
> (e.g. `Stripe-Signature` HMAC-SHA256 for Stripe webhooks).

`csrf_exempt()` only skips *validation* — the CSRF token is still generated and available, so a handler on an exempt path can still use `Request`/`RuniqueContext` normally (session, template context, …) alongside its own verification logic.

---

## Forms sent from another site

Off by default. Turn it on if you write handlers that read their form with an Axum extractor (`Form<T>`, `Multipart`) instead of `request.form()`: nobody checks their token then.

```rust
.middleware(|m| m.with_cross_site_form_guard())
```

The middleware refuses (403) a form sent from a page of **another site** or of a **subdomain**, before the handler. It knows from the `Sec-Fetch-Site` header, set by the browser: a page can neither write nor remove it.

| Request | Effect |
| --- | --- |
| Form sent from your site (`same-origin`) | goes through, the token is checked as before |
| Form sent from another site (`cross-site`) or a subdomain (`same-site`) | **403** |
| No `Sec-Fetch-Site` header (old browser, curl, tests) | goes through, left to the token |
| With the `X-CSRF-Token` header | decided by that header, as before |
| GET, HEAD | never checked |
| Path in `csrf_exempt()` | not checked |

Runique's forms and the admin are sent from your site: they don't change. A Runique form posted from another site was already refused by its token (page shown again with the error); it now gets a 403 earlier, nothing written either way.

It doesn't replace the token: a client without the header is stopped by the token alone. A third party posting a form to you (a payment provider's return) goes in `csrf_exempt()`.

---

## See also

| Section | Description |
| --- | --- |
| [CSP & headers](/docs/en/middleware/csp) | Content Security Policy |
| [Builder](/docs/en/middleware/builder) | Builder configuration |

## Back to summary

- [Middleware & Security](/docs/en/middleware)
