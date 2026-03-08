# Builder Configuration

`RuniqueApp::builder(config)` is the single entry point. All middleware configuration goes through `.middleware(|m| { ... })`.

## Full example

```rust
let app = RuniqueApp::builder(config)
    .routes(url::routes())
    .with_database(db)
    .with_error_handler(true)
    .middleware(|m| {
        m.with_csp(true)               // CSP & security headers
         .with_host_validation(true)   // Host validation
         .with_cache(true)             // No-cache in dev
    })
    .statics()
    .build()
    .await?;
```

---

## Customizing middlewares

```rust
let app = RuniqueApp::builder(config)
    .routes(url::routes())
    .with_database(db)
    .middleware(|m| {
        m.with_csp(false)              // Disable CSP
         .with_host_validation(false)  // Disable host validation
    })
    .build()
    .await?;
```

> In `DEBUG=true` mode, `with_csp` and `with_host_validation` are already disabled by default.

---

## See also

| Section | Description |
| --- | --- |
| [CSRF](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/csrf/csrf.md) | CSRF protection |
| [CSP & headers](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/csp/csp.md) | Content Security Policy |
| [Hosts & cache](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/hosts-cache/hosts-cache.md) | Host validation |

## Back to summary

- [Middleware & Security](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/08-middleware.md)
