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
        list_display: [                                // visible columns in list view
            ["username", "Username"],
            ["email", "Email"],
            ["is_active", "Active"],
        ],
        list_filter: [                                 // sidebar filters
            ["is_active", "Active"],                   // default: 10 values per page
            ["is_superuser", "Superuser", 5],          // explicit per-column limit
        ],
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
| `list_display` | *(empty — all columns)* | Visible columns and their labels in the list view |
| `list_filter` | *(empty — no sidebar)* | Fields available in the sidebar filter (optional per-column limit as 3rd element, default `10`) |
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

#### `list_display`

Declare which columns are shown in the list view and their labels:

```rust
admin! {
    users: users::Model => RegisterForm {
        title: "Users",
        permissions: ["admin"],
        list_display: [
            ["username", "Username"],
            ["email", "Email"],
            ["is_active", "Active"],
        ]
    }
}
```

Each entry is a `["column_name", "Label"]` pair. If absent, **all entity columns** are displayed.

Columns declared in `list_display` also serve as the **sorting whitelist**: only these columns (and `id`) accept a `sort_by` parameter.

#### `list_filter`

Declare fields available in the sidebar filter:

```rust
admin! {
    doc_page: doc_page::Model => DocPageForm {
        title: "Doc — Pages",
        permissions: ["admin"],
        list_filter: [
            ["lang", "Language"],          // default: 10 values per page
            ["block_type", "Type", 5],     // explicit limit: 5 values per page
        ]
    }
}
```

Each entry is a `["column_name", "Group label", optional_limit]` triplet. The 3rd element is optional — if absent, the default limit is `10` values per page.

For each field, the daemon generates a SQL query that loads distinct values with server-side pagination (`LIMIT` / `OFFSET`). The `‹ 1/N ›` navigation in the sidebar reloads the page via `fp_{column}` URL parameters — no JavaScript required.

> Do not use `list_filter` on foreign key (FK) or `id` columns — the raw value (`35`, `128`…) is not human-readable. Good candidates: booleans, enumerations, short codes (`lang`, `status`, `block_type`).

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
- visible columns in the list view (`list_display`)
- sidebar filters with optional per-column limit (`list_filter`)
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
