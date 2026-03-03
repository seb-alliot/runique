# Runique — Django-inspired Rust Framework

![Rust](https://img.shields.io/badge/rust-1.75%2B-orange)
![Tests](https://img.shields.io/badge/tests-1356%2F1356%20passing-brightgreen)
![License](https://img.shields.io/badge/license-MIT-green)
![Version](https://img.shields.io/badge/version-1.1.30-blue)
![Crates.io](https://img.shields.io/crates/v/runique)

Runique is a web framework built on Axum, focused on type-safe forms, security middleware, template rendering, ORM integration, and a code-generated admin workflow.

🌍 **Languages**: [English](https://github.com/seb-alliot/runique/blob/main/README.md) | [Français](https://github.com/seb-alliot/runique/blob/main/README.fr.md)

---

## Core capabilities

- Type-safe form system (`forms`, extractors, validators, renderers)
- Routing macros and URL helpers
- Tera template integration and context helpers
- Security middleware (CSRF, CSP, allowed hosts, sanitization, auth/session)
- SeaORM integration + migration tooling
- Flash message system
- Admin beta (`admin!` macro + daemon-generated CRUD code)

---

## Installation

```toml
[dependencies]
runique = "1.1.30"
```

For a specific database backend only:

```toml
runique = { version = "1.1.30", default-features = false, features = ["postgres"] }
```

Available features: `sqlite`, `postgres`, `mysql`, `mariadb`, `all-databases` (default).

---

## Quick usage

```rust
use runique::prelude::*;

#[tokio::main]
async fn main() {
    let config = RuniqueConfig::from_env();
    let app = RuniqueApp::builder(config).build().await.unwrap();
    app.run().await.unwrap();
}
```

---

## CLI

```bash
runique new <name>
runique start [--main src/main.rs] [--admin src/admin.rs]
runique create-superuser
runique makemigrations --entities src/entities --migrations migration/src
runique migration up|down|status --migrations migration/src
```

---

## Documentation

- [Installation](https://github.com/seb-alliot/runique/blob/main/docs/en/01-installation.md)
- [Architecture](https://github.com/seb-alliot/runique/blob/main/docs/en/02-architecture.md)
- [Configuration](https://github.com/seb-alliot/runique/blob/main/docs/en/03-configuration.md)
- [Routing](https://github.com/seb-alliot/runique/blob/main/docs/en/04-routing.md)
- [Forms](https://github.com/seb-alliot/runique/blob/main/docs/en/05-forms.md)
- [Model/Schema](https://github.com/seb-alliot/runique/blob/main/docs/en/12-model.md)
- [Templates](https://github.com/seb-alliot/runique/blob/main/docs/en/06-templates.md)
- [ORM](https://github.com/seb-alliot/runique/blob/main/docs/en/07-orm.md)
- [Middleware](https://github.com/seb-alliot/runique/blob/main/docs/en/08-middleware.md)
- [Authentication](https://github.com/seb-alliot/runique/blob/main/docs/en/13-authentification.md)
- [Flash Messages](https://github.com/seb-alliot/runique/blob/main/docs/en/09-flash-messages.md)
- [Admin beta](https://github.com/seb-alliot/runique/blob/main/docs/en/11-Admin.md)

---

## License

MIT — [github.com/seb-alliot/runique](https://github.com/seb-alliot/runique)
