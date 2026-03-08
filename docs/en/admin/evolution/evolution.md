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
| [Setup](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/setup/setup.md) | Wire the admin into an existing project, create a superuser |
| [CLI](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/declaration/cli.md) | `runique start` command, general workflow |
| [Permissions](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/permission/permissions.md) | Roles, `is_staff` / `is_superuser`, runtime check |
| [Templates](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/template/templates.md) | Template hierarchy, blocks, visual override |

## Back to summary

- [Admin Summary](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/11-Admin.md)
