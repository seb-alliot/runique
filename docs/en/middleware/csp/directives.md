# CSP Directives

Each CSP directive is configurable via the builder — environment variables are no longer used.

---

## Available directives

| Builder method | CSP Directive | Default |
| --- | --- | --- |
| `.default_src(vec![...])` | `default-src` | `'self'` |
| `.scripts(vec![...])` | `script-src` | `'self'` |
| `.styles(vec![...])` | `style-src` | `'self'` |
| `.images(vec![...])` | `img-src` | `'self'` |
| `.fonts(vec![...])` | `font-src` | `'self'` |
| `.connect(vec![...])` | `connect-src` | `'self'` |
| `.objects(vec![...])` | `object-src` | `'none'` |
| `.media(vec![...])` | `media-src` | `'self'` |
| `.frames(vec![...])` | `frame-src` | `'none'` |
| `.frame_ancestors(vec![...])` | `frame-ancestors` | `'none'` |
| `.base_uri(vec![...])` | `base-uri` | `'self'` |
| `.form_action(vec![...])` | `form-action` | `'self'` |

### Toggles

| Builder method | Default | Description |
| --- | --- | --- |
| `.with_nonce(bool)` | `true` | Per-request nonce injected into `script-src` and `style-src` |
| `.with_header_security(bool)` | `false` | HSTS, X-Frame-Options, COEP, COOP, CORP… |
| `.with_upgrade_insecure(bool)` | `false` | `upgrade-insecure-requests` |

### Presets

| Builder method | Description |
| --- | --- |
| `.policy(SecurityPolicy::default())` | Default policy — `'self'` everywhere, nonce active |
| `.policy(SecurityPolicy::strict())` | Strict — mandatory nonce, `upgrade-insecure-requests`, `frame-ancestors 'none'` |
| `.policy(SecurityPolicy::permissive())` | Permissive — `unsafe-eval` allowed, images from `https:` |

---

## Common examples

### Minimal — CSP enabled with no customization

```rust,ignore
RuniqueApp::new()
    .middleware(|m| {
        m.with_csp(|c| c)
    })
    .build()
    .await?;
```

### CDN for scripts and styles (e.g. Bootstrap)

```rust,ignore
RuniqueApp::new()
    .middleware(|m| {
        m.with_csp(|c| {
            c.scripts(vec!["'self'", "https://cdn.jsdelivr.net"])
             .styles(vec!["'self'", "https://cdn.jsdelivr.net"])
        })
    })
    .build()
    .await?;
```

### Google Fonts + base64 images

```rust,ignore
RuniqueApp::new()
    .middleware(|m| {
        m.with_csp(|c| {
            c.fonts(vec!["'self'", "https://fonts.gstatic.com"])
             .styles(vec!["'self'", "https://fonts.googleapis.com"])
             .images(vec!["'self'", "data:"])
        })
    })
    .build()
    .await?;
```

### WebSocket + iframes

```rust,ignore
RuniqueApp::new()
    .middleware(|m| {
        m.with_csp(|c| {
            c.connect(vec!["'self'", "wss://ws.example.com"])
             .frames(vec!["'self'"])
             .frame_ancestors(vec!["'self'"])
        })
    })
    .build()
    .await?;
```

### Full configuration (production)

```rust,ignore
RuniqueApp::new()
    .middleware(|m| {
        m.with_csp(|c| {
            c.with_header_security(true)
             .with_nonce(true)
             .with_upgrade_insecure(true)
             .scripts(vec!["'self'", "https://cdn.jsdelivr.net"])
             .styles(vec!["'self'", "https://cdn.jsdelivr.net", "https://fonts.googleapis.com"])
             .fonts(vec!["'self'", "https://fonts.gstatic.com"])
             .images(vec!["'self'", "data:", "https://cdn.example.com"])
             .connect(vec!["'self'", "wss://ws.example.com"])
        })
    })
    .build()
    .await?;
```

### Strict preset with security headers

```rust,ignore
RuniqueApp::new()
    .middleware(|m| {
        m.with_csp(|c| {
            c.policy(SecurityPolicy::strict())
             .with_header_security(true)
        })
    })
    .build()
    .await?;
```

---

## Nonce behavior on `script-src` and `style-src`

When the nonce is active (`.with_nonce(true)`):

- `'nonce-{value}'` is automatically appended to `script-src` and `style-src`
- `'unsafe-inline'` is **automatically removed** from those directives if present

This ensures inline scripts without a nonce are blocked, even if `'unsafe-inline'` was manually configured.

```text
# Generated header with active nonce:
Content-Security-Policy: default-src 'self'; script-src 'self' 'nonce-abc123'; style-src 'self' 'nonce-abc123'; ...
```

---

## Back

- [CSP — Overview](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/csp/csp.md)
