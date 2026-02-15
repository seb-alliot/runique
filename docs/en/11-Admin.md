
---

##  Administration View (Beta)

Runique’s administration view is built on a **declarative macro (`admin!`)** combined with a **code generation daemon**.

The goal is to provide a **transparent, auditable, and type-safe** approach:
the generated admin code is regular Rust code, readable, inspectable, and modifiable when needed.

---

## 1) Declaring resources with `admin!`

Administrative resources are declared in the `src/admin.rs` file.

Each resource is defined by:

* a **key** (`users`, `blog`, …)
* a **model** (Rust type path)
* a **form**
* a **title**
* a list of **authorized roles**

Example:

```rust
admin! {
    users: users::Model => RegisterForm {
        title: "Users",
        permissions: ["admin", "staff"]
    }
}
```

The macro generates an `admin_config()` function that builds an `AdminRegistry` and registers each resource using `AdminResource`.

 **Type-safe**
The macro includes *compile-time* checks: if a referenced model or form does not exist, compilation fails with an explicit error.

---

## 2) What can be declared in the `admin!` macro

The `admin!` macro is intentionally limited to **essential admin metadata**.
It does **not** describe business logic, authentication, or HTML rendering.

### Supported fields

For each resource, the following fields are **mandatory**:

| Field         | Description                                            |
| ------------- | ------------------------------------------------------ |
| `key`         | Resource identifier (used in routes: `/admin/{key}/…`) |
| `model`       | Rust model path (e.g. `users::Model`)                  |
| `form`        | Runique form type                                      |
| `title`       | Title displayed in the admin interface                 |
| `permissions` | List of authorized roles                               |

### Permissions

```rust
permissions: ["admin", "staff"]
```

* Permissions are expressed as **roles**
* In the current version, the list applies **uniformly to all CRUD operations**

### ❌ What is intentionally not declarable

The `admin!` macro does **not** allow:

* per-CRUD-operation permissions
* conditional rules
* HTML or template configuration
* business logic
* complex filters or relations

These limitations are **intentional**: the macro remains simple and declarative, while logic lives in the generated Rust code.

---

## 3) Parsing: reading `src/admin.rs`

When running `runique start`, the daemon:

* reads `src/admin.rs`
* parses the `admin! { ... }` macro using `syn`
* extracts resources as `ResourceDef` values

Each resource contains:

* `key`
* `model_type`
* `form_type`
* `title`
* `permissions`

The parser validates required fields and reports explicit errors in case of invalid syntax or missing data.

---

## 4) Generation: creating `src/admins/`

From the parsed resources, Runique automatically generates the following directory:

```
src/admins/
  ├─ README.md
  ├─ mod.rs
  ├─ router.rs
  └─ handlers.rs
```

* **`router.rs`**: assembles CRUD routes (`list`, `create`, `detail`, `edit`, `delete`)
* **`handlers.rs`**: SeaORM + form handlers (GET/POST, validation, rendering)
* **`mod.rs`**: admin module entry point
* **`README.md`**: warning that the folder is auto-generated

---

## 5) Daemon / watcher: automatic regeneration

The `runique start` command launches a watcher (based on `notify`) that monitors `src/admin.rs`.

On each detected change:

1. the file is read
2. the macro is parsed
3. the `src/admins/` directory is regenerated
4. a simple result is displayed (✅ or ❌)

A debounce mechanism prevents multiple regenerations for a single save.

---

##  Explicit trade-off: `src/admins/` is overwritten

This workflow comes with an intentional trade-off:

* `runique start` **fully removes and regenerates** the `src/admins/` directory
* any manual changes inside this directory will be **overwritten**

If manual changes are required, do **not** use `runique start`.
Instead, switch to a `cargo run` workflow to avoid automatic regeneration.

---

##  Permissions and roles (based on the `users` table)

The permission system relies on the authenticated user and data stored in the **`users` table**.

Access checks are based on fields such as:

* **`is_active`**: the user must be active
* **`is_staff`**: grants access to the admin interface
* **`is_superuser`**: full access to all resources and operations
* **`roles`** *(optional)*: custom roles (e.g. `"admin"`, `"editor"`)

Permissions declared in the macro:

```rust
permissions: ["admin", "staff"]
```

are matched against the current user’s roles and attributes.
A user is authorized if they have **at least one matching role** or an equivalent status (e.g. superuser).

>  The `roles` field enables flexible role management without enforcing a rigid schema.

---

##  Important notes

* The `admin!` macro defines **declarative rules**, not authentication logic
* Authorization checks are performed **at runtime** via admin middlewares
* The `users` table remains the **source of truth** for authorization

---

##  Current state (beta)

* Automatic generation of CRUD routes and handlers
* Centralized resource registry (`AdminRegistry`)
* Global permissions per resource
* Feedback is mainly **structural** (missing daemon, missing file, invalid declaration)

Improvements to error feedback, permission granularity, and workflow safety are planned.

---

###  Conclusion

This architecture prioritizes:

* **readability**
* **type safety**
* **developer control over generated code**

The admin view is intentionally simple, explicit, and designed to evolve.

---
