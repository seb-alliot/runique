# Declaring the admin

The admin is plain Rust: `AdminStaging` for the site, `AdminSite` for models, one `ModelAdmin<E>` per entity.

## Flow

```text
main.rs
  .with_admin(admin::config)
       │
       ▼
admin.rs :: config(AdminStaging) -> AdminStaging
  .site_title / .path / .page_size / …
  .site(|site| site.register(…).accounts_users(…))
       │
       ▼
At build: validate() then mount CRUD routes
```

No code generation, no daemon, no `src/admins/`.

## `AdminStaging` — the site

Received by the `.with_admin(...)` closure:

```rust
pub fn config(admin: AdminStaging) -> AdminStaging {
    admin
        .site_title("Administration")
        .path("/admin/")
        .prefix("backoffice")   // optional
        .page_size(20)
        .with_rate_limiter(RateLimiter::new().max_requests(20).retry_after(3600))
        .with_login_guard(LoginGuard::new().max_attempts(20).lockout_secs(300))
        .templates(|t| t.with_dashboard("admin/my_dashboard.html"))
        .site(site)
}
```

## `AdminSite` — the models

```rust
pub fn site(site: AdminSite) -> AdminSite {
    site.accounts_users(accounts())
        .register(blog())
        .register(article())
        .register(ModelAdmin::<tag::Entity>::new())
}
```

| Method | Role |
| --- | --- |
| `.register(ModelAdmin<E>)` | Add a business model |
| `.accounts_users(ModelAdmin<E>)` | Replace the accounts admin (`eihwaz_users` extension) |

Accounts and groups are always present. Registering a second model under the accounts key is refused at boot.

## `ModelAdmin<E>` — one model

```rust
fn blog() -> ModelAdmin<blog::Entity> {
    use blog::Column::*;
    ModelAdmin::new()
        .title("Blog")
        .list(|l| { /* ListConfig */ })
        .form(|f| { /* FormConfig */ })
        .templates(|t| { /* per-model page overrides */ })
}
```

Without `.list().display(...)`, every non-secret column is shown (on accounts: username, email, is_active).

Without `.form().fields(...)`, the form exposes every model field (except secrets and rank flags).

## Useful imports

```rust
use runique::admin::{AdminSite, ModelAdmin};
use runique::app::staging::AdminStaging;
use sea_orm::Order;
```

## See also

| Section | Description |
| --- | --- |
| [ModelAdmin builder](/docs/en/admin/declaration-macro) | Full API |
| [Boot validation](/docs/en/admin/declaration-daemon) | Startup checks |
| [Setup](/docs/en/admin/setup) | Wiring in main |

## Back to summary

- [Admin summary](/docs/en/admin)
