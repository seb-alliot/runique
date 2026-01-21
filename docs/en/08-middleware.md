# 🛡️ Middleware & Security

## Middleware Stack

Middleware executes in **REVERSE** order of declaration:

```rust
use runique::prelude::*;
use tower_sessions::SessionManagerLayer;

pub fn middleware_stack(db: Arc<DatabaseConnection>) -> Router {
    Router::new()
        // ⚠️ Declaration REVERSE:
        // 5. Extension injection (last)
        .layer(Extension(RuniqueEngine {
            db: db.clone(),
            config: RuniqueConfig::from_env(),
        }))
        
        // 4. Error handler
        .layer(axum::middleware::from_fn(error_handler))
        
        // 3. Flash messages
        .layer(axum::middleware::from_fn(flash_messages_layer))
        
        // 2. CSRF protection
        .layer(axum::middleware::from_fn(csrf_middleware))
        
        // 1. Sessions (first)
        .layer(SessionManagerLayer::new(tower_sessions::MemoryStore::new()))
}

// Execution = Session → CSRF → Flash → Error → Extension
```

---

## CSRF Protection

### Automatic for POST/PUT/PATCH/DELETE

```rust
// In config_runique/mod.rs
pub async fn csrf_middleware(
    session: Session,
    request: Request,
    next: Next,
) -> Result<Response> {
    if matches!(request.method(), &Method::POST | &Method::PUT | &Method::PATCH | &Method::DELETE) {
        let token_header = request
            .headers()
            .get("X-CSRF-Token")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());

        let token_form = request
            .form_data()
            .and_then(|f| f.get("csrf_token"))
            .map(|s| s.to_string());

        let token = token_header.or(token_form).ok_or("Missing CSRF token")?;

        let session_token = session.get::<String>("csrf_token")?;
        
        if !verify_csrf_token(&token, &session_token) {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    Ok(next.run(request).await)
}
```

### In Forms

```html
<form method="post">
    {{ '' | csrf_field }}
    <!-- Generates: -->
    <!-- <input type="hidden" name="csrf_token" value="masked_token"> -->
</form>
```

### AJAX Endpoints

```javascript
// Get fresh token
fetch('/api/csrf-token')
    .then(r => r.json())
    .then(data => {
        fetch('/api/users', {
            method: 'POST',
            headers: {
                'X-CSRF-Token': data.csrf_token,
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({username: "john"})
        })
    });
```

---

## ALLOWED_HOSTS

Protection against Host Header Injection:

```rust
// .env
ALLOWED_HOSTS=localhost,127.0.0.1,example.com

// runique/src/config_runique/config_struct.rs
impl RuniqueConfig {
    pub fn validate_host(&self, host: &str) -> bool {
        self.allowed_hosts.iter().any(|h| h == host)
    }
}

// Middleware
pub async fn validate_host_middleware(
    headers: HeaderMap,
    next: Next,
) -> Result<Response> {
    let host = headers
        .get("host")
        .and_then(|h| h.to_str().ok())
        .ok_or("Missing Host header")?;

    if !CONFIG.validate_host(host) {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(next.run(request).await)
}
```

---

## Sessions

### Configuration

```rust
use tower_sessions::{SessionManagerLayer, MemoryStore};
use tower_sessions::session::Config;
use time::Duration;

let session_config = Config::default()
    .with_table_name("sessions")
    .with_cookie_name("RUNIQUE_SID")
    .with_cookie_path("/")
    .with_cookie_same_site(SameSite::Lax)
    .with_secure(!DEBUG)        // HTTPS in production
    .with_http_only(!DEBUG);    // No JS access

let session_layer = SessionManagerLayer::new(
    MemoryStore::new()
).with_config(session_config);
```

### Using Sessions

```rust
use tower_sessions::Session;

async fn login(
    session: Session,
    Form(credentials): Form<LoginForm>,
) -> Response {
    if let Ok(Some(user)) = authenticate(&credentials).await {
        session.insert("user_id", user.id).unwrap();
        session.insert("username", &user.username).unwrap();
        
        Redirect::to("/dashboard").into_response()
    } else {
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

## CSP - Content Security Policy

### Nonce Generation

```rust
use runique::utils::generate_csp_nonce;

async fn index(
    template: TemplateContext,
) -> Response {
    let nonce = generate_csp_nonce();
    
    template.render("index.html", &context! {
        "csp_nonce" => nonce
    })
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

### HTTP Headers

```rust
use axum::http::header::{HeaderMap, HeaderValue};

async fn csp_middleware(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let nonce = generate_csp_nonce();

    response.headers_mut().insert(
        "Content-Security-Policy",
        HeaderValue::from_str(&format!(
            "script-src 'nonce-{}' 'strict-dynamic'; style-src 'nonce-{}'",
            nonce, nonce
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
    format!("Welcome, {}!", user.username)
}
```

---

## Next Steps

← [**ORM & Database**](./07-orm.md) | [**Flash Messages**](./09-flash-messages.md) →
