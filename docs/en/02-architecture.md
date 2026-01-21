# 🏗️ Architecture

## Overview

Runique 2.0 follows a **modular, layered architecture** inspired by Django and modern Rust frameworks:

```
┌─────────────────────────────────────┐
│        HTTP Handlers (Routes)       │
├─────────────────────────────────────┤
│     RuniqueContext (Per-Request)    │
├─────────────────────────────────────┤
│    Middleware Layer (Session, CSRF) │
├─────────────────────────────────────┤
│    RuniqueEngine (Application State)│
├─────────────────────────────────────┤
│     Templates, ORM, Config          │
├─────────────────────────────────────┤
│      SeaORM + Database              │
└─────────────────────────────────────┘
```

---

## Key Concepts

### RuniqueEngine

Global application state container:

```rust
pub struct RuniqueEngine {
    pub db: Arc<DatabaseConnection>,
    pub config: RuniqueConfig,
}
```

**Usage**: Access via `RuniqueContext` in handlers
```rust
async fn handler(ctx: RuniqueContext) -> Response {
    let db = ctx.engine.db.clone();
    // Query database
}
```

---

### RuniqueContext

Per-request context injected by middleware:

```rust
pub struct RuniqueContext {
    pub engine: Arc<RuniqueEngine>,
    pub session: Session,
    pub flash: FlashMessageProcessor,
}
```

**Features**:
- Access to application state
- Session management
- Flash message handling
- Template context building

**Usage**:
```rust
async fn handler(mut ctx: RuniqueContext) -> Response {
    ctx.session.insert("key", "value")?;
    success!(ctx.flash => "Success!");
}
```

---

### TemplateContext

Template rendering helper:

```rust
impl TemplateContext {
    pub fn render(
        &self,
        template_name: &str,
        context: &Context,
    ) -> Response
}
```

**Usage**:
```rust
async fn index(template: TemplateContext) -> Response {
    template.render("index.html", &context! {
        "title" => "Home"
    })
}
```

---

### ExtractForm<T>

Form extraction and validation:

```rust
async fn handler(
    ExtractForm(form): ExtractForm<LoginForm>,
) -> Response {
    if form.is_valid().await {
        // Process valid form
    }
}
```

---

## Module Structure

```
runique/
├── config_runique/          # Configuration
│   ├── mod.rs
│   └── config_struct.rs
├── data_base_runique/       # Database layer
│   ├── mod.rs
│   ├── config.rs
│   └── orm_wrapper.rs
├── formulaire/              # Form system
│   ├── mod.rs
│   ├── builder_form/
│   └── utils/
├── gardefou/                # Middleware
│   ├── mod.rs
│   ├── middleware_struct.rs
│   └── composant_middleware/
├── macro_runique/           # Macros
│   ├── mod.rs
│   ├── flash_message/
│   ├── router/
│   └── sea/
├── moteur_engine/           # Template engine
│   ├── mod.rs
│   └── engine_struct.rs
├── request_context/         # Request handling
│   ├── mod.rs
│   ├── request_struct.rs
│   ├── template_context.rs
│   ├── processor.rs
│   └── tera_tool/
├── runique_body/            # Application builder
│   ├── mod.rs
│   └── composant_app/
└── utils/                   # Utilities
    ├── mod.rs
    ├── generate_token.rs
    ├── parse_html.rs
    ├── csp_nonce.rs
    └── response_helpers.rs
```

---

## Request Lifecycle

### 1. Request Arrives

HTTP request received by Axum router

### 2. Middleware Stack (Reverse Order)

Middleware executes in **reverse** of declaration:

```
Request
  ↓
Session Layer      (1. First to execute)
  ↓
CSRF Validation    (2. Second)
  ↓
Flash Messages     (3. Third)
  ↓
Error Handler      (4. Fourth)
  ↓
Extension Inject   (5. Fifth - extracts RuniqueContext)
  ↓
Handler
```

### 3. Handler Extraction

Axum extracts dependencies:
- `RuniqueContext` - Injected by middleware
- `TemplateContext` - Built from RuniqueEngine
- `ExtractForm<T>` - Parse form from request
- `Path<T>`, `Query<T>` - Extract parameters

### 4. Handler Processing

```rust
async fn handler(
    mut ctx: RuniqueContext,           // From middleware
    template: TemplateContext,         // From engine
    ExtractForm(form): ExtractForm<T>, // Parse form
) -> Response {
    // 1. Validate form
    if !form.is_valid().await { }
    
    // 2. Access database
    let db = ctx.engine.db.clone();
    
    // 3. Process business logic
    // 4. Set flash message
    // 5. Return response
}
```

### 5. Response

HTML, JSON, Redirect, or other response type

### 6. Cleanup

- Flash messages persisted to session
- Response sent to client

---

## Dependency Injection

### Method 1: Via RuniqueContext

```rust
async fn handler(ctx: RuniqueContext) -> Response {
    let engine = &ctx.engine;
    let config = &engine.config;
}
```

### Method 2: Via Extension

```rust
async fn handler(Extension(engine): Extension<Arc<RuniqueEngine>>) -> Response {
    // Direct access to engine
}
```

### Method 3: Via Custom Extractor

```rust
pub struct CurrentUser(pub User);

#[async_trait]
impl FromRequest for CurrentUser {
    async fn from_request(req: &mut Request) -> Result<Self> {
        let session: Session = req.extract().await?;
        let user_id = session.get("user_id")?;
        // Fetch from database
    }
}

async fn handler(user: CurrentUser) -> Response {
    // user.0 contains User
}
```

---

## Data Flow Patterns

### Pattern 1: Read + Render

```
Handler
  ↓
Load from DB (via engine.db)
  ↓
Build context
  ↓
template.render()
  ↓
HTML Response
```

### Pattern 2: Form Submit + Redirect

```
Handler
  ↓
Extract form
  ↓
Validate
  ↓
Save to DB
  ↓
Set flash message
  ↓
Redirect
```

### Pattern 3: API Endpoint

```
Handler
  ↓
Query/Insert/Update/Delete
  ↓
Return JSON
  ↓
JSON Response (with status code)
```

---

## Security Layers

### Layer 1: CSRF Protection
- Token in forms (automatic via `csrf_field` filter)
- Validation middleware (automatic for POST/PUT/PATCH/DELETE)

### Layer 2: Session Security
- HTTP-only cookies
- Secure flag (production)
- 24-hour expiry

### Layer 3: Input Validation
- Form validation (RuniqueForm)
- Database constraints

### Layer 4: Host Validation
- ALLOWED_HOSTS check
- Prevents Host header injection

### Layer 5: Output Encoding
- Template escaping (Tera default)
- CSP nonce generation

---

## Best Practices

### 1. Clone Arc<DatabaseConnection>

```rust
let db = ctx.engine.db.clone();  // Cheap clone of Arc pointer
// Then use &*db in queries
```

### 2. Use Extractors for Validation

```rust
#[derive(Deserialize)]
pub struct UserQuery {
    #[validate(length(min = 1, max = 50))]
    pub username: String,
}

async fn search(Query(q): Query<UserQuery>) -> Response { }
```

### 3. Flash Messages on Redirects

```rust
success!(ctx.flash => "Saved!");
Redirect::to("/dashboard").into_response()
```

### 4. Handle Errors Gracefully

```rust
match operation.await {
    Ok(result) => handle_success(result),
    Err(e) => {
        error!(ctx.flash => format!("Error: {}", e));
        // Return error response
    }
}
```

### 5. Use Type System

```rust
// Good: Type-safe form extraction
ExtractForm(form): ExtractForm<LoginForm>

// Avoid: Loose form data
Form(loose_data): Form<HashMap<String, String>>
```

---

## Performance Considerations

### 1. Connection Pooling
SeaORM manages connection pool automatically

### 2. Arc<DatabaseConnection>
Cheap Arc clones instead of expensive connections

### 3. Template Caching
Tera caches compiled templates automatically

### 4. Session In-Memory Store
MemoryStore for development/testing, upgrade for production

---

## Extending Runique

### Custom Middleware

```rust
use axum::middleware::Next;

pub async fn custom_middleware(
    request: Request,
    next: Next,
) -> Response {
    // Before handler
    let response = next.run(request).await;
    // After handler
    response
}
```

### Custom Extractors

```rust
#[async_trait]
impl FromRequest for CustomData {
    async fn from_request(req: &mut Request) -> Result<Self> {
        // Extract custom data
    }
}
```

### Custom Template Filters

```rust
engine.register_filter("custom_filter", |value, _args| {
    Ok(Value::String("result".to_string()))
});
```

---

## Next Steps

← [**Configuration**](./03-configuration.md) | [**Routing**](./04-routing.md) →
