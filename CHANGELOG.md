# Changelog - Runique Framework

All notable changes to Runique are documented in this file.

---

## [1.1.1] - 24 January 2026

### Fixed
- Documentation links in README files converted to absolute GitHub URLs for crates.io compatibility
- Both English (README.md) and French (README.fr.md) READMEs updated

### Documentation
- All cross-references now use `https://github.com/seb-alliot/runique/blob/main/docs/...` format
- Links work consistently on GitHub, crates.io, and docs.rs

---

## [1.1.0] - 24 January 2026

### Major Changes - Complete Architecture Refactoring

#### Module Restructuring
- **Database module**: `database/orm` → `db/`
  - `db/config.rs` - Database configuration & connection pooling
  - `db/objects.rs` - Django-like query builder pattern
  - `db/query.rs` - Query response handling
- **Middleware auth**: `login_requiert` → `auth`
  - All authentication middleware consolidated in `middleware/auth.rs`
  - Functions: `login_required()`, `redirect_if_authenticated()`, `load_user_middleware()`, `CurrentUser` struct
- **Config reorganization**: Split into sub-modules
  - `config/app.rs` - Main app config
  - `config/server.rs` - Server settings
  - `config/security.rs` - Security configuration
  - `config/settings.rs` - General settings
  - `config/static_files.rs` - Static file configuration
  - `config/router.rs` - Routing config

#### New Form System (Complete Rewrite)
- **New field types**:
  - `TextField` (text, email, password, URL)
  - `NumericField` (integer, decimal)
  - `BooleanField` (checkbox, toggle)
  - `ChoiceField` (select, radio options)
  - `DateTimeField` (date, time, datetime)
  - `FileField` (file uploads)
  - `HiddenField` (hidden form fields)
  - `ColorField` (color picker)
  - `JSONField` (JSON input)
  - `SlugField` (slug generation)

- **Form framework**:
  - `FormField` trait for custom fields
  - `GenericField` generic implementation
  - `Forms` manager for form collections
  - `FieldKind` enum for field categorization

- **Validation system**: Prisme framework integration
  - `csrf_gate.rs` - CSRF token validation
  - `rules.rs` - Validation rules
  - `aegis.rs` - Security checks
  - `sentinel.rs` - Sentinel pattern

- **Templates**:
  - Unified form field templates in `templates/field_html/`
  - Supports all field types with consistent rendering
  - Custom HTML classes and error display

#### Middleware Improvements
- **CSRF Protection**: Enhanced token generation and validation
- **CSP (Content-Security-Policy)**: Improved header management with nonce support
- **Allowed Hosts**: Host validation with security edge case fixes
- **Cache Middleware**: Cache control headers
- **Error Handling**: Improved error middleware
- **Sanitizer**: XSS protection with HTML parsing

#### Prelude Simplification
```rust
// Before (1.0.86)
use runique::forms::Forms;
use runique::forms::fields::TextField;
use runique::context;
use runique::middleware::login_requiert;

// After (1.1.0)
use runique::prelude::*;  // Everything in one import!
```

**Prelude exports**:
- App: `RuniqueApp`, `RuniqueAppBuilder`, `RuniqueEngine`, `RuniqueConfig`
- Forms: All field types, `Forms` manager, validation traits
- Middleware: Auth, CSRF, CSP, allowed hosts
- Utils: Response helpers, CSRF utils, CSP nonce generation
- Context: `AppError`, `AppResult`, template context

#### Database/ORM
- SeaORM integration simplified
- `impl_objects!` macro for Django-like managers
- Query builder pattern with chainable API

#### Demo Application
- **Relocation**: `examples/demo-app/` → `demo-app/`
- **Expanded examples**:
  - User authentication example
  - Blog post creation
  - File uploads
  - Form validation
  - Database migrations

- **Assets added**:
  - `demo-app/media/toshiro.avif` - Example image
  - `demo-app/media/favicon/favicon.ico` - Favicon
  - Enhanced CSS for form styling
  - JavaScript for form interaction

#### Documentation
- **Complete rewrite**: 20+ pages
  - 🇬🇧 English documentation in `docs/en/`
  - 🇫🇷 French documentation in `docs/fr/`

- **10 Main Topics**:
  1. Installation - Setup & configuration
  2. Architecture - Framework design
  3. Configuration - App & server settings
  4. Routing - URL patterns & routes
  5. Forms - Field types & validation
  6. Templates - Tera integration
  7. ORM - Database queries
  8. Middleware - Security & utilities
  9. Flash Messages - Session notifications
  10. Examples - Real-world use cases

- **Guides included**:
  - Getting started tutorial
  - Migration guide from 1.0.x
  - API reference
  - Best practices

#### Breaking Changes
- **Import paths changed**: Use `runique::prelude::*` instead of specific imports
- **Middleware auth**: `login_requiert` module removed, use `runique::middleware::auth`
- **Database module**: `database`/`orm` → `db`
- **Form API**: Old form system replaced with new field-based system
- **Tera functions**: Reorganized in `context/tera/` with same functionality
- **Response types**: Consolidated response helpers in `utils/response_helpers.rs`

#### Migration Guide for 1.0.86 → 1.1.0

**Step 1: Update imports**
```rust
// Old
use runique::middleware::login_requiert::{login_required, CurrentUser};
use runique::forms::Forms;

// New
use runique::prelude::*;
```

**Step 2: Middleware references**
```rust
// Old
.layer(axum::middleware::from_fn(runique::middleware::login_requiert::login_required))

// New
.layer(axum::middleware::from_fn(runique::middleware::auth::login_required))
```

**Step 3: Database access**
```rust
// Old
use runique::database::config::DatabaseConfig;

// New
use runique::prelude::DatabaseConfig;  
// or
use runique::db::config::DatabaseConfig;
```

**Step 4: Forms**
```rust
// Old (legacy)
form.add_field("username", TextField { ... })

// New
let field = TextField::text("username")
    .label("Username")
    .required("This field is required");
form.field(&field);
```

#### Code Quality
- ✅ **Clippy**: Zero warnings (`-D warnings`)
- ✅ **Tests**: 36/36 passing (20 unit + 16 integration)
- ✅ **Doctests**: 30/30 passing
- ✅ **Type Safety**: Full type checking, no `unsafe` code
- ✅ **Documentation**: Comprehensive examples in all modules

#### Dependencies
- **axum**: 0.8.7 (HTTP framework)
- **tokio**: 1.x (async runtime)
- **sea-orm**: 2.0-rc.28 (database ORM)
- **tera**: 1.20.1 (templates)
- **tower**: 0.5.3 (middleware)

### Deprecated (Removed)
- `runique::middleware::login_requiert` module
- Old form field system
- Legacy database module structure
- Old template function paths (moved to `context/tera/`)

### Removed Files
- `informations/` directory (documentation moved to `docs/`)
- `examples/demo-app/` (relocated to `demo-app/`)
- Old middleware file structure

### Added Files & Directories
- `docs/` - Complete documentation (EN & FR)
- `demo-app/` - Enhanced example application
- `runique/src/app/builder.rs` - Application builder
- `runique/src/app/templates.rs` - Template loader
- `runique/src/config/` - Configuration modules
- `runique/src/context/` - Request context
- `runique/src/db/` - Database integration
- `runique/src/engine/` - Core engine
- `runique/src/flash/` - Flash messages
- `runique/src/forms/fields/` - All field types
- `runique/src/macros/` - Routing & convenience macros
- `runique/src/middleware/` - Reorganized middleware
- `runique/src/utils/` - Utilities consolidated

### Performance
- No performance regressions
- Prelude optimization reduces compile-time overhead of imports
- Form system maintains zero-cost abstractions

---

## [1.0.86] - Previous Release

### Final 1.0.x Stable
- Last version in 1.0.x series
- Full feature parity with 1.1.0 features
- Different module structure & import paths
- Legacy form system
- Maintained for backward compatibility

### Notable 1.0.x Features (now in 1.1.0)
- Django-inspired web framework
- Type-safe forms
- SeaORM integration
- Security middleware (CSRF, CSP)
- Flash message system
- Tera template engine
- Comprehensive test suite

---

## Version Comparison

| Aspect | 1.0.86 | 1.1.0 |
|--------|--------|-------|
| **Import System** | Scattered | Unified prelude |
| **Module Paths** | `login_requiert`, `database` | `auth`, `db` |
| **Form System** | Legacy | Complete rewrite |
| **Middleware** | Separate files | Consolidated |
| **Documentation** | Basic | Comprehensive (20+ pages) |
| **Examples** | Minimal | Full demo-app |
| **Tests** | Core tests | 36 tests |
| **Type Safety** | Good | Excellent |
| **API Stability** | Stable | Stable + Improved |

---

## Upgrade Path

### From 1.0.86 to 1.1.0+

**Difficulty**: Medium (breaking changes, but well-documented)

**Time to migrate**: 1-4 hours depending on project size

**Key steps**:
1. Update all imports to use prelude
2. Replace middleware references
3. Update form definitions (new API)
4. Update database module references
5. Test thoroughly

**Migration guide**: See [Migration from 1.0.x](docs/en/README.md#migration-guide)

---

## Future Roadmap

### Planned for 1.2.0+
- WebSocket support
- GraphQL integration
- Enhanced caching
- Performance optimizations
- Additional middleware

### Community Contributions Welcome
- Report bugs on GitHub
- Suggest features
- Contribute code
- Improve documentation

---

**Latest Version**: 1.1.1
**Release Date**: 24 January 2026
**License**: MIT
