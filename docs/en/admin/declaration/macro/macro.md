# The `admin!` macro

## Full syntax

```rust
admin! {
    key: path::Model => FormType {
        title: "Displayed title",
        permissions: ["role1", "role2"]
    },
    other_key: other::Model => OtherForm {
        title: "Other resource",
        permissions: ["admin"]
    }
}
```

The macro is parsed by the daemon (`runique start`) which generates the `admin_register()` function in `src/admins/admin_panel.rs`. This function builds the `HashMap<String, ResourceEntry>` loaded at boot.

---

## Fields

### Required

| Field | Type | Description |
| --- | --- | --- |
| `key` (positional) | identifier | Used in routes `/admin/{key}/…` |
| `model` (positional) | type path | e.g. `users::Model` |
| `form` (positional) | type path | Runique form type for create/edit |
| `title` | `&str` | Title displayed in the interface |
| `permissions` | `[&str; N]` | Roles declared for this resource (⚠️ not enforced — see [Permissions](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/permission/permissions.md)) |

### Optional — template override

These fields allow replacing a CRUD template with a custom one (see [Template override](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/template/surcharge/surcharge.md)).

| Field | Default value | Description |
| --- | --- | --- |
| `template_list` | `admin/list.html` | List template |
| `template_create` | `admin/create.html` | Create form |
| `template_edit` | `admin/edit.html` | Edit form |
| `template_detail` | `admin/detail.html` | Detail page |
| `template_delete` | `admin/delete.html` | Delete confirmation page |

---

## Example with multiple resources

```rust
admin! {
    users: users::Model => RegisterForm {
        title: "Users",
        permissions: ["admin"]
    },
    articles: articles::Model => ArticleForm {
        title: "Articles",
        permissions: ["admin", "editor"]
    },
    comments: comments::Model => CommentForm {
        title: "Comments",
        permissions: ["moderator", "admin"]
    }
}
```

---

## What can be declared

The `admin!` macro covers only **registry metadata**:

- the resource route identifier
- the SeaORM model used
- the Runique form for create/edit
- the display title
- the allowed roles (uniformly across all CRUD operations)

## What cannot be declared

| Feature | Reason for exclusion |
| --- | --- |
| Per-CRUD-operation permissions | Not supported — roles apply globally |
| Conditional rules | Business logic, to be written in the generated code |
| HTML rendering / templates | Separation of concerns |
| Filters or complex relations | Too specific to be declarative |
| Authentication logic | Handled by admin middlewares |

---

## Compile-time errors

The macro includes a static check: if a referenced type (`Model` or form) does not exist in scope, **compilation fails** with an explicit error message.

```text
error[E0412]: cannot find type `RegisterForm` in module `users`
```

Missing required fields also produce compile-time errors, not runtime errors.

---

## See also

| Section | Description |
| --- | --- |
| [CLI](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/declaration/cli.md) | How `runique start` works |
| [Daemon & generation](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/declaration/daemon/generation.md) | Generated files |

## Back to summary

- [Admin Summary](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/11-Admin.md)
