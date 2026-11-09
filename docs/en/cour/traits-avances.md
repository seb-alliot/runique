# Advanced traits

**Associated types, operator overloading, supertraits, blanket implementations, sealed traits, async traits**: the tools that let you design safe and expressive APIs, with the different ways of reaching the same result.

## Course objectives

By the end of this course, you will know how to:

- Choose between an associated type and a generic trait parameter
- Overload operators (`+`, `-`, `*`, `[]`, `+=`, `-x`)
- Require another trait with a supertrait
- Write blanket implementations and marker traits
- Seal a trait so that nobody else can implement it
- Encode states in types (typestate) and improve error messages with `#[diagnostic::on_unimplemented]`
- Write `async` methods in a trait and know their limits
- Use the full `<T as Trait>::` syntax to remove an ambiguity

Prerequisites: `traits-basics`, `generics`, `box-dynamiques`.

## Table of contents

1. Associated types
2. Associated type or generic parameter
3. Default trait parameters
4. Operator overloading
5. Supertraits
6. Blanket implementations
7. Marker traits
8. Sealed traits
9. Typestate: states checked at compile time
10. Custom error messages
11. `async` methods in a trait
12. Full syntax and ambiguities
13. Implementing a trait for `&T`, `Box<T>`, `Vec<T>`
14. Generic associated types (GAT)
15. The same API written in several ways
16. Practical exercises
17. Cheat sheet

## 1. Associated types

### 1.1 Definition

```rust
pub trait Iterator {
    type Item;                                   // associated type: chosen by each implementation
    fn next(&mut self) -> Option<Self::Item>;
}
```

```rust
struct Counter { n: u32 }

impl Iterator for Counter {
    type Item = u32;                             // here, Item is u32
    fn next(&mut self) -> Option<u32> {
        self.n += 1;
        (self.n <= 3).then_some(self.n)
    }
}
```

An associated type is a "hole" in the trait, filled **once** by each implementation.

### 1.2 Using it in a bound

```rust
fn total<I>(it: I) -> u32
where
    I: Iterator<Item = u32>,          // bound on the associated type
{
    it.sum()
}

fn print_all<I>(it: I)
where
    I: Iterator,
    I::Item: std::fmt::Debug,         // bound on the associated type itself
{
    for x in it {
        println!("{x:?}");
    }
}
```

### 1.3 An associated type with a bound

```rust
trait Store {
    type Key: std::hash::Hash + Eq + Clone;      // every implementation must choose a hashable key
    type Value;
    type Error: std::error::Error;

    fn read(&self, key: &Self::Key) -> Result<Option<Self::Value>, Self::Error>;
}
```

## 2. Associated type or generic parameter

### 2.1 The same idea written twice

```rust
// 1. Generic parameter: a type can implement the trait SEVERAL times
trait ConvertTo<T> {
    fn convert(&self) -> T;
}

struct Meters(f64);
impl ConvertTo<f64> for Meters { fn convert(&self) -> f64 { self.0 } }
impl ConvertTo<String> for Meters { fn convert(&self) -> String { format!("{} m", self.0) } }

// 2. Associated type: ONE single implementation per type
trait Container {
    type Element;
    fn first(&self) -> Option<&Self::Element>;
}

struct Stack(Vec<i32>);
impl Container for Stack {
    type Element = i32;
    fn first(&self) -> Option<&i32> { self.0.first() }
}
```

### 2.2 Consequences for the caller

```rust
let m = Meters(3.0);
// let x = m.convert();                 // ERROR: to which type?
let x: f64 = m.convert();               // the caller must specify
let y = ConvertTo::<String>::convert(&m);

let p = Stack(vec![1]);
let z = p.first();                       // no ambiguity: only one possible Element
```

| | Generic parameter `Trait<T>` | Associated type `type T` |
|---|---|---|
| Implementations per type | several | only one |
| The caller specifies the type | often | never |
| Standard examples | `From<T>`, `Add<Rhs>`, `PartialEq<Rhs>` | `Iterator::Item`, `Deref::Target`, `FromStr::Err` |
| Question to ask | "can a type have several answers?" | "is the answer determined by the type?" |

`String` can be built from `&str`, `char`, `Box<str>`...: `From<T>` is generic. An iterator only produces **one** kind of element: `Item` is associated.

## 3. Default trait parameters

```rust
pub trait Add<Rhs = Self> {        // by default, we add two values of the same type
    type Output;
    fn add(self, rhs: Rhs) -> Self::Output;
}
```

The default `Rhs = Self` keeps the common case simple (`impl Add for Point`) while still allowing combinations (`impl Add<Duration> for Instant`).

## 4. Operator overloading

Each operator corresponds to a trait in `std::ops`.

| Operator | Trait | Operator | Trait |
|---|---|---|---|
| `a + b` | `Add` | `a += b` | `AddAssign` |
| `a - b` | `Sub` | `a -= b` | `SubAssign` |
| `a * b` | `Mul` | `a *= b` | `MulAssign` |
| `a / b` | `Div` | `a % b` | `Rem` |
| `-a` | `Neg` | `!a` | `Not` |
| `a[i]` | `Index` | `a[i] = v` | `IndexMut` |
| `a & b`, `a ^ b` (and bitwise or) | `BitAnd`, `BitXor`, `BitOr` | `a << b` | `Shl` |
| `a == b` | `PartialEq` | `a < b` | `PartialOrd` |

### 4.1 Adding two values of the same type

```rust
use std::ops::{Add, AddAssign, Neg, Sub};

#[derive(Debug, Clone, Copy, PartialEq)]
struct Vector { x: f64, y: f64 }

impl Add for Vector {
    type Output = Vector;
    fn add(self, o: Vector) -> Vector { Vector { x: self.x + o.x, y: self.y + o.y } }
}

impl Sub for Vector {
    type Output = Vector;
    fn sub(self, o: Vector) -> Vector { Vector { x: self.x - o.x, y: self.y - o.y } }
}

impl Neg for Vector {
    type Output = Vector;
    fn neg(self) -> Vector { Vector { x: -self.x, y: -self.y } }
}

impl AddAssign for Vector {
    fn add_assign(&mut self, o: Vector) { self.x += o.x; self.y += o.y; }
}

let a = Vector { x: 1.0, y: 2.0 };
let b = Vector { x: 3.0, y: 4.0 };
let mut c = a + b - Vector { x: 1.0, y: 1.0 };
c += a;
let d = -c;
```

### 4.2 Combining two different types

```rust
use std::ops::Mul;

impl Mul<f64> for Vector {                   // Vector * f64
    type Output = Vector;
    fn mul(self, k: f64) -> Vector { Vector { x: self.x * k, y: self.y * k } }
}

impl Mul<Vector> for f64 {                   // f64 * Vector: allowed, Vector is local
    type Output = Vector;
    fn mul(self, v: Vector) -> Vector { v * self }
}

let e = a * 2.0;
let f = 2.0 * a;
```

### 4.3 Operators on references

```rust
use std::ops::Add;

#[derive(Debug, Clone)]
struct Big(Vec<u64>);                            // not Copy: a + b would consume a and b

impl Add<&Big> for &Big {
    type Output = Big;
    fn add(self, o: &Big) -> Big {
        Big(self.0.iter().zip(&o.0).map(|(x, y)| x + y).collect())
    }
}

let a = Big(vec![1, 2]);
let b = Big(vec![3, 4]);
let c = &a + &b;                                 // a and b remain usable
```

### 4.4 Indexing

```rust
use std::ops::{Index, IndexMut};

struct Grid { width: usize, cells: Vec<char> }

impl Index<(usize, usize)> for Grid {
    type Output = char;
    fn index(&self, (x, y): (usize, usize)) -> &char {
        &self.cells[y * self.width + x]
    }
}

impl IndexMut<(usize, usize)> for Grid {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut char {
        &mut self.cells[y * self.width + x]
    }
}

let mut g = Grid { width: 3, cells: vec!['.'; 9] };
g[(1, 1)] = '#';
println!("{}", g[(1, 1)]);
```

### 4.5 When not to overload

An operator must do what everyone expects (`+` adds, with no side effect and no possible error). For an operation that can fail or that has a business name, a named method is clearer (`account.debit(amount)` rather than `account - amount`).

## 5. Supertraits

### 5.1 Requiring another trait

```rust
use std::fmt::Display;

trait Label: Display {                           // every Label type must also be Display
    fn label(&self) -> String {
        format!("[{}]", self)                    // allowed: Display is guaranteed
    }
}

struct Code(u32);
impl Display for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "C{}", self.0) }
}
impl Label for Code {}

// struct Raw;
// impl Label for Raw {}                         // ERROR: Raw does not implement Display
```

### 5.2 Several supertraits

```rust
trait Entity: Clone + std::fmt::Debug + Send + Sync + 'static {
    fn id(&self) -> u64;
}
```

A supertrait is not inheritance: `Entity` inherits no code, it **requires** its implementors to also have `Clone`, `Debug`, etc. The default methods of `Entity` can then use them.

### 5.3 Supertrait or bound on the function

```rust
// 1. Supertrait: the bound applies everywhere the trait is used
trait Showable1: Display { fn title(&self) -> String; }

// 2. Local bound: only this function requires Display
trait Showable2 { fn title(&self) -> String; }
fn show<T: Showable2 + Display>(x: &T) { println!("{x} {}", x.title()); }
```

Use a supertrait when the trait has **no meaning** without the other (an error without `Display`, a shared trait object without `Send + Sync`). Otherwise, a local bound leaves more freedom.

## 6. Blanket implementations

```rust
trait Shout {
    fn shout(&self) -> String;
}

impl<T: std::fmt::Display> Shout for T {          // for ALL displayable types
    fn shout(&self) -> String {
        self.to_string().to_uppercase()
    }
}

println!("{}", "hello".shout());
println!("{}", 42.shout());
```

The standard library contains many of them:

```rust
// impl<T: Display + ?Sized> ToString for T          // .to_string() on any Display
// impl<T, U: From<T>> Into<U> for T                  // Into for free for any From
// impl<T: ?Sized> Borrow<T> for T
// impl<I: Iterator> IntoIterator for I               // an iterator is usable in for
```

Limit: a blanket implementation **occupies** all the types concerned. After `impl<T: Display> Shout for T`, you can no longer write `impl Shout for i32` (conflict, error E0119). Only the crate that defines the trait can write a blanket implementation.

## 7. Marker traits

A trait with no method, used only as a **label** checked at compile time.

```rust
trait Valid {}                                    // "this value has been checked"

struct Raw(String);
struct Checked(String);
impl Valid for Checked {}

fn save<T: Valid>(_v: T) {}

// save(Raw("x".into()));                         // ERROR: Raw is not Valid
save(Checked("x".into()));
```

Standard examples: `Copy`, `Eq`, `Send`, `Sync`, `Sized`, `Unpin`. `Eq` has no method: it is a promise (equality is total).

## 8. Sealed traits

### 8.1 The problem

A public trait can be implemented by **any** crate. If your library relies on a closed list of implementations (for a safety guarantee, or to be able to add a method without breaking anyone), you must prevent that.

### 8.2 The technique

```rust
mod private {
    pub trait Seal {}                             // public, but in a private module: unreachable
}

pub trait Format: private::Seal {                 // the supertrait cannot be implemented elsewhere
    fn extension(&self) -> &'static str;
}

pub struct Json;
pub struct Csv;

impl private::Seal for Json {}
impl private::Seal for Csv {}
impl Format for Json { fn extension(&self) -> &'static str { "json" } }
impl Format for Csv { fn extension(&self) -> &'static str { "csv" } }
```

Outside the crate, you can **use** `Format` (`fn f(x: impl Format)`), but not **implement** it: `private::Seal` cannot be named.

| Advantage | For |
|---|---|
| adding a required method is no longer a breaking change | the evolution of a library |
| the list of implementations is known | a guarantee (safety, invariant) |

## 9. Typestate: states checked at compile time

The state of an object is encoded in its **type**: a method only exists in the state where it makes sense.

### 9.1 The same rule written twice

```rust
// 1. Run-time check: a boolean and a possible error
struct Connection1 { authenticated: bool }
impl Connection1 {
    fn send(&self, _msg: &str) -> Result<(), String> {
        if !self.authenticated {
            return Err("not authenticated".into());
        }
        Ok(())
    }
}

// 2. Compile-time check: two state types
use std::marker::PhantomData;

struct Anonymous;
struct Authenticated;

struct Connection<State> { _state: PhantomData<State> }

impl Connection<Anonymous> {
    fn new() -> Self { Connection { _state: PhantomData } }
    fn login(self, _password: &str) -> Connection<Authenticated> { Connection { _state: PhantomData } }
}

impl Connection<Authenticated> {
    fn send(&self, _msg: &str) {}                 // only exists once logged in
}

fn main() {
    let c = Connection::new();
    // c.send("x");                               // ERROR: no method send for Connection<Anonymous>
    let c = c.login("secret");
    c.send("x");
}
```

### 9.2 In Runique: one-time builder settings

Runique's builder prevents calling twice a setting that only makes sense once (`routes()`, `with_log()`, `with_public_url()`...). Each setting has a type parameter that is `No` and then `Yes`:

```rust
// runique/src/app/builder/mod.rs (excerpt)
pub mod state {
    pub struct No;
    pub struct Yes;
    pub type Fresh = (No, No, No, No, No, No, No);

    mod sealed {
        pub trait Sealed {}
        impl Sealed for super::No {}
    }

    // For each setting, a sealed trait implemented only by No
    // pub trait RoutesUnset: sealed::Sealed {}
    // impl RoutesUnset for No {}
}
```

The `routes()` method is only available if the matching parameter implements `RoutesUnset`, that is, if it is `No`. After the call, it is `Yes`, and a second `routes()` no longer compiles. The trait is **sealed** (section 8): no outside code can implement it for `Yes` and get around the rule.

## 10. Custom error messages

A typestate or a marker trait produces generic errors ("the trait bound `Yes: RoutesUnset` is not satisfied"). The `#[diagnostic::on_unimplemented]` attribute (Rust 1.78+) replaces them with a message of your choice.

```rust
#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot be saved: it has not been validated",
    label = "value not validated",
    note = "first pass the value to `validate()`"
)]
trait Valid {}
```

Runique uses it for its builder:

```rust
// runique/src/app/builder/mod.rs (excerpt, generated by a macro)
#[diagnostic::on_unimplemented(
    message = "`routes()` has already been called on this builder",
    label = "a second `routes()` would replace every route of the first",
    note = "build a single `Router` (merge or nest the others into it) and pass it once"
)]
pub trait RoutesUnset: sealed::Sealed {}
```

The developer who calls `routes()` twice reads the cause and the fix directly, instead of an incomprehensible trait error.

## 11. `async` methods in a trait

### 11.1 Since Rust 1.75

```rust
trait Storage {
    async fn read(&self, key: &str) -> Option<String>;
}

struct Memory(std::collections::HashMap<String, String>);

impl Storage for Memory {
    async fn read(&self, key: &str) -> Option<String> {
        self.0.get(key).cloned()
    }
}
```

### 11.2 The limit: `Send` and `dyn`

```rust
async fn use_storage<S: Storage>(s: &S) {
    let _ = s.read("a").await;
    // tokio::spawn(async move { s.read("a").await });   // is the returned future Send? unknown
}
```

The future returned by `read` has no `Send` bound: it cannot be passed to `tokio::spawn` in generic code, and the trait cannot be used as `dyn Storage`.

### 11.3 Three ways to write an async trait

```rust
use std::future::Future;
use std::pin::Pin;

// 1. Native async fn: simple, no dyn, no Send guarantee
trait Storage1 {
    async fn read(&self, key: &str) -> Option<String>;
}

// 2. Return impl Future + Send: guarantees Send, still no dyn
trait Storage2 {
    fn read(&self, key: &str) -> impl Future<Output = Option<String>> + Send;
}

// 3. Boxed future: compatible with dyn and Send, one allocation per call
trait Storage3: Send + Sync {
    fn read<'a>(&'a self, key: &'a str) -> Pin<Box<dyn Future<Output = Option<String>> + Send + 'a>>;
}
```

The `async-trait` crate generates form 3 from an `async fn` notation:

```rust
#[async_trait::async_trait]
trait Storage4: Send + Sync {
    async fn read(&self, key: &str) -> Option<String>;
}
```

| Form | `Send` guaranteed | `dyn` possible | Allocation |
|---|---|---|---|
| native `async fn` | no | no | no |
| `-> impl Future + Send` | yes | no | no |
| `Pin<Box<dyn Future>>` / `async-trait` | yes | yes | one per call |

Runique uses `async-trait` for `FormField` (`forms/base.rs`): the fields of a form are `Box<dyn FormField>` (see `box-dynamiques`) and some of their methods are asynchronous, which requires form 3. In `db/adb.rs`, it is used to implement `ConnectionTrait` and `TransactionTrait`, SeaORM traits that are themselves declared with `async-trait`.

## 12. Full syntax and ambiguities

### 12.1 Associated function with the same name in two traits

```rust
trait Animal { fn name() -> String; }

struct Dog;
impl Dog { fn name() -> String { "Rex".into() } }
impl Animal for Dog { fn name() -> String { "dog".into() } }

fn main() {
    Dog::name();                      // "Rex": the type's own function
    // Animal::name();                // ERROR: for which type?
    <Dog as Animal>::name();          // "dog"
}
```

### 12.2 In a generic

```rust
fn description<T: Animal>() -> String {
    <T as Animal>::name()             // or simply T::name() if there is no ambiguity
}
```

### 12.3 Associated type of a specific trait

```rust
fn first<I: Iterator>(mut it: I) -> Option<<I as Iterator>::Item> {
    it.next()
}
```

`<I as Iterator>::Item` is usually written `I::Item`; the full form is for when `I` implements two traits that each have an `Item` type.

## 13. Implementing a trait for `&T`, `Box<T>`, `Vec<T>`

### 13.1 Forwarding the implementation

```rust
trait Shape { fn area(&self) -> f64; }

impl<T: Shape + ?Sized> Shape for &T {
    fn area(&self) -> f64 { (**self).area() }
}

impl<T: Shape + ?Sized> Shape for Box<T> {
    fn area(&self) -> f64 { (**self).area() }
}

fn total(shapes: &[impl Shape]) -> f64 {
    shapes.iter().map(|s| s.area()).sum()
}
```

With these two implementations, `total` accepts a `Vec<Square>`, a `Vec<&Square>` and a `Vec<Box<dyn Shape>>`.

### 13.2 For a collection

```rust
impl<T: Shape> Shape for Vec<T> {
    fn area(&self) -> f64 { self.iter().map(|s| s.area()).sum() }   // a list of shapes is a shape
}
```

Possible because `Shape` is local (orphan rule, see `traits-basics`).

## 14. Generic associated types (GAT)

Since Rust 1.65, an associated type can have its own parameters, notably a lifetime.

```rust
trait Lender {
    type View<'a> where Self: 'a;                  // the type depends on the lifetime of the borrow
    fn view<'a>(&'a self) -> Self::View<'a>;
}

struct Text(String);

impl Lender for Text {
    type View<'a> = &'a str;
    fn view<'a>(&'a self) -> &'a str { &self.0 }
}
```

What it is for: traits that return views borrowed from `self` (iterators that lend their elements, generic collections). It is a library tool; in application code you rarely need it.

## 15. The same API written in several ways

A task queue that accepts several kinds of tasks.

```rust
// 1. Enum: the kinds of tasks are known
enum Task { Email(String), Cleanup }
fn run1(t: &Task) {
    match t {
        Task::Email(a) => println!("email to {a}"),
        Task::Cleanup => println!("cleanup"),
    }
}

// 2. Trait + generic: each queue holds a single kind
trait Runnable { fn run(&self); }
struct Queue<T: Runnable> { tasks: Vec<T> }
impl<T: Runnable> Queue<T> {
    fn all(&self) { self.tasks.iter().for_each(|t| t.run()); }
}

// 3. Trait object: mixed kinds, extensible by the user
struct DynQueue { tasks: Vec<Box<dyn Runnable + Send>> }

// 4. Sealed trait + trait object: mixed kinds, CLOSED list guaranteed
mod sealed { pub trait Seal {} }
pub trait InternalTask: sealed::Seal + Send { fn run(&self); }
```

| Version | Mixed types | Extensible outside the crate | Exhaustiveness checked |
|---|---|---|---|
| enum | yes | no | yes (`match`) |
| generic | no | yes | — |
| trait object | yes | yes | — |
| sealed trait | yes | no | — |

## 16. Practical exercises

### Exercise 1: a shape with an associated type

Create a `Shape` trait with an associated type `Measure` (the type of the area: `f64` for real shapes, `u64` for a pixel grid) and two implementations.

```rust
// Solution:
trait Shape {
    type Measure;
    fn area(&self) -> Self::Measure;
}

struct Circle(f64);
struct PixelRectangle { w: u64, h: u64 }

impl Shape for Circle {
    type Measure = f64;
    fn area(&self) -> f64 { std::f64::consts::PI * self.0 * self.0 }
}

impl Shape for PixelRectangle {
    type Measure = u64;
    fn area(&self) -> u64 { self.w * self.h }
}
```

### Exercise 2: operators on an amount

Write `Amount(i64)` (in cents) with `+`, `-`, `+=`, `Amount * i64`, and a `Display` showing "€12.50".

```rust
// Solution:
use std::fmt;
use std::ops::{Add, AddAssign, Mul, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Amount(i64);

impl Add for Amount { type Output = Amount; fn add(self, o: Amount) -> Amount { Amount(self.0 + o.0) } }
impl Sub for Amount { type Output = Amount; fn sub(self, o: Amount) -> Amount { Amount(self.0 - o.0) } }
impl AddAssign for Amount { fn add_assign(&mut self, o: Amount) { self.0 += o.0; } }
impl Mul<i64> for Amount { type Output = Amount; fn mul(self, k: i64) -> Amount { Amount(self.0 * k) } }

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sign = if self.0 < 0 { "-" } else { "" };
        let a = self.0.abs();
        write!(f, "{sign}€{}.{:02}", a / 100, a % 100)
    }
}
```

### Exercise 3: typestate

Write a `Form<State>` with two states `Unvalidated` and `Validated`: `validate(self) -> Result<Form<Validated>, String>` only exists for `Unvalidated`, `save(&self)` only for `Validated`.

```rust
// Solution:
use std::marker::PhantomData;

struct Unvalidated;
struct Validated;

struct Form<S> { email: String, _s: PhantomData<S> }

impl Form<Unvalidated> {
    fn new(email: &str) -> Self { Self { email: email.into(), _s: PhantomData } }
    fn validate(self) -> Result<Form<Validated>, String> {
        if self.email.contains('@') {
            Ok(Form { email: self.email, _s: PhantomData })
        } else {
            Err("invalid email".into())
        }
    }
}

impl Form<Validated> {
    fn save(&self) { println!("saved: {}", self.email); }
}
```

## 17. Cheat sheet

```rust
trait T { type A; fn f(&self) -> Self::A; }          // associated type
fn g<I: Iterator<Item = u32>>(i: I)                    // bound on associated type
trait Add<Rhs = Self> { type Output; .. }              // default parameter
impl Add for X { type Output = X; fn add(self, o: X) -> X { .. } }
impl Index<usize> for X { type Output = Y; fn index(&self, i: usize) -> &Y { .. } }
trait A: Display + Send {}                             // supertraits
impl<T: Display> MyTrait for T {}                      // blanket implementation
trait Marker {}                                        // marker trait
mod private { pub trait Seal {} } pub trait S: private::Seal {}   // sealed trait
struct X<State> { _s: PhantomData<State> }             // typestate
#[diagnostic::on_unimplemented(message = "..")]
trait A { async fn f(&self); }                         // native async
fn f(&self) -> impl Future<Output = T> + Send;        // async + Send
<Type as Trait>::function()                            // full syntax
type View<'a> where Self: 'a;                          // GAT
```

### Key points

- Associated type if the type is determined by the implementation, generic parameter if several implementations make sense.
- Operators are traits in `std::ops`; only overload what behaves like the expected operator.
- A supertrait requires another trait; it is not code inheritance.
- A blanket implementation occupies all the types concerned.
- A sealed trait can be used but not implemented outside its crate.
- Typestate turns a run-time error into a compile error; `on_unimplemented` makes the message readable.
- `async fn` in a trait: native without `Send` or `dyn`; `impl Future + Send` for `Send`; `async-trait` for `dyn`.
