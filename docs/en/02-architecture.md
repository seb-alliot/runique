# 🏗️ Architecture

## Overview

Runique 1.1.11 is organized into **functional modules** based on responsibility:

```
runique/src/
├── config_runique/          # ⚙️ Configuration & Settings
│   ├── config_struct.rs
│   └── mod.rs
├── data_base_runique/       # 🗄️ ORM & Database
│   ├── config.rs
│   ├── orm_wrapper.rs
│   └── mod.rs
├── formulaire/              # 📋 Form System
│   ├── builder_form/
│   ├── utils/
│   └── mod.rs
├── gardefou/                # 🛡️ Middleware (Security)
│   ├── composant_middleware/
│   ├── utils_gardefou/
│   └── mod.rs
├── macro_runique/           # 🎯 Utility Macros
│   ├── context_macro/
│   ├── flash_message/
│   ├── router/
│   └── mod.rs
├── moteur_engine/           # ⚡ Main Engine
│   ├── engine_struct.rs
│   └── mod.rs
├── request_context/         # 📨 Request Context
│   ├── composant_request/
│   ├── tera_tool/
│   ├── request_struct.rs
│   ├── template_context.rs
│   ├── processor.rs
│   └── mod.rs
├── runique_body/            # 🏭 App Builder
│   ├── composant_app/
│   └── mod.rs
├── utils/                   # 🛠️ Utilities
│   ├── generate_token.rs
│   ├── parse_html.rs
│   ├── csp_nonce.rs
│   └── response_helpers.rs
├── lib.rs
└── prelude.rs
```

---

## Key Concepts

### 1. RuniqueEngine

**Main application state** (replaces the old `AppState`).

```rust
pub struct RuniqueEngine {
    pub db: Arc<DatabaseConnection>,
    pub tera: Arc<Tera>,
    pub config: Arc<RuniqueConfig>,
}
```

**Used by:**
- `RuniqueContext` - Available in handlers
- Axum extension injection

### 2. TemplateContext

**Template context** injected into each handler for rendering.

```rust
pub struct TemplateContext {
    pub context: Context,
    // Access to Tera for rendering
}

// FromRequestParts extractor
pub async fn my_handler(
    mut template: TemplateContext,
) -> Response {
    template.context.insert("title", "Welcome to Runique");
    template.render("view.html")
}
```

### 3. TemplateContext

**Template context** with auto-injection of `debug` and `csrf_token`.

```rust
pub struct TemplateContext {
    pub engine: Arc<RuniqueEngine>,
    pub flash: FlashManager,
    pub csrf_token: String,
}

// Automatic rendering
    context_update!(template => {
        "title" => "Your title here",
        "form" => &form,
    });
    template.render("view.html")

```

### 4. ExtractForm<T>

**Axum extractor** for forms.

```rust
// Automatically:
// 1. Parse the body
// 2. Create a MyForm instance
// 3. Inject the CSRF token
// 4. Fill in the data

pub async fn handler(
    mut template: TemplateContext,
    Prisme(mut form): Prisme<RegisterForm>,
) -> AppResult<Response> {
    let db = template.engine.db.clone();
    if form.is_valid().await {
        Ok()...
    }
}
```

---


**Important:** Middleware declared first = Executed last!

---

## Global State vs Instance

### ❌ Old design (problematic)

```rust
// Shared form in state
struct AppState {
    form: MyForm,  // ⚠️ Race condition!
}

// Request 1 fills the form
// Request 2 fills the form
// Request 3 reads the form → ??? Conflicts!
```

### ✅ New design (correct)

```rust
// Copy per request
pub async fn handler(
    ExtractForm(form): ExtractForm<MyForm>
) -> AppResult<Response> {
    // Each request = isolated form
    // Zero concurrency
}
```

---

## Detailed Modules

### config_runique/
Configuration management:
- Load from `.env`
- Settings validation
- Builder pattern

### data_base_runique/
ORM abstraction:
- SeaORM wrapper
- Objects manager (django-like)
- Database connection management

### formulaire/
Form system:
- RuniqueForm derive macro
- Field types (text, email, textarea, etc.)
- Validation
- Prisme extractor

### middleware/
Security middleware:
- CSRF protection
- ALLOWED_HOSTS validation
- Nonce
- Login required middleware
- Redirect if authenticated

### macro_runique/
Utility macros:
- `context!` - Create template context
- `success!`, `error!`, `warning!`, `info!` - Flash messages
- `urlpatterns!` - Define routes

### moteur_engine/
Main engine:
- RuniqueEngine struct
- Initialization
- Extension injection

### request_context/
Request context:
- RuniqueContext extractor
- TemplateContext extractor
- Message extractor
- Tera tool filters

### runique_body/
Application builder:
- RuniqueApp struct
- `.with_database()`
- `.with_routes()`
- `.build()`
- `.run()`

### utils/
Miscellaneous utilities:
- CSRF token generation
- CSP nonce generation
- Response helpers (json, html, redirect)
- HTML parsing

⚠️  The nonce is manually added to your application builder via:

```rust

.layer(middleware::from_fn_with_state(
    engine.clone(),
    security_headers_middleware,
))
```
---

## Dependency Injection

Via **Axum Extensions**:

```rust
// Registered in middleware:
extension_injection
    .layer(Extension(engine))
    .layer(Extension(tera))
    .layer(Extension(config))
    .layer(Extension(session))

// Used in handlers:
pub async fn handler(
    template: TemplateContext,
) -> AppResult<Response> { }
```

---

## Lifecycle

### App Startup

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Application configuration
    let config = RuniqueConfig::from_env();

    // Database connection
    let db_config = DatabaseConfig::from_env()?.build();
    let db = db_config.connect().await?;

    // Create and launch the application
    RuniqueApp::builder(config)
        .routes(url::routes())
        .with_database(db)
        .build()
        .await?
        .run()
        .await?;

    Ok(())
}

```

### Request Handling

```
1. Middleware (reverse order)
2. Handler called with extractors
3. Handler returns response
4. Middleware (forward order)
5. HTTP response sent
```

---

## Best Practices

1. **Clone Arcs:**
   ```rust
       let db = template.engine.db.clone();
   ```

2. **Forms = copies:**
   ```rust
       let form = template.form::<Form>();

   // No shared state
   ```

3. **Templates auto-context:**
   ```rust
   template.context.insert("data", value);
   template.render("page.html")
   // csrf_token auto-injected into context
   ```

4. **Flash messages:**
   ```rust
   Message(mut messages): Message,
   messages.success(format!("Welcome {}, your account has been created!", user.username));
   ```

5. **Middleware order:**
   ```rust
   // Declared first = Executed last!
   .layer(a)  // Executed 3rd
   .layer(b)  // Executed 2nd
   .layer(c)  // Executed 1st (entry point)
   ```

---

## Next Steps

→ [**Configuration**](https://github.com/seb-alliot/runique/blob/main/docs/en/03-configuration.md)
