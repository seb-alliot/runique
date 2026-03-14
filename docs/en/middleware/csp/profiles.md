# CSP Profiles

Runique provides three built-in profiles. The active profile is set in the application builder.

---

## Profile comparison

| Directive | `default()` | `strict()` | `permissive()` |
|-----------|:-----------:|:----------:|:--------------:|
| `default-src` | `'self'` | `'self'` | `'self'` |
| `script-src` | `'self'` + nonce | `'self'` + nonce | `'self'` + `'unsafe-inline'` + `'unsafe-eval'` |
| `style-src` | `'self'` + nonce | `'self'` + nonce | `'self'` + `'unsafe-inline'` |
| `img-src` | `'self'` | `'self'` | `'self'` + `data:` + `https:` |
| `font-src` | `'self'` | `'self'` | `'self'` + `data:` |
| `connect-src` | `'self'` | `'self'` | `'self'` |
| `frame-ancestors` | `'none'` | `'none'` | `'self'` |
| `base-uri` | `'self'` | `'self'` | `'self'` |
| `form-action` | `'self'` | `'self'` | `'self'` |
| Nonce | ✅ active | ✅ active | ❌ disabled |

---

## `SecurityPolicy::default()`

Recommended policy for production. All inline scripts and styles are allowed **only via nonce**. No external images or fonts.

```rust
// Default behavior — no configuration needed
RuniqueApp::new()
    .build()
    .await?;
```

Each directive can be overridden via env vars without touching the code. See [Directives & env vars](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/csp/directives.md).

---

## `SecurityPolicy::strict()`

Identical to `default()`. Use explicitly to signal strict policy intent in the code.

```rust
RuniqueApp::new()
    .with_security_csp(SecurityPolicy::strict())
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

```rust
RuniqueApp::new()
    .with_security_csp(SecurityPolicy::permissive())
    .build()
    .await?;
```

---

## Custom policy

For a fully custom policy, build a `SecurityPolicy` manually:

```rust
use runique::middleware::SecurityPolicy;

let policy = SecurityPolicy {
    script_src: vec!["'self'".into(), "https://cdn.example.com".into()],
    img_src: vec!["'self'".into(), "data:".into()],
    ..SecurityPolicy::default()
};

RuniqueApp::new()
    .with_security_csp(policy)
    .build()
    .await?;
```

Unspecified directives inherit values from `default()`.

---

## Back

- [CSP — Overview](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/csp/csp.md)
