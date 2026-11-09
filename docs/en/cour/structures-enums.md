# Structs and Enums in Rust

**Modelling your data** — structs, enums, `Option`, `Result`, methods and derived traits, with the different ways to build, transform and read each value.

## Course objectives

By the end of this course, you will be able to:

- Define the three forms of struct (named fields, tuple, unit) and know which one to choose
- Build a value in every way: literal, shorthands, update, `new`, `Default`, builder
- Write methods that read, modify or consume
- Derive the standard traits and know what each one allows
- Define enums with or without data, with discriminants, recursive
- Handle `Option` and `Result` with `match`, `if let`, `?` and their methods
- Choose between struct, enum and booleans to represent a state

## Table of contents

1. Structs with named fields
2. Building a struct
3. Methods and associated functions
4. Tuple structs and newtype
5. Unit structs
6. Derived traits
7. Enums
8. Enums with data
9. `Option<T>`
10. `Result<T, E>`
11. Reading an enum: all the syntaxes
12. Modelling well
13. Practical examples
14. Exercises
15. Cheat sheet

## 1. Structs with named fields

### 1.1 — Definition

```rust
struct User {
    name: String,
    email: String,
    age: u32,
    active: bool,
}
```

Type names in `PascalCase`, fields in `snake_case`. A struct groups values that go **together**: it is passed, returned and stored as a whole.

### 1.2 — Accessing fields

```rust
let mut u = User {
    name: String::from("Alice"),
    email: String::from("alice@example.com"),
    age: 30,
    active: true,
};

println!("{}", u.name);
u.age += 1;          // the whole variable must be mut
```

Mutability applies to the **whole** variable: you cannot make a single field modifiable. For that you need a method that only modifies that field, or interior mutability (`Cell`, `RefCell`, see `smart-pointers`).

### 1.3 — Field visibility

```rust
pub struct Account {
    pub name: String,    // readable and modifiable outside the module
    balance: i64,        // private: only the module that defines Account can access it
}

impl Account {
    pub fn new(name: String) -> Self {
        Self { name, balance: 0 }
    }
    pub fn balance(&self) -> i64 {
        self.balance
    }
}
```

What it is for: a private field guarantees an invariant. Here, nobody can set an arbitrary balance: they must go through the methods. Consequence: a single private field prevents building the struct with a literal outside the module; you need a constructor.

### 1.4 — Struct that holds references

```rust
struct Excerpt<'a> {
    text: &'a str,
}
```

A struct that borrows must declare a lifetime: it cannot live longer than what it borrows. See the `lifetimes` course. To start with, prefer owned fields (`String`, `Vec`).

## 2. Building a struct

### 2.1 — The full literal

```rust
let u = User {
    name: String::from("Alice"),
    email: String::from("a@ex.com"),
    age: 30,
    active: true,
};
```

All the fields are required, in any order.

### 2.2 — Shorthand when the variable has the name of the field

```rust
fn create(name: String, email: String) -> User {
    // Long form
    User { name: name, email: email, age: 0, active: true }
}

fn create_short(name: String, email: String) -> User {
    // Shorthand
    User { name, email, age: 0, active: true }
}
```

### 2.3 — Update from another value

```rust
let u1 = User { name: "A".into(), email: "a@ex.com".into(), age: 30, active: true };

let u2 = User {
    email: String::from("b@ex.com"),
    ..u1                          // the other fields come from u1
};

// println!("{}", u1.name);       // ERROR: name was MOVED to u2
println!("{}", u1.age);           // OK: age is Copy, it was copied
```

`..other` moves the non-`Copy` fields. To keep `u1` whole, write `..u1.clone()`.

### 2.4 — `new` constructor

```rust
impl User {
    pub fn new(name: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            email: email.into(),
            age: 0,
            active: true,
        }
    }
}

let u = User::new("Alice", "a@ex.com");
```

`new` is not a keyword, only a convention. You write other constructors with meaningful names: `User::guest()`, `Color::from_hex("#fff")`.

### 2.5 — Constructor that can fail

```rust
pub struct Email(String);

impl Email {
    pub fn parse(text: &str) -> Result<Self, String> {
        if text.contains('@') {
            Ok(Email(text.to_string()))
        } else {
            Err(format!("invalid address: {text}"))
        }
    }
}
```

What it is for: once built, an `Email` is **necessarily** valid. Any function that receives an `Email` no longer has to check. This is the "parse, don't validate" principle.

### 2.6 — `Default`

```rust
#[derive(Default, Debug)]
struct Options {
    port: u16,        // 0
    host: String,     // ""
    verbose: bool,    // false
}

let o = Options::default();
let o2 = Options { port: 8080, ..Default::default() };   // only what changes
```

Write `Default` by hand when the default values are not the "zeros":

```rust
struct Config { port: u16, timeout_s: u64 }

impl Default for Config {
    fn default() -> Self {
        Self { port: 8080, timeout_s: 30 }
    }
}
```

Deriving `Default` on a struct of limits gives 0 everywhere, which can block everything (a maximum size of 0). In that case, `Default` is always written by hand.

### 2.7 — Builder

```rust
pub struct Request { url: String, timeout: u64, retries: u32 }

pub struct RequestBuilder { url: String, timeout: u64, retries: u32 }

impl RequestBuilder {
    pub fn new(url: &str) -> Self {
        Self { url: url.to_string(), timeout: 30, retries: 1 }
    }
    pub fn timeout(mut self, s: u64) -> Self { self.timeout = s; self }
    pub fn retries(mut self, n: u32) -> Self { self.retries = n; self }
    pub fn build(self) -> Request {
        Request { url: self.url, timeout: self.timeout, retries: self.retries }
    }
}

let r = RequestBuilder::new("https://ex.com").timeout(5).retries(3).build();
```

### 2.8 — Choosing

| Situation | Construction |
|---|---|
| Few fields, all meaningful | literal or `new` |
| An invariant to guarantee | private fields + `new` or `parse` returning `Result` |
| Many fields, most of them default | `Default` + `..Default::default()` |
| Many options, public API that will evolve | builder |

## 3. Methods and associated functions

### 3.1 — The four receivers

```rust
struct Rectangle { width: u32, height: u32 }

impl Rectangle {
    // Associated function: no self, called with ::
    fn square(side: u32) -> Self {
        Self { width: side, height: side }
    }

    // &self: reads
    fn area(&self) -> u32 {
        self.width * self.height
    }

    // &mut self: modifies
    fn grow(&mut self, n: u32) {
        self.width += n;
        self.height += n;
    }

    // self: consumes and returns something else
    fn into_tuple(self) -> (u32, u32) {
        (self.width, self.height)
    }
}

fn main() {
    let mut r = Rectangle::square(3);
    let a = r.area();
    r.grow(1);
    let (w, h) = r.into_tuple();
    // r.area();        // ERROR: r was consumed
}
```

| Receiver | The caller after the call | Typical use |
|---|---|---|
| none | — | constructors, utilities |
| `&self` | keeps the value intact | getters, computations |
| `&mut self` | keeps the value, modified | setters, adding elements |
| `self` | loses the value | conversions (`into_*`), builders, closing a resource |

### 3.2 — Calling by method or by path

```rust
let r = Rectangle::square(2);
let a1 = r.area();               // method syntax
let a2 = Rectangle::area(&r);    // path syntax: the same function
```

The method syntax automatically takes the reference (`&r` or `&mut r`) depending on what the method asks for. The path syntax is used to resolve an ambiguity when two traits have a method with the same name.

### 3.3 — Several `impl` blocks

```rust
impl Rectangle {
    fn width(&self) -> u32 { self.width }
}

impl Rectangle {
    fn height(&self) -> u32 { self.height }
}
```

Allowed: useful to separate methods by theme, or to put some under `#[cfg(feature = "...")]`.

### 3.4 — Associated constants

```rust
impl Rectangle {
    const MAX_SIDE: u32 = 10_000;

    fn is_valid(&self) -> bool {
        self.width <= Self::MAX_SIDE && self.height <= Self::MAX_SIDE
    }
}
```

### 3.5 — `Self`

In an `impl` block, `Self` refers to the current type. `Self { .. }`, `-> Self` and `Self::CONSTANT` avoid repeating the name, and stay correct if you rename the type.

### 3.6 — Getter: by reference or by copy

```rust
pub struct Profile { name: String, age: u32 }

impl Profile {
    pub fn name(&self) -> &str { &self.name }   // reference: no copy
    pub fn age(&self) -> u32 { self.age }       // Copy: return the value
    pub fn name_owned(&self) -> String { self.name.clone() }  // only if the caller must own it
}
```

In Rust, a getter has the name of the field (`name()`, not `get_name()`).

## 4. Tuple structs and newtype

### 4.1 — Definition

```rust
struct Color(u8, u8, u8);
struct Point(i32, i32);

let black = Color(0, 0, 0);
let red = black.0;
let Point(x, y) = Point(1, 2);
```

`Color` and `Point` are two **different** types even if their fields have similar types.

### 4.2 — The newtype: a type for a meaning

```rust
struct UserId(u32);
struct OrderId(u32);

fn cancel(order: OrderId) {}

fn main() {
    let u = UserId(5);
    // cancel(u);          // ERROR: expected OrderId, found UserId
    cancel(OrderId(12));
}
```

What it is for:

- preventing the mixing of two values of the same raw type (a user identifier passed in place of an order identifier);
- attaching a validation (`Email(String)` built by `parse`);
- implementing a foreign trait on a foreign type (orphan rule, see `traits-avances`);
- hiding a representation: `struct Password(String)` without a derived `Debug` is not printed by accident in logs.

The newtype costs nothing at runtime: `UserId` takes exactly the space of a `u32`.

### 4.3 — Accessing the value of a newtype

```rust
struct Meters(f64);

impl Meters {
    fn value(&self) -> f64 { self.0 }
}

impl From<Meters> for f64 {
    fn from(m: Meters) -> f64 { m.0 }
}

let m = Meters(3.0);
let a = m.0;            // direct field (if visible)
let b = m.value();      // method
let c: f64 = m.into();  // conversion
```

## 5. Unit structs

```rust
struct Marker;

struct TestMode;
struct ProdMode;

trait Mode { const NAME: &'static str; }
impl Mode for TestMode { const NAME: &'static str = "test"; }
impl Mode for ProdMode { const NAME: &'static str = "prod"; }
```

A struct without fields takes no space. It is used to carry a trait or a type without data: a type marker, the implementation of a behaviour (a `struct Argon2Hasher;` that implements a hashing trait), the state of a builder checked at compile time.

## 6. Derived traits

### 6.1 — What each `derive` allows

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
struct Version {
    major: u32,
    minor: u32,
}
```

| Trait | What it allows | Condition |
|---|---|---|
| `Debug` | `{:?}`, `dbg!`, `assert_eq!` | all fields `Debug` |
| `Clone` | `.clone()`: explicit copy | all fields `Clone` |
| `Copy` | implicit copy on assignment | `Clone` too, `Copy` fields (no `String`, `Vec`) |
| `PartialEq` | `==`, `!=` | `PartialEq` fields |
| `Eq` | total equality (required by `HashMap`) | no `f32`/`f64` |
| `Hash` | `HashMap` key, `HashSet` element | with `Eq` |
| `PartialOrd` | `<`, `>`, `<=`, `>=` | |
| `Ord` | `.sort()`, `.max()`, `BTreeMap` key | with `Eq` and `PartialOrd` |
| `Default` | `::default()` | `Default` fields |

### 6.2 — How the derived comparison works

```rust
#[derive(PartialEq, PartialOrd)]
struct Version { major: u32, minor: u32 }

let a = Version { major: 1, minor: 9 };
let b = Version { major: 2, minor: 0 };
assert!(a < b);     // compares major first, then minor: field order
```

The derived order follows the **declaration order** of the fields, and for an enum, the order of the variants.

### 6.3 — Equality by hand

```rust
struct User { id: u32, name: String }

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id       // two users are equal if they have the same id
    }
}
```

### 6.4 — `Display`: the output for the user

`Display` cannot be derived: it is written by hand.

```rust
use std::fmt;

struct Version { major: u32, minor: u32 }

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

let v = Version { major: 4, minor: 0 };
println!("{v}");             // 4.0
let text = v.to_string();    // to_string comes with Display
```

### 6.5 — `Clone` or `Copy`?

`Copy` is reserved for small values without a resource (numbers, coordinates, identifiers). As soon as a field allocates memory, only `Clone` is possible, and the `.clone()` call stays visible in the code: this is intended, because a copy has a cost.

## 7. Enums

### 7.1 — Simple definition

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
enum Direction {
    North,
    South,
    East,
    West,
}

let d = Direction::North;
```

An enum is a type whose value is **exactly one** of the variants.

### 7.2 — Importing the variants

```rust
use Direction::*;          // avoid outside a small local block

fn opposite(d: Direction) -> Direction {
    match d {
        North => South,
        South => North,
        East => West,
        West => East,
    }
}
```

Pitfall: if you misspell a variant in a `match` with `use Direction::*`, it becomes a **variable** that catches everything. The compiler warns, but it is better to write `Direction::North` or `Self::North`.

### 7.3 — Discriminants

```rust
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum Status {
    Draft = 0,
    Published = 1,
    Archived = 9,
}

let code = Status::Archived as u8;   // 9
```

The reverse direction (`u8` to `Status`) does not exist automatically: a value like `5` would have no variant. You write it with `TryFrom`:

```rust
impl TryFrom<u8> for Status {
    type Error = String;
    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Status::Draft),
            1 => Ok(Status::Published),
            9 => Ok(Status::Archived),
            _ => Err(format!("unknown status: {v}")),
        }
    }
}

let s = Status::try_from(1);    // Ok(Published)
let s: Result<Status, _> = 5_u8.try_into();   // Err; the u8 suffix says which conversion to use
```

### 7.4 — Methods on an enum

```rust
impl Direction {
    fn is_vertical(self) -> bool {
        matches!(self, Direction::North | Direction::South)
    }

    const ALL: [Direction; 4] = [Direction::North, Direction::South, Direction::East, Direction::West];
}

for d in Direction::ALL {
    println!("{d:?} {}", d.is_vertical());
}
```

Rust provides no standard way to iterate over the variants: a constant like `ALL` is the simple solution (or the `strum` crate).

## 8. Enums with data

### 8.1 — Three forms of variants

```rust
enum Message {
    Quit,                              // no data
    Move { x: i32, y: i32 },           // named fields
    Write(String),                     // tuple
    Color(u8, u8, u8),
}

let m1 = Message::Quit;
let m2 = Message::Move { x: 10, y: 20 };
let m3 = Message::Write(String::from("hello"));
```

Each variant carries its own data. The compiler guarantees that you only read `x` if the value really is a `Move`.

### 8.2 — Size in memory

An enum takes the size of **its largest variant**, plus enough to know which one is active. A very large variant makes every value of the enum bigger: you then put it in a `Box`.

```rust
enum Event {
    Click(u16, u16),
    Report(Box<[u8; 4096]>),   // without Box, every Click would weigh 4 KB
}
```

### 8.3 — Recursive enum

```rust
enum Expr {
    Number(f64),
    Sum(Box<Expr>, Box<Expr>),
    Product(Box<Expr>, Box<Expr>),
}

fn evaluate(e: &Expr) -> f64 {
    match e {
        Expr::Number(n) => *n,
        Expr::Sum(a, b) => evaluate(a) + evaluate(b),
        Expr::Product(a, b) => evaluate(a) * evaluate(b),
    }
}

let e = Expr::Sum(Box::new(Expr::Number(2.0)), Box::new(Expr::Number(3.0)));
println!("{}", evaluate(&e)); // 5
```

Without `Box`, the size of `Expr` would be infinite: `Box` stores the sub-expression elsewhere and only keeps a pointer.

### 8.4 — Conversion to an enum

```rust
enum Value { Integer(i64), Text(String) }

impl From<i64> for Value {
    fn from(n: i64) -> Self { Value::Integer(n) }
}
impl From<&str> for Value {
    fn from(s: &str) -> Self { Value::Text(s.to_string()) }
}

let v: Value = 5.into();
let t: Value = "abc".into();
```

### 8.5 — `#[non_exhaustive]`

```rust
#[non_exhaustive]
pub enum ApiError {
    NotFound,
    Forbidden,
}
```

Outside the crate that defines it, a `match` on `ApiError` must have a `_` arm. What it is for: a library can add a variant later without it being a compatibility break. Inside the crate, the `match` stays exhaustive.

## 9. `Option<T>`

### 9.1 — Definition

```rust
enum Option<T> {
    None,
    Some(T),
}
```

`Option` replaces `null`: a value that can be absent has the type `Option<T>`, and you cannot use the `T` without handling the `None`.

### 9.2 — Extracting the value: all the ways

```rust
let x: Option<i32> = Some(5);

// 1. match: all the cases, the most explicit
let v = match x {
    Some(n) => n,
    None => 0,
};

// 2. if let: only one case interests you
if let Some(n) = x {
    println!("{n}");
}

// 3. let else: extract or leave the function
// let Some(n) = x else { return; };

// 4. Fallback value
let v = x.unwrap_or(0);                  // value computed in every case
let v = x.unwrap_or_else(|| compute());  // computed only if None
let v = x.unwrap_or_default();           // Default of the type (0, "", empty vec)

// 5. Propagation with ? (in a function that returns Option)
// let n = x?;

// 6. Panic if None (tests, prototypes, truly impossible cases)
let v = x.unwrap();
let v = x.expect("x is always set after initialisation");

fn compute() -> i32 { 0 }
```

| Form | When to use it |
|---|---|
| `match` | the two cases do different, long things |
| `if let` | only `Some` triggers an action |
| `let ... else` | `None` must stop the function |
| `unwrap_or*` | a fallback value makes sense |
| `?` | the caller must decide |
| `expect` | `None` would be a bug: the message says why it is impossible |
| `unwrap` | tests and prototypes |

### 9.3 — Transforming without extracting

```rust
let name: Option<&str> = Some("alice");

let length = name.map(|n| n.len());                      // Some(5)
let uppercase = name.map(str::to_uppercase);             // Some("ALICE")
let short = name.filter(|n| n.len() < 3);                // None
let id = name.and_then(|n| n.parse::<u32>().ok());       // None: and_then for a step that returns Option
let fallback = name.or(Some("guest"));
let res: Result<&str, &str> = name.ok_or("missing name"); // Option -> Result
let all = name.map_or(0, |n| n.len());                    // transform or fallback value
```

### 9.4 — Testing

```rust
let x = Some(5);
x.is_some();
x.is_none();
x.is_some_and(|n| n > 3);     // true
x.is_none_or(|n| n > 3);      // true if None or if the condition is true
```

### 9.5 — Borrowing the content

```rust
let name: Option<String> = Some(String::from("alice"));

let r: Option<&String> = name.as_ref();     // without moving name
let s: Option<&str> = name.as_deref();      // Option<String> -> Option<&str>

let mut taken = Some(3);
let old = taken.take();                     // taken becomes None, old = Some(3)
let mut slot = Some(1);
let previous = slot.replace(2);             // slot = Some(2), previous = Some(1)
let mut cache: Option<Vec<i32>> = None;
let v = cache.get_or_insert_with(Vec::new); // creates if absent, returns &mut
v.push(1);
```

`as_deref` is the common way to pass an `Option<String>` to a function that expects `Option<&str>`.

### 9.6 — Combining several `Option` values

```rust
let a = Some(2);
let b = Some(3);

// 1. match on the tuple
let sum = match (a, b) {
    (Some(x), Some(y)) => Some(x + y),
    _ => None,
};

// 2. zip
let sum = a.zip(b).map(|(x, y)| x + y);

// 3. ? in a function
fn add(a: Option<i32>, b: Option<i32>) -> Option<i32> {
    Some(a? + b?)
}
```

## 10. `Result<T, E>`

### 10.1 — Definition

```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

`Result` carries either the value, or **the reason** for the failure. It is marked `#[must_use]`: ignoring a `Result` causes a warning.

### 10.2 — Extracting and transforming

```rust
let r: Result<i32, String> = "42".parse::<i32>().map_err(|e| e.to_string());

let v = match &r {
    Ok(n) => *n,
    Err(e) => { eprintln!("{e}"); 0 }
};

let v = r.clone().unwrap_or(0);
let v = r.clone().unwrap_or_else(|e| { eprintln!("{e}"); 0 });
let double = r.clone().map(|n| n * 2);                  // transforms Ok
let msg = r.clone().map_err(|e| format!("error: {e}")); // transforms Err
let next = r.clone().and_then(|n| if n > 0 { Ok(n) } else { Err("negative".into()) });
let opt: Option<i32> = r.clone().ok();                  // Result -> Option (the error is lost)
let is_ok = r.is_ok();
```

### 10.3 — `?`: propagating

```rust
fn read_age(text: &str) -> Result<u32, std::num::ParseIntError> {
    let age = text.trim().parse::<u32>()?;   // Err: the function returns the error right away
    Ok(age)
}
```

`?` is the equivalent of:

```rust
let age = match text.trim().parse::<u32>() {
    Ok(v) => v,
    Err(e) => return Err(e.into()),   // into: converts the error if From exists
};
```

Errors, their types and their conversion are detailed in `gestion-des-erreurs`.

### 10.4 — `Option` or `Result`?

- `Option`: the absence is **normal** and needs no explanation (`v.first()`, `map.get(k)`).
- `Result`: the failure has a **cause** that the caller may want to know or display (file not found, invalid input).

## 11. Reading an enum: all the syntaxes

The same function, which returns the text of a message, written in five ways.

### 11.1 — `match` with destructuring

```rust
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
}

fn describe(m: &Message) -> String {
    match m {
        Message::Quit => "quit".to_string(),
        Message::Move { x, y } => format!("go to {x},{y}"),
        Message::Write(text) => text.clone(),
    }
}
```

### 11.2 — Method on the enum with `Self`

```rust
impl Message {
    fn describe(&self) -> String {
        match self {
            Self::Quit => "quit".into(),
            Self::Move { x, y } => format!("go to {x},{y}"),
            Self::Write(t) => t.clone(),
        }
    }
}
```

### 11.3 — A single case: `if let` or `let else`

```rust
fn text(m: &Message) -> Option<&str> {
    if let Message::Write(t) = m {
        Some(t.as_str())
    } else {
        None
    }
}

fn text_alt(m: &Message) -> Option<&str> {
    let Message::Write(t) = m else { return None };
    Some(t.as_str())
}
```

### 11.4 — Testing the variant: `matches!`

```rust
fn is_move(m: &Message) -> bool {
    matches!(m, Message::Move { .. })
}

fn is_far_move(m: &Message) -> bool {
    matches!(m, Message::Move { x, .. } if x.abs() > 100)
}
```

### 11.5 — Ignored fields, binding and guards

```rust
fn summary(m: &Message) -> &'static str {
    match m {
        Message::Move { x: 0, y: 0 } => "in place",
        Message::Move { x: 0, .. } => "vertical",
        Message::Move { .. } => "elsewhere",
        Message::Write(t) if t.is_empty() => "empty message",
        Message::Write(_) => "message",
        Message::Quit => "end",
    }
}
```

All the available patterns are in the `pattern-matching` course.

## 12. Modelling well

### 12.1 — Booleans that contradict each other

```rust
// Bad: four combinations, one of them impossible (paid AND cancelled?)
struct Order {
    paid: bool,
    cancelled: bool,
}

// Good: only one possible value at a time
enum OrderState {
    Pending,
    Paid,
    Cancelled,
}
```

Make impossible states **impossible to write**: the compiler then does the checking work.

### 12.2 — Data that exists in only one state

```rust
// Bad: shipped_on only makes sense if shipped == true
struct Parcel {
    shipped: bool,
    shipped_on: Option<String>,
    tracking: Option<String>,
}

// Good: each state carries exactly its data
enum Parcel2 {
    Preparing,
    Shipped { date: String, tracking: String },
    Delivered { date: String },
}
```

### 12.3 — Struct, enum or trait?

| Situation | Choice |
|---|---|
| Several values present **together** | struct |
| **One** value among a known, closed set | enum |
| **Open** set: other crates must be able to add cases | trait (see `box-dynamiques`) |
| A state with common data and specific data | struct with an enum field |

```rust
struct Payment {
    amount_cents: i64,
    date: String,
    method: Method,        // common data + variable part
}

enum Method {
    Card { last_four: String },
    Transfer { iban: String },
    Cash,
}
```

## 13. Practical examples

### Example 1 — accounts and roles

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Role {
    User,
    Moderator,
    Admin,
}

#[derive(Debug)]
struct Account {
    id: u32,
    name: String,
    role: Role,
}

impl Account {
    fn new(id: u32, name: impl Into<String>) -> Self {
        Self { id, name: name.into(), role: Role::User }
    }

    fn promote(&mut self, role: Role) {
        self.role = role;
    }

    fn can_moderate(&self) -> bool {
        self.role >= Role::Moderator      // possible thanks to PartialOrd: the order of the variants
    }
}

fn main() {
    let mut c = Account::new(1, "Alice");
    println!("{}", c.can_moderate());     // false
    c.promote(Role::Admin);
    println!("{}", c.can_moderate());     // true
    println!("{c:?}");
}
```

Warning: the `>=` comparison depends on the declaration order. Adding a variant in the wrong place silently changes the permissions. For permissions, an explicit `match` is safer:

```rust
impl Account {
    fn can_moderate_explicit(&self) -> bool {
        match self.role {
            Role::Admin | Role::Moderator => true,
            Role::User => false,
        }
    }
}
```

Adding a variant then forces you to decide its permissions here, otherwise the code does not compile.

### Example 2 — calculator with `Result`

```rust
#[derive(Debug, Clone, Copy)]
enum Operation {
    Addition,
    Subtraction,
    Multiplication,
    Division,
}

impl TryFrom<char> for Operation {
    type Error = String;
    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '+' => Ok(Self::Addition),
            '-' => Ok(Self::Subtraction),
            '*' => Ok(Self::Multiplication),
            '/' => Ok(Self::Division),
            _ => Err(format!("unknown operator: {c}")),
        }
    }
}

fn calculate(a: f64, b: f64, op: Operation) -> Result<f64, String> {
    match op {
        Operation::Addition => Ok(a + b),
        Operation::Subtraction => Ok(a - b),
        Operation::Multiplication => Ok(a * b),
        Operation::Division if b == 0.0 => Err("division by zero".to_string()),
        Operation::Division => Ok(a / b),
    }
}

fn evaluate(a: f64, c: char, b: f64) -> Result<f64, String> {
    let op = Operation::try_from(c)?;
    calculate(a, b, op)
}

fn main() {
    match evaluate(10.0, '/', 0.0) {
        Ok(v) => println!("{v}"),
        Err(e) => println!("error: {e}"),
    }
}
```

## 14. Exercises

### Exercise 1 — `Book` struct

Create a `Book` struct (title, author, pages), a `new` constructor, an `is_long` method (more than 300 pages) and a `Display` output "title, by author".

```rust
// Solution:
use std::fmt;

struct Book {
    title: String,
    author: String,
    pages: u32,
}

impl Book {
    fn new(title: impl Into<String>, author: impl Into<String>, pages: u32) -> Self {
        Self { title: title.into(), author: author.into(), pages }
    }
    fn is_long(&self) -> bool {
        self.pages > 300
    }
}

impl fmt::Display for Book {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}, by {}", self.title, self.author)
    }
}
```

### Exercise 2 — traffic light

Create a `Light` enum (Red, Yellow, Green) with a `next` method and a `duration_s` method (30, 5, 25).

```rust
// Solution:
#[derive(Debug, Clone, Copy, PartialEq)]
enum Light { Red, Yellow, Green }

impl Light {
    fn next(self) -> Self {
        match self {
            Light::Red => Light::Green,
            Light::Green => Light::Yellow,
            Light::Yellow => Light::Red,
        }
    }
    fn duration_s(self) -> u32 {
        match self {
            Light::Red => 30,
            Light::Yellow => 5,
            Light::Green => 25,
        }
    }
}
```

### Exercise 3 — `Option` in three syntaxes

Write `first_initial(name: Option<&str>) -> Option<char>` with `match`, with `?`, then with `and_then`.

```rust
// Solution:
fn v1(name: Option<&str>) -> Option<char> {
    match name {
        Some(n) => n.chars().next(),
        None => None,
    }
}

fn v2(name: Option<&str>) -> Option<char> {
    name?.chars().next()
}

fn v3(name: Option<&str>) -> Option<char> {
    name.and_then(|n| n.chars().next())
}
```

### Exercise 4 — validated newtype

Create `Percentage(u8)` that can only be built with a value from 0 to 100.

```rust
// Solution:
pub struct Percentage(u8);

impl TryFrom<u8> for Percentage {
    type Error = String;
    fn try_from(v: u8) -> Result<Self, Self::Error> {
        if v <= 100 {
            Ok(Percentage(v))
        } else {
            Err(format!("{v} is not a percentage"))
        }
    }
}
```

Outside the module, the private field prevents writing `Percentage(150)`.

## 15. Cheat sheet

### Structs

```rust
struct S { a: i32, b: String }      // named fields
struct T(i32, i32);                 // tuple struct
struct U;                           // unit struct
struct Id(u32);                     // newtype

S { a, b }                          // shorthand
S { a: 1, ..other }                 // update (moves the non-Copy fields)
S { a: 1, ..Default::default() }

impl S {
    fn new() -> Self { .. }         // associated function
    fn read(&self) {}
    fn modify(&mut self) {}
    fn consume(self) {}
    const MAX: i32 = 10;
}
```

### Enums

```rust
enum E { A, B(i32), C { x: i32 } }
#[repr(u8)] enum F { X = 1, Y = 2 }   // F::X as u8
#[non_exhaustive] pub enum G { .. }   // adding variants without a break
enum Tree { Leaf, Node(Box<Tree>, Box<Tree>) }
```

### Option and Result

```rust
opt.unwrap_or(v)   opt.unwrap_or_else(|| v)   opt.unwrap_or_default()
opt.map(f)   opt.and_then(f)   opt.filter(p)   opt.ok_or(e)   opt.zip(other)
opt.as_ref()   opt.as_deref()   opt.take()   opt.is_some_and(p)
res.map(f)   res.map_err(f)   res.and_then(f)   res.ok()   res?
```

### Key points

- A struct groups what goes together, an enum chooses one value among several.
- `..other` moves the non-`Copy` fields.
- A private field + a constructor guarantee an invariant.
- The newtype tells apart values of the same raw type, at no cost.
- A derived `Default` gives zeros: write it by hand when zero makes no sense.
- `Option` replaces `null`, `Result` replaces exceptions.
- Make impossible states impossible to write: an enum rather than booleans.
