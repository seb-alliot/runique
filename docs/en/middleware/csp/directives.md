# CSP Directives & Environment Variables

Each CSP directive is configurable via environment variable in `.env`, without modifying code. Values are comma-separated lists.

---

## Configurable directives

| CSP Directive | Env var | Default |
|---------------|---------|---------|
| `default-src` | `RUNIQUE_POLICY_CSP_DEFAULT` | `'self'` |
| `script-src` | `RUNIQUE_POLICY_CSP_SCRIPTS` | `'self'` |
| `style-src` | `RUNIQUE_POLICY_CSP_STYLES` | `'self'` |
| `img-src` | `RUNIQUE_POLICY_CSP_IMAGES` | `'self'` |
| `font-src` | `RUNIQUE_POLICY_CSP_FONTS` | `'self'` |
| `object-src` | `RUNIQUE_POLICY_CSP_OBJECTS` | `'none'` |
| `media-src` | `RUNIQUE_POLICY_CSP_MEDIA` | `'self'` |
| `frame-src` | `RUNIQUE_POLICY_CSP_FRAMES` | `'none'` |
| Nonce active | `RUNIQUE_POLICY_CSP_STRICT_NONCE` | `true` |

> The `connect-src`, `frame-ancestors`, `base-uri` and `form-action` directives are not yet overridable via env vars. Use a custom `SecurityPolicy` if needed.

---

## Common examples

### Allow a CDN for scripts

```env
RUNIQUE_POLICY_CSP_SCRIPTS='self',https://cdn.jsdelivr.net
```

### Allow inline base64 images (avatars, rich-text editors)

```env
RUNIQUE_POLICY_CSP_IMAGES='self',data:
```

### Allow Google Fonts

```env
RUNIQUE_POLICY_CSP_FONTS='self',https://fonts.gstatic.com
RUNIQUE_POLICY_CSP_STYLES='self',https://fonts.googleapis.com
```

### Allow iframes from same domain

```env
RUNIQUE_POLICY_CSP_FRAMES='self'
```

### Allow embedded objects (Flash plugins, etc.)

```env
RUNIQUE_POLICY_CSP_OBJECTS='self'
```

### Allow media from a CDN

```env
RUNIQUE_POLICY_CSP_MEDIA='self',https://cdn.example.com
```

---

## Nonce behavior on `script-src` and `style-src`

When the nonce is active (`RUNIQUE_POLICY_CSP_STRICT_NONCE=true`, default):

- `'nonce-{value}'` is automatically appended to `script-src` and `style-src`
- `'unsafe-inline'` is **automatically removed** from those directives if present

This ensures inline scripts without a nonce are blocked, even if `'unsafe-inline'` was manually configured.

```
# Generated header with active nonce:
Content-Security-Policy: default-src 'self'; script-src 'self' 'nonce-abc123'; ...
```

---

## Fixed directives (not configurable via env)

These directives are defined in the profile and cannot be changed via env var:

| Directive | Value (default/strict) | Role |
|-----------|----------------------|------|
| `connect-src` | `'self'` | Restricts XHR/fetch/WebSocket connections |
| `frame-ancestors` | `'none'` | Prevents embedding in iframes (clickjacking) |
| `base-uri` | `'self'` | Prevents `<base>` tag injection |
| `form-action` | `'self'` | Prevents form submissions to external domains |

To override them, use a custom `SecurityPolicy`:

```rust
use runique::middleware::SecurityPolicy;

RuniqueApp::new()
    .with_security_csp(SecurityPolicy {
        connect_src: vec!["'self'".into(), "wss://ws.example.com".into()],
        frame_ancestors: vec!["'self'".into()],
        ..SecurityPolicy::default()
    })
    .build()
    .await?;
```

---

## Back

- [CSP — Overview](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/csp/csp.md)
