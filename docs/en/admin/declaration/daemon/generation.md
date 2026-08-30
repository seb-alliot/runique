# Daemon & code generation

## Daemon behaviour

`runique start` is **not** a background watcher: it's a **sequential one-shot generation**, followed by a blocking launch of the application.

1. `src/admin.rs` is read once
2. The `admin! { ... }` macro is parsed via `syn`, producing `ResourceDef` structures
3. The contents of `src/admins/` are rewritten in place (files are truncated and rewritten — the folder is never deleted beforehand)
4. `cargo fmt --all` runs
5. `cargo run --release` runs, blocking, in the same process

There is no continuous file watching and no debounce: an earlier implementation based on a separate thread was removed because it caused a race condition. To regenerate after a change, run `runique start` again.

---

## Generated structure

```text
src/admins/
  ├── README.md       ← warning: do not edit manually
  ├── mod.rs          ← exposes `routes` and `admin_state`
  └── admin.rs        ← main file: DynForm wrappers + admin_register()
```

### `admin.rs`

Contains for each resource declared in `admin!`:

- A `DynForm` wrapper around the concrete Runique form
- The closures `list_fn`, `get_fn`, `create_fn`, `update_fn`, `delete_fn`, `count_fn`, `partial_update_fn` (always generated, used for bulk edit/group actions)
- If `list_filter` is declared: a `filter_fn` closure per field, loading distinct values from the database (up to 10 by default)
- The `admin_register()` function that builds the `HashMap<String, ResourceEntry>` loaded at boot

### `mod.rs`

Re-exports `routes` and `admin_state` from `admin`.

---

## The trade-off: automatic overwriting

Every run of `runique start` **rewrites** the contents of `src/admins/` (files truncated and regenerated, never deleted beforehand).

Any manual modifications inside this folder will be **lost** on the next `runique start`.

## When to switch to `cargo run`

If manual changes to the generated code are needed (specific business logic, custom handler), you must **stop `runique start`** and switch to a standard workflow:

```bash
cargo run
```

In this mode, `runique start` never runs, so `src/admins/` is never rewritten. Changes persist.

> The `README.md` generated inside `src/admins/` reminds you of this behaviour directly in the repository.

## Related sections

| Section | Description |
| --- | --- |
| [CLI](/docs/en/admin/declaration) | How `runique start` works |
| [Macro `admin!`](/docs/en/admin/declaration-macro) | Declaring administrable resources |

## Back to summary

- [Admin Summary](/docs/en/admin)
