# Modules and Organization

**Structuring a Rust project**: modules, files, visibility, paths, `use`, re-exports, library and binaries, with the different ways of writing the same organization.

## Course objectives

By the end of this course, you will know how to:

- Declare a module in the same file, in a separate file or in a folder
- Tell `mod` (declare) and `use` (import) apart
- Choose the right visibility level: private, `pub`, `pub(crate)`, `pub(super)`, `pub(in path)`
- Write an absolute or relative path, and import in every possible way
- Build a clean public API with re-exports and a prelude
- Separate the logic into a library and the interface into binaries

## Table of contents

1. Why modules
2. Module in the same file
3. Module in a separate file
4. Module tree
5. `mod` and `use`: the difference
6. Visibility
7. Visibility of structs and enums
8. Paths
9. `use`: all the forms
10. Import conventions
11. Re-exports and public API
12. Conditional modules
13. Library and binaries
14. A complete project
15. Common errors
16. Exercises
17. Cheat sheet

## 1. Why modules

A module groups code that goes together and decides what it shows to the rest of the program. It is used to:

- **organize**: find the code for a topic in a single place;
- **hide**: what is private can change without breaking the rest;
- **name**: `users::create` and `posts::create` coexist without conflict.

The root of a crate is `src/lib.rs` (library) or `src/main.rs` (binary). All the other modules descend from it.

## 2. Module in the same file

```rust
mod geometry {
    pub fn square_area(side: f64) -> f64 {
        side * side
    }

    fn internal_helper() {}          // private: invisible outside geometry

    pub mod circle {
        pub fn area(radius: f64) -> f64 {
            std::f64::consts::PI * radius * radius
        }
    }
}

fn main() {
    println!("{}", geometry::square_area(2.0));
    println!("{}", geometry::circle::area(1.0));
    // geometry::internal_helper();   // ERROR: function `internal_helper` is private
}
```

What it is for: grouping a small set of functions, or the test module at the bottom of a file (`mod tests`).

## 3. Module in a separate file

### 3.1 — Declare, then write the file

```rust
// src/main.rs
mod geometry;     // tells the compiler: the module is in geometry.rs

fn main() {
    geometry::square_area(2.0);
}
```

```rust
// src/geometry.rs — no `mod geometry { }` around it: the file IS the module
pub fn square_area(side: f64) -> f64 {
    side * side
}
```

A `.rs` file that is declared by **no** `mod` is not compiled at all: its errors do not even show up.

### 3.2 — Module with submodules: two layouts

```
Layout 1 (2018+ edition, recommended)         Layout 2 (historical)
src/                                          src/
├── main.rs                                   ├── main.rs
├── geometry.rs         ← mod circle;         └── geometry/
└── geometry/                                     ├── mod.rs      ← mod circle;
    └── circle.rs                                 └── circle.rs
```

Both are equivalent for the compiler. The first avoids having ten `mod.rs` tabs open in the editor; the second keeps the whole module in a single folder. Pick one and stick to it across the whole project (you cannot have both `geometry.rs` and `geometry/mod.rs`).

### 3.3 — Custom path

```rust
#[path = "tools/old_code.rs"]
mod compat;
```

Keep this for special cases (generated code, a file shared between several crates).

## 4. Module tree

```
src/
├── lib.rs
├── auth.rs
├── auth/
│   ├── login.rs
│   └── session.rs
└── forms.rs
```

```rust
// src/lib.rs
pub mod auth;
pub mod forms;
```

```rust
// src/auth.rs
pub mod login;
mod session;          // private: usable only in auth and its submodules
```

The resulting tree:

```
crate
├── auth
│   ├── login
│   └── session   (private)
└── forms
```

Each module only sees its parent and its siblings through paths (section 8).

## 5. `mod` and `use`: the difference

| | `mod name;` | `use path::Name;` |
|---|---|---|
| Role | **declares** a module: makes it exist, compiles its file | **imports** an existing name into the current scope |
| How many times | once in the whole project, in the parent module | as many times as needed, in each file that needs it |
| Without it | the file is not compiled | you have to write the full path |

```rust
// src/lib.rs
mod utils;            // declaration: only once

// src/forms.rs
use crate::utils::clean;      // import: in each file that uses it
// mod utils;                 // common ERROR: it would look for src/forms/utils.rs
```

`mod` is **not** an `#include`: you do not write it in each file that uses the module.

## 6. Visibility

### 6.1 — Private by default

Every item (function, struct, module, constant, trait) is private by default: visible in the module where it is defined **and its descendants**.

```rust
mod parent {
    fn secret() {}

    pub mod child {
        pub fn call() {
            super::secret();      // OK: a child sees the private items of its ancestors
        }
    }
}

fn main() {
    parent::child::call();    // OK
    // parent::secret();      // ERROR: private
}
```

The rule reads in one direction: a module sees upward (its ancestors), not downward (the private items of its children).

### 6.2 — The levels

```rust
// These declarations are in a submodule of crate::auth
// (pub(in ...) must name an ancestor of the current module).
pub fn everywhere() {}                 // visible to all, including other crates
pub(crate) fn in_the_crate() {}        // visible in the whole crate, not outside
pub(super) fn at_the_parent() {}       // visible in the parent module
pub(in crate::auth) fn in_auth() {}    // visible in crate::auth and its descendants
pub(self) fn here() {}                 // same as private
fn private() {}
```

| Level | Visible to | Usage |
|---|---|---|
| private | the module and its descendants | implementation details |
| `pub(super)` | the parent module | helper shared between siblings |
| `pub(in path)` | a specific subtree | internal API of a subsystem |
| `pub(crate)` | the whole crate | tools internal to the library |
| `pub` | everyone | public API |

### 6.3 — What restricted visibility is for

Everything that is `pub` in a published library is part of its **API**: changing it breaks compatibility. Everything that is `pub(crate)` or more restricted can be renamed or removed freely.

Runique uses this for a security guarantee: the `Authorized` permission proof has a `pub(in crate::auth::gate)` constructor. Only the module that decides permissions can build one; any other code, even inside the crate, has to go through it.

### 6.4 — `pub` in a private module

```rust
mod internal {
    pub fn tool() {}
}
```

`tool` is `pub`, but `internal` is private: outside the crate, you cannot reach it (`internal::tool` is not accessible). The effective visibility is **the most restrictive** one along the path. The compiler does not complain: `pub` is useful here for the other modules of the crate.

## 7. Visibility of structs and enums

### 7.1 — Struct: each field has its own visibility

```rust
pub mod bank {
    pub struct Account {
        pub owner: String,   // public
        balance: i64,        // private
    }

    impl Account {
        pub fn new(owner: String) -> Self {
            Self { owner, balance: 0 }
        }
        pub fn balance(&self) -> i64 {
            self.balance
        }
    }
}

fn main() {
    let c = bank::Account::new("Ada".into());
    println!("{}", c.owner);         // OK
    println!("{}", c.balance());     // OK through the method
    // c.balance = 1_000;            // ERROR: private field
    // bank::Account { owner: "x".into(), balance: 5 };   // ERROR: cannot be built outside the module
}
```

Making `Account` public does not make its fields public. A single private field forces the other modules to go through a constructor.

### 7.2 — Enum: all or nothing

```rust
pub mod network {
    pub enum State {
        Connected,       // the variants of a pub enum are all public
        Disconnected,
    }
}

let e = network::State::Connected;
```

A public enum exposes all its variants and all their data. To hide a representation, wrap it in a struct with a private field.

### 7.3 — Methods

Methods have their own visibility: an `impl` on a public type can contain private methods.

```rust
impl bank::Account {
    // in the bank module:
    // pub fn deposit(&mut self, m: i64) { self.check(m); self.balance += m; }
    // fn check(&self, m: i64) {}
}
```

## 8. Paths

### 8.1 — Absolute or relative

```rust
// Absolute: from the root of the crate
crate::auth::login::verify();

// Relative: from the current module
login::verify();            // a submodule of the current module
self::login::verify();      // same, explicit

// Relative to the parent
super::forms::validate();
super::super::config();     // two levels up

// Another crate (dependency)
serde_json::to_string(&x);
::serde_json::to_string(&x);  // :: forces an external crate in case of ambiguity with a local module
```

### 8.2 — Which one to choose

- `crate::`: stable if the current file moves; it is the most readable form in a medium-sized project.
- `super::`: handy in a submodule that uses its parent, especially `use super::*;` in tests.
- `self::`: rarely needed.

## 9. `use`: all the forms

```rust
use std::collections::HashMap;                     // one name
use std::collections::{HashMap, HashSet};          // several names
use std::io::{self, Read, Write};                  // the module itself + items
use std::{fmt, collections::BTreeMap};             // nested
use std::fmt::Result as FmtResult;                 // rename
use std::io::Result as IoResult;
use std::collections::*;                           // everything (glob): avoid outside tests and preludes
use crate::auth::login;                            // a module of your own crate
use super::Config;                                 // from the parent
use self::internal::tool;                          // from a submodule
```

### 9.1 — Scope of a `use`

A `use` applies to the **module** (or block) where it is written, not to its submodules.

```rust
use std::collections::HashMap;

mod child {
    // HashMap is not visible here
    use std::collections::HashMap;   // you have to import it again
    // or: use super::HashMap;
}

fn f() {
    use std::fmt::Write;   // import limited to the function
}
```

### 9.2 — Resolving a name conflict

```rust
use std::fmt;
use std::io;

// Option 1: keep the module as a prefix
fn a() -> fmt::Result { Ok(()) }
fn b() -> io::Result<()> { Ok(()) }

// Option 2: rename at import
use std::fmt::Result as FmtResult;
fn c() -> FmtResult { Ok(()) }
```

### 9.3 — Importing a trait for its methods

```rust
use std::io::Write;     // without this import, .write_all() does not exist

fn write(f: &mut std::fs::File) -> std::io::Result<()> {
    f.write_all(b"hello")
}

use std::fmt::Write as _;   // import the trait without reserving its name
```

The methods of a trait can only be called if the trait is **in scope**. `as _` is for when another item is already called `Write` (for example `std::io::Write` and `std::fmt::Write` in the same file).

## 10. Import conventions

| Item | Convention | Example |
|---|---|---|
| Function | import the **parent module** | `use std::fs;` then `fs::read_to_string(..)` |
| Struct, enum, trait | import the **type** directly | `use std::collections::HashMap;` |
| Two types with the same name | import the modules, or `as` | `fmt::Result`, `io::Result` |
| Prelude of a crate | glob | `use runique::prelude::*;` |

What the convention on functions is for: `fs::read_to_string` says where the function comes from, whereas `read_to_string` alone could come from anywhere.

`cargo fmt` sorts the `use` lines and `rustfmt.toml` can group them (`imports_granularity`, `group_imports`): no need to spend time on it by hand.

## 11. Re-exports and public API

### 11.1 — `pub use`: exposing under a shorter path

```rust
// src/lib.rs
mod forms;
mod auth;

pub use forms::fields::TextField;      // users write my_lib::TextField
pub use auth::login::{login, logout};
```

The internal organization (`forms::fields::...`) can change without breaking anyone: only the re-exported path is part of the API.

### 11.2 — A prelude

```rust
// src/prelude.rs
pub use crate::forms::{Form, TextField};
pub use crate::auth::{login, logout};
pub use crate::Request;
```

```rust
// src/lib.rs
pub mod prelude;
```

```rust
// On the user's side
use my_lib::prelude::*;
```

This is what `runique::prelude` does: a single import gives access to the common types. Keep the prelude for items that are really used everywhere, otherwise it clashes with the user's own names.

### 11.3 — Re-exporting a dependency

```rust
// src/lib.rs
pub use sea_orm;          // users write my_lib::sea_orm::...
```

What it is for: the user does not have to declare the dependency themselves, and they use **exactly** the version your library uses. Without this, if they declare another version, the types of one are not the types of the other.

### 11.4 — Hiding from the documentation

```rust
#[doc(hidden)]
pub mod __internal {
    pub fn used_by_macros() {}
}
```

An item sometimes has to be `pub` because a macro calls it from the user's crate. `#[doc(hidden)]` removes it from the documentation and signals that it is not a stable API.

## 12. Conditional modules

```rust
#[cfg(test)]
mod tests;                   // compiled only for cargo test

#[cfg(feature = "admin")]
pub mod admin;               // compiled only with the admin feature

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;
```

A disabled module is neither compiled nor checked. Features and their effects are detailed in `cargo-dependances`.

### The test module

```rust
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn internal(x: i32) -> i32 { x * 2 }

#[cfg(test)]
mod tests {
    use super::*;          // imports the whole parent module, private items included

    #[test]
    fn adds() {
        assert_eq!(add(2, 3), 5);
        assert_eq!(internal(2), 4);     // unit tests see private items
    }
}
```

## 13. Library and binaries

### 13.1 — One package, several crates

```
my_project/
├── Cargo.toml
└── src/
    ├── lib.rs          ← library crate `my_project`
    ├── main.rs         ← binary `my_project`
    └── bin/
        ├── import.rs   ← binary `import`
        └── tool/
            └── main.rs ← binary `tool` (with its own modules)
```

```bash
cargo run                  # runs main.rs
cargo run --bin import
```

### 13.2 — The binary uses the library like a dependency

```rust
// src/main.rs
use my_project::config::load;     // we go through the crate name, not crate::

fn main() {
    let c = load();
}
```

A binary is a **separate crate**: it only sees what is `pub` in the library. A `pub(crate)` item in `lib.rs` is invisible from `main.rs` or `src/bin/`. For a command-line tool to use it, it must be `pub`.

### 13.3 — Why separate them

```rust
// Option 1: everything in main.rs — impossible to test with integration tests
// Option 2: logic in lib.rs, main.rs only reads the arguments and calls

// src/main.rs
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Err(e) = my_project::run(&args) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
```

The second form lets the tests (`tests/*.rs`), the other binaries and other projects reuse the logic.

### 13.4 — Several crates: the workspace

When a project contains several libraries (for example a framework, its procedural macros and a demo application), they are split into packages of the same workspace. See `cargo-dependances`, Workspaces section.

## 14. A complete project

```
my_api/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── main.rs
    ├── models.rs
    ├── models/
    │   ├── user.rs
    │   └── post.rs
    ├── api.rs
    ├── api/
    │   └── routes.rs
    └── database.rs
```

```rust
// src/lib.rs
pub mod models;
pub mod api;
mod database;                         // private: implementation detail

pub use models::{Post, User};         // short API
pub use api::routes::configure;
```

```rust
// src/models.rs
mod post;
mod user;

pub use post::Post;                   // the submodules stay private,
pub use user::User;                   // only the types are exposed
```

```rust
// src/models/user.rs
#[derive(Debug)]
pub struct User {
    pub id: u32,
    pub name: String,
}

impl User {
    pub fn new(id: u32, name: impl Into<String>) -> Self {
        Self { id, name: name.into() }
    }
}
```

```rust
// src/database.rs
use crate::models::User;

pub(crate) fn save(u: &User) {
    println!("saving {}", u.name);
}
```

```rust
// src/api/routes.rs
use crate::database;
use crate::models::User;

pub fn configure() {
    let u = User::new(1, "Alice");
    database::save(&u);
}
```

```rust
// src/main.rs
use my_api::{configure, User};

fn main() {
    configure();
    println!("{:?}", User::new(2, "Bob"));
}
```

Reading: `database` is private to the crate, `save` is `pub(crate)` (usable by `api`, invisible to `main.rs`), `User` and `configure` are exposed at the root by re-export.

## 15. Common errors

| Message | Cause | Fix |
|---|---|---|
| `file not found for module` | `mod x;` without `x.rs` or `x/mod.rs` in the right place | create the file next to the parent module |
| `unresolved import` | wrong path, module not declared by `mod` | check the `mod` in the parent, use `crate::` |
| `function is private` | item without `pub` | add the minimal visibility level needed |
| `field is private` | field without `pub` | go through a method, or make the field `pub` |
| `no method named ... found` although it exists | trait not imported | `use path::Trait;` |
| The code of a file is never checked | no `mod` declares it | add `mod name;` in the parent |
| `pub(crate)` not found from `src/bin/` | a binary is another crate | make the item `pub` |

## 16. Exercises

### Exercise 1 — split a file

Move this code into `src/maths.rs` and `src/maths/stats.rs`, and call it from `main.rs`.

```rust
mod maths {
    pub fn square(x: i32) -> i32 { x * x }
    pub mod stats {
        pub fn mean(v: &[f64]) -> f64 { v.iter().sum::<f64>() / v.len() as f64 }
    }
}
```

```rust
// Solution:
// src/main.rs
mod maths;
fn main() {
    println!("{}", maths::square(3));
    println!("{}", maths::stats::mean(&[1.0, 2.0]));
}

// src/maths.rs
pub mod stats;
pub fn square(x: i32) -> i32 { x * x }

// src/maths/stats.rs
pub fn mean(v: &[f64]) -> f64 { v.iter().sum::<f64>() / v.len() as f64 }
```

### Exercise 2 — minimal visibility

In a library, `hash` must only be callable from the `auth` module and its submodules, `verify` from the whole crate, `login` by everyone. Give the declarations.

```rust
// Solution:
pub mod auth {
    pub(in crate::auth) fn hash(pwd: &str) -> String { pwd.to_string() }
    pub(crate) fn verify(pwd: &str, h: &str) -> bool { hash(pwd) == h }
    pub fn login(pwd: &str) -> bool { verify(pwd, "x") }
}
```

### Exercise 3 — short API

Users write `my_lib::forms::fields::text::TextField`. Make it so they can write `my_lib::TextField` and `use my_lib::prelude::*;`, without moving the file.

```rust
// Solution:
// src/lib.rs
pub mod forms;
pub mod prelude;
pub use forms::fields::text::TextField;

// src/prelude.rs
pub use crate::TextField;
```

## 17. Cheat sheet

```rust
mod a;                    // declares a (a.rs or a/mod.rs), once, in the parent
mod b { }                 // inline module
pub mod c;                // public module

use crate::a::X;          // absolute path
use super::Y;             // parent
use self::b::Z;           // submodule
use std::io::{self, Read};
use std::fmt::Result as FmtResult;
use std::io::Write as _;  // trait in scope, without reserving the name
pub use a::X;             // re-export

pub            pub(crate)            pub(super)            pub(in crate::a)
#[cfg(test)] mod tests { use super::*; }
#[cfg(feature = "x")] pub mod x;
```

### Key points

- `mod` declares a module once; `use` imports a name as many times as needed.
- A file not declared by `mod` is not compiled.
- Everything is private by default; a child sees the private items of its ancestors, not the other way around.
- The real visibility is the most restrictive one along the path.
- A single private field is enough to force a constructor.
- `pub use` decouples the internal organization from the public API.
- A binary is a separate crate: it only sees the `pub` items of the library.
- Dependencies, features and workspaces: see `cargo-dependances`.
