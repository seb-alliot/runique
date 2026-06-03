# Planned features

Features planned for future versions of the Runique admin.

---

## Admin inlines

Allow editing related models directly within the parent form, similar to Django inlines:

```rust
admin! {
    order {
        model: entities::order::Entity,
        inline: [
            ["lines", "crate::entities::order_line", "order_id"],
        ],
    }
}
```

The inline would display an editable table of related entries (add, remove, update) within the parent resource's create/edit page.

---

## `view!{}` — public CRUD router with overridable templates

A routing macro that merges route declaration and overridable default templates, analogous to Django's generic views (`ListView`, `DetailView`, `CreateView`…).

```rust
view! {
    Article {
        model: entities::article::Entity,
        prefix: "/articles",
        templates: {
            list:   "articles/list.html",    // optional override
            detail: "articles/detail.html",
            create: "articles/create.html",
        },
        page_size: 10,
        login_required: true,
    }
}
```

Generated routes: `GET /articles` (paginated list), `GET /articles/<id>` (detail), `GET/POST /articles/new` (create), `GET/POST /articles/<id>/edit`, `POST /articles/<id>/delete`. Default templates are provided by the framework and overridable block by block, exactly like the admin.

## See also

| Section | Description |
| --- | --- |
| [Setup](/docs/en/admin/setup) | Wire the admin into an existing project, create a superuser |
| [CLI](/docs/en/admin/declaration) | `runique start` command, general workflow |
| [Permissions](/docs/en/admin/permission) | Roles, `is_staff` / `is_superuser`, runtime check |
| [Templates](/docs/en/admin/template) | Template hierarchy, blocks, visual override |

## Back to summary

- [Admin Summary](/docs/en/admin)
