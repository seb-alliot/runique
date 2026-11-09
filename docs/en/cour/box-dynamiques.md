# Box and dynamic types

> `Box<T>` to place a value on the heap, `dyn Trait` to handle different types behind the same interface: when to use them, how, and what they cost compared to generics and enums.

## Objectives

- Know when a value must go into a `Box`
- Understand what a trait object (`dyn Trait`) and a "wide" pointer are
- Write `&dyn Trait`, `Box<dyn Trait>`, `Arc<dyn Trait>` and choose between them
- Know the `dyn` compatibility rules of a trait, and how to work around them
- Store trait objects in structs and collections
- Clone, compare and convert trait objects
- Choose between enum, generic and `dyn Trait` for the same problem

## Table of contents

1. `Box<T>`: a value on the heap
2. When to use a `Box`
3. Trait objects
4. The different ways to hold a trait object
5. How a dynamic call works
6. `dyn` compatibility of a trait
7. Trait objects in structs
8. Collections of trait objects
9. Cloning a trait object
10. Getting the concrete type back: `Any`
11. Upcasting to a supertrait
12. Enum, generic or `dyn`: the same feature three times
13. Design patterns
14. Real examples in Runique
15. Exercises
16. Key points

## 1. `Box<T>`: a value on the heap

```rust
let b = Box::new(5);           // 5 is on the heap, b (a pointer) on the stack
println!("{}", b);             // Box dereferences automatically
let x: i32 = *b;               // * to get the value back
```

A `Box<T>` **owns** its value: when the `Box` is freed, the value is freed too. It behaves like the value itself (methods, operators) thanks to `Deref`.

```rust
let s = Box::new(String::from("abc"));
println!("{}", s.len());      // String method, called through the Box
let owned: String = *s;       // move the value out of the Box (the Box is consumed)
```

Its size on the stack is always that of a pointer (8 bytes on a 64-bit machine), whatever the size of `T`.

## 2. When to use a `Box`

### 2.1 Recursive type

```rust
// enum List { End, Item(i32, List) }       // ERROR E0072: recursive type has infinite size

enum List {
    End,
    Item(i32, Box<List>),
}

let l = List::Item(1, Box::new(List::Item(2, Box::new(List::End))));
```

To compute the size of `List`, the compiler would need the size of `List`. With `Box`, the next item becomes a fixed-size pointer.

### 2.2 Large value that is moved often

```rust
struct Image { pixels: [u8; 1_000_000] }

fn process(img: Box<Image>) -> Box<Image> { img }    // only moves a pointer
```

Moving a value copies its bytes; moving a `Box` only copies the pointer.

### 2.3 Reducing the size of an enum

```rust
enum Message {
    Ping,
    Big(Box<[u8; 4096]>),     // without Box, every Ping would take 4 KB
}
```

### 2.4 Owned trait object

```rust
trait Shape { fn area(&self) -> f64; }
let f: Box<dyn Shape>;          // see the rest of the course
```

### 2.5 Fixed-size slices and strings

```rust
let slice: Box<[i32]> = vec![1, 2, 3].into_boxed_slice();   // like a Vec, without extra capacity
let text: Box<str> = String::from("fixed").into_boxed_str();
```

What it is for: storing a lot of data that will not grow anymore, with only a pointer and a length (16 bytes instead of 24 for a `Vec` or a `String`).

### 2.6 When not to use `Box`

For an ordinary value, with no recursion or trait object: the stack is faster and `Box` only adds an allocation. `Box<Vec<T>>` is almost always useless: the `Vec` is already on the heap.

## 3. Trait objects

A **trait object** is a value of which you only know the trait, not the concrete type. It is written `dyn Trait`.

```rust
trait Animal {
    fn name(&self) -> String;
    fn sound(&self) -> String;
}

struct Dog;
struct Cat;

impl Animal for Dog {
    fn name(&self) -> String { "dog".into() }
    fn sound(&self) -> String { "woof".into() }
}

impl Animal for Cat {
    fn name(&self) -> String { "cat".into() }
    fn sound(&self) -> String { "meow".into() }
}

fn introduce(a: &dyn Animal) {
    println!("the {} says {}", a.name(), a.sound());
}

fn main() {
    introduce(&Dog);
    introduce(&Cat);
    let zoo: Vec<Box<dyn Animal>> = vec![Box::new(Dog), Box::new(Cat)];
    for a in &zoo {
        introduce(a.as_ref());
    }
}
```

`dyn Animal` has no known size (a `Dog` and a `Cat` do not necessarily have the same one): you handle it **only behind a pointer** (`&`, `Box`, `Arc`, `Rc`).

## 4. The different ways to hold a trait object

| Syntax | Owns the value | Sharing | Usage |
|---|---|---|---|
| `&dyn Trait` | no | — | function parameter, temporary access |
| `&mut dyn Trait` | no | — | parameter that modifies |
| `Box<dyn Trait>` | yes, alone | no | struct field, collection element, function return |
| `Rc<dyn Trait>` | yes, several owners | one thread | graph, value shared within a single thread |
| `Arc<dyn Trait>` | yes, several owners | across threads | service shared in a server |

```rust
use std::rc::Rc;
use std::sync::Arc;

let a: &dyn Animal = &Dog;
let b: Box<dyn Animal> = Box::new(Dog);
let c: Rc<dyn Animal> = Rc::new(Cat);
let d: Arc<dyn Animal + Send + Sync> = Arc::new(Cat);
```

### 4.1 Adding constraints

```rust
Box<dyn Animal + Send>             // can be sent to another thread
Box<dyn Animal + Send + Sync>      // can be shared between threads
Box<dyn Animal + 'a>               // can hold references of lifetime 'a
Box<dyn Animal>                    // = Box<dyn Animal + 'static>
```

For a web server (tokio, axum), shared trait objects are almost always `Send + Sync`.

### 4.2 Parameter: `&dyn` or generic

```rust
fn introduce_dyn(a: &dyn Animal) { println!("{}", a.name()); }
fn introduce_gen(a: &impl Animal) { println!("{}", a.name()); }
fn introduce_gen2<A: Animal + ?Sized>(a: &A) { println!("{}", a.name()); }   // also accepts &dyn Animal
```

All three accept `&Dog`. The generic version is copied for each type (fast), the `dyn` version exists in a single copy (more compact). `?Sized` lets the generic version accept a `&dyn Animal` too.

## 5. How a dynamic call works

### 5.1 The wide pointer

```rust
use std::mem::size_of;

size_of::<&Dog>();             // 8 bytes: one address
size_of::<&dyn Animal>();      // 16 bytes: one address + one vtable address
size_of::<Box<dyn Animal>>();  // 16 bytes
```

A reference to a trait object holds two pointers:

```
&dyn Animal
┌────────────┬────────────┐
│ data    ───┼──► Dog     │
│ vtable  ───┼──► [ drop, size, alignment, name(), sound() ] for Dog
└────────────┴────────────┘
```

### 5.2 The cost

| | Static call (generic, concrete type) | Dynamic call (`dyn`) |
|---|---|---|
| Choice of the function | at compile time | at run time, through the vtable |
| Inlining | possible | generally impossible |
| Code size | one copy per type | a single copy |
| Cost of a call | none | one memory read + one indirect call |

This cost is negligible compared to a network request or a database read. It matters in a very tight loop (millions of calls per second).

## 6. `dyn` compatibility of a trait

Not every trait can become a trait object. A trait is "dyn-compatible" (formerly *object safe*) if its methods can be called without knowing the concrete type.

### 6.1 What prevents `dyn`

```rust
trait Copyable {
    fn copy(&self) -> Self;                    // returns Self: unknown size behind dyn
}

trait Converter {
    fn convert<T>(&self, x: T) -> String;      // generic method: infinitely many versions
}

trait Buildable {
    fn create() -> Self;                       // no self: nothing to call on an object
}

// let x: Box<dyn Copyable>;                   // ERROR E0038: the trait cannot be made into an object
```

| Rule | Rejected example |
|---|---|
| no method that returns `Self` | `fn clone(&self) -> Self` |
| no generic method | `fn f<T>(&self, x: T)` |
| no associated function without `self` | `fn new() -> Self` |
| no associated constant | `const N: usize;` |
| the trait must not require `Sized` | `trait T: Sized` |

This is why `Clone` is not dyn-compatible: `Box<dyn Clone>` does not exist.

### 6.2 Excluding a method with `where Self: Sized`

```rust
trait Shape {
    fn area(&self) -> f64;

    fn new_square(side: f64) -> Self           // cannot be called on a dyn Shape...
    where
        Self: Sized;                           // ...but no longer prevents dyn Shape

    fn duplicate(&self) -> Self where Self: Sized;
}
```

Methods marked `where Self: Sized` only exist on concrete types. The rest of the trait remains usable as `dyn`.

### 6.3 Replacing a generic method

```rust
// trait Journal { fn write<T: std::fmt::Display>(&self, x: T); }    // not dyn

trait Journal {
    fn write(&self, x: &dyn std::fmt::Display);                          // dyn
}
```

## 7. Trait objects in structs

### 7.1 An injectable behavior

```rust
trait Notifier {
    fn send(&self, to: &str, msg: &str) -> Result<(), String>;
}

struct Email;
impl Notifier for Email {
    fn send(&self, to: &str, msg: &str) -> Result<(), String> {
        println!("email to {to}: {msg}");
        Ok(())
    }
}

struct Sms;
impl Notifier for Sms {
    fn send(&self, to: &str, msg: &str) -> Result<(), String> {
        println!("sms to {to}: {msg}");
        Ok(())
    }
}

struct Service {
    notifier: Box<dyn Notifier>,
}

impl Service {
    fn new(notifier: Box<dyn Notifier>) -> Self {
        Self { notifier }
    }
    fn register(&self, contact: &str) -> Result<(), String> {
        self.notifier.send(contact, "Welcome")
    }
    fn switch(&mut self, n: Box<dyn Notifier>) {
        self.notifier = n;               // the behavior changes at run time
    }
}

fn main() {
    let mut s = Service::new(Box::new(Email));
    s.register("ada@ex.com").unwrap();
    s.switch(Box::new(Sms));
}
```

### 7.2 The same struct with a generic

```rust
struct ServiceGen<N: Notifier> {
    notifier: N,
}

impl<N: Notifier> ServiceGen<N> {
    fn register(&self, contact: &str) -> Result<(), String> {
        self.notifier.send(contact, "Welcome")
    }
}

let s = ServiceGen { notifier: Email };
// cannot replace Email with Sms in the same variable: ServiceGen<Email> and ServiceGen<Sms> are two types
```

| | `Box<dyn Notifier>` | `ServiceGen<N>` |
|---|---|---|
| Change implementation at run time | yes | no |
| Choose from the configuration (`.env`) | yes, easily | possible with a `match` that creates different types |
| Type parameter to propagate | no | yes, everywhere `ServiceGen` appears |
| Performance | indirect call | direct call |

### 7.3 Option and default value

```rust
struct Config {
    journal: Option<Box<dyn Fn(&str)>>,      // optional behavior
}

impl Config {
    fn log(&self, msg: &str) {
        if let Some(f) = &self.journal {
            f(msg);
        }
    }
}
```

## 8. Collections of trait objects

### 8.1 Different types in the same `Vec`

```rust
trait Shape {
    fn area(&self) -> f64;
    fn name(&self) -> &str;
}

struct Square(f64);
struct Circle(f64);

impl Shape for Square {
    fn area(&self) -> f64 { self.0 * self.0 }
    fn name(&self) -> &str { "square" }
}
impl Shape for Circle {
    fn area(&self) -> f64 { std::f64::consts::PI * self.0 * self.0 }
    fn name(&self) -> &str { "circle" }
}

fn main() {
    let shapes: Vec<Box<dyn Shape>> = vec![Box::new(Square(2.0)), Box::new(Circle(1.0))];

    let total: f64 = shapes.iter().map(|f| f.area()).sum();
    let largest = shapes.iter().max_by(|a, b| a.area().total_cmp(&b.area()));
    for f in &shapes {
        println!("{}: {:.2}", f.name(), f.area());
    }
}
```

### 8.2 A table of handlers

```rust
use std::collections::HashMap;

type Command = Box<dyn Fn(&[&str]) -> String>;

fn commands() -> HashMap<&'static str, Command> {
    let mut m: HashMap<&'static str, Command> = HashMap::new();
    m.insert("echo", Box::new(|args| args.join(" ")));
    m.insert("count", Box::new(|args| args.len().to_string()));
    m
}

fn main() {
    let c = commands();
    if let Some(f) = c.get("echo") {
        println!("{}", f(&["a", "b"]));
    }
}
```

### 8.3 Insertion order

`IndexMap` (the `indexmap` crate) keeps insertion order, which a `HashMap` does not. Useful for a list of form fields, displayed in the declared order.

## 9. Cloning a trait object

`Clone` is not dyn-compatible (section 6): `Box<dyn Shape>` cannot be cloned directly.

### 9.1 Writing the method yourself

```rust
trait Shape {
    fn area(&self) -> f64;
    fn boxed_clone(&self) -> Box<dyn Shape>;
}

#[derive(Clone)]
struct Square(f64);

impl Shape for Square {
    fn area(&self) -> f64 { self.0 * self.0 }
    fn boxed_clone(&self) -> Box<dyn Shape> { Box::new(self.clone()) }
}

impl Clone for Box<dyn Shape> {
    fn clone(&self) -> Self { self.boxed_clone() }
}
```

### 9.2 With the `dyn-clone` crate

```rust
use dyn_clone::DynClone;

trait Shape: DynClone {
    fn area(&self) -> f64;
}
dyn_clone::clone_trait_object!(Shape);

#[derive(Clone)]
struct Circle(f64);
impl Shape for Circle {
    fn area(&self) -> f64 { 3.14 * self.0 * self.0 }
}

let a: Box<dyn Shape> = Box::new(Circle(1.0));
let b = a.clone();          // works: the macro wrote the Clone impl
```

The crate generates exactly the code of section 9.1. Runique uses it for its form fields (section 14).

### 9.3 Sharing instead of cloning

```rust
use std::sync::Arc;

let a: Arc<dyn Shape> = Arc::new(Circle(1.0));
let b = Arc::clone(&a);     // same object, reference count increased
```

If the copies do not need to be modified separately, `Arc` avoids any copy.

## 10. Getting the concrete type back: `Any`

```rust
use std::any::Any;

trait Component: Any {
    fn name(&self) -> &str;
    fn as_any(&self) -> &dyn Any;
}

struct Button { label: String }

impl Component for Button {
    fn name(&self) -> &str { "button" }
    fn as_any(&self) -> &dyn Any { self }
}

fn label(c: &dyn Component) -> Option<&str> {
    c.as_any().downcast_ref::<Button>().map(|b| b.label.as_str())
}
```

`downcast_ref` returns `Some` if the concrete type is the right one, `None` otherwise. If you need it often, it is a sign that a method is missing from the trait, or that an enum would fit better.

## 11. Upcasting to a supertrait

Since Rust 1.86, a trait object converts to the trait object of a **supertrait**:

```rust
trait Named {
    fn name(&self) -> String;
}

trait Animal: Named {
    fn sound(&self) -> String;
}

struct Dog;
impl Named for Dog { fn name(&self) -> String { "dog".into() } }
impl Animal for Dog { fn sound(&self) -> String { "woof".into() } }

fn show_name(n: &dyn Named) {
    println!("{}", n.name());
}

fn main() {
    let a: Box<dyn Animal> = Box::new(Dog);
    show_name(&*a);                            // &dyn Animal -> &dyn Named
    let n: Box<dyn Named> = a;                 // Box<dyn Animal> -> Box<dyn Named>
}
```

Before 1.86, you had to add a method `fn as_named(&self) -> &dyn Named` to the trait.

## 12. Enum, generic or `dyn`: the same feature three times

A payment system with several payment methods.

```rust
// 1. Enum: CLOSED set, known in advance
enum Method {
    Card { number: String },
    Transfer { iban: String },
}

impl Method {
    fn pay(&self, amount: u32) -> String {
        match self {
            Method::Card { number } => format!("card {}: {amount}", &number[number.len() - 4..]),
            Method::Transfer { iban } => format!("transfer {iban}: {amount}"),
        }
    }
}

// 2. Trait + generic: open set, one type per use
trait Payment {
    fn pay(&self, amount: u32) -> String;
}

struct Card { number: String }
impl Payment for Card {
    fn pay(&self, amount: u32) -> String { format!("card: {amount}") }
}

fn collect<P: Payment>(p: &P, amount: u32) -> String { p.pay(amount) }

// 3. Trait object: open set, types mixed at run time
fn collect_all(methods: &[Box<dyn Payment>], amount: u32) -> Vec<String> {
    methods.iter().map(|m| m.pay(amount)).collect()
}
```

| Criterion | Enum | Generic | `dyn Trait` |
|---|---|---|---|
| Add a case without touching existing code | no (modify the enum and the `match`es) | yes (new `impl`) | yes (new `impl`) |
| Add an operation to all cases | easy (one more `match`) | modify the trait and every `impl` | same |
| The compiler checks that all cases are handled | yes | — | — |
| Mixed types in a collection | yes | no | yes |
| A library user can add a case | no | yes | yes |
| Performance | direct `match` | direct call | indirect call |
| No allocation | yes | yes | no (`Box`) except `&dyn` |

Rule: **enum** if the cases are known and few (states, commands, message types); **generic** for an abstraction chosen at compile time; **`dyn`** for plugins, behaviors chosen at run time or a heterogeneous collection.

## 13. Design patterns

### 13.1 Strategy: changing an algorithm at run time

This is the `Service` of section 7.1: the algorithm (email, SMS) is a replaceable trait object.

### 13.2 State: each state decides the next one

```rust
trait State {
    fn publish(self: Box<Self>) -> Box<dyn State>;
    fn name(&self) -> &'static str;
}

struct Draft;
struct InReview;
struct Published;

impl State for Draft {
    fn publish(self: Box<Self>) -> Box<dyn State> { Box::new(InReview) }
    fn name(&self) -> &'static str { "draft" }
}
impl State for InReview {
    fn publish(self: Box<Self>) -> Box<dyn State> { Box::new(Published) }
    fn name(&self) -> &'static str { "in review" }
}
impl State for Published {
    fn publish(self: Box<Self>) -> Box<dyn State> { self }
    fn name(&self) -> &'static str { "published" }
}

struct Post { state: Option<Box<dyn State>> }

impl Post {
    fn publish(&mut self) {
        if let Some(s) = self.state.take() {
            self.state = Some(s.publish());
        }
    }
}
```

`self: Box<Self>` takes ownership of the state in order to replace it. The same state machine is written more simply with an enum (`enum State { Draft, InReview, Published }`) and a `match`: the `dyn` version is only preferable if new states must be addable from the outside.

### 13.3 Command: stored and undoable actions

```rust
trait Action {
    fn execute(&mut self, doc: &mut String);
    fn undo(&mut self, doc: &mut String);
}

struct Append(String);

impl Action for Append {
    fn execute(&mut self, doc: &mut String) { doc.push_str(&self.0); }
    fn undo(&mut self, doc: &mut String) {
        let n = doc.len() - self.0.len();
        doc.truncate(n);
    }
}

struct Editor { doc: String, history: Vec<Box<dyn Action>> }

impl Editor {
    fn apply(&mut self, mut a: Box<dyn Action>) {
        a.execute(&mut self.doc);
        self.history.push(a);
    }
    fn revert(&mut self) {
        if let Some(mut a) = self.history.pop() {
            a.undo(&mut self.doc);
        }
    }
}
```

### 13.4 Observer: notifying several subscribers

```rust
struct Events {
    subscribers: Vec<Box<dyn Fn(&str) + Send + Sync>>,
}

impl Events {
    fn subscribe(&mut self, f: impl Fn(&str) + Send + Sync + 'static) {
        self.subscribers.push(Box::new(f));
    }
    fn emit(&self, e: &str) {
        for s in &self.subscribers {
            s(e);
        }
    }
}
```

## 14. Real examples in Runique

### 14.1 Form fields

```rust
// runique/src/forms/base.rs
pub trait FormField: CommonFieldConfig + DynClone + std::fmt::Debug + Send + Sync { /* ... */ }

// runique/src/forms/field.rs
dyn_clone::clone_trait_object!(FormField);

// runique/src/utils/aliases/definition.rs
pub type FieldsMap = IndexMap<String, Box<dyn FormField>>;
```

A form holds fields of different types (`TextField`, `MediaField`, `SelectField`...) in a single ordered table: this is exactly the use case of a collection of trait objects (section 8). `IndexMap` keeps the declaration order for HTML rendering; `DynClone` allows cloning the whole form; `Send + Sync` allows sharing it between server tasks.

### 14.2 Password hashing algorithms

```rust
// runique/src/utils/password/mod.rs
pub trait PasswordHandler: Send + Sync + DynClone { /* ... */ }
dyn_clone::clone_trait_object!(PasswordHandler);
```

The algorithm (Argon2, bcrypt, scrypt) is chosen by the configuration: a trait object lets you decide it at startup without a generic parameter propagated through the whole framework (the strategy pattern of section 13.1).

## 15. Exercises

### Exercise 1: virtual file system

Define a trait `Node` with `name(&self) -> &str` and `size(&self) -> u64`. Implement `File { name, size }` and `Folder { name, children: Vec<Box<dyn Node>> }` whose size is the sum of its children.

```rust
// Solution:
trait Node {
    fn name(&self) -> &str;
    fn size(&self) -> u64;
}

struct File { name: String, size: u64 }
struct Folder { name: String, children: Vec<Box<dyn Node>> }

impl Node for File {
    fn name(&self) -> &str { &self.name }
    fn size(&self) -> u64 { self.size }
}

impl Node for Folder {
    fn name(&self) -> &str { &self.name }
    fn size(&self) -> u64 { self.children.iter().map(|e| e.size()).sum() }
}
```

### Exercise 2: extensible calculator

Create a trait `Operation { fn symbol(&self) -> char; fn compute(&self, a: f64, b: f64) -> Option<f64>; }` and a calculator that stores its operations in a `HashMap<char, Box<dyn Operation>>`. Add addition and division (which returns `None` on zero).

```rust
// Solution:
use std::collections::HashMap;

trait Operation {
    fn symbol(&self) -> char;
    fn compute(&self, a: f64, b: f64) -> Option<f64>;
}

struct Addition;
struct Division;

impl Operation for Addition {
    fn symbol(&self) -> char { '+' }
    fn compute(&self, a: f64, b: f64) -> Option<f64> { Some(a + b) }
}
impl Operation for Division {
    fn symbol(&self) -> char { '/' }
    fn compute(&self, a: f64, b: f64) -> Option<f64> { (b != 0.0).then(|| a / b) }
}

struct Calculator { ops: HashMap<char, Box<dyn Operation>> }

impl Calculator {
    fn new() -> Self { Self { ops: HashMap::new() } }
    fn add(&mut self, op: Box<dyn Operation>) { self.ops.insert(op.symbol(), op); }
    fn compute(&self, a: f64, s: char, b: f64) -> Option<f64> {
        self.ops.get(&s)?.compute(a, b)
    }
}
```

### Exercise 3: same problem, enum

Rewrite exercise 2 with an enum `Operation { Addition, Division }`. Which version lets a user of your library add multiplication without changing your code?

Answer: the `dyn` version. With the enum, adding an operation requires modifying the enum and its `match`.

## 16. Key points

- `Box<T>` places a value on the heap and owns it; it is used for recursive types, large values and trait objects.
- `dyn Trait` is always handled behind a pointer: `&dyn`, `Box<dyn>`, `Arc<dyn>`.
- A pointer to a trait object is two words wide: the data and the vtable.
- A trait is only usable as `dyn` without `Self` in a return type, without generic methods, and without functions lacking `self` (except `where Self: Sized`).
- `Clone` on a trait object: a `boxed_clone` method or the `dyn-clone` crate.
- Enum for a closed set, generic for a compile-time choice, `dyn` for an open set or one mixed at run time.
- Add `Send + Sync` to trait objects shared in a server.
