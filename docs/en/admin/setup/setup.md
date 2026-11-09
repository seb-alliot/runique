# Admin setup

Step-by-step guide to enable the admin with the `ModelAdmin` builder.

## Prerequisites

- A Runique project with a configured database and applied migrations (`runique migration up`)
- The `admin` feature enabled in `Cargo.toml`:

  ```toml
  runique = { version = "4.0.0", features = ["postgres", "admin"] }
  ```

- Models declared via `model!` (generated SeaORM entities)

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
        .list(|l| l.display([Title, Status]).filter([Status]))
        .form(|f| f.fields([Title, Body, Status]))
}
```

Notes:

- `accounts_users(...)` replaces the framework's accounts admin with yours, built on your `extend!` of `eihwaz_users`. It is the only way to replace it: registering a model under the accounts' key refuses the boot.
- `register(...)` adds a business model. **Register every `model!{}`**: at boot, the rights targeting an unregistered model (the groups' as well as the admins' grids) are **permanently deleted** from the database.
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

In production (`RUNIQUE_ENV=prod`), add `.with_public_url("https://my-site.com")`: without it, the boot is refused as soon as the admin is enabled. Links sent by email (account creation, reset) are built on this address, never on the request's `Host` header, which an attacker can forge. With `RUNIQUE_ENV=dev`, it is always `http://127.0.0.1:{PORT}`.

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
| `.filter_size(8)` | Default values per page of the filter groups |
| `.search_max_len(n)` | Max search term length |
| `.with_rate_limiter(...)` | Login / action rate limit |
| `.with_login_guard(...)` | Lockout after N failures |
| `.templates(…)` | Template overrides, see [Templates](/docs/en/admin/template-surcharge) |
| `.resource_order([…])` | Order of the models in the menu (unlisted keys last) |
| `.view_site_url("…")` | "Back to the site" link when the public site is served elsewhere (default: `/`) |
| `.sitemap("https://…")` | Sitemap directive in `/robots.txt` |
| `.no_robots_txt()` | Disable auto robots.txt |
| `.disable()` / `.enable()` | Turn the panel off or on |

## Create a superuser

```bash
runique create-superuser
```

The command asks for the hashing algorithm, the username, the email and the password, then creates an active account with `is_superuser`. The superuser has every right; then name a dirigeant from an account's page (rank form) for everyday use. See [Permissions](/docs/en/admin/permission).

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
