# Declarative macros
> `macro_rules!`: writing code that writes code, by pattern matching on syntax. All the fragments, all the repetition forms, recursion techniques, and pitfalls.

## Objectives

- Know when a macro is justified rather than a function or a generic
- Write a macro with one or several arms
- Know all the fragment types (`expr`, `ident`, `ty`, `tt`...)
- Use the repetitions `*`, `+`, `?`, nested, with separators
- Write recursive macros, internal rules and "tt munchers"
- Generate functions, structs, `impl` blocks and tests
- Avoid pitfalls: multiple evaluation, operator precedence, hygiene
- Debug a macro and produce clear errors

## Table of contents

1. Why macros
2. Basic syntax
3. Fragments
4. Repetitions
5. Several arms
6. Simple examples
7. Generating code: functions, structs, impl, tests
8. Recursion and internal rules
9. Counting and accumulating
10. Pitfalls
11. Hygiene
12. Errors and debugging
13. Standard library macros
14. Examples in Runique
15. Macro, function or generic: the same thing three times
16. Exercises
17. Cheat sheet

---

## 1. Why macros

A macro is expanded **at compile time**: it receives code (tokens) and produces code. It can do what a function cannot:

| Need | Function | Macro |
|---|---|---|
| variable number of arguments | no | `vec![1, 2, 3]`, `println!("{} {}", a, b)` |
| new syntax | no | `urlpatterns! { "/" => index, }` |
| generate declarations (struct, impl, fn) | no | yes |
| check something at compile time (format string) | no | `format!("{x}")` |
| receive a variable name, a type, a pattern | no | yes |
| be readable, debuggable, autocompleted by the editor | yes | less so |

Rule: **a function first, a generic next, a macro last**. A macro is justified when it removes real repetition that neither a function nor a trait can remove.

---

## 2. Basic syntax

```rust
macro_rules! hello {
    () => {
        println!("Hello!");
    };
}

hello!();
```

- `macro_rules! name { ... }` declares the macro.
- Each **arm** has the form `(pattern) => { expansion };`.
- You call it with `name!`, and your choice of parentheses, brackets or braces: `hello!()`, `hello![]`, `hello!{}` are equivalent.

Convention: `()` for a macro that looks like a function call, `[]` for a collection (`vec![]`), `{}` for a block of declarations (`urlpatterns! { }`). A macro called with `{}` in item position does not need a `;`.

### With an argument

```rust
macro_rules! square {
    ($x:expr) => {
        $x * $x
    };
}

let a = square!(4);         // 16
```

`$x:expr` captures an **expression** and names it `$x`. In the expansion, `$x` is replaced by the code received.

### Declaration order

A `macro_rules!` macro must be declared **before** it is used in the file (unlike functions). To use it in other modules or other crates: see `macros-export`.

---

## 3. Fragments

| Fragment | Captures | Example |
|---|---|---|
| `expr` | an expression | `2 + 3`, `f(x)`, `vec![]`, `if a { 1 } else { 2 }` |
| `ident` | an identifier | `x`, `my_function`, `Point` |
| `ty` | a type | `i32`, `Vec<String>`, `&'a str` |
| `path` | a path | `std::collections::HashMap`, `crate::a::b` |
| `pat` | a pattern (alternatives included in edition 2021+) | `Some(x)`, `1..=5`, `_` |
| `pat_param` | a pattern without top-level alternatives | same |
| `stmt` | a statement (without the trailing `;`) | `let x = 5` |
| `block` | a block between braces | `{ println!("a"); 1 }` |
| `item` | a declaration | `fn f() {}`, `struct S;`, `impl T for S {}` |
| `literal` | a literal | `42`, `"text"`, `'c'`, `true` |
| `lifetime` | a lifetime | `'a`, `'static` |
| `vis` | a visibility (may be empty) | `pub`, `pub(crate)`, nothing |
| `meta` | the content of an attribute | `derive(Debug)`, `doc = "x"` |
| `tt` | a single token tree | `x`, `+`, `(a, b)`, `{ ... }` |

### 3.1 Examples

```rust
macro_rules! declare {
    ($vis:vis $name:ident : $t:ty = $value:expr) => {
        $vis const $name: $t = $value;
    };
}

declare!(pub MAX: u32 = 100);
declare!(MIN: u32 = 0);         // empty vis: private

macro_rules! is {
    ($value:expr, $pattern:pat) => {
        matches!($value, $pattern)
    };
}

let ok = is!(Some(3), Some(1..=5));
```

### 3.2 `tt`: the universal fragment

A `tt` (*token tree*) is a single token, or everything between `()`, `[]` or `{}`. With `$($t:tt)*`, you capture **anything** without analyzing it: this is the basis of recursive macros (section 8).

### 3.3 What can follow a fragment

To keep the grammar unambiguous, some fragments can only be followed by certain tokens:

| Fragment | Can be followed by |
|---|---|
| `expr`, `stmt` | `=>`, `,`, `;` |
| `ty`, `path` | `=>`, `,`, `=`, `;`, `:`, `>`, `[`, `{`, `as`, `where`, the vertical bar |
| `pat` | `=>`, `,`, `=`, `if`, `in` |
| `ident`, `tt`, `literal`, `lifetime`, `block` | anything |

```rust
// macro_rules! m { ($a:expr $b:expr) => {}; }   // ERROR: `$a:expr` is followed by `$b:expr`
macro_rules! m { ($a:expr, $b:expr) => {}; }     // OK: a comma separates them
```

### 3.4 `expr` and the 2024 edition

In edition 2024, `expr` also accepts `const { ... }` and `_`. To keep the old behavior (edition 2021), there is `expr_2021`.

---

## 4. Repetitions

### 4.1 The three operators

| Operator | Meaning |
|---|---|
| `$( ... )*` | zero times or more |
| `$( ... )+` | one time or more |
| `$( ... )?` | zero or one time |

The separator goes before the operator: `$( ... ),*` = repetitions separated by commas.

### 4.2 Recreating `vec!`

```rust
macro_rules! my_vec {
    ( $( $x:expr ),* ) => {{
        let mut v = Vec::new();
        $( v.push($x); )*          // one generated line per captured element
        v
    }};
}

let v = my_vec![1, 2, 3];
let empty: Vec<i32> = my_vec![];
```

The **double braces** `{{ ... }}`: the first pair delimits the expansion, the second forms a block, so that the variables (`v`) do not leak and the whole is a single expression.

### 4.3 Optional trailing comma

```rust
macro_rules! list {
    ( $( $x:expr ),* $(,)? ) => {
        vec![ $( $x ),* ]
    };
}

list![1, 2, 3];
list![1, 2, 3,];          // accepted thanks to $(,)?
```

### 4.4 Several variables in a repetition

```rust
macro_rules! table {
    ( $( $key:expr => $value:expr ),* $(,)? ) => {{
        let mut m = ::std::collections::HashMap::new();
        $( m.insert($key, $value); )*
        m
    }};
}

let t = table! {
    "one" => 1,
    "two" => 2,
};
```

The variables of the same repetition advance **together**: `$key` and `$value` must have been captured the same number of times.

### 4.5 Nested repetitions

```rust
macro_rules! matrix {
    ( $( [ $( $x:expr ),* ] ),* ) => {
        vec![ $( vec![ $( $x ),* ] ),* ]
    };
}

let m = matrix!([1, 2], [3, 4], [5, 6]);   // Vec<Vec<i32>>
```

### 4.6 Optional part

```rust
macro_rules! field {
    ($name:ident : $t:ty $(= $default:expr)?) => {
        // $default can only be used inside a $( ... )? repetition
        let $name: $t = field!(@default $($default)?);
    };
    (@default $e:expr) => { $e };
    (@default) => { Default::default() };
}
```

---

## 5. Several arms

The arms are tried **in order**; the first one that matches wins.

```rust
macro_rules! log {
    ($msg:expr) => {
        println!("[INFO] {}", $msg)
    };
    ($level:ident, $msg:expr) => {
        println!("[{}] {}", stringify!($level), $msg)
    };
    ($level:ident, $fmt:literal, $($arg:tt)*) => {
        println!(concat!("[{}] ", $fmt), stringify!($level), $($arg)*)   // concat! requires a literal
    };
}

log!("starting");
log!(WARN, "disk almost full");
log!(ERROR, "code {}", 500);
```

Put the **specific arms before the general ones**. An arm that starts with a literal keyword (`(admin $path:expr ...)`) must come before a generic arm (`($path:expr ...)`).

---

## 6. Simple examples

### 6.1 Timing a block

```rust
macro_rules! timed {
    ($name:expr, $body:block) => {{
        let start = ::std::time::Instant::now();
        let r = $body;
        println!("{}: {:?}", $name, start.elapsed());
        r
    }};
}

let total = timed!("total", { (1..1_000_000u64).sum::<u64>() });
```

### 6.2 Checking at the start of a function

```rust
macro_rules! require {
    ($cond:expr, $err:expr) => {
        if !$cond {
            return Err($err.into());
        }
    };
}

fn register(age: u32, email: &str) -> Result<(), String> {
    require!(age >= 18, "you must be an adult");
    require!(email.contains('@'), "invalid email");
    Ok(())
}
```

This is the principle of `anyhow::ensure!`. A macro can `return` **from the calling function**, which a function cannot do.

### 6.3 Printing an expression and its value

```rust
macro_rules! show {
    ($($e:expr),+ $(,)?) => {
        $( println!("{} = {:?}", stringify!($e), $e); )+
    };
}

show!(1 + 2, "abc".len());
// 1 + 2 = 3
// "abc".len() = 3
```

`stringify!` turns the **code** received into text: this is how `assert!` and `dbg!` display the expression that failed.

---

## 7. Generating code: functions, structs, impl, tests

### 7.1 Getters

```rust
macro_rules! getters {
    ($type:ident { $( $field:ident : $t:ty ),* $(,)? }) => {
        impl $type {
            $(
                pub fn $field(&self) -> &$t {
                    &self.$field
                }
            )*
        }
    };
}

struct User { name: String, age: u32 }
getters!(User { name: String, age: u32 });
```

### 7.2 A complete newtype

```rust
macro_rules! identifier {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(pub i64);

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, "{}#{}", stringify!($name), self.0)
            }
        }

        impl From<i64> for $name {
            fn from(v: i64) -> Self { $name(v) }
        }
    };
}

identifier!(UserId);
identifier!(OrderId);
```

Two lines generate two distinct types with six traits each: this is the typical case where a macro removes repetition that no generic can remove (each type must be **named**).

### 7.3 A trait for several types

```rust
trait IsZero { fn is_zero(&self) -> bool; }

macro_rules! impl_is_zero {
    ($($t:ty),*) => {
        $( impl IsZero for $t { fn is_zero(&self) -> bool { *self == 0 as $t } } )*
    };
}

impl_is_zero!(i8, i16, i32, i64, u8, u16, u32, u64, f32, f64);
```

The standard library implements the numeric traits this way for each integer type.

### 7.4 Tests

```rust
fn double(x: i32) -> i32 { x * 2 }

macro_rules! double_tests {
    ($($name:ident : $input:expr => $expected:expr),* $(,)?) => {
        $(
            #[test]
            fn $name() {
                assert_eq!(double($input), $expected);
            }
        )*
    };
}

double_tests! {
    zero: 0 => 0,
    positive: 3 => 6,
    negative: -2 => -4,
}
```

### 7.5 Creating a new identifier

`macro_rules!` cannot **build** a name (`get_` + `name`): it can only use the identifiers it receives. To concatenate names, you need the `paste` crate or a procedural macro (later courses).

---

## 8. Recursion and internal rules

### 8.1 A macro that calls itself

```rust
macro_rules! maximum {
    ($x:expr) => { $x };
    ($x:expr, $($rest:expr),+) => {{
        let a = $x;
        let b = maximum!($($rest),+);
        if a > b { a } else { b }
    }};
}

let m = maximum!(3, 9, 2, 7);    // 9
```

Each call handles the first element and calls the macro again on the rest, until the base case (a single element).

### 8.2 Internal rules with `@`

An arm that starts with a token such as `@step` is, by convention, **internal**: the user does not call it, the macro uses it for its own steps.

```rust
macro_rules! sum_of_squares {
    ($($x:expr),*) => { sum_of_squares!(@acc 0; $($x),*) };
    (@acc $acc:expr;) => { $acc };
    (@acc $acc:expr; $x:expr $(, $rest:expr)*) => {
        sum_of_squares!(@acc $acc + $x * $x; $($rest),*)
    };
}

let s = sum_of_squares!(1, 2, 3);   // 14
```

### 8.3 The "tt muncher"

To parse free-form syntax, the macro "eats" the tokens one group at a time with `$($rest:tt)*` and calls itself again on what follows.

```rust
macro_rules! routes {
    (@parse $v:ident;) => {};
    (@parse $v:ident; get $path:literal => $f:ident, $($rest:tt)*) => {
        $v.push(("GET", $path, stringify!($f)));
        routes!(@parse $v; $($rest)*);
    };
    (@parse $v:ident; post $path:literal => $f:ident, $($rest:tt)*) => {
        $v.push(("POST", $path, stringify!($f)));
        routes!(@parse $v; $($rest)*);
    };
    ($($t:tt)*) => {{
        let mut v: Vec<(&str, &str, &str)> = Vec::new();
        routes!(@parse v; $($t)*);
        v
    }};
}

let r = routes! {
    get "/" => index,
    post "/contact" => contact,
};
```

This is exactly the structure of `urlpatterns!` in Runique (section 14).

### 8.4 Recursion limit

Each level of recursion counts. Beyond 128 levels by default, the compiler stops (`recursion limit reached`). You raise it with `#![recursion_limit = "256"]` at the top of the crate, but a macro that needs this is often better rewritten.

---

## 9. Counting and accumulating

### 9.1 Counting the arguments

```rust
macro_rules! count {
    () => { 0usize };
    ($head:tt $($rest:tt)*) => { 1usize + count!($($rest)*) };
}

const N: usize = count!(a b c d);   // 4
```

### 9.2 Counting without recursion

```rust
macro_rules! replace { ($_t:tt $sub:expr) => { $sub }; }

macro_rules! count_fast {
    ($($t:tt)*) => { <[()]>::len(&[$(replace!($t ())),*]) };
}

const M: usize = count_fast!(a b c d e);   // 5
```

Each element becomes a `()` in an array, whose length is taken: no recursion limit.

### 9.3 Building an array of known size

```rust
macro_rules! name_array {
    ($($name:ident),* $(,)?) => {
        [$(stringify!($name)),*]
    };
}

const FIELDS: [&str; 3] = name_array!(name, email, age);
```

---

## 10. Pitfalls

### 10.1 Multiple evaluation

```rust
macro_rules! naive_square {
    ($x:expr) => { $x * $x };
}

let mut counter = 0;
let mut next = || { counter += 1; counter };
let c = naive_square!(next());   // next() is called TWICE: 1 * 2 = 2, not 1
```

The code received is **copied** at each `$x`. An expression with a side effect (or an expensive one) is run as many times. Fix: evaluate it once in a variable.

```rust
macro_rules! square {
    ($x:expr) => {{
        let v = $x;
        v * v
    }};
}
```

### 10.2 Operator precedence

An `expr` fragment is inserted **as a single block** (in implicit parentheses), which avoids the C preprocessor pitfall:

```rust
macro_rules! double { ($x:expr) => { $x * 2 }; }
let r = double!(1 + 2);      // 6: (1 + 2) * 2, not 1 + 2 * 2
```

But with `tt`, the tokens are copied as they are:

```rust
macro_rules! double_tt { ($($x:tt)*) => { $($x)* * 2 }; }
let r = double_tt!(1 + 2);   // 5: 1 + 2 * 2
```

Rule: capture an expression with `expr`, unless you need to analyze its tokens.

### 10.3 Absolute paths

```rust
macro_rules! new_map {
    () => { HashMap::new() };               // fails if the user has not imported HashMap
}

macro_rules! new_map_ok {
    () => { ::std::collections::HashMap::new() };   // works everywhere
}
```

In the expansion of a macro, write full paths: `::std::...` for the standard library, `$crate::...` for your own crate (see `macros-export`).

### 10.4 A misplaced macro

A macro can only produce what is valid **at the call site**: an expression where an expression is expected, items (`fn`, `struct`) where declarations are expected. A macro that produces several statements in expression position must wrap them in a block `{{ ... }}`.

---

## 11. Hygiene

### 11.1 Local variables are protected

```rust
macro_rules! with_temp {
    ($e:expr) => {{
        let temp = 10;
        $e + temp
    }};
}

let temp = 1;
let r = with_temp!(temp);    // 11: the caller's `temp` is not the macro's one
```

A variable declared **inside** a macro cannot collide with a variable of the caller, even with the same name: `macro_rules!` is **hygienic** for local variables.

### 11.2 What is not protected

```rust
macro_rules! declare_function {
    () => { fn help() -> u32 { 42 } };
}

declare_function!();
let x = help();              // the function is visible: items (fn, struct, impl) are not hygienic
```

The names of functions, types, modules and traits generated by a macro are visible as usual. Paths in the expansion are resolved **at the call site**, hence the absolute path rule (section 10.3).

### 11.3 Giving access to a variable

For the caller's code to use a variable created by the macro, its name must **come from the caller**:

```rust
macro_rules! for_each {
    ($var:ident in $list:expr => $body:block) => {
        for $var in $list $body
    };
}

for_each!(x in vec![1, 2] => { println!("{x}"); });
```

---

## 12. Errors and debugging

### 12.1 A clear error message

```rust
macro_rules! color {
    (red) => { (255, 0, 0) };
    (green) => { (0, 255, 0) };
    ($other:tt) => {
        compile_error!(concat!("unknown color: ", stringify!($other)))
    };
}

let c = color!(red);
// let d = color!(blue);   // ERROR: unknown color: blue
```

Without the last arm, the error would be "no rules expected the token `blue`", which is less helpful.

### 12.2 Seeing the generated code

```bash
cargo install cargo-expand
cargo expand                  # the whole crate expanded
cargo expand module::path     # a single module
```

This is the essential tool for understanding an error in a macro: you read the code produced, and compile the faulty part by hand.

### 12.3 Reading an error

An error in the **generated** code is reported at the line of the macro call, with a note `in this macro invocation`. `cargo expand` shows the line that is really at fault.

---

## 13. Standard library macros

| Macro | Role |
|---|---|
| `println!`, `print!`, `eprintln!`, `format!`, `write!`, `writeln!` | formatting (string checked at compile time) |
| `vec!` | build a `Vec` |
| `assert!`, `assert_eq!`, `assert_ne!`, `debug_assert!` | checks |
| `panic!`, `todo!`, `unimplemented!`, `unreachable!` | stops |
| `matches!` | test a pattern |
| `dbg!` | print an expression and its value |
| `concat!`, `stringify!` | assemble literals, turn code into text |
| `include_str!`, `include_bytes!` | include a file at compile time |
| `env!`, `option_env!` | environment variable **at compile time** |
| `cfg!` | test a configuration (feature, OS) |
| `file!`, `line!`, `column!`, `module_path!` | position in the source code |
| `compile_error!` | stop compilation with a message |

```rust
const VERSION: &str = env!("CARGO_PKG_VERSION");          // version from Cargo.toml, fixed at compile time
const README: &str = include_str!("../README.md");        // the file is embedded in the binary
```

---

## 14. Examples in Runique

### 14.1 `urlpatterns!`: a tt muncher

```rust
// runique/src/macros/routeur/router.rs (start)
macro_rules! urlpatterns {
    (@parse $routes:ident;) => {};

    (@parse $routes:ident;
        admin $path:expr => $handler:expr,
        name = $name:expr,
        for_model = $model:ty,
        overrides = $view:expr,
        $($rest:tt)*
    ) => {
        $routes.add_admin_route::<$model>($path, $name, $handler, $view);
        $crate::urlpatterns!(@parse $routes; $($rest)*);
    };
    // ... other arms for the other route forms ...
}
```

All the techniques of this course appear here: `@parse` internal rules, a base case (`(@parse $routes:ident;) => {}`), capturing the rest with `$($rest:tt)*`, the recursive call, specific arms (starting with the word `admin`) placed before the general arms, and `$crate::` so that the recursive call works from the user's crate.

### 14.2 `impl_from_error!`: generating `impl` blocks

```rust
// runique/src/macros/context/impl_error.rs
#[macro_export]
macro_rules! impl_from_error {
    ($($err:ty => $method:ident),*) => {
        $(
            impl From<$err> for AppError {
                fn from(err: $err) -> Self { Self { context: ErrorContext::$method(&err) } }
            }
            impl From<$err> for Box<AppError> {
                fn from(err: $err) -> Self { Box::new(AppError::from(err)) }
            }
        )*
    };
}

impl_from_error!(
    anyhow::Error => from_anyhow,
    DbErr => database,
    tower_sessions::session::Error => database
);
```

A repetition with two variables (`$err` and `$method`), generating two `impl` blocks per entry: three call lines replace six `impl` blocks.

### 14.3 The builder's one-time settings

Runique's builder generates its typestate traits with a `once_only!` macro that receives a documentation attribute (`$(#[$doc:meta])*`), a name (`$name:ident`) and messages (`$message:literal`), then produces a sealed trait with `#[diagnostic::on_unimplemented]` (see `traits-avances`, sections 9 and 10).

---

## 15. Macro, function or generic: the same thing three times

Creating a `HashMap` from pairs.

```rust
use std::collections::HashMap;
use std::hash::Hash;

// 1. Generic function: readable, autocompleted, debuggable
fn map_from<K: Hash + Eq, V, const N: usize>(pairs: [(K, V); N]) -> HashMap<K, V> {
    HashMap::from(pairs)
}

// 2. Standard library: nothing to write
let a = HashMap::from([("one", 1), ("two", 2)]);

// 3. Macro: lighter syntax, nothing more
macro_rules! map {
    ($($k:expr => $v:expr),* $(,)?) => {
        ::std::collections::HashMap::from([$(($k, $v)),*])
    };
}
let b = map! { "one" => 1, "two" => 2 };
let c = map_from([("one", 1)]);
```

Here, the macro only brings a bit of syntax: `HashMap::from` is enough. It is justified when it generates **declarations** (section 7) or a **syntax** that no function can accept (section 8.3).

---

## 16. Exercises

### Exercise 1 — `min!`

Write `min!` that accepts one or more arguments and evaluates each only once.

```rust
// Solution:
macro_rules! min {
    ($x:expr) => { $x };
    ($x:expr, $($rest:expr),+ $(,)?) => {{
        let a = $x;
        let b = min!($($rest),+);
        if a < b { a } else { b }
    }};
}

assert_eq!(min!(5, 2, 8), 2);
```

### Exercise 2 — enum with a list

Write `enum_list!` that generates an enum and a constant `ALL` containing all its variants.

```rust
// Solution:
macro_rules! enum_list {
    ($vis:vis enum $name:ident { $($variant:ident),* $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        $vis enum $name { $($variant),* }

        impl $name {
            pub const ALL: &'static [$name] = &[$($name::$variant),*];
        }
    };
}

enum_list!(pub enum Color { Red, Green, Blue });
assert_eq!(Color::ALL.len(), 3);
```

### Exercise 3 — clear error

Write `size!` that accepts `small`, `medium`, `large` and returns 1, 2, 3, with a readable compile error for any other word.

```rust
// Solution:
macro_rules! size {
    (small) => { 1 };
    (medium) => { 2 };
    (large) => { 3 };
    ($x:tt) => { compile_error!(concat!("unknown size: ", stringify!($x), " (small, medium or large)")) };
}
```

## 17. Cheat sheet

```rust
macro_rules! name {
    () => { .. };                                   // no argument
    ($x:expr) => { .. };                            // an expression
    ($a:ident, $t:ty) => { .. };                    // identifier, type
    ($($x:expr),* $(,)?) => { $( f($x); )* };       // repetition, optional trailing comma
    ($($k:expr => $v:expr),*) => { .. };            // pairs
    (@internal $acc:expr; $($rest:tt)*) => { .. };  // internal rule + tt muncher
    ($x:tt) => { compile_error!("..") };            // last resort: clear error
}
```

Fragments: `expr` `ident` `ty` `path` `pat` `stmt` `block` `item` `literal` `lifetime` `vis` `meta` `tt`.
Repetitions: `*` (0+), `+` (1+), `?` (0-1).

### Key points

- Function first, generic next, macro last.
- The arms are tried in order: specific before general.
- `{{ ... }}` so that an expansion is a single expression.
- Evaluate in a variable, once, any argument used several times.
- Absolute paths in the expansion: `::std::...`, `$crate::...`.
- Local variables are hygienic, generated items are not.
- `compile_error!` for a readable error, `cargo expand` to see the code produced.
