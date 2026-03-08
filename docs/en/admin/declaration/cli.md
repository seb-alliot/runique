# `runique start`

The `runique start` command is the entry point of the admin workflow.
It orchestrates two operations in parallel: **watching `src/admin.rs`** and **launching the server**.

---

## Detecting the admin in `main.rs`

On startup, `runique start` reads `src/main.rs` and looks for the presence of `.with_admin(`:

```rust
// src/main.rs
RuniqueApp::new()
    .with_admin(|a| a.routes(admins::routes("/admin")))
    // ...
```

Detection is done by simple string search in the source file.
**It works even if the line is commented out** (`// .with_admin(...)`).

| Detection result | Behaviour |
| --- | --- |
| `.with_admin(` found | Daemon + `cargo run` launched |
| Absent | Info message, clean exit |

> The path to `main.rs` is configurable: `runique start --main src/main.rs`

---

## What happens when `.with_admin(` is detected

`runique start` launches **two processes simultaneously**:

1. **The admin daemon** — a separate thread that watches `src/admin.rs` and regenerates `src/admins/` on every change
2. **`cargo run`** — launches the application server (blocking until program exit)

```text
runique start
  ├── daemon thread → watch(src/admin.rs) [immediate initial generation]
  └── cargo run     → HTTP server (blocking)
```

The daemon performs an **initial generation on startup** — there is no need to modify `src/admin.rs` for code to be produced.

---

## Related sections

| Section | Description |
| --- | --- |
| [Daemon & generation](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/declaration/daemon/generation.md) | Watcher, generated files |
| [Macro `admin!`](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/declaration/macro/macro.md) | Declaring administrable resources |

## See also

| Section | Description |
| --- | --- |
| [Setup](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/setup/setup.md) | Wire the admin into an existing project, create a superuser |
| [Permissions](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/permission/permissions.md) | Roles, `is_staff` / `is_superuser`, runtime check |
| [Templates](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/template/templates.md) | Template hierarchy, blocks, visual override |
| [Roadmap](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/evolution/evolution.md) | Planned features and beta status |

## Back to summary

- [Admin Summary](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/11-Admin.md)
