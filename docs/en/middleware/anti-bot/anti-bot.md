# Anti-Bot Honeypot

Runique provides a honeypot middleware that automatically protects forms against simple bots — without any modification to your handlers.

## How it works

1. On first visit, the middleware generates a random 16-character hex field name and stores it in the session.
2. On every request, the name is injected as an Axum extension (`HoneypotFieldName`).
3. `Request::form()` reads it and adds the field to the rendered HTML (invisible via `hp.css`).
4. On POST, if the field is non-empty → `force_invalid = true` → `form.is_valid()` returns `false` immediately.

The handler sees a normal form validation failure — no special case needed.

## Activation

```rust
.middleware(|m| m.with_anti_bot())
```

That is the only change required in `main.rs`.

## Template rendering

The honeypot field is injected automatically:

- **`{{ form | form }}`** (full render): honeypot appended after the last field
- **Field by field** (`{{ form | form(field="name") }}`): honeypot appended after the last field

The `honeypot_html` key is also available directly in the Tera context if you need to place it manually:

```html
{{ form.honeypot_html | safe }}
```

## Security properties

| Property | Value |
| --- | --- |
| Field name | Random hex-16, session-bound |
| Rotation | New name per session (persists across GET/POST) |
| Visibility | Hidden via external CSS (`hp.css`) — CSP-safe, no inline style |
| Bot resistance | Blocks form-fillers that fill all fields unconditionally |
| Human impact | None — the field is invisible and ignored by browsers |

The field name has no recognizable prefix — a bot cannot skip it by pattern matching.

## Local testing constraint

The session cookie uses `Secure=true` when `DEBUG=false`. On `http://localhost`, browsers refuse to send a Secure cookie, which means the honeypot field name is regenerated on every request and the trap never triggers.

**Test locally with `DEBUG=true`.** On the VPS over HTTPS, the middleware works correctly without any override.

## Slot

`65` — between CSRF (60) and HostValidation (70). Requires the session middleware (slot 50) to be active.

## Back to summary

- [Middleware & Security](/docs/en/middleware)
