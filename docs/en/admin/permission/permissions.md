# Roles and permissions

## Source of truth: the `users` table

The permission system relies on the authenticated user, whose data is read from the `users` table.

## Access control fields

| Field | Type | Status | Role |
| --- | --- | --- | --- |
| `is_staff` | `bool` | ✅ Active | Grants access to the admin interface |
| `is_superuser` | `bool` | ✅ Active | Full access, bypasses all checks |
| `is_active` | `bool` | ⏳ In development | Planned to block disabled accounts |
| `roles` | `Option<Vec<String>>` | ✅ Active | User roles — accessible in templates via `current_user.roles` |

---

## What is actually enforced today

### Entering the admin

The middleware checks only `is_staff` and `is_superuser`:

```
is_staff = true  OR  is_superuser = true  →  access granted
both = false                               →  redirect to /admin/login
```

### is_superuser

A user with `is_superuser = true` bypasses **all** checks — admin entry and per-resource permissions.

---

## What is declared but not yet enforced

### is_active

The field exists in the `users` model but is not yet checked by the admin middleware. An account with `is_active = false` can still log in if `is_staff` or `is_superuser` is `true`.

### roles

The `roles` field is available in all admin templates via `current_user.roles`.

#### Setting roles in the admin interface

Roles are entered as free text, comma-separated:

```
editor
editor, moderator
admin, editor
```

#### Using roles in templates

```html
{% if current_user and "editor" in current_user.roles %}
    <a href="...">Edit</a>
{% endif %}
```

`is_superuser = true` always bypasses role conditions — a superuser sees everything.

#### Per-resource permissions (declarative)

The `admin!` macro allows declaring allowed roles per resource:

```rust
admin! {
    articles: articles::Model => ArticleForm {
        title: "Articles",
        permissions: ["editor", "admin"]
    }
}
```

The `ResourcePermissions` structure and the `check_permission()` function exist in the code, but **are not called** in the generated CRUD handlers. Permissions are stored without being checked at this stage.

---

## Current access logic (actual state)

```
authenticated?
  └─ no  → redirect to /admin/login
  └─ yes → is_staff OR is_superuser?
               └─ neither → redirect to /admin/login
               └─ is_superuser → GRANTED (full access)
               └─ is_staff only → GRANTED (no role check)
```

---

## Notes

- `is_active` and `roles` are planned on the roadmap — see [Roadmap](/docs/en/admin/evolution).
- The `admin!` macro defines only declarative rules; the enforcement logic lives in the middlewares.
- Per-CRUD-operation granularity (list/create/edit/delete) is not supported in the current version.

---

## See also

| Section | Description |
| --- | --- |
| [Setup](/docs/en/admin/setup) | Wire the admin into an existing project, create a superuser |
| [CLI](/docs/en/admin/declaration) | `runique start` command, general workflow |
| [Templates](/docs/en/admin/template) | Template hierarchy, blocks, visual override |
| [Roadmap](/docs/en/admin/evolution) | Planned features and beta status |

## Back to summary

- [Admin Summary](/docs/en/admin)
