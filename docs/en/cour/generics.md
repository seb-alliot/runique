# Generics
> Write a function, struct or enum once for several types, saying precisely what those types must be able to do.

## Objectives

- Write generic functions, structs, enums and `impl` blocks
- Express bounds in every way: inline, `where`, `impl Trait`
- Write methods reserved for certain types
- Understand monomorphization and its cost
- Choose between generics and `dyn Trait`
- Use const generics, default parameters and `PhantomData`
- Read and fix missing-bound errors

## Table of contents

1. Why generics
2. Generic functions
3. Bounds (trait bounds)
4. The three ways to write a bound
5. Generic structs and enums
6. Generic `impl` blocks
7. Conditional methods and blanket implementations
8. Monomorphization
9. Generic or `dyn Trait`
10. Const generics
11. Default parameters and `PhantomData`
12. `?Sized`
13. The same function written in several ways
14. Common errors
15. Exercises
16. Cheat sheet

---

## 1. Why generics

```rust
fn largest_i32(list: &[i32]) -> i32 {
    let mut max = list[0];
    for &x in list {
        if x > max { max = x; }
    }
    max
}

fn largest_char(list: &[char]) -> char {
    let mut max = list[0];
    for &x in list {
        if x > max { max = x; }
    }
    max
}
```

The two functions are identical except for one type. A generic replaces them with a single one:

```rust
fn largest<T: PartialOrd + Copy>(list: &[T]) -> T {
    let mut max = list[0];
    for &x in list {
        if x > max { max = x; }
    }
    max
}

let a = largest(&[3, 7, 2]);       // T = i32
let b = largest(&['a', 'z']);      // T = char
```

`T` is a **type parameter**. `PartialOrd + Copy` says what `T` must be able to do: compare itself (`>`) and copy itself (`let mut max = list[0]`).

---

## 2. Generic functions

### 2.1 — Syntax

```rust
fn identity<T>(x: T) -> T {
    x
}

fn pair<A, B>(a: A, b: B) -> (A, B) {
    (a, b)
}

fn first<T>(v: &[T]) -> Option<&T> {
    v.first()
}
```

Without a bound, you can do almost nothing with a `T`: move it, return it, store it. Not print it, compare it or copy it.

### 2.2 — Specifying the type at the call

```rust
let a = identity(5);                  // inferred: i32
let b = identity::<u8>(5);            // turbofish
let c: u64 = identity(5);             // inferred from the expected type

let v = Vec::<String>::new();         // turbofish on a type
let n = "42".parse::<i32>();
```

The turbofish (`::<T>`) is used when nothing lets the compiler guess the type.

---

## 3. Bounds (trait bounds)

A bound says: "`T` must implement this trait". It **opens up** the operations of that trait in the body of the function.

```rust
use std::fmt::Display;

fn show<T: Display>(x: T) {
    println!("{x}");              // allowed thanks to Display
}

fn show_twice<T: Display + Clone>(x: T) {
    let copy = x.clone();         // allowed thanks to Clone
    println!("{x} {copy}");
}
```

### Common bounds and what they allow

| Bound | Allows |
|---|---|
| `Clone` | `.clone()` |
| `Copy` | copying by assignment (`let a = x; let b = x;`) |
| `Debug` | `{:?}` |
| `Display` | `{}`, `.to_string()` |
| `PartialEq` | `==`, `!=` |
| `PartialOrd` | `<`, `>`, `<=`, `>=` |
| `Ord` | `.cmp()`, `.max()`, sorting |
| `Hash + Eq` | `HashMap` key |
| `Default` | `T::default()` |
| `Add<Output = T>` | `a + b` |
| `Into<String>` | `x.into()` to a `String` |
| `AsRef<str>` | `x.as_ref()` to a `&str` |
| `IntoIterator<Item = U>` | `for x in t` |
| `Send`, `Sync` | pass or share between threads |
| `'static` | contains no temporary reference |

---

## 4. The three ways to write a bound

### 4.1 — The same function written three times

```rust
use std::fmt::{Debug, Display};

// 1. Inline
fn summary_v1<T: Display + Clone, U: Debug>(a: T, b: U) -> String {
    format!("{a} / {b:?}")
}

// 2. where clause: more readable when the bounds are long
fn summary_v2<T, U>(a: T, b: U) -> String
where
    T: Display + Clone,
    U: Debug,
{
    format!("{a} / {b:?}")
}

// 3. impl Trait as an argument: without naming the type
fn summary_v3(a: impl Display + Clone, b: impl Debug) -> String {
    format!("{a} / {b:?}")
}
```

### 4.2 — The differences

| | Inline / `where` | `impl Trait` as an argument |
|---|---|---|
| Name the type in the body or elsewhere | yes (`T`) | no |
| Two arguments of the **same** type | yes (`a: T, b: T`) | no: each `impl` is a separate type |
| Turbofish at the call | possible | impossible |
| Bounds on something other than the parameters | yes with `where` (`Vec<T>: Debug`) | no |

```rust
// Two parameters MUST have the same type: only the named form allows it
fn equal<T: PartialEq>(a: T, b: T) -> bool { a == b }

// Here a and b can be of different types
fn two(a: impl Display, b: impl Display) {}
```

### 4.3 — `where` on constructed types

```rust
use std::collections::HashMap;
use std::hash::Hash;

fn count<T>(items: &[T]) -> HashMap<&T, usize>
where
    T: Hash + Eq,
{
    let mut m = HashMap::new();
    for e in items {
        *m.entry(e).or_insert(0) += 1;
    }
    m
}

fn show_all<I>(it: I)
where
    I: IntoIterator,
    I::Item: std::fmt::Display,       // bound on the type of the elements
{
    for x in it {
        println!("{x}");
    }
}
```

### 4.4 — `impl Trait` as a return type

```rust
fn evens(limit: u32) -> impl Iterator<Item = u32> {
    (0..limit).filter(|x| x % 2 == 0)
}
```

As a return type, `impl Trait` means "one precise type, which I do not name". It is **not** a generic chosen by the caller: the function chooses, and it must always return **the same** type.

---

## 5. Generic structs and enums

### 5.1 — Struct

```rust
struct Point<T> {
    x: T,
    y: T,
}

let integers = Point { x: 5, y: 10 };
let floats = Point { x: 1.0, y: 4.0 };
// let mixed = Point { x: 5, y: 4.0 };   // ERROR: x and y must have the same T

struct Point2<T, U> {
    x: T,
    y: U,
}
let mixed = Point2 { x: 5, y: 4.0 };     // OK
```

### 5.2 — Enum

The two most used enums in Rust are generic:

```rust
enum Option<T> { Some(T), None }
enum Result<T, E> { Ok(T), Err(E) }
```

```rust
enum Tree<T> {
    Leaf(T),
    Node(Box<Tree<T>>, Box<Tree<T>>),
}
```

### 5.3 — Where to put the bound

```rust
// 1. On the struct: every use of SortedVec1<T> requires Ord, even those that do not sort
struct SortedVec1<T: Ord> { v: Vec<T> }

// 2. On the impl block only (recommended): the struct stays freely usable
struct SortedVec2<T> { v: Vec<T> }

impl<T: Ord> SortedVec2<T> {
    fn insert(&mut self, x: T) {
        let pos = self.v.binary_search(&x).unwrap_or_else(|p| p);
        self.v.insert(pos, x);
    }
}
```

Putting the bound on the struct forces you to repeat it **everywhere** the type appears (every `impl`, every function that receives it). Put it on the struct only if the type makes no sense without it.

---

## 6. Generic `impl` blocks

### 6.1 — For all `T`

```rust
impl<T> Point<T> {
    fn x(&self) -> &T {
        &self.x
    }
}
```

`impl<T>` declares `T`, `Point<T>` uses it. Without the `<T>` after `impl`, Rust would look for a type that is really named `T`.

### 6.2 — For a specific type

```rust
impl Point<f64> {
    fn distance_from_origin(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

let p = Point { x: 3.0, y: 4.0 };
p.distance_from_origin();            // 5.0
let q = Point { x: 3, y: 4 };
// q.distance_from_origin();         // ERROR: only exists for Point<f64>
```

### 6.3 — A method with its own type parameters

```rust
impl<T> Point2<T, T> {
    fn mix<V>(self, other: Point2<V, V>) -> Point2<T, V> {
        Point2 { x: self.x, y: other.y }
    }
}
```

`T` comes from the type, `V` belongs to the method and is chosen at each call.

---

## 7. Conditional methods and blanket implementations

### 7.1 — Methods that exist only if `T` allows it

```rust
use std::fmt::Display;

struct Pair<T> { a: T, b: T }

impl<T> Pair<T> {
    fn new(a: T, b: T) -> Self { Self { a, b } }       // for all T
}

impl<T: Display + PartialOrd> Pair<T> {
    fn print_largest(&self) {                           // only if T is comparable and displayable
        if self.a >= self.b {
            println!("{}", self.a);
        } else {
            println!("{}", self.b);
        }
    }
}
```

`Pair<Vec<i32>>` exists, but does not have `print_largest` (`Vec` is not `Display`).

### 7.2 — Implementing a trait for every type that has another one

```rust
trait Summary {
    fn summary(&self) -> String;
}

impl<T: std::fmt::Display> Summary for T {
    fn summary(&self) -> String {
        let s = self.to_string();
        if s.chars().count() > 10 {
            format!("{}…", s.chars().take(10).collect::<String>())   // in characters, not bytes
        } else {
            s
        }
    }
}

println!("{}", 12345.summary());
println!("{}", "a fairly long text".summary());
```

This is a **blanket implementation** (blanket impl). The standard library does the same: `impl<T: Display> ToString for T` gives `.to_string()` to every displayable type. You can write only one per trait and per form: it then prevents you from implementing the same trait by hand for a `Display` type.

---

## 8. Monomorphization

### 8.1 — What the compiler does

```rust
fn double<T: std::ops::Add<Output = T> + Copy>(x: T) -> T { x + x }

double(2_i32);
double(2.5_f64);
```

The compiler generates **one copy** of the function per type used, as if you had written:

```rust
fn double_i32(x: i32) -> i32 { x + x }
fn double_f64(x: f64) -> f64 { x + x }
```

### 8.2 — Consequences

| Advantage | Cost |
|---|---|
| no runtime cost: each copy is optimised for its type | bigger binary (one copy per type) |
| direct calls, often inlined | longer compilation |

### 8.3 — Reducing duplication

```rust
// The whole function is duplicated for each type that implements AsRef<Path>
pub fn read_v1<P: AsRef<std::path::Path>>(path: P) -> std::io::Result<String> {
    let path = path.as_ref();
    // ... fifty lines ...
    std::fs::read_to_string(path)
}

// Only the conversion is generic; the body is compiled once
pub fn read_v2<P: AsRef<std::path::Path>>(path: P) -> std::io::Result<String> {
    fn inner(path: &std::path::Path) -> std::io::Result<String> {
        // ... fifty lines ...
        std::fs::read_to_string(path)
    }
    inner(path.as_ref())
}
```

The standard library uses this technique (`std::fs::read_to_string` itself).

---

## 9. Generic or `dyn Trait`

The same function, which adds up a list of shapes, written both ways.

```rust
trait Shape {
    fn area(&self) -> f64;
}

struct Square(f64);
struct Circle(f64);
impl Shape for Square { fn area(&self) -> f64 { self.0 * self.0 } }
impl Shape for Circle { fn area(&self) -> f64 { 3.14159 * self.0 * self.0 } }

// 1. Generic: all the elements have THE SAME type
fn total_generic<S: Shape>(shapes: &[S]) -> f64 {
    shapes.iter().map(|s| s.area()).sum()
}

// 2. dyn Trait: each element can have a different type
fn total_dyn(shapes: &[Box<dyn Shape>]) -> f64 {
    shapes.iter().map(|s| s.area()).sum()
}

fn main() {
    total_generic(&[Square(1.0), Square(2.0)]);
    // total_generic(&[Square(1.0), Circle(2.0)]);   // ERROR: different types
    total_dyn(&[Box::new(Square(1.0)), Box::new(Circle(2.0))]);
}
```

| | Generic `<T: Trait>` | `dyn Trait` |
|---|---|---|
| Choice of type | at compile time | at runtime |
| Collection of mixed types | no | yes |
| Call cost | direct, inlinable | indirect (method table) |
| Binary size | one copy per type | a single copy |
| Constraints on the trait | none | the trait must be "dyn-compatible" (see `box-dynamiques`) |

Default: generic. Use `dyn` when types are mixed, chosen at runtime, or when binary size matters.

---

## 10. Const generics

A generic parameter can be a **constant value** (integer, `bool`, `char`) instead of a type.

```rust
fn sum<const N: usize>(t: [i32; N]) -> i32 {
    t.iter().sum()
}

sum([1, 2, 3]);           // N = 3
sum([1, 2, 3, 4, 5]);     // N = 5

struct Buffer<const SIZE: usize> {
    data: [u8; SIZE],
    length: usize,
}

impl<const SIZE: usize> Buffer<SIZE> {
    fn new() -> Self {
        Self { data: [0; SIZE], length: 0 }
    }
    fn capacity(&self) -> usize { SIZE }
}

let t = Buffer::<64>::new();
```

What it is for: fixed-size arrays without allocation, matrices whose dimensions are checked at compile time, buffers of known size in embedded code. Before const generics, the standard library only implemented some traits for arrays of up to 32 elements.

---

## 11. Default parameters and `PhantomData`

### 11.1 — Default type

```rust
struct Counter<T = u32> {
    value: T,
}

let a: Counter = Counter { value: 5 };          // T = u32
let b: Counter<u64> = Counter { value: 5 };
```

This is how `HashMap<K, V, S = RandomState>` lets you choose the hashing algorithm without imposing it on everyone.

### 11.2 — `PhantomData`: a parameter without data

```rust
use std::marker::PhantomData;

struct Id<T> {
    value: i64,
    _type: PhantomData<T>,       // Id<T> "carries" T without storing it
}

struct User;
struct Order;

impl<T> Id<T> {
    fn new(value: i64) -> Self { Self { value, _type: PhantomData } }
}

fn cancel(_id: Id<Order>) {}

fn main() {
    let u: Id<User> = Id::new(1);
    // cancel(u);                   // ERROR: Id<User> is not Id<Order>
    cancel(Id::new(7));
}
```

Rust rejects an unused type parameter (`parameter T is never used`). `PhantomData<T>` uses it without taking up any memory. What it is for: telling apart at compile time values with the same representation (identifiers, units of measure, builder states).

---

## 12. `?Sized`

By default, every type parameter must have a **size known** at compile time (implicit `T: Sized`). `str`, `[T]` and `dyn Trait` do not.

```rust
fn length_v1<T: AsRef<str>>(x: T) -> usize { x.as_ref().len() }

// Accept &str directly as a &T with T = str
fn show<T: std::fmt::Display + ?Sized>(x: &T) {
    println!("{x}");
}

show("text");          // T = str: possible thanks to ?Sized
show(&5);
```

`?Sized` lifts the implicit bound. You can then only handle `T` behind a reference or a pointer (`&T`, `Box<T>`).

---

## 13. The same function written in several ways

Find the largest element of a list.

```rust
// 1. Copy: returns a copy, reserved for small types
fn max_v1<T: PartialOrd + Copy>(v: &[T]) -> Option<T> {
    let mut iter = v.iter().copied();
    let mut max = iter.next()?;
    for x in iter {
        if x > max { max = x; }
    }
    Some(max)
}

// 2. Reference: also works with String, without copying
fn max_v2<T: PartialOrd>(v: &[T]) -> Option<&T> {
    let mut max = v.first()?;
    for x in v {
        if x > max { max = x; }
    }
    Some(max)
}

// 3. Ord + standard library
fn max_v3<T: Ord>(v: &[T]) -> Option<&T> {
    v.iter().max()
}

// 4. Any iterator, not only a slice
fn max_v4<I>(it: I) -> Option<I::Item>
where
    I: IntoIterator,
    I::Item: Ord,
{
    it.into_iter().max()
}

fn main() {
    max_v1(&[3, 9, 2]);                     // Some(9)
    max_v2(&[String::from("a"), String::from("b")]);
    max_v3(&["apple", "kiwi"]);
    max_v4(vec![1, 5, 3]);
    max_v4((1..10).map(|x| x * x % 7));
    max_v2(&[1.5, 0.2]);                    // f64: PartialOrd is enough for max_v2
    // max_v3(&[1.5, 0.2]);                 // ERROR: f64 is not Ord
}
```

All of them return an `Option`: an empty list has no maximum, whereas the version in section 1 panicked on `list[0]`.

---

## 14. Common errors

### 14.1 — Operation without a bound

```rust
// fn sum<T>(a: T, b: T) -> T { a + b }
// ERROR E0369: cannot add `T` to `T`
use std::ops::Add;
fn sum<T: Add<Output = T>>(a: T, b: T) -> T { a + b }
```

The compiler often suggests the missing bound in its message (`consider restricting type parameter T`).

### 14.2 — Method not found

```rust
// fn show<T>(x: T) { println!("{}", x); }
// ERROR E0277: `T` doesn't implement `std::fmt::Display`
fn show<T: std::fmt::Display>(x: T) { println!("{x}"); }
```

### 14.3 — Type that cannot be guessed

```rust
// let v = Vec::new();            // ERROR E0282: type annotations needed
let v: Vec<i32> = Vec::new();
let w = Vec::<i32>::new();
```

### 14.4 — Moving out of a slice

```rust
// fn first<T>(v: &[T]) -> T { v[0] }   // ERROR: cannot move out of index
fn first_ref<T>(v: &[T]) -> &T { &v[0] }
fn first_copy<T: Copy>(v: &[T]) -> T { v[0] }
fn first_clone<T: Clone>(v: &[T]) -> T { v[0].clone() }
```

---

## 15. Exercises

### Exercise 1 — generic stack

Write `Stack<T>` with `new`, `push`, `pop -> Option<T>`, `peek -> Option<&T>` and `is_empty`.

```rust
// Solution:
struct Stack<T> {
    items: Vec<T>,
}

impl<T> Stack<T> {
    fn new() -> Self { Self { items: Vec::new() } }
    fn push(&mut self, x: T) { self.items.push(x); }
    fn pop(&mut self) -> Option<T> { self.items.pop() }
    fn peek(&self) -> Option<&T> { self.items.last() }
    fn is_empty(&self) -> bool { self.items.is_empty() }
}
```

### Exercise 2 — conditional method

Add to `Stack<T>` a `print` method that only exists if `T: Debug`.

```rust
// Solution:
impl<T: std::fmt::Debug> Stack<T> {
    fn print(&self) {
        println!("{:?}", self.items);
    }
}
```

### Exercise 3 — generic average

Write an average for any type convertible to `f64` without loss (`i32`, `u8`, `f32`...).

```rust
// Solution:
fn average<T: Copy + Into<f64>>(v: &[T]) -> Option<f64> {
    if v.is_empty() {
        return None;
    }
    let total: f64 = v.iter().map(|&x| -> f64 { x.into() }).sum();   // target type written out: sum would have two choices
    Some(total / v.len() as f64)
}

average(&[1_i32, 2, 3]);      // Some(2.0)
average(&[1.5_f32, 2.5]);     // Some(2.0)
// average(&[1_i64]);         // ERROR: i64 -> f64 can lose precision, no From
```

## 16. Cheat sheet

```rust
fn f<T>(x: T) -> T                         // generic
fn f<T: A + B>(x: T)                       // inline bounds
fn f<T>(x: T) where T: A + B               // where
fn f(x: impl A + B)                        // impl Trait as an argument
fn f() -> impl Iterator<Item = u32>        // impl Trait as a return type
struct S<T> { x: T }                       // generic struct
impl<T> S<T> { }                           // for all T
impl S<f64> { }                            // for a specific type
impl<T: Display> S<T> { }                  // conditional methods
impl<T: Display> MyTrait for T { }         // blanket implementation
fn f<const N: usize>(t: [u8; N])           // const generic
struct S<T = u32>                          // default type
PhantomData<T>                             // parameter without data
fn f<T: ?Sized>(x: &T)                     // accepts str, [T], dyn Trait
f::<u8>(5)                                 // turbofish
```

### Key points

- A bound opens up operations: without it, a `T` can do almost nothing.
- `where` for readability, `impl Trait` when you do not need to name the type.
- Put bounds on the `impl` block, not on the struct.
- A generic is copied for each type: fast at runtime, heavier to compile.
- Generic by default, `dyn Trait` to mix types at runtime.
- `PhantomData` tells types apart at no memory cost.
