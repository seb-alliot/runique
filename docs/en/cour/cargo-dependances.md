# Cargo & Dependencies
> Manage a Rust project from A to Z: Cargo.toml, versions, features, workspaces, profiles, publishing, and what each declaration really changes at compile time.

## Objectives

- Understand what Cargo does with each command
- Read and write a complete `Cargo.toml`
- Choose a version constraint and understand `Cargo.lock`
- Declare dependencies in every way (crates.io, git, path, platform, renaming)
- Design features: optional, additive, exclusive
- Understand feature unification and the resolver
- Organise a workspace and share versions, dependencies and lints
- Tune the build profiles
- Publish to crates.io

---

## Table of contents

1. Introduction to Cargo
2. The `[package]` manifest
3. Declaring a dependency
4. Versions and `Cargo.lock`
5. The different kinds of dependencies
6. Features: declaration
7. Features: what they change at compile time
8. Feature unification and the resolver
9. Exclusive features
10. Workspaces
11. Targets: lib, binaries, examples
12. Build profiles
13. Lints in Cargo.toml
14. Essential commands
15. Publishing to crates.io
16. Cheat sheet

---

## 1. Introduction to Cargo

Cargo is Rust's build tool **and** its package manager. It:

- downloads dependencies from crates.io (or git, or a local folder);
- picks a compatible version for each one and freezes it in `Cargo.lock`;
- calls `rustc` for each crate, in the right order, with the right features;
- runs the tests, the documentation, the linter, the formatter;
- publishes a crate.

### Structure of a project

```
my_project/
├── Cargo.toml        ← the manifest: name, version, dependencies, features
├── Cargo.lock        ← exact versions chosen (generated, do not edit by hand)
├── src/
│   ├── main.rs       ← entry point of a binary
│   ├── lib.rs        ← root of a library (optional)
│   └── bin/
│       └── tool.rs   ← extra binary
├── tests/            ← integration tests
├── examples/         ← runnable examples (cargo run --example name)
├── benches/          ← benchmarks
├── build.rs          ← script run before compilation (optional)
└── target/           ← build output (do not commit)
```

Cargo finds these files by itself from their location: no declaration is needed as long as you follow this structure.

### A crate, a package

- A **package** is a folder with a `Cargo.toml`.
- A **crate** is a compilation unit: a library (`lib.rs`) or a binary (`main.rs`, `src/bin/*.rs`).
- A package contains **at most one** library and as many binaries as you want.

---

## 2. The `[package]` manifest

### 2.1 Main fields

```toml
[package]
name = "my_project"
version = "0.1.0"
edition = "2024"
rust-version = "1.85"
authors = ["Your Name <you@email.com>"]
description = "A short description"
license = "MIT OR Apache-2.0"
repository = "https://github.com/me/my_project"
readme = "README.md"
keywords = ["web", "framework"]
categories = ["web-programming::http-server"]
```

| Field | What it is for |
|---|---|
| `name` | name of the crate; `-` becomes `_` in code (`use my_project::...`) |
| `version` | SemVer version of your crate |
| `edition` | version of the **language** used to compile this crate |
| `rust-version` | minimum compiler version (MSRV); Cargo refuses to build with an older one |
| `license`, `description` | required to publish |
| `keywords` (5 max), `categories` | search on crates.io |

### 2.2 The edition: what it changes

The edition is **not** the compiler version: a recent compiler can build the 2015, 2018, 2021 and 2024 editions. It fixes some language rules for the crate.

| Edition | Examples of changes |
|---|---|
| 2018 | `use crate::...`, `async`/`await` become keywords |
| 2021 | `array.into_iter()` yields values, closures capture fields, resolver 2 by default |
| 2024 | let-chains (`if let ... && ...`), `if let` temporaries dropped before `else`, `gen` reserved, `unsafe extern`, resolver 3 by default |

Each crate has its own edition: your 2024 project can depend on a 2018 crate without any problem. `cargo fix --edition` migrates a crate to the next edition.

The 2024 edition requires at least Rust 1.85, hence the matching `rust-version`.

### 2.3 Inheriting fields from the workspace

```toml
[package]
name = "runique"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
```

See section 10.

---

## 3. Declaring a dependency

### 3.1 All the ways to write it

```toml
[dependencies]
# Short form: a version from crates.io
serde_json = "1.0"

# Inline table form: version + options
serde = { version = "1.0", features = ["derive"] }

# Without the default features, then pick yours
imagesize = { version = "0.15", default-features = false, features = ["heif"] }

# Optional: compiled only if a feature asks for it
sea-orm = { version = "2.0", optional = true }

# From git (branch, tag or commit)
my_crate = { git = "https://github.com/user/my_crate" }
my_crate2 = { git = "https://github.com/user/my_crate", branch = "dev" }
my_crate3 = { git = "https://github.com/user/my_crate", tag = "v1.2.0" }
my_crate4 = { git = "https://github.com/user/my_crate", rev = "a1b2c3d" }

# From a local folder
utils = { path = "../utils" }

# Local during development, crates.io when publishing
derive_form = { path = "../derive_form", version = "4.0.0" }

# Rename a dependency: it is used under the name `json` in the code
json = { package = "serde_json", version = "1.0" }
```

### 3.2 Expanded table form

Both notations are strictly equivalent; the expanded table is easier to read when there are many options.

```toml
[dependencies]
tokio = { version = "1", features = ["rt-multi-thread", "macros", "net"] }

[dependencies.tokio]
version = "1"
features = ["rt-multi-thread", "macros", "net"]
```

### 3.3 Adding a dependency from the command line

```bash
cargo add serde --features derive        # [dependencies]
cargo add tokio -F full                  # -F is short for --features
cargo add pretty_assertions --dev        # [dev-dependencies]
cargo add cc --build                     # [build-dependencies]
cargo add sea-orm --optional             # optional = true
cargo add imagesize --no-default-features -F heif
cargo add utils --path ../utils
cargo remove serde
```

`cargo add` writes the latest version compatible with your `rust-version`.

### 3.4 `default-features = false`: what it is for

Most crates enable default features. Disabling them lets you:

- **reduce compile time and size**: build only what you use;
- **choose an implementation**: for example a TLS backend (`rustls` rather than `native-tls`);
- **build without the standard library** (`no_std`) for embedded targets.

```toml
# The crate pulls native-tls by default; we want rustls instead
reqwest = { version = "0.13", default-features = false, features = ["rustls", "json"] }
```

Careful: `default-features = false` only applies to **your** declaration. If another crate in the tree depends on `reqwest` with its default features, they will be enabled anyway (section 8).

---

## 4. Versions and `Cargo.lock`

### 4.1 SemVer in brief

A version is written `MAJOR.MINOR.PATCH`:

- **MAJOR** changes: a breaking change is possible;
- **MINOR** changes: compatible additions;
- **PATCH** changes: compatible fixes.

For `0.x` versions, Cargo treats **the first non-zero digit** as the breaking digit: going from `0.3` to `0.4` can break your code.

### 4.2 Version constraints

| Notation | Accepted versions | Use |
|---|---|---|
| `"1.2.3"` or `"^1.2.3"` | `>=1.2.3, <2.0.0` | default (caret) |
| `"1.2"` | `>=1.2.0, <2.0.0` | same, any patch |
| `"1"` | `>=1.0.0, <2.0.0` | any 1.x |
| `"0.3.5"` | `>=0.3.5, <0.4.0` | in 0.x, the minor is the breaking digit |
| `"0.0.4"` | `>=0.0.4, <0.0.5` | in 0.0.x, any change can break |
| `"~1.2.3"` | `>=1.2.3, <1.3.0` | tilde: patches only |
| `"1.*"` | `>=1.0.0, <2.0.0` | wildcard |
| `"=2.0.4"` | exactly `2.0.4` | pin a version |
| `">=1.2, <1.5"` | explicit range | rare cases |
| `"*"` | all | **rejected** by crates.io when publishing |

The default form (`"1.2"`) is right in almost every case: it lets Cargo take patches without accepting a breaking change.

### 4.3 When to pin with `=`

```toml
sea-orm = "=2.0.4"
```

Pinning is useful when a crate breaks compatibility **without** changing its breaking digit (it happens), or when two crates must stay exactly aligned (`sea-orm` and `sea-orm-migration`). The price: your crate cannot live alongside another one that asks for `sea-orm = "2.0.5"`, because Cargo cannot pick a version that satisfies both.

### 4.4 Pre-release versions

A version such as `2.0.0-rc.3` is **never** chosen by a constraint that does not name it: `"2"` will not pick an `-rc`. You have to write it explicitly (`"2.0.0-rc.3"`).

### 4.5 `Cargo.lock`

`Cargo.toml` says **what you accept**; `Cargo.lock` says **what was chosen**, down to the patch, for all dependencies, including indirect ones.

```bash
cargo update                 # takes the latest compatible version of everything
cargo update -p serde        # only serde
cargo update -p serde --precise 1.0.200   # a precise version
```

| Project type | Commit `Cargo.lock`? | Why |
|---|---|---|
| Binary, application, server | **yes** | the whole chain (CI, production) builds exactly the same versions |
| Library | also recommended | it serves your CI; it is **ignored** by projects that depend on your library |

Key point: the `Cargo.lock` of a library has **no** effect on its users. They resolve versions according to the constraints in your `Cargo.toml`. If your library only builds with a precise version of a dependency, you must write it in `Cargo.toml`.

`cargo install` also ignores the `Cargo.lock` of a published crate, unless you pass `--locked`.

### 4.6 Several versions of the same crate

If two dependencies ask for `rand = "0.8"` and `rand = "0.9"`, Cargo builds **both**: they are not compatible with each other, so they are two different crates. Consequences:

- longer compile times;
- a `rand::Rng` from 0.8 is not the same trait as the one from 0.9: passing a value from one to the other gives a confusing type error (`expected Rng, found Rng`).

```bash
cargo tree -d                # lists the crates present in several versions
```

---

## 5. The different kinds of dependencies

### 5.1 `[dependencies]`

Compiled into your crate and visible to those who depend on it (for a library).

### 5.2 `[dev-dependencies]`

```toml
[dev-dependencies]
pretty_assertions = "1.4"
reqwest = { version = "0.13", features = ["json"] }
```

Used **only** by tests, examples and benchmarks. They are never compiled into the shipped library, and they are not downloaded by those who depend on your crate.

Extra use: enabling a feature **only for tests**.

```toml
[dependencies]
runique = { version = "4.0", features = ["postgres"] }

[dev-dependencies]
runique = { version = "4.0", features = ["test-utils"] }
```

During `cargo test`, the two declarations are merged: `postgres` and `test-utils` are both active. With `cargo build`, only `postgres` is.

### 5.3 `[build-dependencies]`

```toml
[build-dependencies]
cc = "1.0"
toml = "1"
```

Used by `build.rs`, a program compiled and run **before** your crate (generate code, compile C, read a configuration file). They are not available in `src/`. If you need the same crate in both, declare it in both sections (Runique does this for `toml`).

### 5.4 Platform-specific dependencies

```toml
[target.'cfg(unix)'.dependencies]
libc = "0.2"

[target.'cfg(windows)'.dependencies]
windows-sys = "0.59"

[target.'cfg(target_arch = "wasm32")'.dependencies]
getrandom = { version = "0.3", features = ["wasm_js"] }
```

They are compiled only for the target platform. The code that uses them guards itself with the same `#[cfg(unix)]`.

### 5.5 Replacing a dependency: `[patch]`

```toml
[patch.crates-io]
sea-orm = { git = "https://github.com/SeaQL/sea-orm", branch = "master" }
```

Replaces `sea-orm` **everywhere in the tree**, including when it is an indirect dependency. What it is for: testing a fix that is not published yet. It only applies to the root project (it is ignored when your crate is used as a dependency).

### 5.6 A workspace declaration does nothing on its own

```toml
# Root Cargo.toml
[workspace.dependencies]
time = "=0.3.55"
```

This line forces **nothing** as long as no member writes `time.workspace = true`. To pin an indirect dependency (one you do not use yourself), you must declare it directly in the relevant member: this is what Runique does for `time`, otherwise `cargo install` would pick a newer version incompatible with `cookie`.

---

## 6. Features: declaration

A **feature** is a name that enables conditional code in your crate and/or optional dependencies or features of your dependencies.

### 6.1 The four things a feature can enable

```toml
[features]
default = ["json"]                    # 1. other features of your crate
json = ["dep:serde_json"]             # 2. an optional dependency
derive = ["serde/derive"]             # 3. a feature of a dependency
full = ["json", "derive", "async"]    # group of features
async = ["dep:tokio", "tokio/rt"]     # combination
admin = []                            # 4. nothing else: only code under #[cfg]

[dependencies]
serde = "1.0"
serde_json = { version = "1.0", optional = true }
tokio = { version = "1", optional = true }
```

### 6.2 The `default` feature

```toml
[features]
default = ["orm"]
```

Enabled automatically unless the user writes `default-features = false` (or `--no-default-features`). Keep it **small**: everything listed there will be compiled for all your users, and they can only remove it as a whole.

### 6.3 Optional dependency: `dep:` or implicit feature

```toml
[dependencies]
serde_json = { version = "1.0", optional = true }
```

Notation 1: **without** `dep:`, Cargo automatically creates a feature named `serde_json`. Users can enable it directly (`features = ["serde_json"]`), and this name becomes part of your public API.

```toml
[features]
json = ["serde_json"]       # json enables the implicit serde_json feature
```

Notation 2: **with** `dep:` (Rust 1.60+), the implicit feature is no longer created. Only `json` is visible.

```toml
[features]
json = ["dep:serde_json"]
```

The second form is preferable: you choose the exposed names, and you can replace `serde_json` with another crate without breaking anyone. Runique uses it: `orm = ["dep:sea-orm", "dep:sea-orm-migration"]`.

### 6.4 "Weak" feature: `?`

```toml
[features]
serde = ["dep:serde", "chrono?/serde"]

[dependencies]
serde = { version = "1.0", optional = true }
chrono = { version = "0.4", optional = true }
```

- `"chrono/serde"` enables chrono's `serde` **and** forces chrono to be compiled.
- `"chrono?/serde"` enables chrono's `serde` **only if** chrono is already enabled elsewhere.

What it is for: "if you use dates, make them serialisable", without forcing dates on someone who only wants serde.

### 6.5 Enabling features on a dependency

```toml
[dependencies]
runique = { version = "4.0", features = ["postgres", "admin"] }
```

```bash
cargo build --features "json,async"      # features of the current package
cargo build -F json -F async             # same
cargo build --features runique/admin     # feature of a dependency, from the command line
cargo build --all-features               # all of them (careful: section 9)
cargo build --no-default-features        # no default feature
cargo build --no-default-features -F json
```

---

## 7. Features: what they change at compile time

A feature is **not** an option read at run time: it changes **the compiled code**. Each combination of features produces a different program.

### 7.1 `#[cfg(feature = "...")]`: the code disappears

```rust
#[cfg(feature = "json")]
pub mod json {
    pub fn parse(input: &str) -> serde_json::Value {
        serde_json::from_str(input).unwrap_or_default()
    }
}

#[cfg(feature = "admin")]
pub fn with_admin(self) -> Self { self }
```

Without the feature, the module does not exist: it is neither compiled, nor checked, nor present in the binary. A user who calls `my_project::json::parse` without the feature gets "cannot find module `json`".

Direct consequence: **an error inside a `#[cfg(feature = "x")]` block only shows up if you compile with `x`**. So you must test every combination that matters (this is why Runique's CI builds several batches of features).

### 7.2 Combining `cfg`

```rust
#[cfg(all(feature = "postgres", feature = "admin"))]
fn a() {}

#[cfg(any(feature = "sqlite", feature = "postgres"))]
fn b() {}

#[cfg(not(feature = "admin"))]
fn c() {}

#[cfg(all(unix, not(test)))]
fn d() {}
```

### 7.3 `cfg!`: the code stays, the value changes

```rust
fn engine() -> &'static str {
    if cfg!(feature = "postgres") {
        "postgres"
    } else {
        "sqlite"
    }
}
```

`cfg!(...)` is replaced at compile time by `true` or `false`. Unlike `#[cfg]`, **both branches are compiled** and so must be valid even when the feature is absent. You cannot call a function there that only exists under the feature.

| Form | Is the inactive code compiled? | Use |
|---|---|---|
| `#[cfg(feature = "x")]` | no, it disappears | modules, functions, `use`, fields that depend on an optional crate |
| `cfg!(feature = "x")` | yes, checked but never run | small choice of value, message, path |

### 7.4 `#[cfg_attr]`: a conditional attribute

```rust
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Point {
    pub x: i32,
    pub y: i32,
}
```

`Point` only implements `Serialize` if the user enables `serde`. What it is for: offering integration with a crate without forcing it.

### 7.5 On `use`, fields and variants

```rust
#[cfg(feature = "orm")]
use sea_orm::DatabaseConnection;

pub struct Engine {
    pub name: String,
    #[cfg(feature = "orm")]
    pub db: DatabaseConnection,
}
```

Careful: a field or an enum variant under `#[cfg]` changes the **shape** of the type. User code that builds `Engine { name }` will stop compiling the day someone else in the tree enables `orm` (section 8). For a public type, prefer a field that is always present, or a constructor.

### 7.6 Showing the feature in the documentation

```toml
[package.metadata.docs.rs]
features = ["all-databases"]
# or: all-features = true
```

docs.rs builds your documentation with these features; without this section, only the default features are documented and all the code under `#[cfg(feature)]` is missing from the docs.

---

## 8. Feature unification and the resolver

### 8.1 The principle: the union

Within **a single build**, a crate is compiled **only once** per version. If several crates in the tree depend on it with different features, Cargo enables **the union** of all the requested features.

```
my_app ──► serde (features: [])
   └─► my_lib ──► serde (features: ["derive"])

Result: serde is compiled ONCE, with "derive".
```

Consequences:

- `default-features = false` on your side does not prevent another crate from re-enabling the default features;
- your code may compile **by accident** thanks to a feature enabled by another crate, then break the day it leaves the tree. Always declare the features you use yourself.

### 8.2 Golden rule: features are additive

Since Cargo can enable a feature without you asking, **enabling a feature must never remove or change an existing behaviour**, only add some.

```toml
# Bad: no_std removes the standard library
[features]
no_std = []

# Good: std adds it, enabled by default
[features]
default = ["std"]
std = []
```

```rust
// Bad: the feature changes the meaning of an existing function
#[cfg(feature = "fast")]
pub fn hash(x: &str) -> u64 { /* algorithm A */ 0 }
#[cfg(not(feature = "fast"))]
pub fn hash(x: &str) -> u64 { /* algorithm B */ 1 }
// Another crate in the tree enables "fast": your stored hashes no longer match.
```

### 8.3 The resolver: when the union applies

```toml
[package]
edition = "2024"      # resolver 3 by default

# In a workspace, it is declared at the root:
[workspace]
resolver = "3"
```

| Resolver | Behaviour |
|---|---|
| `"1"` | union of everything, everywhere: the features of dev-dependencies and build-dependencies leak into the normal build |
| `"2"` (default in the 2021 edition) | does not merge the features of **build-dependencies** and procedural macros with those of the normal code; ignores dependencies of another platform; only enables the features of **dev-dependencies** when building tests and examples |
| `"3"` (default in the 2024 edition) | like 2, and also picks versions compatible with `rust-version` |

A **virtual** workspace (a root without `[package]`) has no edition: you must write `resolver` explicitly, otherwise Cargo falls back to resolver 1 and prints a warning.

### 8.4 Unification pitfalls in a workspace

If you build the whole workspace (`cargo build` at the root), the features requested by **all** the members are unified. If you build a single member (`cargo build -p my_lib`, or from its folder), only its own requests count.

Result: `cargo test` at the root can succeed while `cargo test -p my_lib` fails (or the opposite). This is why Runique's tests are run from `runique/`: at the root, the features of `demo-app` would mix with those of the framework.

### 8.5 Seeing who enables what: `cargo tree`

```bash
cargo tree                       # dependency tree
cargo tree -i serde              # who depends on serde (inverted tree)
cargo tree -e features           # shows the features enabled on each edge
cargo tree -e features -i serde  # who enables which feature of serde
cargo tree -d                    # crates in several versions
cargo tree --no-default-features -F postgres   # the tree for a precise combination
```

---

## 9. Exclusive features

### 9.1 The problem

Sometimes two features **cannot** coexist: two database engines, two TLS backends, two primary key types. Unification (section 8) makes this dangerous: another crate can enable the second one without your knowing.

### 9.2 Blocking at compile time: `compile_error!`

```rust
#[cfg(all(feature = "postgres", feature = "sqlite"))]
compile_error!("features `postgres` and `sqlite` are exclusive: pick a single engine");
```

The build stops with a clear message, instead of inconsistent behaviour or an obscure type error.

Runique does exactly this for `sqlite`, `postgres` and `mysql`, with an explicit escape hatch:

```rust
#[cfg(all(
    not(doc),
    not(feature = "all-databases"),
    feature = "postgres",
    feature = "sqlite"
))]
compile_error!("features `postgres` and `sqlite` are mutually exclusive");
```

- `not(doc)`: `cargo doc` and docs.rs can document all the engines;
- `all-databases`: a feature you have to **ask for by name**, for tools that talk to all three engines.

### 9.3 Consequence: `--all-features` no longer works

With exclusive features, `cargo test --all-features` triggers the `compile_error!`. You must test **each valid combination** separately:

```bash
cargo test --features postgres
cargo test --features sqlite
cargo test --features big-pk,all-databases
cargo test --features pk-uuid,all-databases
```

In CI, a matrix runs these batches in parallel.

### 9.4 The alternative: choose at run time

When possible, avoid exclusivity: compile both implementations and choose at run time (environment variable, configuration). Features stay additive, at the cost of a bigger binary. Exclusive features are justified when the two implementations cannot live in the same binary, or change public types (like the primary key type).

---

## 10. Workspaces

A workspace groups several packages that share a single `Cargo.lock` and a single `target/` folder.

### 10.1 Root

```toml
# Root Cargo.toml (virtual workspace: no [package])
[workspace]
members = [
    "runique/runique_dsl",
    "runique/derive_form",
    "runique",
    "demo-app",
]
resolver = "3"

[workspace.package]
version = "4.0.0"
edition = "2024"
rust-version = "1.94"
license = "MIT"

[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
derive_form = { path = "runique/derive_form", version = "4.0.0" }
```

`members` accepts wildcards (`members = ["crates/*"]`) and `exclude` removes folders.

### 10.2 Members

```toml
# runique/Cargo.toml
[package]
name = "runique"
version.workspace = true
edition.workspace = true

[dependencies]
serde.workspace = true
# Inherit and add: features are added to those of the workspace
tokio = { workspace = true, features = ["test-util"] }
# Inherit and make optional
sea-orm = { workspace = true, optional = true }
derive_form.workspace = true
```

What you can add to an inherited dependency: `features` (added), `optional`. You cannot change the version or remove a feature.

### 10.3 What it is for

- a single version of each dependency for the whole project;
- common dependencies compiled once;
- a common version for crates published together;
- `cargo test --workspace` tests everything at once.

```bash
cargo build -p runique          # a single member
cargo test --workspace          # all members
cargo build --workspace --exclude demo-app
```

### 10.4 Publishing order

A member that depends on another through `path` must also give a `version` to be publishable. You publish in dependency order: for Runique, `runique_dsl`, then `derive_form`, then `runique`.

---

## 11. Targets: lib, binaries, examples

### 11.1 Implicit and explicit declaration

```toml
# Implicit: src/lib.rs, src/main.rs, src/bin/*.rs, examples/*.rs are found automatically.

# Explicit: different name or path
[lib]
name = "runique"
path = "src/lib.rs"

[[bin]]
name = "runique"
path = "src/bin/runique.rs"

[[example]]
name = "demo"
required-features = ["admin"]
```

`[[bin]]` (double brackets) is an array: you can declare several.

### 11.2 `required-features`

```toml
[[bin]]
name = "admin-tool"
required-features = ["admin"]
```

The binary is only built if the feature is active. Without it, `cargo build` skips it without an error.

### 11.3 A binary is a separate crate

A binary in `src/bin/` uses the library of the same package **as an external dependency**: it only sees what is `pub`, not what is `pub(crate)`.

```bash
cargo run --bin tool
cargo run --example demo
cargo install --path . --features postgres
```

---

## 12. Build profiles

### 12.1 The default profiles

| Profile | Used by | Default setting |
|---|---|---|
| `dev` | `cargo build`, `cargo run` | `opt-level = 0`, full debug info, integer overflow checks on |
| `release` | `--release` | `opt-level = 3`, no overflow checks |
| `test` | `cargo test` | inherits from `dev` |
| `bench` | `cargo bench` | inherits from `release` |

### 12.2 Common settings

```toml
[profile.release]
opt-level = 3          # 0, 1, 2, 3, "s" (size), "z" (minimal size)
lto = "fat"            # link-time optimisation: faster binary, slower build
codegen-units = 1      # less parallelism, better optimisation
strip = true           # removes symbols: smaller binary
panic = "abort"        # no stack unwinding: smaller, but no more catch_unwind
debug = false

[profile.dev]
opt-level = 0
debug = true
incremental = true
```

### 12.3 Optimising dependencies in dev

```toml
[profile.dev.package."*"]
opt-level = 2
```

All the dependencies are optimised, your code stays at `opt-level = 0` (fast build). What it is for: some crates are unusable in debug. Hashing a password with argon2 can take several seconds without optimisation and a few dozen milliseconds with it.

### 12.4 Custom profile

```toml
[profile.profiling]
inherits = "release"
debug = true
```

```bash
cargo build --profile profiling
```

### 12.5 What a profile changes for your code

```rust
fn main() {
    // Value read at run time: with a hard-coded constant,
    // the compiler would detect the overflow and refuse to compile.
    let x: u8 = std::env::args().count() as u8 + 254;
    let y = x + 1;     // dev: panics "attempt to add with overflow"
                       // release: equals 0 (wrap-around), no error
}
```

Profiles only belong to the root project: those declared in your dependencies are ignored.

---

## 13. Lints in Cargo.toml

Since Rust 1.74, lints are declared in the manifest rather than at the top of `lib.rs`.

```toml
[lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"

[lints.clippy]
unwrap_used = "warn"
pedantic = { level = "warn", priority = -1 }
module_name_repetitions = "allow"
```

In a workspace:

```toml
# root
[workspace.lints.clippy]
unwrap_used = "warn"

# member
[lints]
workspace = true
```

`priority = -1` applies the group first, so that a precise rule (`allow`) can override it afterwards.

---

## 14. Essential commands

```bash
cargo new my_project          # new binary
cargo new my_lib --lib        # new library
cargo init                    # in an existing folder

cargo check                   # analyses and type-checks without producing a binary: the fastest
cargo build                   # builds in debug
cargo build --release         # builds in release
cargo run -- arg1 arg2        # builds, runs, passes the arguments after --

cargo test                    # all tests
cargo test test_name          # tests whose name contains this text
cargo test -- --nocapture     # show the println! output of tests

cargo clippy --all-targets -- -D warnings   # linter, warnings as errors
cargo fmt                     # format
cargo fmt --check             # check without modifying (CI)

cargo doc --open              # local documentation
cargo tree                    # dependency tree
cargo update                  # update Cargo.lock
cargo clean                   # empty target/

cargo install ripgrep         # install a published binary
cargo install --path .        # install the binary of the current project
```

`cargo check` does all the analysis (types, borrows) but stops before machine code generation: it is the command to run during development.

---

## 15. Publishing to crates.io

```bash
cargo login <your_token>      # once
cargo package --list          # files that will be uploaded
cargo publish --dry-run       # checks that the package builds on its own
cargo publish
```

### Checklist before publishing

- `name`, `version`, `description`, `license` filled in
- `README.md` present (shown on crates.io)
- No `"*"` constraint
- Each `path` or `git` dependency also has a `version` (otherwise crates.io refuses)
- `exclude` or `include` to avoid uploading heavy tests, SQLite databases, local files
- `///` documentation on public items, `[package.metadata.docs.rs]` if features are needed
- Passing tests **for each combination of features** that matters

### After publishing

- A published version is **final**: you can neither modify it nor delete it.
- `cargo yank --version 1.2.3` prevents **new** projects from choosing it; those who already have it in their `Cargo.lock` keep using it.
- Any break in the public API requires bumping the breaking digit (the major, or the minor in 0.x). Removing a feature or making one mandatory is a breaking change.

---

## 16. Cheat sheet

### Dependencies

```toml
a = "1.2"                                        # >=1.2.0, <2.0.0
b = "0.3"                                        # >=0.3.0, <0.4.0
c = "=2.0.4"                                     # exactly
d = { version = "1", features = ["x"] }
e = { version = "1", default-features = false }
f = { version = "1", optional = true }
g = { path = "../g", version = "1" }
h = { git = "https://...", tag = "v1" }
i = { package = "real_name", version = "1" }     # renaming
j.workspace = true
```

### Features

```toml
[features]
default = ["a"]
a = []                       # code under #[cfg(feature = "a")]
b = ["dep:opt_crate"]        # optional dependency without an implicit feature
c = ["serde/derive"]         # feature of a dependency
d = ["chrono?/serde"]        # only if chrono is already active
```

```rust
#[cfg(feature = "a")]            // the code disappears without the feature
if cfg!(feature = "a") { }       // compiled in all cases, value fixed
#[cfg_attr(feature = "s", derive(serde::Serialize))]
#[cfg(all(feature = "x", feature = "y"))]
compile_error!("x and y are exclusive");
```

### Key points

- The edition fixes the language rules of a crate, not the compiler version.
- In `0.x`, the minor is the breaking digit.
- The `Cargo.lock` of a library has no effect on its users.
- A feature changes the compiled code: test each important combination.
- Cargo enables the union of the features requested in the tree: features must be additive.
- Exclusive features protect themselves with `compile_error!` and forbid `--all-features`.
- `dep:` avoids exposing the names of your optional dependencies.
- An entry in `[workspace.dependencies]` does nothing until a member references it.
- `cargo tree -e features -i crate` answers "who enables this feature?".
