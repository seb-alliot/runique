# Macros: visibility and export
> Where a macro is visible, how to make it usable in other modules and other crates, and how to write a macro that works in the user's code whatever their imports are.

## Objectives

- Understand the "textual" scope of a `macro_rules!`
- Use a macro in other modules of the same crate, in two ways
- Export a macro to other crates with `#[macro_export]`
- Write `$crate::` and absolute paths so the macro works everywhere
- Hide helper functions and macros with `#[doc(hidden)]`
- Re-export procedural macros and the dependencies they use
- Document and test an exported macro

## Table of contents

1. Default scope: file order
2. Using a macro in other modules
3. `#[macro_export]`: to other crates
4. `$crate`: a path that works everywhere
5. Dependencies used by a macro
6. Helper items: `#[doc(hidden)]`
7. Re-exporting macros
8. `#[macro_use]`: the old way
9. Documenting and testing an exported macro
10. Examples in Runique
11. Exercises
12. Cheat sheet

---

## 1. Default scope: file order

A `macro_rules!` without an attribute has a **textual** scope: it is visible from its declaration to the end of the module that contains it, **and in the modules declared after it**.

```rust
// before!();                   // ERROR: the macro is not declared yet

macro_rules! before {
    () => { 1 };
}

fn f() -> i32 { before!() }      // OK: after the declaration

mod child {
    pub fn g() -> i32 { before!() }   // OK: module declared after the macro
}
```

Unlike a function, **order matters**. In `lib.rs`, the module that defines the macros must therefore be declared **before** the ones that use them:

```rust
// src/lib.rs
#[macro_use]
mod macros;        // defines the macros (see section 2.1)
mod forms;         // can use them
mod views;         // too
```

---

## 2. Using a macro in other modules

### 2.1 Way 1: `#[macro_use]` on the module

```rust
// src/lib.rs
#[macro_use]
mod macros;                     // the macros of this module stay visible after it
mod users;

// src/macros.rs
macro_rules! check {
    ($c:expr) => { assert!($c) };
}

// src/users.rs
pub fn create() {
    check!(true);               // visible thanks to #[macro_use] and the order of the mod items
}
```

### 2.2 Way 2: a path import (edition 2018+)

```rust
// src/macros.rs
macro_rules! check {
    ($c:expr) => { assert!($c) };
}
pub(crate) use check;           // the macro becomes an ordinary item of the module

// src/users.rs
use crate::macros::check;       // explicit import, like a function

pub fn create() {
    check!(true);
}
```

`pub(crate) use macro_name;` right after the definition turns the macro into an item that can be imported by its path. The order of the `mod` items no longer matters, and the editor knows where the macro comes from.

| | `#[macro_use]` on `mod` | `pub(crate) use` |
|---|---|---|
| Depends on the order of `mod` | yes | no |
| You can see where the macro comes from | no | yes (`use crate::macros::check`) |
| For | small projects | recent code, medium and large projects |

---

## 3. `#[macro_export]`: to other crates

```rust
// In the crate `my_lib`, any file
#[macro_export]
macro_rules! say {
    ($msg:expr) => { println!("my_lib: {}", $msg) };
}
```

`#[macro_export]` places the macro **at the root of the crate**, whatever module it is written in. It is then used like this:

```rust
// In a crate that depends on my_lib
use my_lib::say;               // from the root, never my_lib::module::say

fn main() {
    say!("hello");
    my_lib::say!("also possible without use");
}
```

In the crate that defines it, you call it with `crate::say!` (or directly by its name, depending on the textual scope).

A `#[macro_export]` macro is **always public**: it is part of the crate's API. Changing it (removing an arm, changing the syntax) is a breaking change.

---

## 4. `$crate`: a path that works everywhere

### 4.1 The problem

```rust
// In my_lib
pub fn log_line(msg: &str) { println!("{msg}"); }

#[macro_export]
macro_rules! log_bad {
    ($m:expr) => { log_line($m) };     // resolved at the CALL site, in the user's code
}
```

In the user's code, `log_line` does not exist (or means something else): the macro breaks. Writing `my_lib::log_line` is not enough either: the user may have renamed the dependency (`my_lib2 = { package = "my_lib" }`), and inside `my_lib` itself, `my_lib::` refers to nothing.

### 4.2 The solution

```rust
#[macro_export]
macro_rules! log {
    ($m:expr) => { $crate::log_line($m) };
}
```

`$crate` is replaced by **the path of the crate that defines the macro**, wherever it is called from and whatever name it is imported under. It is the only reliable way to refer to items of your own crate in an exported macro.

### 4.3 All the references of an exported macro

```rust
#[macro_export]
macro_rules! new_table {
    ($($k:expr => $v:expr),*) => {{
        let mut m = ::std::collections::HashMap::new();   // std: absolute path ::std
        $( m.insert($k, $v); )*
        $crate::Table::from(m)                             // own crate: $crate
    }};
}
```

| What the macro uses | Path to write |
|---|---|
| an item of its own crate | `$crate::path::Item` |
| the standard library | `::std::...` or `::core::...` |
| a macro of its own crate | `$crate::other_macro!` |
| a dependency of its crate | `$crate::__dep::...` (section 5) |
| a trait whose methods are needed | full call: `$crate::MyTrait::method(&x)` |

The last point is a frequent trap: `x.method()` in the expansion only works if the user imported the trait. `$crate::MyTrait::method(&x)` always works.

### 4.4 Recursion

An exported macro that calls itself must do so through `$crate::`:

```rust
#[macro_export]
macro_rules! total {
    () => { 0 };
    ($x:expr $(, $rest:expr)*) => { $x + $crate::total!($($rest),*) };
}
```

---

## 5. Dependencies used by a macro

If the expansion of a macro uses a **dependency** of your crate (serde, sea-orm...), the user may not have that dependency, or not the same version.

```rust
// In my_lib (which depends on serde_json)
#[doc(hidden)]
pub use serde_json as __serde_json;      // hidden re-export

#[macro_export]
macro_rules! json_of {
    ($v:expr) => { $crate::__serde_json::to_string(&$v) };
}
```

The user has nothing to declare in their `Cargo.toml`, and the macro uses **exactly** the version of `serde_json` chosen by your library.

The `__` prefix and `#[doc(hidden)]` signal that this path is internal: it does not appear in the documentation and is not considered a stable API.

---

## 6. Helper items: `#[doc(hidden)]`

An exported macro can only call **public** items. Helper functions must therefore be `pub`, even though they are not meant to be called by hand.

```rust
pub struct Routes { /* ... */ }

impl Routes {
    #[doc(hidden)]
    pub fn new() -> Self { Routes { /* ... */ } }         // called by the macro

    #[doc(hidden)]
    pub fn add(&mut self, path: &str) { /* ... */ }       // called by the macro
}

#[macro_export]
macro_rules! routes {
    ($($path:literal),* $(,)?) => {{
        let mut r = $crate::Routes::new();
        $( r.add($path); )*
        r
    }};
}
```

Same thing for a helper macro: `#[macro_export] #[doc(hidden)] macro_rules! __step { .. }`, called by `$crate::__step!`.

`#[doc(hidden)]` is **not** a protection: the item can still be called. It is a signal ("not a public API, may change"). For a real guarantee (a constructor that must not be bypassed), you need a restricted visibility, and therefore a design where the macro does not need it.

---

## 7. Re-exporting macros

### 7.1 A declarative macro

```rust
// src/macros/mod.rs
#[macro_export]
macro_rules! my_macro { () => {} }

// src/lib.rs
pub mod macros;
// pub use macros::my_macro;      // useless: #[macro_export] already puts it at the root
```

A `#[macro_export]` macro is already at the root. You can add it to a prelude:

```rust
// src/prelude.rs
pub use crate::my_macro;
```

### 7.2 A procedural macro

A procedural macro (`derive`, attribute, function-like) **must** live in a separate crate of type `proc-macro` (see `macros-derive`). So that the user only has one dependency to declare, the main library re-exports it:

```toml
# my_lib/Cargo.toml
[dependencies]
my_lib_macros = { path = "../my_lib_macros", version = "1.0" }
```

```rust
// my_lib/src/lib.rs
pub use my_lib_macros::{MyDerive, my_attribute};

pub mod prelude {
    pub use my_lib_macros::{MyDerive, my_attribute};
}
```

```rust
// In the user's code: a single dependency
use my_lib::prelude::*;

#[derive(MyDerive)]
struct S;
```

### 7.3 The code generated by a procedural macro

A procedural macro does not know `$crate`. It usually generates `::my_lib::...` paths. Two consequences:

- the user must have the dependency under the name `my_lib` (not renamed);
- in the tests of `my_lib` itself, `::my_lib::` refers to nothing: add, under `cfg(test)`, `extern crate self as my_lib;` at the top of `lib.rs`, which gives the crate a name for itself.

---

## 8. `#[macro_use]`: the old way

Before the 2018 edition, the macros of a dependency were imported with:

```rust
#[macro_use]
extern crate serde_derive;
```

This style is still valid but discouraged:

```rust
// Old
#[macro_use]
extern crate my_lib;
fn main() { say!("x"); }

// Modern: an explicit use
use my_lib::say;
fn main() { say!("x"); }
```

`#[macro_use]` remains useful **on a module** of your own crate (section 2.1).

---

## 9. Documenting and testing an exported macro

### 9.1 Documentation

```rust
/// Creates a `HashMap` from `key => value` pairs.
///
/// # Example
///
/// ```
/// let m = my_lib::table! { "a" => 1, "b" => 2 };
/// assert_eq!(m["a"], 1);
/// ```
#[macro_export]
macro_rules! table {
    ($($k:expr => $v:expr),* $(,)?) => {
        ::std::collections::HashMap::from([$(($k, $v)),*])
    };
}
```

The `///` comment goes **above** `#[macro_export]`. The example is run by `cargo test` as a documentation test, from an external crate: it checks exactly what the user will see (paths through `$crate`, required imports).

### 9.2 Testing that a use is rejected

```rust
/// ```compile_fail
/// let x = my_lib::table!(1, 2);     // syntax without =>: must be rejected
/// ```
```

### 9.3 Testing from the outside

An integration test (`tests/macros.rs`) uses the macro exactly like another crate: this is where you detect a forgotten path that worked by chance inside the crate.

---

## 10. Examples in Runique

### 10.1 `urlpatterns!`: `$crate`, hidden methods

```rust
// runique/src/macros/routeur/router.rs
#[macro_export]
macro_rules! urlpatterns {
    // ...
    (@parse $routes:ident; admin $path:expr => $handler:expr, /* ... */ $($rest:tt)*) => {
        $routes.add_admin_route::<$model>($path, $name, $handler, $view);
        $crate::urlpatterns!(@parse $routes; $($rest)*);      // recursion through $crate
    };
}

// runique/src/macros/routeur/routes.rs
impl Routes {
    /// Used by `urlpatterns!`; construct route sets through the macro.
    #[doc(hidden)]
    pub fn new() -> Self { /* ... */ }

    /// Used by `urlpatterns!` to add an ordinary route.
    #[doc(hidden)]
    pub fn add_route(&mut self, path: impl AsRef<str>, handler: MethodRouter) { /* ... */ }
}
```

The macro is exported at the root (`runique::urlpatterns!`), calls itself through `$crate::`, and uses public methods hidden from the documentation, which the user is not supposed to call directly.

### 10.2 A hidden helper macro

```rust
// runique/src/macros/bdd/filter.rs
#[macro_export]
#[doc(hidden)]
macro_rules! search_apply_op { /* eq, icontains, gte... */ }
```

`search!` translates operators (`eq`, `icontains`, `gte`...) into SeaORM conditions through this helper macro, which is exported (so it can be called from the code expanded in the user's crate) but hidden.

### 10.3 Re-exporting the procedural macros

```rust
// runique/src/lib.rs
pub use derive_form::{extend, model};

pub mod prelude {
    pub use derive_form::{extend, form, model};
    // ...
}
```

`model!{}`, `extend!{}` and `#[form]` live in the `derive_form` crate (a `proc-macro` crate). The user only declares `runique` and receives them through `use runique::prelude::*`.

### 10.4 The crate's name for itself

```rust
// runique/src/lib.rs
// In the crate's own tests, `::runique::…` (what `model!{}` and `extend!{}`
// generate) names the crate itself.
#[cfg(test)]
extern crate self as runique;
```

The code generated by `model!{}` contains `::runique::...` paths. In the user's code, `runique` is a dependency: all is well. In the framework's own tests, this line gives the crate the name `runique`, so the same generated code compiles (section 7.3).

---

## 11. Exercises

### Exercise 1: usable everywhere

This macro works in its crate but not in the user's code. Fix it.

```rust
pub struct Counter(pub u32);

#[macro_export]
macro_rules! counter {
    ($n:expr) => { Counter($n) };
}
```

```rust
// Solution:
#[macro_export]
macro_rules! counter {
    ($n:expr) => { $crate::Counter($n) };
}
```

### Exercise 2: crate-internal macro

Make the `check!` macro usable in all the modules of the crate, without depending on the order of the `mod` items and without exporting it to other crates.

```rust
// Solution:
// src/macros.rs
macro_rules! check {
    ($c:expr, $msg:expr) => { if !$c { return Err($msg.into()); } };
}
pub(crate) use check;

// elsewhere
use crate::macros::check;
```

### Exercise 3: trait method call

Why does this exported macro fail in the code of a user who has not imported `Display`? Fix it.

```rust
pub trait Display { fn display(&self) -> String; }

#[macro_export]
macro_rules! show {
    ($x:expr) => { println!("{}", $x.display()) };
}
```

```rust
// Solution: full call, which does not depend on the user's imports.
#[macro_export]
macro_rules! show {
    ($x:expr) => { println!("{}", $crate::Display::display(&$x)) };
}
```

## 12. Cheat sheet

```rust
macro_rules! m { .. }                // visible after its declaration, in the module and the following ones
#[macro_use] mod macros;             // keep the macros visible after the module
pub(crate) use m;                    // import by path inside the crate: use crate::macros::m;
#[macro_export] macro_rules! m { .. } // crate root, for everyone: use my_lib::m;
$crate::Item                         // path to your own crate in the expansion
::std::collections::HashMap          // std as an absolute path
$crate::Trait::method(&x)            // trait method without an import in the user's code
#[doc(hidden)] pub use dep as __dep; // dependency used by the macro
#[doc(hidden)] pub fn helper() {}    // public helper, hidden from the docs
pub use my_lib_macros::MyDerive;     // re-export a procedural macro
#[cfg(test)] extern crate self as my_lib;   // ::my_lib:: in its own tests
```

### Key points

- A `macro_rules!` depends on declaration order; `pub(crate) use` removes this constraint.
- `#[macro_export]` places the macro at the root and makes it public for good.
- In an exported macro, every path must be absolute: `$crate::`, `::std::`.
- Trait methods are called by their full path, not by `.method()`.
- Dependencies used by the macro are re-exported, hidden.
- `#[doc(hidden)]` signals an internal helper, it protects nothing.
- Procedural macros are re-exported from the main crate.
