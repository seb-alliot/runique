# 🛡️ Middleware & Security

## Middleware Stack

Middleware executes in **REVERSE** order of declaration. Runique automatically configures the stack via the builder:

```rust
use runique::prelude::*;

// The builder automatically configures the middleware stack
let app = RuniqueAppBuilder::new(config)
    .with_routes(routes)
    .with_error_handler(true)      // Optional
    .with_sanitize(true)           // Optional
    .build()
    .await?;

// Stack applied automatically (reverse order):
// 1. RequestExtensions injection (Tera, Config, Engine)
// 2. Static files (runique internal)
// 3. Error handler (if enabled)
// 4. Sessions (with MemoryStore by default)
// 5. CSRF protection
// 6. Sanitize (if enabled)
// 7. User routes
```

---

## CSRF Protection

### Automatic for POST/PUT/PATCH/DELETE

The CSRF middleware is automatically configured by `RuniqueAppBuilder`. It generates and verifies tokens for all POST/PUT/PATCH/DELETE requests.

**How it works:**
- Token generated with user context (user_id if authenticated, otherwise session_id)
- Double Submit Cookie pattern with token masking
- Automatic verification via `X-CSRF-Token` header or form field
- Integration with `Prisme` for form validation

### In Forms

```html
<form method="post">
    {% csrf %}
    <!-- Generates automatically: -->
    <!-- <input type="hidden" name="csrf_token" value="masked_token"> -->
</form>
```

**Note:** The token is automatically masked (Double Submit Cookie pattern) and verified by the middleware + `Prisme`.

### AJAX Endpoints

```javascript
// The token is automatically sent in the response header
fetch('/api/users', {
    method: 'POST',
    headers: {
        'X-CSRF-Token': document.querySelector('[name="csrf_token"]').value,
        'Content-Type': 'application/json'
    },
    body: JSON.stringify({username: "john"})
});
```

---

## ALLOWED_HOSTS

Protection against Host Header Injection (inspired by Django):

```rust
use runique::middleware::AllowedHostsValidator;

// Configuration in RuniqueConfig
let validator = AllowedHostsValidator::from_settings(&config);

// The validator supports:
// - Exact match: "example.com"
// - Full wildcard: "*" (dangerous in production!)
// - Subdomain wildcard: ".example.com" matches "api.example.com", etc.

impl AllowedHostsValidator {
    pub fn is_host_allowed(&self, host: &str) -> bool {
        if self.debug { return true; }

        let host = host.split(':').next().unwrap_or(host);

        self.allowed_hosts.iter().any(|allowed| {
            if allowed == "*" {
                true
            } else if allowed.starts_with('.') {
                // Subdomains
                host == &allowed[1..] ||
                (host.ends_with(allowed) && host.as_bytes()[host.len() - allowed.len()] == b'.')
            } else {
                allowed == host
            }
        })
    }
}
```

**Usage:**
```rust
// Automatic validation in requests
match validator.validate(&headers) {
    Ok(_) => { /* OK */ }, Expiry};
use time::Duration;

// The builder automatically configures sessions
let session_layer = SessionManagerLayer::new(MemoryStore::default())
    .with_secure(!config.debug)          // HTTPS in production
    .with_http_only(!config.debug)        // No JS access
    .with_expiry(Expiry::OnInactivity(Duration::hours(2)));

// Applied automatically in RuniqueAppBuilder
```

### Using Sessions

```rust
use tower_sessions::Session;

// Method 1: Direct Session extraction
async fn login(
    session: Session,
    Form(credentials): Form<LoginForm>,
) -> Response {
    // Note: authenticate() is an example - this function will be provided in a future version
    if let Ok(Some(user)) = authenticate(&credentials).await {
        session.insert("user_id", user.id).await.ok();
        session.insert("username", &user.username).await.ok();

        Redirect::to("/dashboard").into_response()
    } else {
        Redirect::to("/login?error=1").into_response()
    }
}

async fn dashboard(session: Session) -> Response {
    let user_id: Option<i32> = session.get("user_id").await.ok().flatten();
    match user_id {
        Some(id) => format!("User ID: {}", id).into_response(),
        None => Redirect::to("/login").into_response()
    }
}

// Method 2: Access via TemplateContext (session is included)
async fn profile(template: TemplateContext) -> Response {
    let user_id: Option<i32> = template.session.get("user_id").await.ok().flatten();

    template.context.insert("user_id", &user_id);
    template.render("profile.html").unwrap()
}
e {
        // Error
    }
}

async fn dashboard(
    session: Session,
) -> Response {
    let user_id: i32 = session.get("user_id").unwrap().unwrap();
    // ...
}

// Logout
async fn logout(session: Session) -> Response {
    session.flush().await.ok();
    Redirect::to("/").into_response()
}
```

---

## CSP - Content Seccsp_nonce::CspNonce;

// The nonce is automatically generated and injected into TemplateContext
async fn index(
    template: TemplateContext,
) -> Response {
    // template.csp_nonce is already available
    template.render("index.html").unwrap()
}
```

### In Templates

```html
<!DOCTYPE html>
<html>
<head>
    <style nonce="{{ csp_nonce }}">
        body { color: #333; }
    </style>
</head>
<body>
    <script nonce="{{ csp_nonce }}">
        console.log("Secure script");
    </script>
</body>
</html>
```

### CSP Configuration

```rust
use runique::middleware::CspConfig;

// Predefined profiles
let strict_csp = CspConfig::strict();      // Strict with nonces
let permissive = CspConfig::permissive();  // More lenient
let default = CspConfig::default();        // Balanced

// Generate the header
let nonce = CspNonce::generate();
let header_value = csp_config.to_header_value(Some(nonce.as_str()));

// Example of generated header:
// "default-src 'self'; script-src 'self' 'nonce-ABC123'; style-src 'self' 'nonce-ABC123'"           nonce, nonce
        )).unwrap()
    );

    response
}
```

---

## Custom Authentication

```rust
// Authentication middleware
pub async fn auth_middleware(
    session: Session,
    request: Request,
    next: Next,
) -> Result<Response> {
    let user_id: Option<i32> = session.get("user_id").ok().flatten();

    if user_id.is_none() && !is_public_route(request.uri()) {
        return Err(Redirect::to("/login").into_response());
    }

    Ok(next.run(request).await)
}

// Custom extractor
pub struct CurrentUser {
    pub id: i32,
    pub username: String,
}

#[async_trait]
impl FromRequest for CurrentUser {
    async fn from_request(request: &mut Request) -> Result<Self> {
        let session: Session = request.extract().await?;

        let user_id: i32 = session.get("user_id")?
            .ok_or("Not authenticated")?;

        let username = session.get("username")?
            .ok_or("Not authenticated")?;

        Ok(CurrentUser { id: user_id, username })
    }
}

// Usage:
async fn dashboard(user: CurrentUser) -> Response {
    format!("Welcome, {}!", user.username).into_response()
}
```

---

## Next Steps

← [**ORM & Database**](https://github.com/seb-alliot/runique/blob/main/docs/en/07-orm.md) | [**Flash Messages**](https://github.com/seb-alliot/runique/blob/main/docs/en/09-flash-messages.md) →
