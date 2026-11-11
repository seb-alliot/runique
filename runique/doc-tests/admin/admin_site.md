Every model of the admin. Accounts and groups are there without any
declaration; `accounts_users` replaces the accounts admin with the project's
own, `register` adds a model of the project.

```rust
use runique::admin::{AdminSite, ModelAdmin};
use runique::auth::user::{Column, Entity};

let site = AdminSite::default().accounts_users(
    ModelAdmin::<Entity>::new().list(|l| l.display([Column::Username, Column::Email])),
);
# let _ = site;
```

In an application, the site is handed to the admin staging:
`.with_admin(|a| a.site(|site| site.register(articles())))`, one function per
model, each declaring its own columns.
