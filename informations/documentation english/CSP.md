# Content Security Policy (CSP) - Rusti Framework

Rusti provides an integrated Content Security Policy (CSP) middleware to protect your application against XSS attacks, code injection, and other security threats.

## Table of Contents

1. [Introduction](#introduction)
2. [Basic Configuration](#basic-configuration)
3. [Predefined Configurations](#predefined-configurations)
4. [Custom Configuration](#custom-configuration)
5. [Nonce Usage](#nonce-usage)
6. [Middleware Integration](#middleware-integration)
7. [Common Patterns](#common-patterns)
8. [Progressive Migration](#progressive-migration)
9. [Debugging](#debugging)
10. [Best Practices](#best-practices)
11. [API Reference](#api-reference)

---

## Introduction

### What is CSP?

Content Security Policy (CSP) is an HTTP security header that allows you to control which resources the browser can load. It acts as a whitelist of trusted sources for:

- Scripts (`script-src`)
- Stylesheets (`style-src`)
- Images (`img-src`)
- Fonts (`font-src`)
- External connections (`connect-src`)
- Frames (`frame-ancestors`)
- And more

### Why Use CSP?

CSP provides strong protection against:

- **XSS attacks**: Blocks execution of malicious injected scripts
- **Data injection**: Prevents loading of untrusted resources
- **Clickjacking**: Controls framing of your pages
- **Mixed content**: Forces HTTPS usage

**Mozilla Observatory Score:** Rusti's strict CSP contributes to an A+ (115/100) score.

---

## Basic Configuration

### CSP Structure

The `CspConfig` struct defines all CSP directives:

```rust
pub struct CspConfig {
    pub default_src: Vec<String>,
    pub script_src: Vec<String>,
    pub style_src: Vec<String>,
    pub img_src: Vec<String>,
    pub font_src: Vec<String>,
    pub connect_src: Vec<String>,
    pub frame_ancestors: Vec<String>,
    pub base_uri: Vec<String>,
    pub form_action: Vec<String>,
    pub use_nonce: bool,
}
```

### Standard Directives

| Directive | Description | Example |
|-----------|-------------|---------|
| `default-src` | Default policy for all resources | `'self'` |
| `script-src` | JavaScript sources | `'self' https://cdn.example.com` |
| `style-src` | CSS sources | `'self' 'unsafe-inline'` |
| `img-src` | Image sources | `'self' data: https:` |
| `font-src` | Font sources | `'self' https://fonts.gstatic.com` |
| `connect-src` | AJAX/WebSocket/EventSource | `'self' https://api.example.com` |
| `frame-ancestors` | Who can frame your page | `'none'` |
| `base-uri` | `<base>` tag restrictions | `'self'` |
| `form-action` | Form submission destinations | `'self'` |

### Special Keywords

- `'self'`: Same origin (protocol + domain + port)
- `'none'`: Block everything
- `'unsafe-inline'`: Allow inline scripts/styles (avoid in production)
- `'unsafe-eval'`: Allow `eval()` and similar (avoid)
- `'nonce-xxx'`: Allow specific inline code with cryptographic nonce

---

## Predefined Configurations

Rusti provides three ready-to-use CSP configurations:

### 1. Development Mode (`default()`)

Permissive for rapid development:

```rust
use rusti::middleware::CspConfig;

let csp = CspConfig::default();

RustiApp::new(settings)
    .await?
    .middleware(csp.into_middleware())
    .routes(routes())
    .run()
    .await?;
```

**Directives:**
```
default-src 'self'
script-src 'self' 'unsafe-inline' 'unsafe-eval'
style-src 'self' 'unsafe-inline'
img-src 'self' data: https:
font-src 'self' data:
connect-src 'self'
frame-ancestors 'self'
base-uri 'self'
form-action 'self'
```

**Use case:** Local development, prototyping

### 2. Production Mode (`strict()`)

Maximum security:

```rust
use rusti::middleware::CspConfig;

let csp = CspConfig::strict();

RustiApp::new(settings)
    .await?
    .middleware(csp.into_middleware())
    .routes(routes())
    .run()
    .await?;
```

**Directives:**
```
default-src 'self'
script-src 'self'
style-src 'self'
img-src 'self' data:
font-src 'self'
connect-src 'self'
frame-ancestors 'none'
base-uri 'self'
form-action 'self'
```

**Use case:** Production deployment, high security

### 3. Testing Mode (`permissive()`)

Very permissive for tests:

```rust
use rusti::middleware::CspConfig;

let csp = CspConfig::permissive();

RustiApp::new(settings)
    .await?
    .middleware(csp.into_middleware())
    .routes(routes())
    .run()
    .await?;
```

**Directives:**
```
default-src *
script-src * 'unsafe-inline' 'unsafe-eval'
style-src * 'unsafe-inline'
img-src * data: blob:
font-src *
connect-src *
frame-ancestors *
base-uri *
form-action *
```

**Use case:** Integration tests, debugging

---

## Custom Configuration

### Manual Configuration

```rust
use rusti::middleware::CspConfig;

let csp = CspConfig {
    default_src: vec!["'self'".to_string()],
    script_src: vec![
        "'self'".to_string(),
        "https://cdn.jsdelivr.net".to_string(),
    ],
    style_src: vec![
        "'self'".to_string(),
        "https://fonts.googleapis.com".to_string(),
        "'unsafe-inline'".to_string(), // For inline styles
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
    use_nonce: true, // Enable nonces for inline scripts
};

RustiApp::new(settings)
    .await?
    .middleware(csp.into_middleware())
    .routes(routes())
    .run()
    .await?;
```

### Modifying a Predefined Configuration

```rust
let mut csp = CspConfig::strict();

// Add a CDN
csp.script_src.push("https://cdn.example.com".to_string());
csp.style_src.push("https://cdn.example.com".to_string());

// Allow images from external sources
csp.img_src.push("https://images.example.com".to_string());

RustiApp::new(settings)
    .await?
    .middleware(csp.into_middleware())
    .routes(routes())
    .run()
    .await?;
```

---

## Nonce Usage

### What is a Nonce?

A nonce (Number used ONCE) is a cryptographic random value that authorizes a specific inline script or style. This avoids using `'unsafe-inline'`.

### Enabling Nonces

```rust
let mut csp = CspConfig::strict();
csp.use_nonce = true; // Enable nonce generation

RustiApp::new(settings)
    .await?
    .middleware(csp.into_middleware())
    .routes(routes())
    .run()
    .await?;
```

### Using in Templates

Rusti automatically injects nonce into the context with the `{{ csp_nonce }}` variable:

```html
<!DOCTYPE html>
<html>
<head>
    <title>My Page</title>

    <!-- Inline style with nonce -->
    <style nonce="{{ csp_nonce }}">
        body {
            background: #f0f0f0;
        }
    </style>
</head>
<body>
    <h1>Hello World</h1>

    <!-- Inline script with nonce -->
    <script nonce="{{ csp_nonce }}">
        console.log('This script is authorized by CSP');
    </script>
</body>
</html>
```

### Alternative Template Tag

You can also use the `{% csp %}` tag:

```html
<script nonce="{% csp %}">
    // Your inline code
</script>

<style nonce="{% csp %}">
    /* Your inline styles */
</style>
```

**Important:** Without a nonce, inline scripts/styles will be blocked if `'unsafe-inline'` is not in your CSP.

---

## Middleware Integration

Rusti provides three methods to integrate CSP:

### 1. `with_csp()` - CSP Only

Adds only the CSP header:

```rust
use rusti::middleware::CspConfig;

let csp = CspConfig::strict();

RustiApp::new(settings)
    .await?
    .with_csp(csp) // CSP only
    .routes(routes())
    .run()
    .await?;
```

### 2. `with_security_headers()` - Complete Security

Adds CSP + all security headers:

```rust
use rusti::middleware::CspConfig;

let csp = CspConfig::strict();

RustiApp::new(settings)
    .await?
    .with_security_headers(csp) // CSP + all headers
    .routes(routes())
    .run()
    .await?;
```

**Added headers:**
- `Content-Security-Policy`
- `X-Content-Type-Options: nosniff`
- `X-Frame-Options: DENY`
- `X-XSS-Protection: 1; mode=block`
- `Referrer-Policy: strict-origin-when-cross-origin`
- `Permissions-Policy: geolocation=(), microphone=(), camera=()`

### 3. `with_csp_report_only()` - Report Mode

Sends CSP in report mode (doesn't block, only reports violations):

```rust
use rusti::middleware::CspConfig;

let csp = CspConfig::strict();

RustiApp::new(settings)
    .await?
    .with_csp_report_only(csp) // Test without blocking
    .routes(routes())
    .run()
    .await?;
```

**Use:** Test CSP without breaking your application. Violations are logged in browser console.

---

## Common Patterns

### Pattern 1: Development

```rust
let csp = CspConfig::default(); // Permissive

RustiApp::new(settings)
    .await?
    .with_csp(csp)
    .routes(routes())
    .run()
    .await?;
```

### Pattern 2: Production with CDN

```rust
let mut csp = CspConfig::strict();

// Add your CDNs
csp.script_src.push("https://cdn.jsdelivr.net".to_string());
csp.style_src.push("https://cdn.jsdelivr.net".to_string());
csp.font_src.push("https://fonts.gstatic.com".to_string());
csp.style_src.push("https://fonts.googleapis.com".to_string());

// Enable nonces for inline scripts
csp.use_nonce = true;

RustiApp::new(settings)
    .await?
    .with_security_headers(csp) // Complete security
    .routes(routes())
    .run()
    .await?;
```

### Pattern 3: API with External Connections

```rust
let mut csp = CspConfig::strict();

// Allow API calls
csp.connect_src.push("https://api.example.com".to_string());
csp.connect_src.push("wss://ws.example.com".to_string());

RustiApp::new(settings)
    .await?
    .with_csp(csp)
    .routes(routes())
    .run()
    .await?;
```

### Pattern 4: Progressive Migration

Start with report-only mode:

```rust
// Phase 1: Report only
let csp = CspConfig::strict();
app.with_csp_report_only(csp);

// Phase 2: After validation, enable enforcement
let csp = CspConfig::strict();
app.with_csp(csp);
```

---

## Progressive Migration

### Step 1: Analyze Current Application

Identify all resources used:
- External scripts (CDN, analytics, ads)
- Inline scripts and styles
- Images from external sources
- Fonts
- API calls

### Step 2: Start with Report Mode

```rust
let csp = CspConfig::strict();

RustiApp::new(settings)
    .await?
    .with_csp_report_only(csp)
    .routes(routes())
    .run()
    .await?;
```

### Step 3: Check Browser Console

Open DevTools (F12) and check for CSP violations:

```
[Report Only] Refused to load the script 'https://example.com/script.js'
because it violates the following Content Security Policy directive: "script-src 'self'"
```

### Step 4: Adjust Configuration

Add missing sources:

```rust
let mut csp = CspConfig::strict();

// Add sources identified in console
csp.script_src.push("https://example.com".to_string());
csp.img_src.push("https://images.example.com".to_string());

app.with_csp_report_only(csp); // Still in report mode
```

### Step 5: Switch to Enforcement

Once there are no more violations:

```rust
let csp = CspConfig::strict();
// ... your adjustments

RustiApp::new(settings)
    .await?
    .with_csp(csp) // Enforcement enabled
    .routes(routes())
    .run()
    .await?;
```

---

## Debugging

### Common CSP Issues

#### 1. Inline Script Blocked

**Error:**
```
Refused to execute inline script because it violates CSP directive: "script-src 'self'"
```

**Solutions:**

a) Use nonces (recommended):
```rust
let mut csp = CspConfig::strict();
csp.use_nonce = true;
```

```html
<script nonce="{{ csp_nonce }}">
    console.log('Authorized');
</script>
```

b) Allow `'unsafe-inline'` (not recommended):
```rust
csp.script_src.push("'unsafe-inline'".to_string());
```

#### 2. External Resource Blocked

**Error:**
```
Refused to load 'https://cdn.example.com/script.js' because it violates CSP
```

**Solution:** Add the domain:
```rust
csp.script_src.push("https://cdn.example.com".to_string());
```

#### 3. Image Blocked

**Error:**
```
Refused to load the image 'https://example.com/image.jpg'
```

**Solution:**
```rust
csp.img_src.push("https://example.com".to_string());
// Or allow all HTTPS images
csp.img_src.push("https:".to_string());
```

#### 4. Font Blocked

**Error:**
```
Refused to load the font 'https://fonts.gstatic.com/...'
```

**Solution:**
```rust
csp.font_src.push("https://fonts.gstatic.com".to_string());
csp.style_src.push("https://fonts.googleapis.com".to_string());
```

### Inspecting CSP in Browser

1. Open DevTools (F12)
2. Go to **Network** tab
3. Click on your page
4. Check **Headers** → **Response Headers**
5. Look for `Content-Security-Policy`

---

## Best Practices

### 1. Use `strict()` in Production

```rust
// Good
let csp = CspConfig::strict();

// Bad (too permissive)
let csp = CspConfig::permissive();
```

### 2. Prefer Nonces over `unsafe-inline`

```rust
// Good
csp.use_nonce = true;

// Bad
csp.script_src.push("'unsafe-inline'".to_string());
```

### 3. Never Use `unsafe-eval`

```rust
// Very bad
csp.script_src.push("'unsafe-eval'".to_string());
```

This allows `eval()`, `new Function()`, and opens major XSS vulnerabilities.

### 4. Be Specific with Domains

```rust
// Good (specific)
csp.script_src.push("https://cdn.jsdelivr.net".to_string());

// Bad (too broad)
csp.script_src.push("https:".to_string());
```

### 5. Block Framing

```rust
// Good (prevents clickjacking)
csp.frame_ancestors = vec!["'none'".to_string()];

// Or allow only your domain
csp.frame_ancestors = vec!["'self'".to_string()];
```

### 6. Use Report-Only for Testing

Always test with `with_csp_report_only()` before deploying a strict CSP.

### 7. Combine with Other Security Headers

Use `with_security_headers()` for comprehensive protection:

```rust
RustiApp::new(settings)
    .await?
    .with_security_headers(csp) // CSP + X-Frame-Options + etc.
    .routes(routes())
    .run()
    .await?;
```

---

## API Reference

### `CspConfig`

```rust
pub struct CspConfig {
    pub default_src: Vec<String>,
    pub script_src: Vec<String>,
    pub style_src: Vec<String>,
    pub img_src: Vec<String>,
    pub font_src: Vec<String>,
    pub connect_src: Vec<String>,
    pub frame_ancestors: Vec<String>,
    pub base_uri: Vec<String>,
    pub form_action: Vec<String>,
    pub use_nonce: bool,
}
```

### Predefined Configurations

```rust
impl CspConfig {
    /// Development configuration (permissive)
    pub fn default() -> Self;

    /// Production configuration (strict)
    pub fn strict() -> Self;

    /// Testing configuration (very permissive)
    pub fn permissive() -> Self;

    /// Convert to middleware
    pub fn into_middleware(self) -> CspMiddleware;
}
```

### Integration Methods

```rust
impl RustiApp {
    /// Add CSP header only
    pub fn with_csp(self, config: CspConfig) -> Self;

    /// Add CSP + all security headers
    pub fn with_security_headers(self, config: CspConfig) -> Self;

    /// Add CSP in report-only mode (doesn't block)
    pub fn with_csp_report_only(self, config: CspConfig) -> Self;
}
```

### Template Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `{{ csp_nonce }}` | Current CSP nonce | `8f7a9b2c...` |
| `{% csp %}` | Template tag for nonce | Same as above |

---

## See Also

- [Configuration Guide](CONFIGURATION.md)
- [Security Guide](SECURITY.md)
- [Getting Started](GETTING_STARTED.md)

Develop securely with Rusti!