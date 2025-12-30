# Rusti CSP Documentation

## Content Security Policy in Rusti Framework

### Overview

Rusti provides a comprehensive Content Security Policy (CSP) implementation that helps protect your web applications against Cross-Site Scripting (XSS), clickjacking, and other code injection attacks. The CSP middleware allows you to control which resources the browser is allowed to load for your application.

### What is Content Security Policy?

Content Security Policy is a security standard that helps prevent various types of attacks by declaring which dynamic resources are allowed to load. It works by defining a whitelist of trusted sources for content such as scripts, styles, images, and other resources.

### Key Features

- **Directive-based configuration**: Fine-grained control over different resource types
- **Nonce generation**: Automatic generation of cryptographic nonces for inline scripts and styles
- **Report-only mode**: Test CSP policies without blocking resources
- **Violation reporting**: Configure endpoints to receive CSP violation reports
- **Middleware integration**: Seamless integration with Rusti's middleware system

### Configuration

#### Basic Setup

Add the CSP middleware to your application's middleware stack:

```rust
use rusti::middleware::csp::{CspMiddleware, CspConfig, CspDirective};

let csp_config = CspConfig {
    directives: vec![
        CspDirective::DefaultSrc(vec!["'self'".to_string()]),
        CspDirective::ScriptSrc(vec!["'self'".to_string(), "'unsafe-inline'".to_string()]),
        CspDirective::StyleSrc(vec!["'self'".to_string(), "'unsafe-inline'".to_string()]),
    ],
    report_only: false,
    report_uri: None,
};

let csp_middleware = CspMiddleware::new(csp_config);
```

#### Available Directives

Rusti supports all standard CSP directives:

- **default-src**: Fallback for all resource types
- **script-src**: Controls JavaScript sources
- **style-src**: Controls CSS sources
- **img-src**: Controls image sources
- **font-src**: Controls font sources
- **connect-src**: Controls fetch, XHR, WebSocket connections
- **media-src**: Controls audio and video sources
- **object-src**: Controls `<object>`, `<embed>`, and `<applet>` elements
- **frame-src**: Controls frame sources
- **worker-src**: Controls Worker, SharedWorker, and ServiceWorker sources
- **manifest-src**: Controls application manifest sources
- **base-uri**: Controls document base URL
- **form-action**: Controls form submission targets
- **frame-ancestors**: Controls which sources can embed the page

#### Using Nonces

Nonces (number used once) are cryptographic tokens that allow specific inline scripts or styles while blocking others:

```rust
let csp_config = CspConfig {
    directives: vec![
        CspDirective::ScriptSrc(vec![
            "'self'".to_string(),
            "'nonce-{nonce}'".to_string(),
        ]),
    ],
    report_only: false,
    report_uri: None,
};
```

In your templates, access the nonce:

```html
<script nonce="{{ csp_nonce }}">
    // Your inline script here
    console.log('Protected by CSP nonce');
</script>
```

### Report-Only Mode

Test your CSP policy without blocking resources:

```rust
let csp_config = CspConfig {
    directives: vec![
        CspDirective::DefaultSrc(vec!["'self'".to_string()]),
    ],
    report_only: true,  // Enable report-only mode
    report_uri: Some("/csp-violation-report".to_string()),
};
```

In report-only mode, violations are reported but resources are not blocked. This is useful for testing policies before enforcement.

### Violation Reporting

Configure an endpoint to receive CSP violation reports:

```rust
let csp_config = CspConfig {
    directives: vec![
        CspDirective::DefaultSrc(vec!["'self'".to_string()]),
    ],
    report_only: false,
    report_uri: Some("/csp-report".to_string()),
};
```

Create a view to handle violation reports:

```rust
use rusti::http::{Request, Response};

pub fn csp_report_handler(request: Request) -> Response {
    // Parse and log the violation report
    let report = request.json::<CspViolationReport>().unwrap();

    eprintln!("CSP Violation: {:?}", report);

    Response::new()
        .status(204)
}
```

### Common Patterns

#### Strict CSP for Maximum Security

```rust
let strict_csp = CspConfig {
    directives: vec![
        CspDirective::DefaultSrc(vec!["'none'".to_string()]),
        CspDirective::ScriptSrc(vec!["'self'".to_string(), "'nonce-{nonce}'".to_string()]),
        CspDirective::StyleSrc(vec!["'self'".to_string(), "'nonce-{nonce}'".to_string()]),
        CspDirective::ImgSrc(vec!["'self'".to_string(), "data:".to_string()]),
        CspDirective::FontSrc(vec!["'self'".to_string()]),
        CspDirective::ConnectSrc(vec!["'self'".to_string()]),
        CspDirective::BaseUri(vec!["'self'".to_string()]),
        CspDirective::FormAction(vec!["'self'".to_string()]),
        CspDirective::FrameAncestors(vec!["'none'".to_string()]),
    ],
    report_only: false,
    report_uri: Some("/csp-report".to_string()),
};
```

#### Relaxed CSP for Development

```rust
let dev_csp = CspConfig {
    directives: vec![
        CspDirective::DefaultSrc(vec!["'self'".to_string()]),
        CspDirective::ScriptSrc(vec![
            "'self'".to_string(),
            "'unsafe-inline'".to_string(),
            "'unsafe-eval'".to_string(),
        ]),
        CspDirective::StyleSrc(vec![
            "'self'".to_string(),
            "'unsafe-inline'".to_string(),
        ]),
    ],
    report_only: false,
    report_uri: None,
};
```

#### CDN and Third-Party Resources

```rust
let csp_with_cdn = CspConfig {
    directives: vec![
        CspDirective::DefaultSrc(vec!["'self'".to_string()]),
        CspDirective::ScriptSrc(vec![
            "'self'".to_string(),
            "https://cdn.jsdelivr.net".to_string(),
            "'nonce-{nonce}'".to_string(),
        ]),
        CspDirective::StyleSrc(vec![
            "'self'".to_string(),
            "https://cdn.jsdelivr.net".to_string(),
            "'nonce-{nonce}'".to_string(),
        ]),
        CspDirective::FontSrc(vec![
            "'self'".to_string(),
            "https://fonts.gstatic.com".to_string(),
        ]),
        CspDirective::ImgSrc(vec![
            "'self'".to_string(),
            "data:".to_string(),
            "https:".to_string(),
        ]),
    ],
    report_only: false,
    report_uri: None,
};
```

### Best Practices

1. **Start with Report-Only Mode**: Test your CSP configuration before enforcing it
2. **Use Nonces for Inline Scripts**: Prefer nonces over `unsafe-inline`
3. **Avoid `unsafe-eval`**: This weakens your CSP significantly
4. **Be Specific**: Use specific domains instead of wildcards when possible
5. **Monitor Violations**: Set up violation reporting to catch issues
6. **Regular Updates**: Review and update your CSP as your application evolves
7. **Test Thoroughly**: Check all pages and features after implementing CSP

### Debugging CSP Issues

When resources are blocked by CSP, browsers log detailed information in the console:

```
Refused to load the script 'https://example.com/script.js' because it violates
the following Content Security Policy directive: "script-src 'self'".
```

Common issues:

- **Inline scripts blocked**: Add nonces or move scripts to external files
- **CDN resources blocked**: Add the CDN domain to appropriate directives
- **eval() blocked**: Refactor code to avoid eval or add `unsafe-eval` (not recommended)
- **Inline styles blocked**: Add nonces or move styles to external files

### Migration Strategy

1. **Audit Current Resources**: Identify all scripts, styles, and external resources
2. **Deploy Report-Only**: Use report-only mode to identify violations
3. **Adjust Policy**: Update CSP based on violation reports
4. **Gradual Enforcement**: Start with strict directives and relax as needed
5. **Monitor**: Continue monitoring violations after enforcement

### Security Considerations

- CSP is a defense-in-depth measure, not a complete security solution
- Always validate and sanitize user input server-side
- Keep dependencies up to date
- Use HTTPS for all resources
- Combine CSP with other security headers (HSTS, X-Frame-Options, etc.)

### API Reference

#### CspConfig

```rust
pub struct CspConfig {
    pub directives: Vec<CspDirective>,
    pub report_only: bool,
    pub report_uri: Option<String>,
}
```

#### CspDirective

```rust
pub enum CspDirective {
    DefaultSrc(Vec<String>),
    ScriptSrc(Vec<String>),
    StyleSrc(Vec<String>),
    ImgSrc(Vec<String>),
    FontSrc(Vec<String>),
    ConnectSrc(Vec<String>),
    MediaSrc(Vec<String>),
    ObjectSrc(Vec<String>),
    FrameSrc(Vec<String>),
    WorkerSrc(Vec<String>),
    ManifestSrc(Vec<String>),
    BaseUri(Vec<String>),
    FormAction(Vec<String>),
    FrameAncestors(Vec<String>),
}
```

#### CspMiddleware

```rust
impl CspMiddleware {
    pub fn new(config: CspConfig) -> Self
    pub fn generate_nonce() -> String
}
```

### Further Reading

- [MDN Content Security Policy](https://developer.mozilla.org/en-US/docs/Web/HTTP/CSP)
- [CSP Level 3 Specification](https://www.w3.org/TR/CSP3/)
- [Google CSP Evaluator](https://csp-evaluator.withgoogle.com/)
- [Content Security Policy Reference](https://content-security-policy.com/)

---

*This documentation is part of the Rusti web framework. For more information, visit the Rusti documentation.*
