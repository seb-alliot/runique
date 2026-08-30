# `runique start`

The `runique start` command is the entry point of the admin workflow.
It runs, **sequentially on a single thread**: admin code generation, `cargo fmt`, then a blocking server launch.

---

## Detecting the admin in `main.rs`

On startup, `runique start` reads `src/main.rs` and looks for the presence of `.with_admin(`:

```rust
// src/main.rs
RuniqueApp::builder(config)
    .with_admin(|a| a.routes(admins::routes("/admin")))
    // ...
```

Detection is done by simple string search in the source file.
**It works even if the line is commented out** (`// .with_admin(...)`).

| Detection result | Behaviour |
| --- | --- |
| `.with_admin(` found | Generation + `cargo run` chained |
| Absent | Info message, clean exit |

> The path to `main.rs` is configurable: `runique start --main src/main.rs`

---

## What happens when `.with_admin(` is detected

`runique start` runs, **in order, on the same thread**:

1. **Generation** — reads `src/admin.rs`, parses `admin!{}`, rewrites `src/admins/`
2. **`cargo fmt --all`**
3. **`cargo run --release`** — blocking until program exit

```text
runique start
  ├── generate_admin(src/admin.rs) → rewrites src/admins/
  ├── cargo fmt --all
  └── cargo run --release          → HTTP server (blocking)
```

An earlier design ran generation on a separate thread in parallel with `cargo run`: this created a race condition (the build could pick up a half-written `admin.rs`, failing unreproducibly). The flow is now strictly sequential to eliminate that. There is **no continuous watching**: to regenerate after changing `src/admin.rs`, run `runique start` again.

---

## Related sections

| Section | Description |
| --- | --- |
| [Admin code generation](/docs/en/admin/declaration-daemon) | Generated files |
| [Macro `admin!`](/docs/en/admin/declaration-macro) | Declaring administrable resources |

## See also

| Section | Description |
| --- | --- |
| [Setup](/docs/en/admin/setup) | Wire the admin into an existing project, create a superuser |
| [Permissions](/docs/en/admin/permission) | Roles, `is_staff` / `is_superuser`, runtime check |
| [Templates](/docs/en/admin/template) | Template hierarchy, blocks, visual override |
| [Roadmap](/docs/en/admin/evolution) | Planned features and beta status |

## Back to summary

- [Admin Summary](/docs/en/admin)
