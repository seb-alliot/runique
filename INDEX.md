# 📁 Project Structure Guide

Navigate the Runique Framework codebase.

## Root Level

```
runique/
├── README.md                 # Main documentation (English)
├── README.fr.md              # French documentation
├── CHANGELOG.md              # Version history & release notes
├── SECURITY.md               # Security policy & guidelines
├── LICENSE-MIT.md            # MIT License
├── Cargo.toml                # Workspace configuration
├── Cargo.lock                # Dependency lock file
├── audit.toml                # Security audit configuration
├── audit.ps1                 # Audit script
│
├── docs/                     # Complete documentation
│   ├── README.md             # Documentation hub
│   ├── en/                   # English guides (10 sections)
│   └── fr/                   # French guides (10 sections)
│
├── runique/                  # Main framework crate
│   ├── Cargo.toml
│   ├── src/
│   ├── tests/
│   └── derive_form/          # Procedural macros
│
├── demo-app/                 # Example application
│   ├── Cargo.toml
│   ├── src/
│   ├── migration/            # Database migrations
│   ├── static/               # CSS & JavaScript
│   ├── templates/            # HTML templates
│   └── media/                # Images & assets
│
└── target/                   # Build output (git-ignored)
```

## Framework Core (`runique/src/`)

### Application Entry Point
- **`lib.rs`** - Module exports, public API, prelude definition

### Modules

#### `app/` - Application Lifecycle
- **`builder.rs`** - `RuniqueApp` builder with middleware setup
- **`mod.rs`** - App module exports
- **`templates.rs`** - Template loader & Tera engine initialization

#### `config/` - Configuration System
- **`app.rs`** - Main config struct (`RuniqueConfig`)
- **`server.rs`** - Server settings (IP, port, domain)
- **`security.rs`** - Security configuration (CSRF, CSP)
- **`settings.rs`** - General app settings
- **`static_files.rs`** - Static file paths configuration
- **`router.rs`** - Route configuration
- **`mod.rs`** - Config module exports

#### `context/` - Request Context
- **`error.rs`** - Error types and handling (`AppError`, `AppResult`)
- **`template.rs`** - Template context builder
- **`request/`**
  - `extension.rs` - Request extensions
  - `extractor.rs` - Custom extractors
  - `template.rs` - Template extraction
  - `mod.rs` - Request module exports
- **`tera/`** - Tera template functions
  - `csp.rs` - CSP nonce injection
  - `csrf.rs` - CSRF token injection
  - `form.rs` - Form field rendering
  - `static_tera.rs` - Static file URL filters
  - `url.rs` - URL routing filters
  - `mod.rs` - Tera module exports
- **`mod.rs`** - Context module exports

#### `db/` - Database & ORM
- **`config.rs`** - Database configuration & connection pooling
- **`objects.rs`** - Django-like query builder pattern (`impl_objects!`)
- **`query.rs`** - Query response handling
- **`mod.rs`** - DB module exports

#### `engine/` - Core Engine
- **`core.rs`** - Main engine logic
- **`mod.rs`** - Engine module exports

#### `flash/` - Flash Messages
- **`flash_struct.rs`** - Flash message data structure
- **`flash_manager.rs`** - Message management & session handling
- **`mod.rs`** - Flash module exports

#### `forms/` - Form System
- **`base.rs`** - Base form structure
- **`field.rs`** - Form field trait definitions
- **`generic.rs`** - Generic field implementation
- **`manager.rs`** - Form manager & collection
- **`fields/`** - Specific field types
  - `text.rs` - Text, email, password, URL fields
  - `number.rs` - Integer & decimal fields
  - `boolean.rs` - Checkbox & toggle fields
  - `choice.rs` - Select & radio fields
  - `datetime.rs` - Date, time, datetime fields
  - `file.rs` - File upload fields
  - `special.rs` - Color, JSON, slug fields
  - `hidden.rs` - Hidden fields
  - `mod.rs` - Fields module exports
- **`options/`** - Field options
  - `config.rs` - Field configuration
  - `length.rs` - Length validation options
  - `bool_choice.rs` - Boolean choice options
  - `mod.rs` - Options module exports
- **`utils/`** - Form utilities
  - `extractor.rs` - Form extractor for handlers
  - `sanitizer.rs` - HTML sanitization
  - `prisme/` - Prisme validation framework
    - `csrf_gate.rs` - CSRF sentinel validation
    - `rules.rs` - Validation rules
    - `aegis.rs` - Security checks
    - `sentinel.rs` - Sentinel pattern
    - `mod.rs` - Prisme module exports
  - `mod.rs` - Utils module exports
- **`mod.rs`** - Forms module exports

#### `macros/` - Convenient Macros
- **`router.rs`** - URL routing macro (`route!`, `get!`, `post!`, etc.)
- **`context.rs`** - Context extraction macro
- **`flash.rs`** - Flash message macros (`success!`, `error!`, `warning!`, `info!`)
- **`db_query.rs`** - Database query helpers
- **`helper.rs`** - General helper macros
- **`impl_objects.rs`** - Django-like object manager pattern
- **`register_name_url.rs`** - URL reverse mapping
- **`get_post.rs`** - HTTP method-specific routing
- **`mod.rs`** - Macros module exports (with `#[macro_use]`)

#### `middleware/` - Security & Utility Middleware
- **`auth.rs`** - Authentication middleware (`login_required`, `redirect_if_authenticated`)
- **`csrf.rs`** - CSRF token validation middleware
- **`csp.rs`** - Content-Security-Policy header injection
- **`cache.rs`** - Caching middleware
- **`config.rs`** - Middleware configuration
- **`error.rs`** - Error handling middleware
- **`sanitizer.rs`** - XSS sanitization middleware
- **`allowed_hosts.rs`** - Host validation middleware
- **`mod.rs`** - Middleware module exports

#### `utils/` - Utility Functions
- **`response_helpers.rs`** - Response builders (JSON, HTML, redirect, text)
- **`csrf.rs`** - CSRF token generation & validation
- **`csp_nonce.rs`** - CSP nonce generation
- **`parse_html.rs`** - HTML parsing utilities
- **`mod.rs`** - Utils module exports

#### `bin/` - CLI Tools
- **`runique.rs`** - Command-line interface for project scaffolding

#### `composant-bin/` - Code Templates
- **`code/`** - Generated code templates for new projects
- **`css/`** - Default CSS files
- **`image/`** - Default images
- **`template/`** - Default HTML templates
- **`readme/`** - README templates

#### `static/` - Framework Static Files
- **`css/`** - Framework CSS (error pages, base styles)
- **`js/`** - Framework JavaScript (CSRF tokens, etc.)

#### `templates/` - Framework Templates
- **`base_form.html`** - Base form template
- **`errors/`** - Error page templates
- **`field_html/`** - Form field HTML templates
- **`message/`** - Flash message templates
- **`csrf/`** - CSRF token templates
- **`runique_index/`** - Framework index page

### Tests
- **`tests/integration_tests.rs`** - Integration test suite
- **`tests/README_INTEGRATION.md`** - Test documentation

### Procedural Macros (`derive_form/`)
- **`src/lib.rs`** - Macro entry point
- **`src/forms.rs`** - Form derivation macro
- **`src/helpers.rs`** - Helper functions
- **`src/models.rs`** - Model field introspection

## Example Application (`demo-app/`)

### Structure
- **`src/main.rs`** - Application entry point
- **`src/prelude.rs`** - Local re-exports
- **`src/forms.rs`** - Form definitions
- **`src/views.rs`** - View handlers
- **`src/url.rs`** - URL routing
- **`src/models/`** - Database models
  - `users.rs` - User model
  - `blog.rs` - Blog model
  - `model_derive.rs` - Derived fields
  - `test.rs` - Test models
  - `mod.rs` - Models module
- **`migration/`** - Database migrations
- **`templates/`** - HTML templates
- **`static/`** - CSS & JavaScript
- **`media/`** - Images & assets

## Documentation (`docs/`)

### Structure
- **`README.md`** - Documentation hub
- **`en/`** - English documentation
  - `01-installation.md`
  - `02-architecture.md`
  - `03-configuration.md`
  - `04-routing.md`
  - `05-forms.md`
  - `06-templates.md`
  - `07-orm.md`
  - `08-middleware.md`
  - `09-flash-messages.md`
  - `10-examples.md`
  - `README.md`
- **`fr/`** - French documentation (same structure)

## Configuration Files

- **`Cargo.toml`** (root) - Workspace definition & shared dependencies
- **`Cargo.toml`** (runique) - Framework crate configuration
- **`Cargo.toml`** (demo-app) - Example app configuration
- **`Cargo.toml`** (derive_form) - Macro crate configuration
- **`.gitignore`** - Git exclusions
- **`audit.toml`** - Cargo audit configuration

## Build & Output

- **`target/debug/`** - Debug build output
- **`target/release/`** - Release build output
- **`target/doc/`** - Generated documentation

## Quick Navigation

### For Framework Development
1. Start: `runique/src/lib.rs`
2. App: `runique/src/app/builder.rs`
3. Forms: `runique/src/forms/manager.rs`
4. Middleware: `runique/src/middleware/`
5. Tests: `runique/tests/integration_tests.rs`

### For Users
1. Installation: `docs/en/01-installation.md`
2. Examples: `demo-app/src/`
3. API: https://docs.rs/runique

### For Contributors
1. Code: `runique/src/`
2. Tests: `runique/tests/`
3. Docs: `docs/`
4. Changelog: `CHANGELOG.md`

---

**Version**: 1.1.1
**Last Updated**: 24 janvier 2026
