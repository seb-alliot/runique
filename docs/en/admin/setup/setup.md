# Admin setup

Step-by-step guide to enable the admin with the `ModelAdmin` builder.

## Prerequisites

- A Runique project with a configured database
- Models declared via `model!` (generated SeaORM entities)
- A staff or superuser account to sign in

## Step 1 — Create `src/admin.rs`

A config function takes `AdminStaging` and registers models:

```rust
// src/admin.rs
use crate::entities::{article, user_profile};
use runique::admin::{AdminSite, ModelAdmin};
use runique::app::staging::AdminStaging;

pub fn config(admin: AdminStaging) -> AdminStaging {
    admin
        .site_title("Administration")
        .path("/admin/")
        .page_size(20)
        .site(site)
}

fn site(site: AdminSite) -> AdminSite {
    site.accounts_users(accounts())
        .register(articles())
}

fn accounts() -> ModelAdmin<user_profile::Entity> {
    use user_profile::Column::*;
    ModelAdmin::new()
        .list(|l| {
            l.display([Username, Email, IsActive])
                .sortable([Username])
                .search([Username, Email])
        })
        .form(|f| f.fields([Username, Email]))
}

fn articles() -> ModelAdmin<article::Entity> {
    use article::Column::*;
    ModelAdmin::new()
        .title("Articles")
        .list(|l| l.display([Title, Status]).links([Title]).filter([Status]))
        .form(|f| f.fields([Title, Body, Status]))
}
```

Notes:

- `accounts_users(...)` replaces the framework accounts admin (table extended via `extend!`)
- `register(...)` adds a business model
- Accounts and groups already exist in the framework; you do not recreate them

## Step 2 — Declare the module

```rust
// src/main.rs
mod admin;
```

No generated `src/admins/` folder: everything lives in your `admin.rs`.

## Step 3 — Wire `.with_admin()`

```rust
use runique::app::builder::RuniqueAppBuilder as builder;

builder::new(config)
    .routes(url::routes())
    .with_database(db)
    .with_admin(admin::config)
    .build()
    .await?
    .run()
    .await?;
```

Or an inline closure:

```rust
.with_admin(|a| {
    a.site_title("Admin")
        .path("/admin/")
        .site(|s| s.register(ModelAdmin::<article::Entity>::new()))
})
```

## Path and prefix

Two separate knobs:

| Method | Role |
| --- | --- |
| `.path("/admin/")` | Where the admin lives (CRUD segment) |
| `.prefix("secret")` | Segment mounted **in front of** the path |

| `path` | `prefix` | Final URL |
| --- | --- | --- |
| `/admin/` | — | `/admin/…` |
| `/admin/` | `secret` | `/secret/admin/…` |
| `/admin-runique/` | `prefix-test` | `/prefix-test/admin-runique/…` |

## Useful `AdminStaging` options

| Method | Effect |
| --- | --- |
| `.site_title("…")` | Title in the UI |
| `.page_size(15)` | Default list page size |
| `.search_max_len(n)` | Max search term length |
| `.with_rate_limiter(...)` | Login / action rate limit |
| `.with_login_guard(...)` | Lockout after N failures |
| `.templates(|t| …)` | Global template overrides |
| `.sitemap("https://…")` | Sitemap directive in `/robots.txt` |
| `.no_robots_txt()` | Disable auto robots.txt |
| `.disable()` / `.enable()` | Turn the panel off or on |

## Create a superuser

```bash
runique createsuperuser
```

## Boot checks

Config is validated at startup. demo-app replays the same checks in a unit test:

```rust
#[test]
fn every_registered_model_passes_the_boot_checks() {
    let staging = super::config(AdminStaging::new().enable());
    if let Err(e) = staging.validate() {
        panic!("{e}");
    }
}
```

## See also

| Section | Description |
| --- | --- |
| [Declaration](/docs/en/admin/declaration) | `AdminSite` and registration |
| [ModelAdmin builder](/docs/en/admin/declaration-macro) | Full list / form API |
| [Permissions](/docs/en/admin/permission) | Who sees what |

## Back to summary

- [Admin summary](/docs/en/admin)
