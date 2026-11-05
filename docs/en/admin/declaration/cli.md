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

Without `.form().fields(...)`, the form exposes every field of the model's form (`ModelMeta::Form`), cut to the user's rights.

## Admin routes declared in `urlpatterns!`

A page of your own, served behind the admin's protection, is declared in `urlpatterns!` with the word `admin` before the path:

```rust
use runique::admin::AdminView;

urlpatterns! {
    "/" => get(views::index), name = "index",

    // A page of its own, attached to the articles model
    admin "/articles/stats" => get(views::article_stats),
        name = "article_stats",
        for_model = article::Entity,

    // Replaces the articles' detail page
    admin "/articles/{id}" => get(views::article_detail),
        name = "article_detail",
        for_model = article::Entity,
        overrides = AdminView::Detail,
}
```

- `name` and `for_model` are required; the model must be registered in the admin.
- Without `overrides`, the page is added (`AdminView::Standalone`), as many as you like per model.
- `overrides = AdminView::Detail` replaces the model's detail page: the list's "view" links lead to your route. The path must hold exactly one `{id}` segment, and nothing else. It is the only page replaced by a route; the others are replaced by a template (see [Template override](/docs/en/admin/template-surcharge)).
- The route keeps its own path, outside the admin prefix, and must overlap neither a public route nor the admin's namespace.
- Access: signed in with admin access → your handler; signed in without it → sent to `/` with a notice, session kept; anonymous → `/` (or the admin login when coming from the admin). The admin prefix is never shown to a stranger.
- The protection only checks **admin access**. What your handler reads or writes goes through the gate, as everywhere in the framework: see [Permissions](/docs/en/admin/permission).

For a page on the admin's layout (stylesheets, menu with the rights applied, top bar), call `admin_page_context` and extend `admin_base.html`:

```rust
use runique::prelude::*;

pub async fn article_stats(
    Extension(admin): Extension<Arc<AdminState>>,
    Extension(site): Extension<Arc<AdminSite>>,
    mut request: Request,
) -> AppResult<Response> {
    admin_page_context(&mut request, &admin, &site, Page::Detail, "articles");
    request.render("admin/article_stats.html")
}
```

`Page` picks the stylesheets to load, and the last argument the model the menu highlights. This function only prepares the display: it grants no right.

When the admin is disabled, declaring an `admin` route refuses the boot.

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
