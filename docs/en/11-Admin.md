##  Admin View (Beta)

Runique’s admin view is based on a **declarative macro (`admin!`)** combined with a **generation daemon**.

The goal is to provide a **transparent, auditable, and type-safe** approach:
the generated admin code is “normal” Rust — readable, inspectable, and modifiable if necessary.

---

## 1) Declaring Resources via `admin!`

Developers declare administrable resources inside `src/admin.rs`.

Each resource is defined by:

* a **key** (`users`, `blog`, …)
* a **model** (Rust type path)
* a **form**
* a **title**
* a list of allowed **roles**

Example:

```rust
admin! {
    users: users::Model => RegisterForm {
        title: "Users",
        permissions: ["admin", "staff"]
    }
}
```

The macro generates an `admin_config()` function that builds an `AdminRegistry` and registers each resource via `AdminResource`.

### Type-safe

The macro includes a *compile-time* verification: if a referenced model or form does not exist, compilation fails with an explicit error.

---

## 2) What Can Be Declared in `admin!`

The `admin!` macro allows declaring **only the essential metadata** of an admin resource.
It does **not** describe business logic, authentication, or HTML rendering.

### Supported Fields

For each resource, the following fields are **required**:

| Field         | Description                                           |
| ------------- | ----------------------------------------------------- |
| `key`         | Resource identifier (used in routes `/admin/{key}/…`) |
| `model`       | Rust model path (e.g., `users::Model`)                |
| `form`        | Runique form type                                     |
| `title`       | Display title in the admin interface                  |
| `permissions` | List of allowed roles                                 |

### Permissions

```rust
permissions: ["admin", "staff"]
```

* Permissions are expressed as **roles**
* The list applies **uniformly to all CRUD operations** in the current version

### What Is Intentionally Not Declarable

The `admin!` macro does not allow declaring:

* Different permissions per CRUD operation
* Conditional rules
* HTML rendering or templates
* Business logic
* Complex filters or relationships

These limitations are intentional: the macro remains simple and readable, while the logic resides in the generated Rust code.

---

## 3) Parsing: Reading `src/admin.rs`

When running `runique start`, the daemon:

* reads `src/admin.rs`
* parses the `admin! { ... }` macro using `syn`
* extracts resources into `ResourceDef` structures

Each resource includes:

* `key`
* `model_type`
* `form_type`
* `title`
* `permissions`

The parser validates the presence of required fields and reports explicit errors for invalid syntax.

---

## 4) Generation: Creating `src/admins/`

From the parsed resources, Runique automatically generates:

```
src/admins/
  ├─ README.md
  ├─ mod.rs
  ├─ router.rs
  └─ handlers.rs
```

* **`router.rs`**: CRUD routes (`list`, `create`, `detail`, `edit`, `delete`)
* **`handlers.rs`**: SeaORM + form handlers (GET/POST, validation, rendering)
* **`mod.rs`**: admin module entry point
* **`README.md`**: warning that the folder is auto-generated

---

## 5) Daemon / Watcher: Automatic Regeneration

The `runique start` command launches a watcher (based on `notify`) that monitors `src/admin.rs`.

On each detected modification:

1. the file is re-read
2. the macro is parsed
3. the `src/admins/` folder is regenerated
4. simple feedback is displayed (✅ or ❌)

A *debounce* mechanism prevents multiple regenerations from a single file save.

---

## ⚠️ Intentional Trade-off: Overwriting `src/admins/`

This workflow involves a deliberate trade-off:

* `runique start` **deletes and fully regenerates** the `src/admins/` folder
* any manual modifications in this folder will be **overwritten**

If manual changes are required, you must **avoid using `runique start`** and switch to a `cargo run` workflow to prevent automatic regeneration.

---

## 🔐 Permissions and Roles (Based on the `users` Table)

The permission system relies on the authenticated user and data stored in the **`users` table**.

Access control checks rely on:

* **`is_active`**: the user must be active
* **`is_staff`**: admin access authorization
* **`is_superuser`**: full access
* **`roles`** *(optional)*: custom roles (e.g., `"admin"`, `"editor"`)

Permissions declared in the macro:

```rust
permissions: ["admin", "staff"]
```

are compared against the current user’s roles and attributes.
A user is authorized if they possess **at least one compatible role** or an equivalent status (e.g., superuser).

> The `roles` field allows flexible permission management without enforcing a rigid schema.

---

## 📌 Important Notes

* The `admin!` macro defines **declarative rules**, not authentication logic
* Access checks are performed **at runtime** via admin middlewares
* The `users` table remains the **single source of truth** for authorization

---

## 🚧 Current State (Beta)

* Automatic generation of CRUD routes and handlers
* Central resource registry (`AdminRegistry`)
* Global permissions per resource
* Feedback mainly **structural** (missing daemon, missing file, invalid declaration)

Improvements in error feedback, permission granularity, and workflow safety are planned evolution areas.

---

### 🏁 Conclusion

This architecture prioritizes:

* **Readability**
* **Type-driven safety**
* **Developer control over generated code**

The admin view is intentionally simple, explicit, and evolvable.

---
