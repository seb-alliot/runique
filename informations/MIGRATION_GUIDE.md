# Migration Guide

This guide helps you migrate from the original code structure to the Rusti framework.

## Overview

The original code has been refactored into a reusable library (`rusti`) that can be included in any project via Cargo.

## Key Changes

### 1. Project Structure

**Before:**
```
my-app/
├── src/
│   ├── main.rs
│   ├── path.rs
│   ├── settings.rs
│   ├── rust_view.rs
│   └── utils/
│       ├── db/
│       ├── middleware/
│       ├── server/
│       └── ...
└── Cargo.toml
```

**After:**
```
my-app/
├── src/
│   ├── main.rs
│   ├── views.rs        # Your view handlers
|   ├── url.rs          # Your route
│   ├── templates/      # Your templates
│   └── static/         # Your static files
|   └── media/          # Your media files
└── Cargo.toml          # Just depends on rusti
```

### 2. Main Entry Point

**Before:**
```rust
// src/main.rs
mod utils;
mod path;
mod settings;
mod rust_view;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = settings::Settings::default_values();
    let db = utils::db::pool::connected_db::connect_db(&config).await?;
    let _ = utils::server::server::runserver(Arc::new(db), Arc::new(config)).await?;
    Ok(())
}
```

**After:**
```rust
// src/main.rs
use rusti::prelude::*;

mod views;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let router = Router::new()
        .route("/", get(views::index));

    let app = RustiApp::new()
        .with_default_config()
        .with_router(router)
        .build()
        .await?;

    app.run().await?;
    Ok(())
}
```

### 3. Configuration

**Before:**
```rust
// src/settings.rs
impl Settings {
    pub fn default_values() -> Self {
        Settings {
            server: ServerSettings::server(),
            base_dir: base_dir.to_string(),
            // ... lots of manual configuration
        }
    }
}
```

**After:**
```rust
// Configuration is handled by the framework
// Use .env file or custom Settings struct

// .env
IP_SERVER=127.0.0.1
PORT=3000
DEBUG=true
SECRET_KEY=your-secret-key

// Or programmatic:
let config = Settings {
    debug: true,
    // ... only override what you need
    ..Default::default()
};

let app = RustiApp::new()
    .with_config(config)
    .build()
    .await?;
```

### 4. Views/Handlers

**Before:**
```rust
// src/rust_view.rs
pub async fn index(
    Extension(tera): Extension<Arc<Tera>>,
    Extension(config): Extension<Arc<Settings>>,
) -> Response {
    let context = Context::from_serialize(json!({
        "title": "Bienvenue",
        "debug": config.debug,
    })).unwrap_or_default();

    return_render(&tera, "index.html", &context, StatusCode::OK, &config)
}
```

**After:**
```rust
// src/views.rs
use rusti::{
    Context,
    Message,
    Path,
    Response,
    Template,
    json,
};
pub async fn index(
    template: Template,
) -> Response {
    let context = Context::from_serialize(json!({
        "title": "Bienvenue sur Rusti",
        "description": "Un framework web moderne inspiré de Django",

    })).unwrap_or_default();

    template.render("index.html", &context)
}
```

### 5. Routing

**Before:**
```rust
// src/path.rs
pub fn path_url(
    static_files: ServeDir,
    media_files: ServeDir,
    config: Arc<Settings>,
    db: Arc<DatabaseConnection>,
    tera: Arc<Tera>,
) -> Router {
    Router::new()
        .route("/", get(rust_view::index))
        .nest_service(&static_url, static_files)
        // ... lots of manual setup
        .layer(Extension(config))
        .layer(Extension(tera))
}
```

**After:**
```rust
// src/url.rs
use rusti::{Router, urlpatterns};
use crate::views;

pub fn urls() -> Router {
    urlpatterns! {
        "/" => get(views::index), name ="index",
        "/about" => get(views::about), name ="about",
        "/user/{id}/{name}" => get(views::user_profile), name ="user_profile",
    }
}
```

### 6. Database

**Before:**
```rust
// src/utils/db/pool/connected_db.rs
pub async fn connect_db(config: &settings::Settings) -> Result<DatabaseConnection, DbErr> {
    // ... manual connection setup
}
```

**After:**
```rust
// Database connection is handled by the framework
let app = RustiApp::new()
    .with_default_config()
    .with_database_custom()  // Automatically connects using config
    .await?
    .build()
    .await?;

// In your view:
pub async fn users(
    Extension(db): Extension<Arc<DatabaseConnection>>,
) -> Response {
    // Use db here
}
```

### 7. Middleware

**Before:**
```rust
// src/utils/middleware/path_error/middleware_error.rs
pub async fn error_handler_middleware(
    Extension(tera): Extension<Arc<Tera>>,
    Extension(config): Extension<Arc<Settings>>,
    request: Request,
    next: Next,
) -> Response {
    // ... manual error handling
}
```

**After:**
```rust
// Error handling is built into the framework
// Custom middleware can still be added:

async fn my_middleware(
    request: Request,
    next: Next,
) -> Response {
    // Your middleware logic
    next.run(request).await
}

let router = Router::new()
    .route("/", get(index))
    .layer(middleware::from_fn(my_middleware));
```

### 8. Templates

Templates remain largely the same, but the directory structure is simplified:

**Before:**
```
src/utils/frontend/templates/
```

**After:**
```
src/templates/
```

## Step-by-Step Migration

### Step 1: Add Rusti Dependency

```toml
# Cargo.toml
[dependencies]
rusti = { path = "path/to/rusti" }  # Or from crates.io when published
tokio = { version = "1", features = ["full"] }
```

### Step 2: Simplify main.rs

Replace your entire main.rs with the simple builder pattern shown above.

### Step 3: Move Templates

```bash
mv src/utils/frontend/templates/* src/templates/
```

### Step 4: Move Static Files

```bash
mv src/utils/frontend/static/* src/static/
```

### Step 5: Convert Views

1. Create `src/views.rs`
2. Move your handler functions
3. Simplify using `render()` helper
4. Update imports to use `rusti::prelude::*`

### Step 6: Configure Environment

Create `.env`:
```env
HOST=127.0.0.1
PORT=3000
DEBUG=true
SECRET_KEY=your-secret-key
```

### Step 7: Remove Old Code

Delete these directories (now provided by framework):
- `src/utils/`
- `src/settings.rs`
- `src/path.rs`

### Step 8: Test

```bash
cargo run
```

Visit `http://127.0.0.1:3000`

## Benefits of Migration

1. **Less Boilerplate**: ~80% less code in your application
2. **Reusability**: Share the framework across multiple projects
3. **Maintainability**: Framework updates benefit all projects
4. **Best Practices**: Built-in patterns for common tasks
5. **Documentation**: Framework is documented and tested
6. **Community**: Potential for community contributions

## Troubleshooting

### Templates Not Found

Make sure templates are in `src/templates/` and your `.env` has:
```env
# Or set in Settings
templates_dir = "src/templates"
```

### Database Connection Issues

Check your `.env` database configuration:
```env
DB_ENGINE=postgres
POSTGRES_USER=user
POSTGRES_PASSWORD=password
POSTGRES_HOST=localhost
POSTGRES_PORT=5432
POSTGRES_DB=mydb
```

### Static Files Not Serving

Verify the paths in your configuration match your directory structure.

## Need Help?

- Check the [examples](./examples/) directory
- Read the [README](./README.md)
- Open an issue on GitHub

## Gradual Migration

You can migrate gradually by:

1. Start with a new small app using Rusti
2. Port one module at a time from your old app
3. Keep both running until fully migrated

This allows you to learn the framework without disrupting your existing application.
