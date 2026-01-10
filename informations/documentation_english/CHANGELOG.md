# Changelog - Runique Framework

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.86/),
and this project adheres to [Semantic Versioning](https://semver.org/).

---

## [1.0.86] - 2025-01-15

### Initial Release

#### Added

**Core Framework**
- `RuniqueApp` structure with builder pattern
- Configuration via `Settings` (default, .env, builder)
- Environment variable support
- Logging with `tracing`

**Routing**
- Django-inspired `urlpatterns!` macro
- Reverse routing support
- Route naming with `name = "..."`
- `reverse()` and `reverse_with_parameters()` functions

**Templates**
- Tera integration
- Template preprocessing at startup
- Django-like custom tags:
  - `{% static "file" %}` - Static files
  - `{% media "file" %}` - Media files
  - `{% csrf %}` - CSRF token
  - `{% messages %}` - Flash messages
  - `{% link "route" %}` - Reverse routing
- Template inheritance support
- Auto-injection of context (csrf_token, messages, debug, csp_nonce)

**Forms**
- `#[runique_form]` macro for custom forms
- `#[derive(DeriveModelForm)]` macro for auto-generation from models
- Field types: CharField, TextField, EmailField, PasswordField, IntegerField, FloatField, BooleanField, DateField, DateTimeField, IPAddressField, URLField, SlugField, JSONField
- Validation with `require()` and `optional()`
- Type-safe `get_value<T>()` method
- Automatic Argon2id hashing for PasswordField
- Automatic XSS sanitization for text fields
- Automatic generation of `validate()`, `to_active_model()` and `save()` for ModelForm

**ORM & Database**
- SeaORM integration
- Django-like API with `impl_objects!`:
  - `Entity::objects.all()`
  - `Entity::objects.filter()`
  - `Entity::objects.exclude()`
  - `Entity::objects.get()`
  - `Entity::objects.count()`
- Multi-database support:
  - SQLite (default)
  - PostgreSQL
  - MySQL / MariaDB
- Automatic configuration from `.env`
- Automatic database engine detection
- Configurable connection pool (default: max=20, min=5)
- Password masking in logs

**Middleware**
- Error handling middleware with detailed debug pages
- CSRF middleware with secure HMAC-SHA256 token generation
- Flash message middleware with levels (success, error, info)
- Optional XSS sanitization middleware
- CSP (Content Security Policy) middleware with 3 predefined configurations
- ALLOWED_HOSTS middleware with wildcard support
- Authentication middleware (`login_required`, `redirect_if_authenticated`)
- Custom middleware support

**Security**
- Built-in CSRF protection with HMAC-SHA256 tokens
- ALLOWED_HOSTS validation with wildcards (`.example.com`)
- CSP with cryptographic nonce generation
- Complete security headers (X-Content-Type-Options, X-Frame-Options, etc.)
- Optional automatic XSS sanitization
- Secure sessions with `tower-sessions`
- Constant token validation
- Argon2id hashing for passwords
- Debug/production modes

**Static Files**
- Automatic static file serving
- Automatic media file serving
- Filtres Tera {% static "css/main.css %} et {% media "media_name.format %}
- Flexible path configuration

**Error Handling**
- Elegant error pages (404, 500)
- Debug mode with detailed information:
  - Complete stack trace
  - HTTP request information
  - Template source with line number
  - List of available templates
  - Environment variables
  - Rust version
- Customizable error templates
- HTML fallback on failure

**Custom Axum Extractors**
- `Template` - Template extraction and rendering
- `Message` - Flash message management
- `ExtractForm<T>` - Form extraction and validation
- Auto-injection of context into templates

**Macros**
- `urlpatterns!` - Django-like route definition
- `context!` - Tera context creation with two syntaxes
- `impl_objects!` - Enable Django-like API for entities
- `#[runique_form]` - Generate Deref/DerefMut and serde(flatten) for forms
- `#[derive(DeriveModelForm)]` - Generate complete form from model
- `reverse!` and `reverse_with_parameters!` - Reverse routing

**Documentation**
- Complete README with examples
- Step-by-step getting started guide (GETTING_STARTED.md)
- Template and tag documentation (TEMPLATES.md)
- Database and ORM guide (DATABASE.md)
- Configuration guide (CONFIGURATION.md)
- Forms guide (FORMULAIRE.md)
- CSP guide (CSP.md)
- Contributing guide (CONTRIBUTING.md)
- Complete documentation (~89 pages)

#### Technical

**Architecture**
- Clear code modularization
- Separation of concerns
- Builder pattern for configuration
- Trait extensions for Tera
- Procedural macro for code generation

**Dependencies**
- `axum` 0.7 - HTTP framework
- `tokio` 1.43 - Async runtime
- `tower` - Middleware
- `tower-http` - HTTP services
- `tower-sessions` - Session management
- `tera` 1.20 - Template engine
- `sea-orm` 1.1 - ORM (optional)
- `serde` / `serde_json` - Serialization
- `tracing` - Logging
- `dotenvy` - Environment variables
- `regex` - Template preprocessing
- `hmac` / `sha2` - CSRF tokens
- `argon2` - Password hashing
- `chrono` - Time management

**Cargo Features**
- `default = ["orm"]` - Enable ORM by default
- `orm` - SeaORM support
- `sqlite` - SQLite driver
- `postgres` - PostgreSQL driver
- `mysql` / `mariadb` - MySQL/MariaDB driver
- `all-databases` - All drivers

#### Examples Provided

- Complete demo application with templates, DB, forms
- Unit tests (75% coverage)
- Integration tests
- Examples in documentation

#### Known Issues / Limitations

**Missing Features:**
- Rate limiting: Flag exists (`Settings.rate_limiting`) but implementation not done
- CLI for migrations: Use `sea-orm-cli` directly
- Native WebSocket: Use Axum/Tower layers
- Admin panel: Not implemented
- Hot reload: Use `cargo-watch`

**Resolved Documentation/Code Inconsistencies:**
- CSP.md: Corrected `CspConfig` structure (flat fields, not enum)
- DATABASE.md: Clarified hardcoded pool parameters
- FORMULAIRE.md: Validated that `DeriveModelForm` generates `validate()`, `to_active_model()` and `save()`

**Technical Limitations:**
- Dynamic variables not supported in custom template tags
- Single level of template preprocessing
- SQLite limited to 1 connection (native limitation)

#### Project Statistics

- Lines of code: ~15,000 LOC
- Documentation: ~89 pages (~150 printed pages)
- Tests: 75% coverage
- Number of tests: 50+ unit and integration tests
- Modules: 20+ modules

#### Security

**Mozilla Observatory Score: A+ (115/100)**

Implemented security headers:
- Content-Security-Policy
- X-Content-Type-Options: nosniff
- X-Frame-Options: DENY
- X-XSS-Protection: 1; mode=block
- Referrer-Policy: strict-origin-when-cross-origin
- Permissions-Policy
- Strict-Transport-Security (recommended in production)

---

## Coming Soon (v1.1.0)

### Planned

**Features**
- Actual rate limiting implementation
- Runique CLI for scaffolding and migrations
- Integrated WebSocket support
- Auto-generated admin panel
- Hot reload in development
- Cache system (Redis, Memcached)
- Complete i18n/l10n support

**Improvements**
- Fully configurable connection pool via Settings
- Dynamic variables in template tags
- Multi-level template preprocessing
- Automatic API documentation generation
- Performance benchmarks
- More form field types

**Documentation**
- Advanced deployment guide
- Video tutorials
- Real application examples
- Patterns and best practices

---

## Version Comparison

### Django → Runique

| Feature | Django | Runique v1.0.86 | Status |
|---------|--------|--------------|--------|
| **Routing** | `urls.py` | `urlpatterns!` | Complete |
| **Templates** | Jinja2-like | Tera + custom tags | Complete |
| **ORM** | Django ORM | SeaORM + Django-like API | Complete |
| **Forms** | Django Forms | `#[runique_form]` + `DeriveModelForm` | Complete |
| **Admin** | Django Admin | Not yet | Coming |
| **Auth** | Built-in | Basic middleware | Partial |
| **Migrations** | `manage.py migrate` | `sea-orm-cli` | Partial |
| **CSRF** | Middleware | Middleware | Complete |
| **Sessions** | Built-in | Built-in | Complete |
| **Static files** | `collectstatic` | Automatic serving | Complete |
| **i18n** | Complete | Not yet | Coming |
| **Cache** | Multiple backends | Not yet | Coming |
| **Rate limiting** | Django-ratelimit | Flag only | Coming |

---

## Contributors

Thanks to all contributors! Your contributions make the difference.

Lead Developer: Itsuki

---

## Links

- [Documentation](informations)
- [GitHub Repository](https://github.com/seb-alliot/runique)
- [Issues](https://github.com/seb-alliot/runique/tree/issues)
- [Discussions](https://github.com/seb-alliot/runique/tree/discussions)
- [Changelog](hinformations/documentation_english/CHANGELOG.md)

---

**Legend**
- Complete feature
- Partial feature
- Not yet implemented
- In development

---

[1.0.86]: https://github.com/seb-alliot/runique/releases/tag/v1.0.86