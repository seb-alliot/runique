# Permissions-Policy

## What it does

Controls which browser APIs are available to the page and any embedded frames.
Sent as the `Permissions-Policy` HTTP response header.

Runique adds this header automatically on every response. The default is a secure preset
that denies sensitive APIs. Override individual directives via the builder.

---

## Default preset

**Denied (all origins):**

| Feature | Category |
| --- | --- |
| `accelerometer` | Sensor |
| `ambient-light-sensor` | Sensor |
| `bluetooth` | Hardware |
| `camera` | Hardware |
| `gyroscope` | Sensor |
| `hid` | Hardware |
| `magnetometer` | Sensor |
| `microphone` | Hardware |
| `midi` | Hardware |
| `serial` | Hardware |
| `usb` | Hardware |
| `geolocation` | Location |
| `idle-detection` | Privacy |
| `display-capture` | Screen capture |
| `payment` | Payments |
| `interest-cohort` | Fingerprinting (disables FLoC) |
| `local-fonts` | Fingerprinting |
| `sync-xhr` | Legacy / deprecated |
| `xr-spatial-tracking` | XR |
| `window-management` | Multi-window |

**Allowed for same origin (`(self)`):**

| Feature | Notes |
| --- | --- |
| `fullscreen` | Standard UX need |
| `picture-in-picture` | Standard UX need |
| `publickey-credentials-create` | WebAuthn / passkeys |
| `publickey-credentials-get` | WebAuthn / passkeys |

---

## Configuration via the builder

```rust
.middleware(|m| {
    m.with_permissions_policy(|p| {
        p.deny("geolocation")
         .allow_self("fullscreen")
         .allow("payment", vec!["https://pay.example.com"])
    })
})
```

---

## Available methods

| Method | Header value | Description |
| --- | --- | --- |
| `.deny("feature")` | `feature=()` | Deny all origins |
| `.allow_self("feature")` | `feature=(self)` | Same origin only |
| `.allow_any("feature")` | `feature=*` | Any origin |
| `.allow("feature", vec!["https://…"])` | `feature=("url1" "url2")` | Specific origins |

Methods override the default for that directive. Directives not mentioned keep their default value.

---

## Keeping the default

Do not call `.with_permissions_policy` — the secure default applies automatically.

---

## See also

| Section | Description |
| --- | --- |
| [CSP & headers](/docs/en/middleware/csp) | Content Security Policy |
| [Builder](/docs/en/middleware/builder) | Builder configuration |

## Back to summary

- [Middleware & Security](/docs/en/middleware)
