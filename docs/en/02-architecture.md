---
# 🏗️ Architecture

## Overview

Runique is organized into **functional modules** based on responsibility:

runique/src/
├── app/                    # 🏭 App Builder, Templates & Smart Builder
│   ├── builder.rs          # 🧠 RuniqueAppBuilder with slots
│   ├── error_build.rs      # Build errors
│   ├── templates.rs        # TemplateLoader (Tera)
│   └── staging/            # Staging structs
│       ├── core_staging.rs
│       │   ├── middleware_staging.rs
│       │   └── static_staging.rs
│       └── error_build.rs  # BuildError & CheckReport
├── config/                 # ⚙️ Configuration & Settings
├── context/                # 📨 Request Context & Tera tools
│   ├── request.rs          # Request struct (extractor)
│   └── tera/               # Tera filters and functions
├── db/                     # 🗄️ ORM & Database
├── engine/                 # ⚡ RuniqueEngine
├── errors/                 # 🚨 Error handling
├── flash/                  # 💬 Flash messages
├── forms/                  # 📋 Form system
├── macros/                 # 🎯 Utility macros
│   ├── context_macro/      # context!, context_update!
│   ├── flash_message/      # success!, error!, info!, warning!, flash_now!
│   └── router/             # urlpatterns!, view!, impl_objects!
├── middleware/             # 🛡️ Middleware (Security)
│   └── security/           # CSRF, CSP, Host, Cache, Error Handler
├── utils/                  # 🛠️ Utilities
├── lib.rs
└── prelude.rs

---

## Key Concepts

### 1. RuniqueEngine

**Main shared state** of the application:

```rust
pub struct RuniqueEngine {
    pub db: Arc<DatabaseConnection>,
    pub tera: Arc<Tera>,
    pub config: Arc<RuniqueConfig>,
}
```

Injected as an Axum Extension, accessible in every handler via `request.engine`.

### 2. Request — Main Extractor

`Request` is Runique's central extractor. It replaces the old `TemplateContext` and contains everything needed:

```rust
pub struct Request {
    pub engine: AEngine,       // Arc<RuniqueEngine>
    pub session: Session,      // Tower-sessions session
    pub notices: Message,      // Flash messages
    pub csrf_token: CsrfToken, // CSRF token
    pub context: Context,      // Tera context
    pub method: Method,        // HTTP method
}
```

**Usage in a handler:**

```rust
pub async fn index(mut request: Request) -> AppResult<Response> {
    context_update!(request => {
        "title" => "Home",
    });
    request.render("index.html")
}
```

**Methods:**

* `request.render("template.html")` — Render with the current context
* `request.is_get()` / `request.is_post()` — Check HTTP method

### 3. Prisme`<T>` — Form Extractor

```rust
pub async fn handler(
    mut request: Request,
    Prisme(mut form): Prisme<RegisterForm>,
) -> AppResult<Response> {
    if request.is_post() && form.is_valid().await {
        let user = form.save(&request.engine.db).await?;
        success!(request.notices => "User created!");
        return Ok(Redirect::to("/").into_response());
    }

    context_update!(request => {
        "form" => &form,
    });
    request.render("form.html")
}
```

Automatically:

1. Parses the request body
2. Creates a form instance
3. Injects the CSRF token
4. Populates submitted data

---

## Rust Macros

Runique provides a set of macros to simplify development:

### Context Macros

| Macro             | Description              | Example                                          |
| ----------------- | ------------------------ | ------------------------------------------------ |
| `context!`        | Create a Tera context    | `context!("title" => "Page")`                    |
| `context_update!` | Add to a Request context | `context_update!(request => { "key" => value })` |

### Flash Message Macros

| Macro        | Description                    | Example                                  |
| ------------ | ------------------------------ | ---------------------------------------- |
| `success!`   | Success message (session)      | `success!(request.notices => "OK!")`     |
| `error!`     | Error message (session)        | `error!(request.notices => "Error")`     |
| `info!`      | Info message (session)         | `info!(request.notices => "Info")`       |
| `warning!`   | Warning (session)              | `warning!(request.notices => "Warning")` |
| `flash_now!` | Immediate message (no session) | `flash_now!(error => "Errors")`          |

### Routing Macros

| Macro           | Description                    | Example                                           |
| --------------- | ------------------------------ | ------------------------------------------------- |
| `urlpatterns!`  | Define routes with names       | `urlpatterns!("/" => view!{...}, name = "index")` |
| `view!`         | Handler for all HTTP methods   | `view!{ GET => handler, POST => handler2 }`       |
| `impl_objects!` | Django-like manager for SeaORM | `impl_objects!(Entity)`                           |

### Error Macros

| Macro              | Description                            |
| ------------------ | -------------------------------------- |
| `impl_from_error!` | Generates `From<Error>` for `AppError` |

---

## Tera Tags and Filters

### Django-like Tags (syntactic sugar)

| Tag                    | Translated to                              | Description            |
| ---------------------- | ------------------------------------------ | ---------------------- |
| `{% static "..." %}`   | `{{ "..." \| static }}`                    | Static file URL        |
| `{% media "..." %}`    | `{{ "..." \| media }}`                     | Media file URL         |
| `{% csrf %}`           | `{% include "csrf/..." %}`                 | Hidden CSRF field      |
| `{% messages %}`       | `{% include "message/..." %}`              | Display flash messages |
| `{% csp_nonce %}`      | `{% include "csp/..." %}`                  | CSP nonce attribute    |
| `{% link "name" %}`    | `{{ link(link='name') }}`                  | Named route URL        |
| `{% form.xxx %}`       | `{{ xxx \| form \| safe }}`                | Full form rendering    |
| `{% form.xxx.field %}` | `{{ xxx \| form(field='field') \| safe }}` | Render a single field  |

### Tera Filters

| Filter           | Description                      |
| ---------------- | -------------------------------- |
| `static`         | App static URL prefix            |
| `media`          | App media URL prefix             |
| `runique_static` | Internal framework static assets |
| `runique_media`  | Internal framework media         |
| `form`           | Render full form or field        |
| `csrf_field`     | Generates a hidden CSRF input    |

### Tera Functions

| Function           | Description                         |
| ------------------ | ----------------------------------- |
| `csrf()`           | Generates a CSRF field from context |
| `nonce()`          | Returns CSP nonce                   |
| `link(link='...')` | Named URL resolution                |

---

## Middleware Stack

Runique applies middlewares in an **optimal order** via the slot system:

```

Incoming request
    ↓
1. Extensions (slot 0)     → Inject Tera, Config, Engine
2. ErrorHandler (slot 10)  → Capture and render errors
3. Custom (slot 20+)       → Custom middlewares
4. CSP (slot 30)           → Content Security Policy & headers
5. Cache (slot 40)         → No-cache in development
6. Session (slot 50)       → Session management
7. CSRF (slot 60)          → CSRF protection
8. Host (slot 70)          → Validate Allowed Hosts
    ↓
Handler (your code)
    ↓
Outgoing response (middlewares in reverse order)
```

> 💡 **Important**: With Axum, the last `.layer()` applied is executed first. The Smart Builder manages this order automatically.

---

## Dependency Injection

Via **Axum Extensions**, automatically injected by the Extensions middleware:

```rust
// Automatically registered by the builder:
// Extension(engine)  → Arc<RuniqueEngine>
// Extension(tera)    → Arc<Tera>
// Extension(config)  → Arc<RuniqueConfig>

// Accessible in handlers via Request:
pub async fn handler(request: Request) -> AppResult<Response> {
    let db = request.engine.db.clone();
    let config = &request.engine.config;
    // ...
}
```

---

## Request Lifecycle

```
1. HTTP request arrives
2. Middlewares traversed (slot order)
3. Extensions injected (Engine, Tera, Config)
4. Session loaded, CSRF checked
5. Handler called with extractors (Request, Prisme<T>)
6. Handler returns AppResult<Response>
7. Middlewares traversed in reverse
8. HTTP response sent
```

---

## Best Practices

1. **Clone Arcs:**

   ```rust
   let db = request.engine.db.clone();
   ```

2. **Forms = per-request copies:**

   ```rust
   Prisme(mut form): Prisme<MyForm>
   // Each request = isolated form, no concurrency
   ```

3. **Use `context_update!` for context:**

   ```rust
   context_update!(request => {
       "title" => "My Page",
       "data" => &my_data,
   });
   ```

4. **Flash messages for redirects:**

   ```rust
   success!(request.notices => "Action successful!");
   return Ok(Redirect::to("/").into_response());
   ```

5. **flash_now! for direct renders:**

   ```rust
   context_update!(request => {
       "messages" => flash_now!(error => "Validation error"),
   });
   ```

---

## Next Steps


← [Installation](https://github.com/seb-alliot/runique/blob/main/docs/en/01-installation.md) | [**Configuration**](https://github.com/seb-alliot/runique/blob/main/docs/en/03-configuration.md) →

---