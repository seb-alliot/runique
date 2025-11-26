# Project Structure

Complete overview of the Rusti framework project structure.

## Root Structure

```
rusti-framework/
├── Cargo.toml                  # Workspace configuration
├── .gitignore                  # Git ignore rules
├── LICENSE-MIT                 # MIT License
├── README.md                   # Main documentation
├── CHANGELOG.md                # Version history
├── CONTRIBUTING.md             # Contribution guidelines
├── MIGRATION_GUIDE.md          # Migration from old code
├── TUTORIAL.md                 # Step-by-step tutorial
├── PROJECT_STRUCTURE.md        # This file
│
├── rusti/                      # Core framework library
│   ├── Cargo.toml              # Library dependencies
│   ├── README.md               # Library documentation
│   └── src/
│       ├── lib.rs              # Library entry point & prelude
│       ├── app.rs              # RustiApp & builder
│       ├── config.rs           # Settings & configuration
│       ├── database.rs         # Database connection (ORM feature)
│       ├── error.rs            # Error types & handling
│       ├── response.rs         # Response helpers
│       ├── server.rs           # Server running logic
│       ├── template.rs         # Template utilities
│       └── middleware/
│           ├── mod.rs
│           └── error_handler.rs
│
└── examples/                   # Example applications
    └── demo-app/
        ├── Cargo.toml
        ├── .env.example
        └── src/
            ├── main.rs
            ├── views.rs
            ├── templates/
            │   ├── base.html
            │   ├── index.html
            │   └── about.html
            └── static/
                └── css/
                    └── main.css
```

## Module Breakdown

### Core Library (`rusti/`)

#### `lib.rs`
- Main entry point
- Exports all public API
- Defines `prelude` module for easy imports

#### `app.rs`
- `RustiApp` struct - main application container
- `RustiAppBuilder` - builder pattern for app creation
- Holds shared state: config, templates, database

#### `config.rs`
- `Settings` struct - Django-inspired configuration
- `ServerSettings` - server-specific config
- `DatabaseSettings` - database configuration (ORM feature)
- Environment variable loading

#### `database.rs` (ORM feature only)
- Database connection utilities
- Connection pooling
- Password masking for logs
- Validation

#### `error.rs`
- `RustiError` enum - framework error types
- `RustiResult<T>` type alias
- Error to Response conversion
- User-friendly error pages

#### `response.rs`
- `RustiTemplate` trait - template rendering helpers
- `RustiResponse` struct - response wrapper
- Helper functions: `render()`, `render_with_status()`

#### `server.rs`
- Server initialization
- Router configuration
- Middleware setup
- Static file serving
- Graceful shutdown
- Logging configuration

#### `template.rs`
- Template engine utilities
- Re-exports Tera types

#### `middleware/`
- `error_handler.rs` - Error handling middleware
- Custom 404/500 pages
- Debug vs production error display

### Example Application (`examples/demo-app/`)

#### `main.rs`
- Application entry point
- Router definition
- App initialization

#### `views.rs`
- View handlers
- Request/response logic
- Template rendering

#### `templates/`
- Base template (layout)
- Page templates
- Tera syntax (Django/Jinja2-like)

#### `static/`
- CSS files
- JavaScript files (can be added)
- Images (can be added)

## Key Concepts

### Builder Pattern

```rust
RustiApp::new()
    .with_default_config()
    .with_router(router)
    .with_database().await?
    .build().await?
    .run().await?
```

### Configuration Layers

1. Default values in `Settings::default()`
2. Environment variables (`.env` file)
3. Programmatic override

### Request Flow

```
HTTP Request
    ↓
Server (Axum)
    ↓
Middleware (Tower)
    ↓
Router (Your routes)
    ↓
View Handler (Your code)
    ↓
Template Rendering (Tera)
    ↓
Response
    ↓
Middleware (Error handling)
    ↓
HTTP Response
```

### Error Handling Flow

```
Error occurs
    ↓
Converted to RustiError
    ↓
Error middleware intercepts
    ↓
Debug mode? 
    ├─ Yes → Detailed error page
    └─ No → Simple error page
```

## Feature Flags

### `default`
Includes ORM support.

### `orm`
Enables SeaORM database integration:
- `database.rs` module
- Database connection in `RustiApp`
- Database-related configuration

### `full`
Enables all features.

## Dependencies

### Core Dependencies
- `axum` - Web framework
- `tokio` - Async runtime  
- `tera` - Template engine
- `tower` - Middleware
- `tower-http` - HTTP utilities
- `serde` - Serialization

### Optional Dependencies
- `sea-orm` - ORM (with `orm` feature)

### Development Dependencies
- `tokio-test` - Testing utilities

## File Naming Conventions

- `snake_case` for Rust files
- `kebab-case` for directories
- `lowercase` for templates
- All caps for markdown docs (README, etc.)

## Important Files

### Must Configure
- `.env` - Environment variables (create from `.env.example`)

### Must Have
- `src/main.rs` - Entry point
- `src/templates/` - Template directory
- `Cargo.toml` - Dependencies

### Should Create
- `src/views.rs` or `src/views/` - View handlers
- `src/static/` - Static files
- `src/templates/base.html` - Base template

## Testing Structure

```
rusti/
├── src/
│   └── *.rs (unit tests inline)
└── tests/
    └── integration/ (future integration tests)
```

## Documentation

### For Users
- `README.md` - Overview & quick start
- `TUTORIAL.md` - Step-by-step guide
- `MIGRATION_GUIDE.md` - Migrating from old code
- Cargo docs - `cargo doc --open`

### For Contributors
- `CONTRIBUTING.md` - How to contribute
- `CHANGELOG.md` - Version history
- Code comments - Rustdoc format

## Deployment Structure

When deploying:

```
production/
├── Cargo.toml
├── .env (production values)
├── src/
│   ├── main.rs
│   ├── views.rs
│   ├── templates/
│   └── static/
└── target/
    └── release/
        └── myapp (binary)
```

## Development Workflow

1. Clone repository
2. `cd` into workspace root
3. `cargo build --workspace`
4. `cd examples/demo-app`
5. `cargo run`
6. Edit code
7. Changes auto-reload (in debug mode)

## Creating New Applications

```bash
# Option 1: Start from example
cp -r examples/demo-app my-new-app
cd my-new-app
# Edit Cargo.toml, src/main.rs, etc.

# Option 2: Start from scratch
cargo new my-new-app
cd my-new-app
# Add rusti dependency
# Create src/templates/, src/static/
# Write main.rs with RustiApp
```

## Best Practices

### Directory Structure
```
src/
├── main.rs              # Entry point only
├── views/               # Split views into modules
│   ├── mod.rs
│   ├── home.rs
│   └── blog.rs
├── models/              # If using ORM
│   ├── mod.rs
│   └── user.rs
├── templates/
│   ├── base.html
│   ├── home/
│   └── blog/
└── static/
    ├── css/
    ├── js/
    └── img/
```

### Configuration
- Use `.env` for development
- Use environment variables in production
- Never commit `.env` to git
- Keep sensitive data out of code

### Templates
- Use template inheritance (`{% extends %}`)
- Break templates into reusable components
- Keep logic in views, not templates

### Views
- Keep views small and focused
- One view per route
- Group related views in modules
- Extract common logic into utilities

## Future Expansion

Planned structure additions:

```
rusti/
└── src/
    ├── forms.rs         # Form handling (future)
    ├── auth.rs          # Authentication (future)
    ├── admin.rs         # Admin interface (future)
    └── cli/             # CLI tools (future)
```

## Quick Reference

### Import Pattern
```rust
use rusti::prelude::*;  // Most common imports
```

### Minimal App
```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    RustiApp::new()
        .with_default_config()
        .build().await?
        .run().await?;
    Ok(())
}
```

### With Database
```rust
RustiApp::new()
    .with_default_config()
    .with_database().await?
    .build().await?
    .run().await?
```

### Custom Config
```rust
let config = Settings { /* ... */ };
RustiApp::new()
    .with_config(config)
    .build().await?
```

## Summary

The Rusti framework provides:

✅ Clear separation between framework and application code  
✅ Reusable library pattern  
✅ Django-inspired architecture  
✅ Type-safe, performant Rust implementation  
✅ Easy to understand and extend  
✅ Comprehensive documentation  
✅ Working examples  

The structure supports both small single-file apps and large modular applications.
