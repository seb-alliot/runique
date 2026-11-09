# Traits — The basics
> A trait describes a behavior that several types can share. This course shows how to define them, implement them, use them as parameters, and implement the standard library traits.

## Objectives

- Define a trait with methods, associated functions and constants
- Implement it for your own types, and for types from other crates
- Write default methods that build on the others
- Use a trait as a parameter and as a return type, in every way
- Implement by hand the most useful standard traits (`Display`, `From`, `FromStr`, `Default`, `PartialEq`, `Ord`)
- Understand the coherence rule (orphan rule) and extension traits
- Resolve an ambiguity when two traits have a method with the same name

## Table of contents

1. What is a trait
2. Defining a trait
3. Implementing a trait
4. Default methods
5. Using a trait: parameters and return values
6. The trait must be imported
7. Deriving or implementing by hand
8. The standard traits to know
9. Implementing `Display`
10. Conversions: `From`, `Into`, `TryFrom`, `FromStr`
11. Comparisons: `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`
12. Several traits on the same type
13. The coherence rule and extension traits
14. The same feature written in several ways
15. Practical exercises
16. Cheat sheet

---

## 1. What is a trait

A **trait** is a set of methods that a type promises to provide. It is the equivalent of interfaces in Java or TypeScript, with two differences:

- you can implement a trait for a type **after the fact**, without changing its definition (even `i32` or `String`);
- a trait can provide default code.

```rust
trait Describe {
    fn describe(&self) -> String;
}

struct Dog { name: String }
struct Car { model: String }

impl Describe for Dog {
    fn describe(&self) -> String { format!("the dog {}", self.name) }
}

impl Describe for Car {
    fn describe(&self) -> String { format!("a {}", self.model) }
}

fn introduce(x: &impl Describe) {
    println!("Here is {}", x.describe());
}
```

`Dog` and `Car` have nothing in common, except this behavior. `introduce` accepts any type that has it.

---

## 2. Defining a trait

```rust
trait Shape {
    // Method to implement: only the signature
    fn area(&self) -> f64;

    // Method that modifies
    fn scale(&mut self, factor: f64);

    // Associated function (no self): a common constructor
    fn unit() -> Self where Self: Sized;

    // Associated constant
    const SIDES: u32;

    // Default method (section 4)
    fn description(&self) -> String {
        format!("shape with {} sides, area {:.2}", Self::SIDES, self.area())
    }
}
```

| Element | Syntax | Call |
|---|---|---|
| method | `fn f(&self)` | `value.f()` |
| method that modifies | `fn f(&mut self)` | `value.f()` |
| method that consumes | `fn f(self)` | `value.f()` |
| associated function | `fn f() -> Self` | `Type::f()` or `T::f()` |
| constant | `const N: u32;` | `Type::N` or `T::N` |

---

## 3. Implementing a trait

```rust
struct Square { side: f64 }

impl Shape for Square {
    fn area(&self) -> f64 { self.side * self.side }
    fn scale(&mut self, factor: f64) { self.side *= factor; }
    fn unit() -> Self { Square { side: 1.0 } }
    const SIDES: u32 = 4;
    // description is not written: the default version applies
}

fn main() {
    let mut c = Square::unit();
    c.scale(3.0);
    println!("{}", c.description());   // shape with 4 sides, area 9.00
}
```

All methods **without** a body must be written, with exactly the same signature. The compiler lists the missing ones (`missing ... in implementation`).

### The `impl Trait for Type` block is separate from the `impl Type` block

```rust
impl Square {
    fn diagonal(&self) -> f64 { self.side * 2f64.sqrt() }   // method specific to Square
}

impl Shape for Square {
    // only the items declared by the trait: area, scale, unit, SIDES
    // (and possibly description, to replace the default version)
}
```

You cannot add an extra method inside an `impl Trait for Type`: it goes in `impl Type`.

---

## 4. Default methods

### 4.1 A default method can call the others

```rust
trait Summary {
    fn author(&self) -> String;

    fn summary(&self) -> String {
        format!("(Read more from {}…)", self.author())
    }
}

struct Tweet { user: String, text: String }

impl Summary for Tweet {
    fn author(&self) -> String { format!("@{}", self.user) }
    // summary is provided
}
```

The trait asks for the **minimum** (`author`) and builds the rest. This is the principle of `Iterator`: you write `next`, you get `map`, `filter`, `sum` and 70 other methods.

### 4.2 Replacing a default method

```rust
struct Article { title: String, author: String }

impl Summary for Article {
    fn author(&self) -> String { self.author.clone() }
    fn summary(&self) -> String {                      // replaces the default version
        format!("{}, by {}", self.title, self.author)
    }
}
```

An implementation that replaces the default method **cannot** call the default version (there is no `super`). To reuse it, put it in a free function that both call.

### 4.3 Designing a trait

Ask for as few required methods as possible, and provide by default everything that can be deduced from them. Adding a method **with** a default later breaks nobody; adding a **required** method breaks all existing implementations.

---

## 5. Using a trait: parameters and return values

### 5.1 As a parameter: the same function in four ways

```rust
// 1. impl Trait
fn notify_v1(x: &impl Summary) {
    println!("{}", x.summary());
}

// 2. Generic with a bound
fn notify_v2<T: Summary>(x: &T) {
    println!("{}", x.summary());
}

// 3. where
fn notify_v3<T>(x: &T)
where
    T: Summary,
{
    println!("{}", x.summary());
}

// 4. Trait object: a single compiled copy, choice at runtime
fn notify_v4(x: &dyn Summary) {
    println!("{}", x.summary());
}
```

The first three are equivalent (generics, see `generics`). The fourth is dynamic (see `box-dynamiques`).

### 5.2 Several bounds

```rust
use std::fmt::Display;

fn show_summary(x: &(impl Summary + Display)) {
    println!("{x}: {}", x.summary());
}

fn show_summary2<T: Summary + Display>(x: &T) {
    println!("{x}: {}", x.summary());
}
```

### 5.3 As a return value

```rust
fn create() -> impl Summary {
    Tweet { user: "rust".into(), text: "1.0".into() }
}
```

`impl Summary` as a return type hides the concrete type, but it must be **unique**:

```rust
// fn create(short: bool) -> impl Summary {
//     if short { Tweet { .. } } else { Article { .. } }    // ERROR: two different types
// }

fn create_dyn(short: bool) -> Box<dyn Summary> {
    if short {
        Box::new(Tweet { user: "a".into(), text: "b".into() })
    } else {
        Box::new(Article { title: "t".into(), author: "a".into() })
    }
}
```

### 5.4 Calling an associated function on a generic

```rust
trait Zero {
    fn zero() -> Self;
}

impl Zero for u32 { fn zero() -> Self { 0 } }

fn fill<T: Zero>(n: usize) -> Vec<T> {
    (0..n).map(|_| T::zero()).collect()        // T::f(): associated function of the generic type
}

let v: Vec<u32> = fill(3);
```

---

## 6. The trait must be imported

```rust
mod shapes {
    pub trait Area { fn area(&self) -> f64; }
    impl Area for f64 { fn area(&self) -> f64 { self * self } }
}

fn main() {
    let x = 2.0_f64;
    // x.area();                 // ERROR: no method named `area` found
    use shapes::Area;            // the trait must be in scope
    x.area();
}
```

The methods of a trait are visible **only if the trait is imported**. This is intended: two crates can add an `area` method to `f64` without conflict, as long as one file does not import both. Preludes (`use runique::prelude::*`, `use std::io::prelude::*`) import the common traits at once.

---

## 7. Deriving or implementing by hand

```rust
#[derive(Debug, Clone, PartialEq, Default)]
struct Point { x: i32, y: i32 }
```

`#[derive(...)]` generates the most obvious implementation, field by field. You write it by hand when the behavior you want is different:

| Trait | Derived | By hand when... |
|---|---|---|
| `Debug` | all fields | a field is secret (password, token) |
| `Clone` | clones each field | rarely |
| `PartialEq` | compares all fields | equality is based on an identifier only |
| `Default` | `Default` of each field | zero makes no sense (port, limit, delay) |
| `PartialOrd`/`Ord` | field order | the order you want is not the field order |
| `Hash` | all fields | must stay consistent with a hand-written `PartialEq` |

### Example: `Debug` without the secret

```rust
use std::fmt;

struct Credentials { user: String, password: String }

impl fmt::Debug for Credentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Credentials")
            .field("user", &self.user)
            .field("password", &"***")
            .finish()
    }
}
```

A `#[derive(Debug)]` would print the password in the logs at the first `{:?}` or the first `dbg!`.

---

## 8. The standard traits to know

| Trait | Used for | Derivable |
|---|---|---|
| `Debug` | `{:?}`, `dbg!`, `assert_eq!` | yes |
| `Display` | `{}`, `.to_string()` | no |
| `Clone`, `Copy` | copying | yes |
| `Default` | default value | yes |
| `PartialEq`, `Eq` | `==` | yes |
| `PartialOrd`, `Ord` | `<`, sorting | yes |
| `Hash` | `HashMap` key | yes |
| `From`, `Into` | conversion that cannot fail | no |
| `TryFrom`, `TryInto` | conversion that can fail | no |
| `FromStr` | `"..".parse::<T>()` | no |
| `AsRef`, `Borrow` | borrowing in another form | no |
| `Iterator`, `IntoIterator` | iterating, `for` | no |
| `Deref`, `Drop` | smart pointers | no |
| `Add`, `Sub`, `Mul`, `Index`... | operators (see `traits-avances`) | no |
| `Error` | error type | no (`thiserror`) |
| `Send`, `Sync` | threads (automatic) | — |
| `Serialize`, `Deserialize` (serde) | serialization | yes, with serde |

---

## 9. Implementing `Display`

```rust
use std::fmt;

struct Price { cents: i64 }

impl fmt::Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{:02} €", self.cents / 100, self.cents % 100)
    }
}

let p = Price { cents: 1250 };
println!("{p}");                 // 12.50 €
let text = p.to_string();        // provided by Display
```

### Respecting the formatting options

```rust
struct Code(&'static str);

impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(self.0)               // pad applies the requested width and alignment
    }
}

println!("[{:>8}]", Code("abc"));  // [     abc]
```

`write!(f, ...)` ignores `{:>8}`; `f.pad(...)` respects it.

---

## 10. Conversions: `From`, `Into`, `TryFrom`, `FromStr`

### 10.1 `From`: you write one, you get the other

```rust
struct Celsius(f64);
struct Fahrenheit(f64);

impl From<Celsius> for Fahrenheit {
    fn from(c: Celsius) -> Self { Fahrenheit(c.0 * 9.0 / 5.0 + 32.0) }
}

let f = Fahrenheit::from(Celsius(100.0));
let f: Fahrenheit = Celsius(100.0).into();     // Into comes for free
```

You **always implement `From`**, never `Into`: the standard library provides `Into` automatically for every `From`.

### 10.2 `TryFrom`: a conversion that can fail

```rust
struct Age(u8);

impl TryFrom<i32> for Age {
    type Error = String;
    fn try_from(v: i32) -> Result<Self, Self::Error> {
        if (0..=150).contains(&v) {
            Ok(Age(v as u8))
        } else {
            Err(format!("impossible age: {v}"))
        }
    }
}

let a = Age::try_from(30);
let b: Result<Age, _> = 200_i32.try_into();   // the suffix says which conversion to use
```

### 10.3 `FromStr`: for `parse`

```rust
use std::str::FromStr;

#[derive(Debug)]
enum Color { Red, Green, Blue }

impl FromStr for Color {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "red" => Ok(Color::Red),
            "green" => Ok(Color::Green),
            "blue" => Ok(Color::Blue),
            other => Err(format!("unknown color: {other}")),
        }
    }
}

let c: Color = "Red".parse().unwrap();
let d = "purple".parse::<Color>();            // Err
```

`FromStr` (text → value) and `Display` (value → text) form a pair: a type that has both can be written out and read back.

---

## 11. Comparisons: `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`

### 11.1 Equality on an identifier

```rust
use std::hash::{Hash, Hasher};

struct User { id: u32, name: String }

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool { self.id == other.id }
}
impl Eq for User {}

impl Hash for User {
    fn hash<H: Hasher>(&self, h: &mut H) { self.id.hash(h); }   // consistent with eq
}
```

Mandatory rule: if `a == b`, then `hash(a) == hash(b)`. A `Hash` that included `name` while `eq` does not compare it would silently break `HashMap`s.

### 11.2 `PartialEq` or `Eq`, `PartialOrd` or `Ord`

- `PartialEq`: `==` exists, but some values may not equal themselves (`f64::NAN != f64::NAN`).
- `Eq`: equality is total (`a == a` always). No method to write: it is a promise.
- `PartialOrd`: some pairs are not comparable (`NaN`).
- `Ord`: everything is comparable; required by `sort()`, `BTreeMap`, `max()`.

### 11.3 A chosen order

```rust
use std::cmp::Ordering;

#[derive(PartialEq, Eq)]
struct Task { priority: u8, title: String }

impl Ord for Task {
    fn cmp(&self, other: &Self) -> Ordering {
        other.priority.cmp(&self.priority)                // priority descending
            .then_with(|| self.title.cmp(&other.title))    // then title ascending
    }
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}
```

`PartialOrd` delegates to `Ord` to stay consistent: this is the standard form.

Often, a one-off sort is enough without implementing `Ord`: `v.sort_by_key(|t| std::cmp::Reverse(t.priority))`.

---

## 12. Several traits on the same type

### 12.1 One type, several behaviors

```rust
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
struct Temperature(f64);

impl fmt::Display for Temperature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{:.1} °C", self.0) }
}

impl From<f64> for Temperature {
    fn from(v: f64) -> Self { Temperature(v) }
}

impl Default for Temperature {
    fn default() -> Self { Temperature(20.0) }
}
```

### 12.2 Two traits with a method of the same name

```rust
trait Pilot { fn fly(&self) -> &'static str; }
trait Wizard { fn fly(&self) -> &'static str; }

struct Human;

impl Pilot for Human { fn fly(&self) -> &'static str { "takeoff" } }
impl Wizard for Human { fn fly(&self) -> &'static str { "broom" } }
impl Human { fn fly(&self) -> &'static str { "flapping arms" } }

fn main() {
    let h = Human;
    h.fly();                         // the type's own method wins: "flapping arms"
    Pilot::fly(&h);                  // "takeoff"
    Wizard::fly(&h);                 // "broom"
    <Human as Pilot>::fly(&h);       // full form
}
```

For an associated function without `self`, only the full form works: `<Human as Pilot>::name()`.

---

## 13. The coherence rule and extension traits

### 13.1 The orphan rule

You can write `impl Trait for Type` only if **the trait or the type** is defined in your crate.

```rust
use std::fmt;

struct MyType;
trait MyTrait {}

impl fmt::Display for MyType {                           // OK: the type is local
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "my type")
    }
}

impl MyTrait for Vec<i32> {}                             // OK: the trait is local

// impl fmt::Display for Vec<i32> { .. }                 // ERROR E0117: neither one
```

Why: if two crates could each implement `Display` for `Vec<i32>`, the compiler would not know which one to choose in a program that uses both.

### 13.2 Working around it: the newtype

```rust
use std::fmt;

struct List(Vec<i32>);

impl fmt::Display for List {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let texts: Vec<String> = self.0.iter().map(|x| x.to_string()).collect();
        write!(f, "[{}]", texts.join(" | "))
    }
}
```

### 13.3 Adding methods to a foreign type: the extension trait

```rust
trait TextExt {
    fn is_blank(&self) -> bool;
    fn truncate_chars(&self, n: usize) -> String;
}

impl TextExt for str {
    fn is_blank(&self) -> bool {
        self.trim().is_empty()
    }
    fn truncate_chars(&self, n: usize) -> String {
        self.chars().take(n).collect()
    }
}

fn main() {
    "   ".is_blank();                     // true
    "hello".truncate_chars(3);            // "hel"
    String::from("abc").truncate_chars(1); // also works on String (Deref to str)
}
```

The trait is local: the orphan rule is respected. Naming convention: `TypeNameExt`. `itertools::Itertools` adds dozens of methods to all iterators this way.

---

## 14. The same feature written in several ways

Computing the price including tax for several kinds of items.

```rust
// 1. One function per type: duplication
fn gross_book(price: f64) -> f64 { price * 1.055 }
fn gross_game(price: f64) -> f64 { price * 1.20 }

// 2. An enum: closed set, one match
enum Item { Book(f64), Game(f64) }
impl Item {
    fn gross(&self) -> f64 {
        match self {
            Item::Book(p) => p * 1.055,
            Item::Game(p) => p * 1.20,
        }
    }
}

// 3. A trait with an associated constant and a default method: open set
trait Taxable {
    const RATE: f64;
    fn net_price(&self) -> f64;
    fn gross(&self) -> f64 { self.net_price() * (1.0 + Self::RATE) }
}

struct Book(f64);
struct Game(f64);
impl Taxable for Book { const RATE: f64 = 0.055; fn net_price(&self) -> f64 { self.0 } }
impl Taxable for Game { const RATE: f64 = 0.20; fn net_price(&self) -> f64 { self.0 } }

fn total<T: Taxable>(items: &[T]) -> f64 {
    items.iter().map(|a| a.gross()).sum()
}
```

| Version | Adding an item type | Adding an operation |
|---|---|---|
| functions | one more function per operation | one function per type |
| enum | change the enum and every `match` | one more `match` |
| trait | one more `impl`, elsewhere | change the trait (with a default: without breaking) |

Note: a trait with an **associated constant** cannot be used as `dyn Taxable` (see `box-dynamiques`); for mixed items in a `Vec`, you would replace `RATE` with a method `fn rate(&self) -> f64`.

---

## 15. Practical exercises

### Exercise 1 — `Convertible` trait

Define `Convertible` with `fn to_text(&self) -> String` and a default method `fn to_uppercase_text(&self) -> String`. Implement it for `i32` and for a struct `Person { name: String }`.

```rust
// Solution:
trait Convertible {
    fn to_text(&self) -> String;
    fn to_uppercase_text(&self) -> String {
        self.to_text().to_uppercase()
    }
}

impl Convertible for i32 {
    fn to_text(&self) -> String { self.to_string() }
}

struct Person { name: String }

impl Convertible for Person {
    fn to_text(&self) -> String { format!("Person({})", self.name) }
}
```

### Exercise 2 — `FromStr` and `Display`

Write `struct Version { major: u32, minor: u32 }` that is read from `"4.0"` and printed as `"4.0"`.

```rust
// Solution:
use std::fmt;
use std::str::FromStr;

struct Version { major: u32, minor: u32 }

impl FromStr for Version {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (a, b) = s.split_once('.').ok_or("expected format: X.Y")?;
        Ok(Version {
            major: a.parse().map_err(|_| "invalid major")?,
            minor: b.parse().map_err(|_| "invalid minor")?,
        })
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

let v: Version = "4.0".parse().unwrap();
assert_eq!(v.to_string(), "4.0");
```

`ok_or("...")?` and `map_err(|_| "...")?` work here because `String: From<&str>`: `?` converts the `&str` into a `String`.

### Exercise 3 — sorting by a chosen order

Sort a `Vec<Version>` from newest to oldest, without implementing `Ord`, then by implementing it.

```rust
// Solution 1: sort_by
fn sort_versions(v: &mut [Version]) {
    v.sort_by(|a, b| (b.major, b.minor).cmp(&(a.major, a.minor)));
}

// Solution 2: Ord (natural ascending order), then reverse
// #[derive(PartialEq, Eq, PartialOrd, Ord)] struct Version { major: u32, minor: u32 }
// v.sort(); v.reverse();
```

With `derive`, the order compares `major` then `minor`: the field order makes the sort order.

---

## 16. Cheat sheet

```rust
trait T {
    fn required(&self) -> u32;
    fn with_default(&self) -> u32 { self.required() * 2 }
    fn associated() -> Self where Self: Sized;
    const N: u32;
}
impl T for MyType { .. }

fn f(x: &impl T)              fn f<X: T>(x: &X)          fn f<X>(x: &X) where X: T
fn f(x: &dyn T)               fn f() -> impl T           fn f() -> Box<dyn T>
X::associated()               <MyType as T>::associated()  T::method(&x)

impl fmt::Display for X       impl From<A> for B         impl TryFrom<A> for B
impl FromStr for X            impl PartialEq for X       impl Ord + PartialOrd
trait StrExt { .. } impl StrExt for str { .. }            // extension trait
```

### Key points

- A trait declares a behavior; a type implements it, even after the fact.
- Ask for few required methods, provide the rest by default.
- The methods of a trait are only visible if the trait is imported.
- Derive when the obvious behavior fits; write by hand for a secret, an equality on an identifier, a non-zero default.
- Implement `From`, never `Into`. `FromStr` + `Display` for a type that can be written out and read back.
- `Hash` must stay consistent with `PartialEq`.
- Orphan rule: the trait or the type must be local; otherwise use a newtype or an extension trait.
