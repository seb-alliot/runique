# Builder Configuration

`RuniqueApp::builder(config)` is the single entry point. All middleware configuration goes through `.middleware(|m| { ... })`.

## Full example

```rust
let app = RuniqueApp::builder(config)
    .routes(url::routes())
    .with_database(db)
    .middleware(|m| {
        m.with_csp(|c| {
              c.policy(SecurityPolicy::strict())
               .with_header_security(true)
           })
         .with_allowed_hosts(|h| {
              h.enabled(true)
               .host("mysite.com")
               .host("www.mysite.com")
           })
         .with_cache(true)
    })
    .statics()
    .build()
    .await?;
```

---

## Conditional by environment

```rust
.middleware(|m| {
    m.with_csp(|c| {
          c.policy(SecurityPolicy::strict())
           .with_upgrade_insecure(!is_debug())
       })
     .with_allowed_hosts(|h| {
          h.enabled(!is_debug())  // disabled in dev, active in prod
           .host("mysite.com")
       })
})
```

> In `DEBUG=true` mode, `is_debug()` returns `true` — security guards can be conditionally disabled.

---

## See also

| Section | Description |
| --- | --- |
| [CSRF](/docs/en/middleware/csrf) | CSRF protection and exempt paths |
| [CSP & headers](/docs/en/middleware/csp) | Content Security Policy |
| [Hosts & cache](/docs/en/middleware/hosts-cache) | Host validation |
| [CORS](/docs/en/middleware/cors) | Cross-Origin Resource Sharing |
| [Open Redirect](/docs/en/middleware/open-redirect) | Open redirect protection |
| [Permissions-Policy](/docs/en/middleware/permissions-policy) | Browser API permissions |
| [Trusted Proxies](/docs/en/middleware/trusted-proxies) | Real client IP extraction |

## Back to summary

- [Middleware & Security](/docs/en/middleware)
