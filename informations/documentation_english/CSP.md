# Content Security Policy (CSP) Guide - Runique Framework

Runique includes built-in Content Security Policy (CSP) support to protect your application against XSS attacks and code injection.

## Table of Contents

1. [What is CSP?](#what-is-csp)
2. [Configuration](#configuration)
3. [Using Nonces](#using-nonces)
4. [Preset Configurations](#preset-configurations)
5. [Complete Examples](#complete-examples)
6. [Debugging](#debugging)

---

## What is CSP?

Content Security Policy (CSP) is an HTTP security header that helps prevent:
- **XSS attacks** (Cross-Site Scripting)
- **Code injection**
- **Clickjacking**
- **Data theft**

CSP works by defining which sources are authorized to load resources (scripts, styles, images, etc.).

### Basic Example

```
Content-Security-Policy: default-src 'self'; script-src 'self' https://cdn.example.com
```

This policy means:
- By default, only load resources from the same origin (`'self'`)
- Scripts can come from the same origin or `cdn.example.com`

---

## Configuration

### CspConfig Structure

The `CspConfig` struct uses **individual named fields** (not a HashMap):

```rust
pub struct CspConfig {
    // Individual directives
    pub default_src: Vec<String>,
    pub script_src: Vec<String>,
    pub style_src: Vec<String>,
    pub img_src: Vec<String>,
    pub font_src: Vec<String>,
    pub connect_src: Vec<String>,
    pub frame_ancestors: Vec<String>,
    pub base_uri: Vec<String>,
    pub form_action: Vec<String>,

    // Nonce configuration
    pub use_nonce: bool,
}
```

### Available Directives

| Directive | Description | Common Values |
|-----------|-------------|---------------|
| `default_src` | Default policy for all resources | `'self'`, `'none'`, domains |
| `script_src` | JavaScript sources | `'self'`, CDN domains |
| `style_src` | CSS sources | `'self'`, `'unsafe-inline'` |
| `img_src` | Image sources | `'self'`, `data:`, domains |
| `font_src` | Font sources | `'self'`, font CDNs |
| `connect_src` | AJAX/WebSocket/EventSource | `'self'`, API domains |
| `frame_ancestors` | Who can embed this page | `'none'`, `'self'` |
| `base_uri` | Allowed `<base>` URLs | `'self'`, `'none'` |
| `form_action` | Form submission targets | `'self'`, specific domains |
| `use_nonce` | Enable cryptographic nonces | `true`, `false` |

### Basic Configuration

```rust
use runique::prelude::*;
use runique::middleware::CspConfig;

let csp_config = CspConfig::default();

RuniqueApp::new(settings).await?
    .with_csp(csp_config)
    .routes(routes())
    .run()
    .await?;
```

### Configuration with Builder Pattern

```rust
let csp = CspConfig::strict()
    .script_src(vec!["'self'", "https://cdn.example.com"])
    .style_src(vec!["'self'", "'unsafe-inline'"])
    .img_src(vec!["'self'", "data:", "https:"])
    .build();
```

### Configuration with External CDNs

```rust
let csp_config = CspConfig {
    default_src: vec!["'self'".to_string()],

    script_src: vec![
        "'self'".to_string(),
        "https://cdn.jsdelivr.net".to_string(),
        "https://unpkg.com".to_string(),
    ],

    style_src: vec![
        "'self'".to_string(),
        "'unsafe-inline'".to_string(),
        "https://fonts.googleapis.com".to_string(),
    ],

    img_src: vec![
        "'self'".to_string(),
        "data:".to_string(),
        "https:".to_string(),
    ],

    font_src: vec![
        "'self'".to_string(),
        "https://fonts.gstatic.com".to_string(),
    ],

    connect_src: vec![
        "'self'".to_string(),
        "https://api.example.com".to_string(),
    ],

    frame_ancestors: vec!["'none'".to_string()],
    base_uri: vec!["'self'".to_string()],
    form_action: vec!["'self'".to_string()],
    use_nonce: false,
};
```

---

## Using Nonces

Nonces (Number used ONCE) are cryptographically random values that allow inline scripts and styles.

### Why Use Nonces?

**Problem without nonce:**
```html
<!-- ❌ Blocked by CSP if script-src doesn't allow 'unsafe-inline' -->
<script>
    console.log("This will be blocked");
</script>
```

**Solution with nonce:**
```html
<!-- ✅ Allowed with nonce -->
<!-- replace nonce="abc123xyz789" by {{ csp }}> -->
<script nonce="abc123xyz789">
    console.log("This is allowed");
</script>
```

### Enabling Nonces

```rust
let csp_config = CspConfig {
    default_src: vec!["'self'".to_string()],
    script_src: vec!["'self'".to_string()],
    style_src: vec!["'self'".to_string()],
    use_nonce: true,  // ✅ Enable nonce generation
    ..Default::default()
};
```

### Using Nonces in Templates

**⚠️ IMPORTANT:** The `{{ nonce() }}` tag generates a nonce **ONLY if `use_nonce: true`** in the CSP configuration.

```html
<!-- Inline script with nonce -->
<script {{ nonce() }}>
    // JavaScript code
    console.log("Script allowed with nonce");
</script>

<!-- Inline style with nonce -->
<style {{ nonce() }}>
    /* CSS code */
    body { background: #f0f0f0; }
</style>
```

**If `use_nonce: false`:**

The `{{ nonce() }}` tag will generate an **empty string**:

```html
<script>
    console.log("No nonce generated");
</script>
```

### When to Use Nonces

| Use Case | Use Nonce? |
|----------|------------|
| Inline scripts necessary | ✅ Yes |
| Inline styles necessary | ✅ Yes |
| External scripts only | ❌ No (use `script-src 'self'`) |
| Strict mode without inline | ❌ No |

### Complete Example with Nonces

```html
{% extends "base.html" %}

{% block content %}
<h2>Dashboard</h2>

<div id="chart"></div>

{% endblock %}

{% block extra_js %}
<!-- External script (no nonce needed) -->
<script src="{% static 'js/chart.min.js' %}"></script>

<!-- Inline script (requires nonce if strict CSP) -->
<script {{ nonce() }}>
    const data = {{ chart_data|json_encode|safe }};

    new Chart(document.getElementById('chart'), {
        type: 'bar',
        data: data,
    });
</script>
{% endblock %}
```

---

## Middleware Integration

Runique provides three middleware functions for CSP:

### 1. csp_middleware()

Adds the CSP header only.

```rust
RuniqueApp::new(settings).await?
    .routes(routes)
    .with_csp(CspConfig::strict())
    .run()
    .await?;
```

### 2. security_headers_middleware()

Adds CSP + 8 additional security headers.

```rust
RuniqueApp::new(settings).await?
    .routes(routes)
    .with_security_headers(CspConfig::strict())
    .run()
    .await?;
```

**Security headers added by `with_security_headers()`:**

- **Content-Security-Policy** (your CSP configuration)
- **X-Content-Type-Options**: nosniff
- **X-Frame-Options**: DENY
- **X-XSS-Protection**: 1; mode=block
- **Referrer-Policy**: strict-origin-when-cross-origin
- **Permissions-Policy**: geolocation=(), microphone=(), camera=()
- **Cross-Origin-Embedder-Policy**: require-corp
- **Cross-Origin-Opener-Policy**: same-origin
- **Cross-Origin-Resource-Policy**: same-origin

### 3. csp_report_only_middleware()

CSP in report-only mode (violations are logged but not blocked).

```rust
RuniqueApp::new(settings).await?
    .routes(routes)
    .with_csp_report_only(CspConfig::strict())
    .run()
    .await?;
```

---

## Preset Configurations

Runique provides three preset CSP configurations:

| Preset | Script-src | Nonce | Unsafe-inline | Usage |
|--------|-----------|-------|----------------|-------|
| `default()` | `'self' 'unsafe-inline'` | ❌ | ✅ | Development |
| `strict()` | `'self'` | ✅ | ❌ | Production |
| `permissive()` | `'self' 'unsafe-inline' 'unsafe-eval'` | ❌ | ✅ | Advanced Dev |

### 1. Default Configuration (Balanced)

Suitable for most applications with modern practices.

```rust
let csp_config = CspConfig::default();
// Equivalent to:
CspConfig {
    default_src: vec!["'self'".to_string()],
    script_src: vec!["'self'".to_string(), "'unsafe-inline'".to_string()],
    style_src: vec!["'self'".to_string(), "'unsafe-inline'".to_string()],
    img_src: vec!["'self'".to_string(), "data:".to_string()],
    font_src: vec!["'self'".to_string()],
    connect_src: vec!["'self'".to_string()],
    frame_ancestors: vec!["'none'".to_string()],
    base_uri: vec!["'self'".to_string()],
    form_action: vec!["'self'".to_string()],
    use_nonce: false,
}
```

**Features:**
- ✅ Blocks most XSS attacks
- ✅ Allows inline scripts and styles (`'unsafe-inline'`)
- ✅ Allows data URIs for images
- ❌ No nonces enabled

### 2. Strict Configuration (Maximum Security)

For applications requiring maximum security.

```rust
let csp_config = CspConfig::strict();
// Equivalent to:
CspConfig {
    default_src: vec!["'self'".to_string()],
    script_src: vec!["'self'".to_string()],
    style_src: vec!["'self'".to_string()],
    img_src: vec!["'self'".to_string()],
    font_src: vec!["'self'".to_string()],
    connect_src: vec!["'self'".to_string()],
    frame_ancestors: vec!["'none'".to_string()],
    base_uri: vec!["'self'".to_string()],
    form_action: vec!["'self'".to_string()],
    use_nonce: true,  // ✅ Nonces enabled for production
}
```

**Features:**
- ✅ Maximum protection
- ✅ Blocks all inline code without nonce
- ✅ No external resources by default
- ⚠️ Requires nonces for inline scripts/styles
- ⚠️ Requires explicit configuration for CDNs

### 3. Permissive Configuration (Development)

For development or legacy applications.

```rust
let csp_config = CspConfig::permissive();
// Equivalent to:
CspConfig {
    default_src: vec!["'self'".to_string()],
    script_src: vec![
        "'self'".to_string(),
        "'unsafe-inline'".to_string(),
        "'unsafe-eval'".to_string(),
    ],
    style_src: vec![
        "'self'".to_string(),
        "'unsafe-inline'".to_string(),
    ],
    img_src: vec![
        "'self'".to_string(),
        "data:".to_string(),
        "https:".to_string(),
    ],
    font_src: vec!["'self'".to_string(), "data:".to_string()],
    connect_src: vec!["'self'".to_string()],
    frame_ancestors: vec!["'self'".to_string()],
    base_uri: vec!["'self'".to_string()],
    form_action: vec!["'self'".to_string()],
    use_nonce: false,
}
```

**Features:**
- ✅ Very permissive, few restrictions
- ✅ Allows inline scripts and styles
- ✅ Allows `eval()`
- ✅ Allows HTTPS images
- ❌ Minimal protection against XSS
- ⚠️ **NOT recommended for production**

---

## Complete Examples

### Example 1: Modern SPA with CDN

```rust
use runique::prelude::*;
use runique::middleware::CspConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::from_env();

    let csp_config = CspConfig {
        default_src: vec!["'self'".to_string()],

        script_src: vec![
            "'self'".to_string(),
            "https://cdn.jsdelivr.net".to_string(),
            "https://unpkg.com".to_string(),
        ],

        style_src: vec![
            "'self'".to_string(),
            "'unsafe-inline'".to_string(),
            "https://fonts.googleapis.com".to_string(),
        ],

        img_src: vec![
            "'self'".to_string(),
            "data:".to_string(),
            "https:".to_string(),
        ],

        font_src: vec![
            "'self'".to_string(),
            "https://fonts.gstatic.com".to_string(),
        ],

        connect_src: vec![
            "'self'".to_string(),
            "https://api.example.com".to_string(),
            "wss://websocket.example.com".to_string(),
        ],

        frame_ancestors: vec!["'none'".to_string()],
        base_uri: vec!["'self'".to_string()],
        form_action: vec!["'self'".to_string()],
        use_nonce: false,
    };

    RuniqueApp::new(settings).await?
        .with_security_headers(csp_config)
        .routes(routes())
        .run()
        .await?;

    Ok(())
}
```

### Example 2: Application with Inline Scripts (Nonces)

```rust
let csp_config = CspConfig::strict();

RuniqueApp::new(settings).await?
    .with_csp(csp_config)
    .routes(routes())
    .run()
    .await?;
```

**Template with nonce:**

```html
<!DOCTYPE html>
<html>
<head>
    <title>Dashboard</title>

    <!-- External style (no nonce needed) -->
    <link rel="stylesheet" href="{% static 'css/style.css' %}">

    <!-- Inline style (requires nonce) -->
    <style {{ nonce() }}>
        .custom-chart { width: 100%; height: 400px; }
    </style>
</head>
<body>
    <div id="app"></div>

    <!-- External script (no nonce needed) -->
    <script src="{% static 'js/vue.min.js' %}"></script>

    <!-- Inline script (requires nonce) -->
    <script {{ nonce() }}>
        new Vue({
            el: '#app',
            data: {{ app_data|json_encode|safe }}
        });
    </script>
</body>
</html>
```

### Example 3: API with Strict CSP

```rust
let csp_config = CspConfig {
    default_src: vec!["'none'".to_string()],
    script_src: vec!["'self'".to_string()],
    style_src: vec!["'self'".to_string()],
    img_src: vec!["'self'".to_string()],
    font_src: vec!["'self'".to_string()],
    connect_src: vec![
        "'self'".to_string(),
        "https://api.backend.com".to_string(),
    ],
    frame_ancestors: vec!["'none'".to_string()],
    base_uri: vec!["'none'".to_string()],
    form_action: vec!["'none'".to_string()],
    use_nonce: true,
};
```

---

## Debugging

### Testing Your CSP

1. **Browser Console**

Open DevTools (F12) and check the Console. CSP violations appear as warnings:

```
Refused to execute inline script because it violates the following
Content Security Policy directive: "script-src 'self'".
```

2. **CSP Report URI** (Advanced)

```rust
let csp_config = CspConfig {
    default_src: vec!["'self'".to_string()],
    // ... other directives
    report_uri: Some("https://example.com/csp-report".to_string()),
    use_nonce: false,
};
```

3. **Mozilla Observatory**

Test your site: [https://observatory.mozilla.org/](https://observatory.mozilla.org/)

### Common Issues

#### Issue 1: Inline Scripts Blocked

**Error:**
```
Refused to execute inline script because it violates CSP
```

**Solutions:**

**Option A:** Use nonces (recommended)
```rust
let csp_config = CspConfig::strict();
```

```html
<script {{ nonce() }}>
    console.log("Now allowed");
</script>
```

**Option B:** Allow `'unsafe-inline'` (not recommended)
```rust
let csp_config = CspConfig::default();  // or permissive()
```

#### Issue 2: External CDN Blocked

**Error:**
```
Refused to load script from 'https://cdn.jsdelivr.net' because it violates CSP
```

**Solution:** Add the CDN to `script_src`
```rust
let csp_config = CspConfig {
    script_src: vec![
        "'self'".to_string(),
        "https://cdn.jsdelivr.net".to_string(),
    ],
    ..Default::default()
};
```

#### Issue 3: Images from Other Domains Blocked

**Solution:**
```rust
let csp_config = CspConfig {
    img_src: vec![
        "'self'".to_string(),
        "data:".to_string(),
        "https:".to_string(),  // All HTTPS images
        // Or specific:
        "https://images.example.com".to_string(),
    ],
    ..Default::default()
};
```

#### Issue 4: Google Fonts Blocked

**Solution:**
```rust
let csp_config = CspConfig {
    style_src: vec![
        "'self'".to_string(),
        "https://fonts.googleapis.com".to_string(),
    ],
    font_src: vec![
        "'self'".to_string(),
        "https://fonts.gstatic.com".to_string(),
    ],
    ..Default::default()
};
```

---

## Best Practices

### 1. Start with a Strict Policy

```rust
// Start strict
let csp_config = CspConfig {
    default_src: vec!["'none'".to_string()],
    script_src: vec!["'self'".to_string()],
    style_src: vec!["'self'".to_string()],
    use_nonce: true,
    ..Default::default()
};
```

Then relax as needed.

### 2. Avoid `'unsafe-inline'` and `'unsafe-eval'`

```rust
// ❌ Bad
script_src: vec!["'self'".to_string(), "'unsafe-inline'".to_string()]

// ✅ Good - use nonces
script_src: vec!["'self'".to_string()]
use_nonce: true
```

### 3. Use Specific Domains, Not Wildcards

```rust
// ❌ Bad
img_src: vec!["*".to_string()]

// ✅ Good
img_src: vec![
    "'self'".to_string(),
    "https://images.example.com".to_string(),
]
```

### 4. Test in Development Mode First

```rust
// Development
let csp_config = if settings.debug {
    // Permissive config
} else {
    // Strict config
};
```

### 5. Monitor CSP Violations

Use `report-uri` or browser console to identify issues.

---

## Migration from Old Documentation

If you were using the old documentation with `HashMap<String, Vec<String>>`:

**Before (incorrect):**
```rust
let mut directives = HashMap::new();
directives.insert("default-src".to_string(), vec!["'self'".to_string()]);
directives.insert("script-src".to_string(), vec!["'self'".to_string()]);
```

**Now (correct):**
```rust
let csp_config = CspConfig {
    default_src: vec!["'self'".to_string()],
    script_src: vec!["'self'".to_string()],
    style_src: vec!["'self'".to_string()],
    // ... other fields
    ..Default::default()
};
```

---

## See Also

- [Security Guide](informations/documentation_english/CSP.md)
- [Configuration](informations/documentation_english/CONFIGURATION.md)
- [Templates](informations/documentation_english/TEMPLATES.md)
- [MDN CSP Documentation](https://developer.mozilla.org/en-US/docs/Web/HTTP/CSP)

Protect your application with Runique's CSP!

---

**Version:** 0.1.86 (January 17, 2026)
**License:** MIT

*Documentation created with ❤️ by Claude for Itsuki*
