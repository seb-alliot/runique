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
| [Setup](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/setup/setup.md) | Wire the admin into an existing project, create a superuser |
| [CLI](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/declaration/cli.md) | `runique start` command, general workflow |
| [Daemon & generation](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/declaration/daemon/generation.md) | Generated files, watcher behaviour |
| [Macro `admin!`](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/declaration/macro/macro.md) | Full syntax, required and optional fields |
| [Permissions](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/permission/permissions.md) | Roles, `is_staff` / `is_superuser`, runtime check |
| [Templates](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/template/templates.md) | Template hierarchy, blocks, visual override |
| [Roadmap](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/evolution/evolution.md) | Planned features and beta status |

## Back to menu

- [English Readme](https://github.com/seb-alliot/runique/blob/main/README.md)
