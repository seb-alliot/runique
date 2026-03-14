# CSP Profiles

Runique provides three built-in profiles, usable via `.policy(...)` in the builder.

---

## Profile comparison

| Directive | `default()` | `strict()` | `permissive()` |
| --- | :-----------: | :----------: | :--------------: |
| `default-src` | `'self'` | `'self'` | `'self'` |
| `script-src` | `'self'` + nonce | `'self'` + nonce | `'self'` + `'unsafe-inline'` + `'unsafe-eval'` |
| `style-src` | `'self'` + nonce | `'self'` + nonce | `'self'` + `'unsafe-inline'` |
| `img-src` | `'self'` | `'self'` | `'self'` + `data:` + `https:` |
| `font-src` | `'self'` | `'self'` | `'self'` + `data:` |
| `object-src` | `'none'` | `'none'` | `'self'` |
| `media-src` | `'self'` | `'self'` | `'self'` + `https:` |
| `frame-src` | `'none'` | `'none'` | `'self'` |
| `connect-src` | `'self'` | `'self'` | `'self'` |
| `frame-ancestors` | `'none'` | `'none'` | `'self'` |
| `base-uri` | `'self'` | `'self'` | `'self'` |
| `form-action` | `'self'` | `'self'` | `'self'` |
| `upgrade-insecure-requests` | ❌ | ✅ | ❌ |
| Nonce | ✅ active | ✅ active | ❌ disabled |

---

## `SecurityPolicy::default()`

Recommended policy for production. All inline scripts and styles are allowed **only via nonce**. No external images or fonts.

```rust,ignore
RuniqueApp::new()
    .middleware(|m| {
        m.with_csp(|c| c)
    })
    .build()
    .await?;
```

---

## `SecurityPolicy::strict()`

More restrictive than `default()`: adds `upgrade-insecure-requests` and enforces the nonce. Use in production for maximum security.

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

## `SecurityPolicy::permissive()`

Relaxed policy for development or legacy integrations. **Do not use in production.**

- `'unsafe-inline'` and `'unsafe-eval'` enabled → CSP no longer protects against XSS
- Nonce disabled
- `data:` and `https:` allowed for images and fonts
- `frame-ancestors 'self'` instead of `'none'`

```rust,ignore
RuniqueApp::new()
    .middleware(|m| {
        m.with_csp(|c| {
            c.policy(SecurityPolicy::permissive())
        })
    })
    .build()
    .await?;
```

---

## Custom policy

For a fully custom policy, use the builder methods directly:

```rust,ignore
RuniqueApp::new()
    .middleware(|m| {
        m.with_csp(|c| {
            c.scripts(vec!["'self'", "https://cdn.example.com"])
             .images(vec!["'self'", "data:"])
             .with_nonce(true)
        })
    })
    .build()
    .await?;
```

Or build a `SecurityPolicy` manually for advanced cases:

```rust,ignore
use runique::middleware::SecurityPolicy;

let policy = SecurityPolicy {
    script_src: vec!["'self'".into(), "https://cdn.example.com".into()],
    img_src: vec!["'self'".into(), "data:".into()],
    ..SecurityPolicy::default()
};

RuniqueApp::new()
    .middleware(|m| {
        m.with_csp(|c| c.policy(policy))
    })
    .build()
    .await?;
```

---

## Back

- [CSP — Overview](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/csp/csp.md)
