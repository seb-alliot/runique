# Runique Admin

Runique's admin generates a full CRUD interface from a declarative macro (`admin!`).
The generated code is plain Rust — readable, auditable, and modifiable if needed.
The approach is intentionally transparent: no hidden magic, no unknown runtime.

## Minimal example

```rust
admin! {
    users: users::Model => RegisterForm {
        title: "Users",
        permissions: ["admin", "staff"]
    }
}
```

## Table of contents

| File | Contents |
| --- | --- |
| [Setup](/docs/en/admin/setup) | Wire the admin into an existing project, create a superuser |
| [CLI](/docs/en/admin/declaration) | `runique start` command, general workflow |
| [Daemon & generation](/docs/en/admin/declaration) | Generated files, watcher behaviour |
| [Macro `admin!`](/docs/en/admin/declaration) | Full syntax, required and optional fields |
| [Permissions](/docs/en/admin/permission) | Roles, `is_staff` / `is_superuser`, runtime check |
| [Templates](/docs/en/admin/template) | Template hierarchy, blocks, visual override |
| [Roadmap](/docs/en/admin/evolution) | Planned features and beta status |

## Back to menu

- [English Readme](https://github.com/seb-alliot/runique/blob/main/README.md)
