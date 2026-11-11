The list page of a model, set inside `ModelAdmin::list`. Only what is declared
is shown or accepted from the URL.

```rust
use runique::admin::ModelAdmin;
use runique::auth::permissions::groupe::{Column, Entity};
use runique::sea_orm::Order;

let groups: ModelAdmin<Entity> = ModelAdmin::new().list(|l| {
    l.display([Column::Nom, Column::IsStaff, Column::IsActive])
        .sortable([Column::Nom])
        .ordering([(Column::Nom, Order::Asc)])
        .search([Column::Nom])
        .filter([Column::IsStaff, Column::IsActive])
        .filter_size(8)
        .per_page(25)
        .action(Column::IsActive, true, "Activate")
        .action(Column::IsActive, false, "Deactivate")
});
# let _ = groups;
```

- `display`: the columns shown, in this order.
- `sortable`, `search`, `filter`: the columns the visitor may sort by, search
  in, filter on.
- `ordering`: the default order.
- `filter_size`: the values each filter group shows before its arrows.
- `action`: a grouped action that sets a column to a value on the selected rows.
