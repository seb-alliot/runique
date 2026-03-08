# Daemon & code generation

## Daemon behaviour

The daemon continuously watches `src/admin.rs` via `notify`.

On each detected change:

1. `src/admin.rs` is re-read
2. The `admin! { ... }` macro is parsed via `syn`, producing `ResourceDef` structures
3. The `src/admins/` folder is deleted and fully regenerated
4. Feedback is displayed (success or parsing error)

A **debounce** mechanism (300 ms) prevents multiple regenerations from a single file save.

An **initial generation** is performed on daemon startup, without waiting for a change.

---

## Generated structure

```text
src/admins/
  ├── README.md       ← warning: do not edit manually
  ├── mod.rs          ← exposes `routes` and `admin_proto_state`
  └── admin_panel.rs  ← main file: DynForm wrappers + admin_register()
```

### `admin_panel.rs`

Contains for each resource declared in `admin!`:

- A `DynForm` wrapper around the concrete Runique form
- The closures `list_fn`, `get_fn`, `create_fn`, `update_fn`, `delete_fn`, `count_fn`
- The `admin_register()` function that builds the `HashMap<String, ResourceEntry>` loaded at boot

### `mod.rs`

Re-exports `routes` and `admin_proto_state` from `admin_panel`.

---

## The trade-off: automatic overwriting

`runique start` **deletes and fully regenerates** `src/admins/` on every change to `src/admin.rs`.

Any manual modifications inside this folder will be **lost** on the next regeneration.

## When to switch to `cargo run`

If manual changes to the generated code are needed (specific business logic, custom handler), you must **stop `runique start`** and switch to a standard workflow:

```bash
cargo run
```

In this mode, `src/admins/` is no longer watched or overwritten. Changes persist.

> The `README.md` generated inside `src/admins/` reminds you of this behaviour directly in the repository.

## Related sections

| Section | Description |
| --- | --- |
| [CLI](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/declaration/cli.md) | How `runique start` works |
| [Macro `admin!`](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/declaration/macro/macro.md) | Declaring administrable resources |

## Back to summary

- [Admin Summary](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/11-Admin.md)
