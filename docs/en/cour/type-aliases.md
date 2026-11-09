# Type aliases in Rust

> Giving a short name to a long type, without creating a new type: what an alias allows, what it does not allow, and when to switch to a newtype.

## Table of contents

1. What an alias is for
2. Syntax
3. An alias is not a new type
4. Generic aliases
5. `Result` aliases
6. Aliases for complex types
7. Aliases chosen at compile time
8. Alias, newtype or `use ... as`
9. Limits and pitfalls
10. Organizing your aliases
11. Real examples in Runique
12. Exercises
13. Key points to remember

## 1. What an alias is for

An alias gives a **name** to an existing type. It is used to:

- **shorten** a long type that comes up often (`Arc<RwLock<HashMap<String, String>>>`);
- **centralize** a decision: change the type in a single place (the primary key type);
- **document** the intent (`type Bytes = Vec<u8>`);
- **fix** a parameter of a generic type (`type Result<T> = std::result::Result<T, MyError>`).

It adds **no** checking: it is a synonym, nothing more.

## 2. Syntax

### 2.1 General form

```rust
type Name = ExistingType;

type Kilometers = f64;
type Bytes = Vec<u8>;
type Grid = [[u8; 9]; 9];
type Callback = fn(i32) -> i32;
```

The name follows the convention for types: `PascalCase`. An alias can be `pub`, `pub(crate)` or private, like any item.

### 2.2 Usage

```rust
type Kilometers = f64;

fn distance(a: Kilometers, b: Kilometers) -> Kilometers {
    (a - b).abs()
}

let trip: Kilometers = 42.0;
let d = distance(trip, 10.0);
```

### 2.3 In a struct

```rust
use std::collections::HashMap;

type Scores = HashMap<String, u32>;

struct Game {
    scores: Scores,
}

impl Game {
    fn new() -> Self {
        Self { scores: Scores::new() }     // associated functions go through the alias
    }
}
```

## 3. An alias is not a new type

```rust
type Meters = f64;
type Seconds = f64;

fn speed(d: Meters, t: Seconds) -> f64 {
    d / t
}

let d: Meters = 100.0;
let t: Seconds = 9.58;
speed(t, d);              // compiles: the arguments are swapped, no error
```

`Meters`, `Seconds` and `f64` are **the same type**: the compiler sees no difference. To make it reject the swap, you need newtypes (section 8).

Consequences:

- all the methods and traits of `f64` are available on `Meters`;
- you can pass an `f64` anywhere a `Meters` is expected, and the other way round;
- error messages often show the real type, not the alias.

## 4. Generic aliases

### 4.1 With parameters

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

type Shared<T> = Arc<Mutex<T>>;
type Index<V> = HashMap<String, V>;

let counter: Shared<u32> = Arc::new(Mutex::new(0));
let cities: Index<u32> = Index::new();
```

### 4.2 Fixing some of the parameters

```rust
use std::collections::HashMap;

type Table<K, V> = HashMap<K, V>;
type TextTable<V> = Table<String, V>;        // K fixed
type CountTable = TextTable<u64>;             // K and V fixed
```

Each level fixes one more parameter: you start from a generic alias and specialize it.

### 4.3 Bounds are not checked

```rust
// type Displayable<T: std::fmt::Display> = Vec<T>;
// warning: bounds on generic parameters are not enforced in type aliases
```

A bound in an alias is **ignored** (the compiler warns). Put it on the functions and `impl` blocks that use the type.

## 5. `Result` aliases

### 5.1 The most common pattern

```rust
#[derive(Debug)]
pub enum Error {
    NotFound,
    Database(String),
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn load(id: u32) -> Result<String> {
    if id == 0 {
        return Err(Error::NotFound);
    }
    Ok("Ada".into())
}
```

The standard library does the same: `std::io::Result<T>`, `std::fmt::Result`. So do SeaORM, anyhow (`anyhow::Result<T>`) and Runique (`AppResult<T>`).

### 5.2 The shadowing pitfall

```rust
use std::io;

type Result<T> = std::result::Result<T, io::Error>;

// fn parse(s: &str) -> Result<i32, std::num::ParseIntError> { ... }
// ERROR: this Result has only one parameter
fn parse(s: &str) -> std::result::Result<i32, std::num::ParseIntError> {
    s.parse()
}
```

An alias named `Result` **shadows** the prelude one in the whole module. Two ways to avoid the confusion:

```rust
// 1. Give it a distinct name
type IoResult<T> = std::result::Result<T, std::io::Error>;

// 2. Use it prefixed from another module: io::Result<T>, my_lib::Result<T>
```

### 5.3 A `Result` alias with a default error

```rust
pub type Outcome<T, E = Error> = std::result::Result<T, E>;

fn a() -> Outcome<u32> { Ok(1) }                         // E = Error
fn b() -> Outcome<u32, String> { Err("other".into()) }   // E chosen
```

## 6. Aliases for complex types

### 6.1 Closures and trait objects

```rust
use std::future::Future;
use std::pin::Pin;

type Listener = Box<dyn Fn(&str) + Send + Sync>;
type SendableFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;
type AsyncHandler = Box<dyn Fn(String) -> SendableFuture<String> + Send + Sync>;

struct Bus {
    listeners: Vec<Listener>,
}
```

Without an alias, `Vec<Box<dyn Fn(&str) + Send + Sync>>` comes back in every signature.

### 6.2 Function pointers

```rust
type Operation = fn(f64, f64) -> f64;

fn apply(op: Operation, a: f64, b: f64) -> f64 {
    op(a, b)
}

let table: [(&str, Operation); 2] = [("+", |a, b| a + b), ("*", |a, b| a * b)];
```

### 6.3 Shared state

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

type Cache = Arc<RwLock<HashMap<String, Vec<u8>>>>;

fn read(cache: &Cache, key: &str) -> Option<Vec<u8>> {
    cache.read().unwrap().get(key).cloned()
}
```

### 6.4 Alias of an enum variant

```rust
type Grade = Option<u8>;

let a: Grade = Grade::Some(15);       // variants go through the alias
let b = Grade::None;
```

## 7. Aliases chosen at compile time

An alias can depend on the platform or on a feature: the same name designates a different type depending on the build.

```rust
#[cfg(feature = "big-numbers")]
pub type Integer = i64;

#[cfg(not(feature = "big-numbers"))]
pub type Integer = i32;

pub fn double(x: Integer) -> Integer { x * 2 }
```

The rest of the code writes `Integer` and does not change. Warning: both versions must compile, so the code must only use what the two types have in common. You must test every combination of features (see `cargo-dependances`).

```rust
#[cfg(unix)]
pub type Descriptor = std::os::unix::io::RawFd;      // i32
#[cfg(windows)]
pub type Descriptor = std::os::windows::io::RawHandle;
```

## 8. Alias, newtype or `use ... as`

### 8.1 The same idea, three syntaxes

```rust
// 1. Alias: a synonym
type UserId = u32;

// 2. Newtype: a new type
struct OrderId(u32);

// 3. Rename on import: another name for an existing item
use std::collections::HashMap as Table;
```

### 8.2 Comparison

| | `type` alias | Newtype `struct X(T)` | `use ... as` |
|---|---|---|---|
| Creates a new type | no | yes | no |
| Prevents mixing with the original type | no | yes | no |
| Methods of the original type | all | must be re-exposed (or `Deref`) | all |
| Implement a foreign trait | no (unless the original type is local) | yes | no |
| Can fix generic parameters | yes | yes | no |
| Runtime cost | none | none | none |

### 8.3 When to use which

- **Alias**: shorten, centralize a type decision, fix an error parameter.
- **Newtype**: prevent a mix-up (identifiers, units), guarantee validation (`Email` built by `parse`), hide a representation, implement a foreign trait.
- **`use ... as`**: resolve a name conflict between two imports (`fmt::Result` and `io::Result`).

### 8.4 The same code, from alias to newtype

```rust
// Alias: nothing prevents the mistake
type Euros = i64;
type Cents = i64;
fn pay(amount: Cents) {}
let price: Euros = 12;
pay(price);                               // compiles: bug (12 cents instead of 12 euros)

// Newtype: the mistake becomes a compile error
struct InEuros(i64);
struct InCents(i64);
impl From<InEuros> for InCents {
    fn from(e: InEuros) -> Self { InCents(e.0 * 100) }
}
fn pay2(amount: InCents) {}
// pay2(InEuros(12));                     // ERROR: expected InCents
pay2(InEuros(12).into());                 // explicit conversion
```

## 9. Limits and pitfalls

### 9.1 No extra checking

See section 3: an alias never protects against a mix-up of values.

### 9.2 No tuple struct constructor through the alias

```rust
struct Point(i32, i32);
type P = Point;

// let p = P(1, 2);               // ERROR E0423: the alias exists only as a type
let p = Point(1, 2);
let q: P = Point(3, 4);
let r = P { 0: 5, 1: 6 };         // the brace form goes through the alias
```

For a struct with named fields, `P { x, y }` works.

### 9.3 Implementing a trait

```rust
type List = Vec<i32>;

// impl std::fmt::Display for List { ... }
// ERROR E0117: Display and Vec come from the standard library (orphan rule)
```

The alias **is** `Vec<i32>`: the implementation would target `Vec<i32>`, which only the standard library can do. A newtype (`struct List(Vec<i32>)`) allows it.

Conversely, on an alias of a **local** type, `impl` works normally, since it targets the local type.

### 9.4 Error messages

The compiler often displays the real type (`HashMap<String, Vec<u8>>`) rather than the alias (`Cache`). With many nested aliases, finding the origin of an error requires unfolding them.

### 9.5 Too many aliases

One alias per use (`FormData`, `Errors`, `Parameters` for the same `HashMap<String, String>`) makes people think there are three different types. Prefer one alias per type **shape**, named after its structure (`StrMap`), unless the use carries a real decision.

## 10. Organizing your aliases

### 10.1 Centralize

```rust
// src/types.rs
use std::collections::HashMap;
use std::sync::Arc;

pub type StrMap = HashMap<String, String>;
pub type Shared<T> = Arc<T>;
pub type Result<T> = std::result::Result<T, crate::Error>;
```

```rust
// src/lib.rs
pub mod types;
pub use types::*;          // or through a prelude
```

### 10.2 Or keep them close to where they are used

An alias used in a single module stays in that module, private. Only shared aliases go in a common file.

### 10.3 Naming conventions observed

| Prefix / form | Meaning | Example |
|---|---|---|
| `A...` | inside an `Arc` | `AEngine = Arc<RuniqueEngine>` |
| `O...` | inside an `Option` | `OCurrentUser = Option<CurrentUser>` |
| `...Map` | a map | `StrMap`, `JsonMap` |
| `...Result<T>` | a `Result` with a fixed error | `AppResult<T>`, `IoResult<T>` |

## 11. Real examples in Runique

Runique's aliases are grouped in `runique/src/utils/aliases/definition.rs`, and most of them are re-exported by `runique::prelude`.

### 11.1 An alias chosen by feature: `Pk`

```rust
// runique/src/utils/config/pk.rs
#[cfg(all(feature = "big-pk", feature = "pk-uuid"))]
compile_error!("features `big-pk` and `pk-uuid` are mutually exclusive — enable at most one of them");

#[cfg(feature = "pk-uuid")]
pub type Pk = uuid::Uuid;

#[cfg(all(feature = "big-pk", not(feature = "pk-uuid")))]
pub type Pk = i64;

#[cfg(not(any(feature = "big-pk", feature = "pk-uuid")))]
pub type Pk = i32;
```

The whole framework writes `Pk`. Changing the primary key type means changing a feature in `Cargo.toml`, not hundreds of signatures. The `compile_error!` prevents enabling two contradictory choices.

### 11.2 An internal alias chosen by feature: `Inner` of `ADb`

```rust
// runique/src/db/adb.rs
#[cfg(not(feature = "test-utils"))]
type Inner = sea_orm::DatabaseConnection;
#[cfg(feature = "test-utils")]
type Inner = crate::db::RuniqueDb;      // real connection or test transaction

#[derive(Clone, Debug)]
pub struct ADb(Arc<Inner>);
```

The alias is private: the user only sees `ADb`, whatever the feature.

### 11.3 When an alias is no longer enough: `ADb`

The database was first an alias, `Arc<DatabaseConnection>`. Problem: every SeaORM call (`.insert(db)`, `.one(db)`, `db.begin()`) expects a type that implements `ConnectionTrait`, and `Arc<DatabaseConnection>` does not. You had to write `.as_ref()` everywhere.

It cannot be fixed with an alias: `impl ConnectionTrait for Arc<DatabaseConnection>` is rejected by the orphan rule (the trait and the type come from other crates, and `Arc` is not `#[fundamental]`). Hence a newtype:

```rust
impl ConnectionTrait for ADb { /* delegates to the connection */ }
impl TransactionTrait for ADb { /* same */ }
```

`ADb` is still as cheap to clone as an `Arc`, and `&ADb` is used directly: `.insert(db)`, without `.as_ref()`. This is the exact illustration of the "implement a foreign trait" row of the table in section 8.2.

### 11.4 Aliases for collections and results

```rust
// runique/src/utils/aliases/definition.rs
pub type AEngine = Arc<RuniqueEngine>;
pub type StrMap = HashMap<String, String>;
pub type StrVecMap = HashMap<String, Vec<String>>;
pub type JsonMap = HashMap<String, serde_json::Value>;
pub type AppResult<T> = Result<T, Box<AppError>>;

// In a handler
pub async fn contact(mut request: Request) -> AppResult<Response> {
    // ...
}
```

### 11.5 Alias for a builder state type

```rust
// runique/src/app/builder/mod.rs
pub mod state {
    pub struct No;
    pub struct Yes;
    pub type Fresh = (No, No, No, No, No, No, No);
}
```

Runique's builder records in its type which one-time settings have already been made (`No` or `Yes` per setting). `Fresh` names the starting state: without the alias, you would have to write the tuple of seven `No` in every signature that mentions it.

## 12. Exercises

### Exercise 1: refactoring

Simplify these signatures with aliases.

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

fn read(c: &Arc<Mutex<HashMap<String, Vec<String>>>>, k: &str) -> Option<Vec<String>> { None }
fn write(c: &Arc<Mutex<HashMap<String, Vec<String>>>>, k: String, v: Vec<String>) {}
fn load() -> Result<Arc<Mutex<HashMap<String, Vec<String>>>>, std::io::Error> { todo!() }
```

```rust
// Solution:
type Groups = HashMap<String, Vec<String>>;
type Shared<T> = Arc<Mutex<T>>;
type IoResult<T> = std::result::Result<T, std::io::Error>;

fn read(c: &Shared<Groups>, k: &str) -> Option<Vec<String>> {
    c.lock().unwrap().get(k).cloned()
}
fn write(c: &Shared<Groups>, k: String, v: Vec<String>) {
    c.lock().unwrap().insert(k, v);
}
fn load() -> IoResult<Shared<Groups>> {
    Ok(Arc::new(Mutex::new(Groups::new())))
}
```

### Exercise 2: alias or newtype

For each case, choose alias or newtype: (a) the error type of your whole library; (b) user and product identifiers, both `i64`; (c) `Box<dyn Fn(&Event) + Send + Sync>` repeated everywhere; (d) an always-valid email address.

Answers: (a) alias `type Result<T> = std::result::Result<T, Error>`; (b) newtypes, so they cannot be mixed up; (c) alias; (d) newtype with a constructor that validates.

### Exercise 3: alias by feature

Write an alias `Timestamp` that equals `chrono::DateTime<chrono::Utc>` with the `chrono` feature, and `u64` (seconds) without it.

```rust
// Solution:
#[cfg(feature = "chrono")]
pub type Timestamp = chrono::DateTime<chrono::Utc>;

#[cfg(not(feature = "chrono"))]
pub type Timestamp = u64;
```

The code that handles `Timestamp` can only use what exists for both types (comparison, copy): everything else must be placed under the same `#[cfg]`.

## 13. Key points to remember

- An alias is a **synonym**: no new type, no checking.
- It is used to shorten, centralize and fix parameters (`type Result<T> = ...`).
- The generic bounds of an alias are ignored.
- An alias named `Result` shadows the prelude one in its module.
- To prevent a mix-up, validate, or implement a foreign trait: a **newtype**.
- `use ... as` renames an import, it does not create a type.
- An alias under `#[cfg]` changes the type depending on the build: test every combination.
