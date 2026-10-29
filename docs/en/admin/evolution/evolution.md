# Planned features

Ideas for future versions of the Runique admin (`ModelAdmin` builder).

## Admin inlines

Edit related models inside the parent form, Django-style:

```rust
ModelAdmin::new()
    .form(|f| f.fields([Title, Status]))
    // future:
    // .inline(lines, |i| i.fk(OrderId).fields([Product, Qty]))
```

The inline would show an editable table (add, remove, update) on the parent's create/edit page.

## `view!{}` — public CRUD

Public routing macro similar to Django generic views, with overridable templates:

```rust
view! {
    Article {
        model: entities::article::Entity,
        prefix: "/articles",
        templates: {
            list:   "articles/list.html",
            detail: "articles/detail.html",
            create: "articles/create.html",
        },
        page_size: 10,
        login_required: true,
    }
}
```

Target routes: paginated list, detail, create, edit, delete — framework templates overridable block by block, like the admin.

## Custom business filters

Today the list scope follows the gate (rights + ownership). A declarative hook for a business filter ("orders in my region") is still open.

## See also

| Section | Description |
| --- | --- |
| [Setup](/docs/en/admin/setup) | Wire the admin |
| [ModelAdmin builder](/docs/en/admin/declaration-macro) | Current API |
| [Permissions](/docs/en/admin/permission) | Runtime gate |

## Back to summary

- [Admin summary](/docs/en/admin)
