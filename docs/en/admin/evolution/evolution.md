# Planned features

Features planned for future versions of the Runique admin.

---

## Granular permissions per CRUD operation

Currently permissions apply uniformly to all operations.
The goal is to allow:

```rust
admin! {
    users: users::Model => UserForm {
        title: "Users",
        permissions: {
            list:   ["staff", "admin"],
            create: ["admin"],
            edit:   ["admin"],
            delete: ["admin"],
        }
    }
}
```

---

## Filters and search on the list view

Adding declarative filters on the `list` view:

```rust
admin! {
    users: users::Model => UserForm {
        title: "Users",
        filters: ["username", "is_active"],
        search: ["username", "email"],
    }
}
```

---

## Relations and computed fields

SeaORM relation support in detail/edit views (display of related entities).

---

## Improved daemon error feedback

Better feedback during generation: Rust compilation errors exposed directly in the terminal with context.

## See also

| Section | Description |
| --- | --- |
| [Setup](/docs/en/admin/setup) | Wire the admin into an existing project, create a superuser |
| [CLI](/docs/en/admin/declaration) | `runique start` command, general workflow |
| [Permissions](/docs/en/admin/permission) | Roles, `is_staff` / `is_superuser`, runtime check |
| [Templates](/docs/en/admin/template) | Template hierarchy, blocks, visual override |

## Back to summary

- [Admin Summary](/docs/en/admin)
