Declares how one model appears in the admin: its title, its list, its form.
Columns are the entity's own `Column` variants, so a column of another model,
or a misspelt one, does not compile.

```rust
use runique::admin::ModelAdmin;
use runique::auth::user::{Column, Entity};

let accounts: ModelAdmin<Entity> = ModelAdmin::new()
    .title("Accounts")
    .list(|l| {
        l.display([Column::Username, Column::Email, Column::IsActive])
            .sortable([Column::Username])
            .search([Column::Username, Column::Email])
    })
    .form(|f| f.fields([Column::Username, Column::Email]));
# let _ = accounts;
```

Rules that cross several declarations (a secret column in a list, a rank flag
as a form field…) are checked when the application starts, not here.
