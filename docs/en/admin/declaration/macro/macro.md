# The `admin!` macro

## Full syntax

```rust
admin! {
    // Required fields only
    articles: articles::Model => ArticleForm {
        title: "Articles",
        permissions: ["admin"]
    },

    // All optional fields
    users: users::Model => RegisterForm {
        title: "Users",
        permissions: ["admin"],
        id_type: I32,                                  // I32 | I64 | Uuid
        edit_form: crate::forms::UserEditForm,         // separate form for edit
        extra: {
            "icon" => "user",
            "color" => "#3b82f6"
        }
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
| `permissions` | `[&str; N]` | Roles declared for this resource (⚠️ not enforced — see [Permissions](/docs/en/admin/permission)) |

### Optional — behaviour

| Field | Default value | Description |
| --- | --- | --- |
| `id_type` | `I32` | Primary key type in routes — `I32`, `I64`, `Uuid` |
| `edit_form` | *(same as `form`)* | Separate form type for edit operations |
| `extra` | *(empty)* | Additional variables injected into all Tera templates for this resource |

#### `id_type`

By default, the `{id}` route segment is converted to `i32`. Declaring a different type generates the appropriate conversion in the CRUD closures:

```rust
admin! {
    posts: posts::Model => PostForm {
        title: "Posts",
        permissions: ["admin"],
        id_type: I64
    }
}
```

Supported types: `I32` (default), `I64`, `Uuid`.

> The column type in the database is defined by the SeaORM entity, not by `id_type`. This field generates no migration or schema change — it only adjusts the String → native type conversion inside the admin handler.

#### `edit_form`

Use a different form for create and edit (common case: the create form includes a password field, the edit form does not):

```rust
admin! {
    users: users::Model => RegisterForm {
        title: "Users",
        permissions: ["admin"],
        edit_form: crate::forms::UserEditForm
    }
}
```

When declared, the edit view uses `edit_form`; the create view keeps using the main form. The `save()` method on the edit wrapper returns `Ok(())` — persistence is handled by `update_fn` via `admin_from_form`.

#### `extra`

Inject Tera variables available in all templates for this resource:

```rust
admin! {
    users: users::Model => RegisterForm {
        title: "Users",
        permissions: ["admin"],
        extra: {
            "icon" => "user",
            "color" => "#3b82f6"
        }
    }
}
```

Keys are accessible via `{{ resource.extra_context.icon }}`.

> Framework reserved keys (`entries`, `form_fields`, `object_id`, `csrf_token`, etc.) take priority over `extra` keys.

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
- the primary key type (`id_type`)
- a separate form for edit operations (`edit_form`)
- additional per-resource Tera variables (`extra`)

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
| [CLI](/docs/en/admin/declaration) | How `runique start` works |
| [Daemon & generation](/docs/en/admin/declaration) | Generated files |

## Back to summary

- [Admin Summary](/docs/en/admin)
