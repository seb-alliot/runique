The create and edit form of a model, set inside `ModelAdmin::form`.

```rust
use runique::admin::ModelAdmin;
use runique::auth::user::{Column, Entity};

let accounts: ModelAdmin<Entity> = ModelAdmin::new().form(|f| {
    f.fields([Column::Username, Column::Email])
        .create_only([Column::Username])
        .labels([(Column::Email, "Email address")])
});
# let _ = accounts;
```

- `fields`: the columns of the form, in this order.
- `readonly`: shown, never written.
- `create_only`: written on creation, read-only afterwards.
- `labels`: a label per column, over the model's own.
