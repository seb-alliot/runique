# Runique Admin

Runique's admin builds a full CRUD UI from a **typed builder** (`ModelAdmin`). No `admin!` macro, no generated files: you declare SeaORM columns, the framework serves routes, list views, forms and permissions.

Columns are enums (`blog::Column::Title`) — the compiler rejects a missing column, an exposed secret, or an inconsistent action.

## Before you start

- **The `admin` feature is required.** Without it, the admin module isn't compiled: `with_admin`, `ModelAdmin`, `AdminSite` don't exist.

  ```toml
  runique = { version = "4.0.0", features = ["postgres", "admin"] }
  ```

- **`with_public_url` is required in production** when the admin is enabled: without it, the boot is refused (links sent by email would be built from the request's `Host` header, which anyone can forge). In debug, the request's `Host` stands in for it.
- **Every `model!{}` of the project must be registered** in the admin (`register`, or `accounts_users` for the accounts' extension). At boot, group rights targeting an unregistered model are deleted from the database.
- The framework's accounts, groups and rights are provided: you don't declare them.

## Minimal example

```rust
use runique::admin::{AdminSite, ModelAdmin};
use runique::app::staging::AdminStaging;
use crate::entities::article;

pub fn config(admin: AdminStaging) -> AdminStaging {
    admin
        .site_title("Administration")
        .path("/admin/")
        .site(|site| site.register(articles()))
}

fn articles() -> ModelAdmin<article::Entity> {
    use article::Column::*;
    ModelAdmin::new()
        .title("Articles")
        .list(|l| {
            l.display([Title, Status, Views])
                .links([Title])
                .sortable([Title])
                .search([Title])
                .filter([Status])
        })
        .form(|f| f.fields([Title, Body, Status]))
}
```

Then in `main`:

```rust
builder::new(config)
    .routes(url::routes())
    .with_database(db)
    .with_admin(admin::config)
    .build()
    .await?
    .run()
    .await?;
```

## Table of contents

| File | Contents |
| --- | --- |
| [Setup](/docs/en/admin/setup) | Wire the admin, path, prefix, superuser |
| [Declaration](/docs/en/admin/declaration) | `AdminStaging`, `AdminSite`, registration |
| [`ModelAdmin` builder](/docs/en/admin/declaration-macro) | List + form API, typed columns |
| [Boot validation](/docs/en/admin/declaration-daemon) | Checks run at startup |
| [User creation](/docs/en/admin/user-creation) | Create → email → activation flow |
| [List view](/docs/en/admin/list) | Pagination, sort, search, filters, actions |
| [Permissions](/docs/en/admin/permission) | Ranks, groups, rights grid |
| [Templates](/docs/en/admin/template) | Override pages and blocks |
| [Roadmap](/docs/en/admin/evolution) | Future ideas |

## Back to menu

- [English Readme](https://github.com/seb-alliot/runique/blob/main/README.md)
