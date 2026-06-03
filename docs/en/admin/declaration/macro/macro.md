# The `admin!` macro

## Full syntax

```rust
admin! {
    // Optional: configure display for any resource (declared or builtin)
    configure {
        users:  { list_display: [["id", "ID"], ["username", "Username"], ["email", "Email"]] },
        droits: { list_display: [["id", "ID"], ["nom", "Name"]] },
        blog:   { list_display: [["id", "ID"], ["title", "Title"], ["created_at", "Created"]] },
    }

    // Required fields only
    articles: articles::Model => ArticleForm {
        title: "Articles",
    },

    // All optional fields
    users: users::Model => RegisterForm {
        title: "Users",
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

### Optional — behaviour

| Field | Default value | Description |
| --- | --- | --- |
| `id_type` | `I32` | Primary key type in routes — `I32`, `I64`, `Uuid` |
| `create_form` | *(same as `form`)* | Separate form type for create operations |
| `edit_form` | *(same as `form`)* | Separate form type for edit operations |
| `list_display` | *(empty — all columns)* | Visible columns and their labels in the list view |
| `list_filter` | *(empty — no sidebar)* | Fields available in the sidebar filter (optional per-column limit as 3rd element, default `10`) |
| `extra` | *(empty)* | Additional variables injected into all Tera templates for this resource |

### `configure {}` block

The `configure {}` block controls display settings for **any** registered resource — whether declared in `admin!{}` or injected as a builtin (users, droits, groupes).

```rust
admin! {
    configure {
        users:  { list_display: [["id", "ID"], ["username", "Username"], ["email", "Email"]] },
        droits: { list_display: [["id", "ID"], ["nom", "Name"]] },
    }
    // ... resource declarations
}
```

Supported keys inside each entry:

| Key | Type | Description |
| --- | --- | --- |
| `list_display` | `[["col", "Label"], …]` | Ordered visible columns in the list view |
| `list_exclude` | `["col1", "col2", …]` | Columns to hide (mutually exclusive with `list_display`) |
| `list_filter` | `[["col", "Label"], …]` | Sidebar filters |
| `group_action` | `[["field", "Label"], …]` | Bulk actions (also available here for builtins) |
| `hidden` | `true` / `false` | Removes the builtin resource from the registry — useful when `extend!{}` takes over |

`list_display` and `list_exclude` are mutually exclusive — the daemon rejects both being declared for the same resource.

When `hidden: true` is declared, all other keys for that entry are ignored — the resource is removed from the registry.

The daemon generates `registry.configure("key", DisplayConfig::new()...)` calls **after** all `registry.register()` calls, so builtin resources registered earlier can also be configured. For `hidden: true`, it generates `registry.remove("key")`.

**Typical example — `extend!{}` takes ownership of `eihwaz_users`:**

```rust
admin! {
    configure {
        users: { hidden: true }
    }
    user_profile: user_profile::Model => user_profile::AdminForm {
        title: "User profiles",
        list_display: [
            ["username", "User"],
            ["email", "Email"],
            ["bio", "Bio"],
            ["is_verified", "Verified"],
        ],
    }
}
```

The builtin "Users" panel disappears; "User profiles" replaces it with the full table (`eihwaz_users` + extended columns).

#### `id_type`

By default, the `{id}` route segment is converted to `i32`. Declaring a different type generates the appropriate conversion in the CRUD closures:

```rust
admin! {
    posts: posts::Model => PostForm {
        title: "Posts",
        id_type: I64
    }
}
```

Supported types: `I32` (default), `I64`, `Uuid`.

> The column type in the database is defined by the SeaORM entity, not by `id_type`. This field generates no migration or schema change — it only adjusts the String → native type conversion inside the admin handler.

#### `create_form`

Use a different form for the create view (common case: a multi-select `CheckboxField` to create several records at once, combined with `bulk_create`):

```rust
admin! {
    horaires: horaire::Model => horaire::AdminForm {
        title: "Schedules",
        create_form: crate::forms::ScheduleGroupForm,
        bulk_create: jour,
    }
}
```

When declared, the create view uses `create_form`; the edit view keeps using the main form (or `edit_form` if declared separately). The `save()` method on the create wrapper returns `Ok(())` — persistence is handled by `create_fn`.

> `create_form` implicitly enables `inject_password: true` on the resource, preserving password field handling if present in the form.

---

#### `edit_form`

Use a different form for create and edit (common case: the create form includes a password field, the edit form does not):

```rust
admin! {
    users: users::Model => RegisterForm {
        title: "Users",
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
        list_display: [
            ["username", "Username"],
            ["email", "Email"],
            ["is_active", "Active"],
        ]
    }
}
```

Each entry is a `["column_name", "Label"]` pair. The column name must **exactly** match the field name in the generated SeaORM entity (generally identical to the name declared in `model!`). Using the wrong name produces a SQL error at runtime.

> If the DSL field is `sort_order`, the column name to declare is `"sort_order"` — not `"order"` or any alias.

If absent, **all entity columns** are displayed.

Columns declared in `list_display` also serve as the **sorting whitelist**: only these columns (and `id`) accept a `sort_by` parameter.

#### `list_filter`

Declare fields available in the sidebar filter:

```rust
admin! {
    doc_page: doc_page::Model => DocPageForm {
        title: "Doc — Pages",
        list_filter: [
            ["lang", "Language"],          // default: 10 values per page
            ["block_type", "Type", 5],     // explicit limit: 5 values per page
        ]
    }
}
```

Each entry is a `["column_name", "Group label", optional_limit]` triplet. The column name follows the same rule as `list_display`: it must exactly match the SeaORM entity field name. The 3rd element is optional — if absent, the default limit is `10` values per page.

`enum` columns are good candidates for `list_filter`: the distinct values shown match the serialized (capitalized) variants. See the [`model!` DSL](/docs/en/model/dsl) for enum behavior.

For each field, the daemon generates a SQL query that loads distinct values with server-side pagination (`LIMIT` / `OFFSET`). The `‹ 1/N ›` navigation in the sidebar reloads the page via `fp_{column}` URL parameters — no JavaScript required.

> Do not use `list_filter` on foreign key (FK) or `id` columns — the raw value (`35`, `128`…) is not human-readable. Good candidates: booleans, enumerations, short codes (`lang`, `status`, `block_type`).

#### `group_action`

Declares bulk actions applicable to a selection of entries in the list view (e.g. bulk activate/deactivate):

```rust
admin! {
    users: users::Model => RegisterForm {
        title: "Users",
        group_action: [
            ["is_active", "Activate"],   // 2-element: boolean field → submits "true"
        ]
    }
}
```

Each 2-element entry `["field_name", "Label"]` targets a **boolean field** and submits `"true"` via `partial_update_fn`.

For **enum fields**, use the 3-element syntax to submit an exact string value:

```rust
admin! {
    reviews: review::Model => review::AdminForm {
        title: "Reviews",
        group_action: [
            ["status", "Approve", "approved"],  // 3-element: submits "approved"
            ["status", "Reject",  "rejected"],  // 3-element: submits "rejected"
        ]
    }
}
```

Multiple entries targeting the **same field** are automatically merged into a single dropdown in the interface, preventing form conflicts.

Also available in `configure {}` for builtin resources.

---

#### `bulk_create`

When declared on a resource, the generated `create_fn` splits the named field by comma and performs an **upsert** per value: update if a record already exists (same split field value), insert otherwise. Designed for `CheckboxField` multi-select (e.g. selecting multiple days of the week to create or update one schedule per day).

```rust
admin! {
    horaires: horaire::Model => horaire::AdminForm {
        title: "Schedules",
        create_form: crate::forms::ScheduleGroupForm,
        bulk_create: jour,   // upsert per value of data["jour"]
    }
}
```

Only the split field behaves differently — all other form fields are copied as-is into each inserted or updated record.

**Multi-select submission pipeline**: a group of HTML checkboxes submits the same key multiple times (`jour=monday&jour=tuesday&jour=thursday`). Prisme, Runique's request body parser, automatically joins these repeated values with a comma → the field becomes `"monday,tuesday,thursday"`. `bulk_create` then splits this string to process each value independently.

```rust
// In the group create form
form.field(
    &CheckboxField::new("jour")
        .label("Days")
        .add_choice("monday", "Monday")
        .add_choice("tuesday", "Tuesday")
        // ...
);
```

**Interaction with `edit_form`**: when `bulk_create` is declared without an explicit `edit_form`, the daemon automatically generates an `edit_form_builder` using `module::AdminForm` (standard single-record form). The individual edit view therefore never uses the multi-select create form.

> The split field must correspond to a `unique` constraint in the model for the upsert to work correctly — this uniqueness is what allows an existing record to be located.

---

#### `m2m`

Declares many-to-many relations managed through a junction table. The daemon generates a `M2mLoaderFn` closure that loads available choices and pre-selects existing associations.

```rust
admin! {
    articles: article::Model => ArticleForm {
        title: "Articles",
        m2m: [
            ["tags", "Tags", "article_tags", "article_id", "tag_id", "tags::Entity", "name"],
        ]
    }
}
```

Each entry is a 7-element array:

| Position | Example | Description |
| --- | --- | --- |
| 1 | `"tags"` | Field name — used as the form field prefix (`m2m_tags__`) |
| 2 | `"Tags"` | Label displayed in the create/edit form |
| 3 | `"article_tags"` | Junction table name |
| 4 | `"article_id"` | This entity's FK column in the junction table |
| 5 | `"tag_id"` | Target entity's FK column in the junction table |
| 6 | `"tags::Entity"` | Target SeaORM entity path |
| 7 | `"name"` | Column from the target table used as display label in the form |

In create/edit forms, all available choices are loaded from the target table and existing associations are pre-selected. On submit, submitted values (prefixed `m2m_field__`) are diffed against the current state — only inserts and deletes are applied.

---

#### Bulk edit

Bulk edit requires no DSL declaration. When entries are selected in the list view and the bulk-edit action is triggered, a form is rendered with all shared editable fields.

**Unique-constrained fields are automatically excluded** from the bulk edit form — setting the same unique value on multiple records would violate the constraint. These fields are detected via the `UNIQUE_FIELDS` constant generated by `derive_form!{}` for each entity.

On submit, each record is updated independently. Only fields with non-empty submitted values are applied — leaving a select blank means "no change".

The bulk edit form uses the same form type as the create/edit view. To customise the template, override `admin/bulk_edit.html`.

---

#### `extra`

Inject Tera variables available in all templates for this resource:

```rust
admin! {
    users: users::Model => RegisterForm {
        title: "Users",
        extra: {
            "icon" => "user",
            "color" => "#3b82f6"
        }
    }
}
```

Keys are accessible via `{{ resource.extra_context.icon }}`.

> The default admin templates do not use `icon` or `color`. These keys only have an effect in a **custom template** that reads them explicitly. See [Template overrides](/docs/en/admin/template/surcharge).
>
> Framework reserved keys (`entries`, `form_fields`, `object_id`, `csrf_token`, etc.) take priority over `extra` keys.

---

## Example with multiple resources

```rust
admin! {
    users: users::Model => RegisterForm {
        title: "Users",
    },
    articles: articles::Model => ArticleForm {
        title: "Articles",
    },
    comments: comments::Model => CommentForm {
        title: "Comments",
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
- the primary key type (`id_type`)
- a separate form for edit operations (`edit_form`)
- visible columns in the list view (`list_display`)
- sidebar filters with optional per-column limit (`list_filter`)
- bulk actions (`group_action`)
- multi-record creation from a comma-split field (`bulk_create`)
- many-to-many relations through a junction table (`m2m`)
- additional per-resource Tera variables (`extra`)
- display configuration for any resource including builtins (`configure {}`)

## What cannot be declared

| Feature | Reason for exclusion |
| --- | --- |
| Resource access permissions | Managed from the panel via scoped droits — see [Permissions](/docs/en/admin/permission) |
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
| [Daemon & generation](/docs/en/admin/declaration-daemon) | Generated files |

## Back to summary

- [Admin Summary](/docs/en/admin)
