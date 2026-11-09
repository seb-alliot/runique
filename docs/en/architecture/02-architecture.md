# Architecture of a Runique Project

A Runique project is a standard Rust binary crate. `runique new` generates the base structure, which you evolve as needed.

## Project structure

```text
my_project/
├── src/
│   ├── main.rs            # Entry point — RuniqueApp builder
│   ├── url.rs             # urlpatterns! — routing table
│   ├── views.rs           # Handlers (async functions)
│   ├── forms/             # Forms (#[form] or declared by hand)
│   ├── entities/          # model!{} / extend!{} models, read by makemigrations
│   └── admin.rs           # ModelAdmin + AdminSite (`admin` feature only)
│
├── templates/             # Tera templates (.html)
├── static/                # Static files (CSS, JS, images)
├── media/                 # Files users upload (FileField);
│                          # MEDIA_ROOT may point elsewhere
├── migration/             # The migrations crate (workspace)
│   ├── Cargo.toml
│   └── src/               # main.rs, lib.rs, migrations, snapshots/
├── .env                   # Environment variables
└── Cargo.toml             # Workspace: the app + migration
```

`src/entities/` must hold only `model!{}` and `extend!{}`: `makemigrations` reads that folder and writes a migration for every model it finds there. A form declared by hand goes in `src/forms/`.

---

## Role of each file

**`main.rs`** — Configures and runs the app through the builder:

```rust
use runique::prelude::*;

mod entities;
mod forms;
mod url;
mod views;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    password_init(PasswordConfig::auto_with(Manual::Argon2));

    let config: RuniqueConfig = RuniqueConfig::from_env();
    let db_config = DatabaseConfig::from_env()?.build();
    let db: DatabaseConnection = db_config.connect().await?;

    RuniqueApp::builder(config)
        .routes(url::routes())
        .with_database(db)
        .statics()
        .build()
        .await?
        .run()
        .await?;

    Ok(())
}
```

**`url.rs`** — Declares the routes with `urlpatterns!`:

```rust
use crate::views;
use runique::prelude::*;

pub fn routes() -> Routes {
    urlpatterns! {
        "/"       => view!{ views::index },  name = "index",
        "/signup" => view!{ views::signup }, name = "signup",
    }
}
```

**`views.rs`** — Request handlers:

```rust
use runique::prelude::*;

pub async fn index(mut request: Request) -> AppResult<Response> {
    context_update!(request => { "title" => "Home" });
    request.render("index.html")
}
```

**`forms/`** — Typed forms. A form declared by hand:

```rust
use runique::prelude::*;
use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct ContactForm {
    pub form: Forms,
}

impl FormAccess for ContactForm {
    fn register_fields(form: &mut Forms) {
        form.field(&TextField::text("name").label("Name").required());
        form.field(&TextField::email("email").label("Email").required());
    }
    impl_form_access!();
}

impl RuniqueForm for ContactForm {}
```

A form taken from a model is one line: `#[form(schema = blog, fields = [title, content])]`. See [Forms](/docs/en/formulaire).

**`admin.rs`** — The admin's typed declaration (`ModelAdmin` + `AdminSite`):

```rust
use crate::entities::demo_category;
use runique::admin::{AdminSite, ModelAdmin};
use runique::app::staging::AdminStaging;
use sea_orm::Order;

pub fn config(admin: AdminStaging) -> AdminStaging {
    admin
        .site_title("Administration")
        .path("/admin/")
        .site(site)
}

fn site(site: AdminSite) -> AdminSite {
    site.register(demo_category())
}

fn demo_category() -> ModelAdmin<demo_category::Entity> {
    use demo_category::Column::*;
    ModelAdmin::new().list(|l| {
        l.display([Title, BackLinkLabel, BackLinkUrl, SortOrder])
            .ordering([(SortOrder, Order::Asc)])
            .search([Title])
            .empty_value("—")
    })
}
```

Wired in `main.rs` with `.with_admin(admin::config)`.

> **The admin is a feature to turn on.** `runique::admin`, `AdminStaging` and `.with_admin(...)` only exist when `runique` is compiled with the `admin` feature:
>
> ```toml
> runique = { version = "4.0.0", features = ["postgres", "admin"] }
> ```
>
> Without it, the compiler reports `unresolved import runique::admin` or `no method named with_admin`. Every `model!{}` of the project must be registered in the `AdminSite`: the rights of a model left out are pruned at boot.

---

## Starting a new project

```bash
runique new my_project
cd my_project
runique migration up
cargo run
```

What each part of the generated project is yours to change:

| File / folder | To change | Role |
| --- | --- | --- |
| `src/main.rs` | Yes | Configures the builder and declares the modules |
| `src/url.rs` | Yes | Routing table |
| `src/views.rs` | Yes | Request handlers |
| `src/entities/` | Yes | Models (`model!{}`, `extend!{}`), read by `makemigrations` |
| `src/forms/` | Yes | Forms and validation |
| `src/admin.rs` | Yes (`admin` feature) | `ModelAdmin` + `AdminSite` registration |
| `templates/` | Yes | Tera templates |
| `static/` | Yes | CSS, JS, images |
| `migration/` | No | Kept by `runique makemigrations` (`lib.rs`, migrations, snapshots). By hand: only the `up()` / `down()` of a column type change, which `makemigrations` can't write yet |
| `.env` | Yes | Environment variables |

---

## Table of Contents

| Section | Content |
| --- | --- |
| [Core Concepts](/docs/en/architecture/concepts) | `RuniqueEngine`, `Request`, `request.form("name")` |
| [Makemigrations Internals](/docs/en/architecture/makemigrations) | Three-pass pipeline: AST extraction, diffing, SeaQuery generation |
| [Macros](/docs/en/architecture/macros) | Context, flash, routing, and error macros |
| [Tera Tags & Filters](/docs/en/architecture/tera) | Django-like tags, filters, functions |
| [Middleware Stack](/docs/en/architecture/middleware) | Slot order and dependency injection |
| [Request Lifecycle](/docs/en/architecture/lifecycle) | Lifecycle and best practices |
| [Testing](/docs/en/architecture/testing) | `runique_test`: business logic tested against the real database, transaction rolled back; `runique test`; boot checks |

---

## Next Steps

← [Installation](/docs/en/installation) | [**Configuration**](/docs/en/configuration) →
