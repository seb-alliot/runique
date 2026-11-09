# Smart pointers

> Smart pointers own a value and add behaviour: sharing (`Rc`, `Arc`), interior mutability (`Cell`, `RefCell`), lazy initialisation (`OnceCell`, `LazyLock`). This course also shows the two traits that make them possible: `Deref` and `Drop`.

## Objectives

- Understand what makes a type a "smart pointer": `Deref` and `Drop`
- Write your own `Drop` and master the order of destruction
- Share a value between several owners with `Rc` and `Arc`
- Modify a value behind a shared reference with `Cell` and `RefCell`
- Combine `Rc<RefCell<T>>` and avoid leaks with `Weak`
- Initialise a value only once with `OnceCell`, `OnceLock`, `LazyLock`
- Choose the right tool for the need

## Table of contents

1. What a smart pointer is
2. The `Deref` trait
3. The `Drop` trait
4. `Rc<T>`: several owners
5. `Arc<T>`: several owners across threads
6. Interior mutability: `Cell<T>`
7. Interior mutability: `RefCell<T>`
8. `Rc<RefCell<T>>`
9. Cycles and `Weak<T>`
10. One-time initialisation: `OnceCell`, `OnceLock`, `LazyLock`
11. Copy on write: `Rc::make_mut`
12. Choosing
13. The same structure written in several ways
14. Exercises
15. Cheat sheet

## 1. What a smart pointer is

A reference `&T` borrows a value. A **smart pointer** **owns** it and adds behaviour. You already use some:

| Type | What it adds |
|---|---|
| `Box<T>` | value on the heap, owned alone (course `box-dynamiques`) |
| `String`, `Vec<T>` | growable heap buffer |
| `Rc<T>`, `Arc<T>` | several owners, freed with the last one |
| `RefCell<T>`, `Mutex<T>` | modification through a shared reference |
| `MutexGuard<T>`, `Ref<T>` | lock released automatically |

What makes them possible: `Deref` (be used like the value) and `Drop` (do something when freed).

## 2. The `Deref` trait

### 2.1 What it allows

```rust
let b = Box::new(String::from("abc"));
let n = b.len();               // method of String, called on a Box<String>
let s: &str = &b;              // &Box<String> becomes &String then &str
```

`Deref` says "I can be seen as a reference to `Target`". The compiler then inserts the conversions by itself (*deref coercion*), as many times as needed.

### 2.2 Implementing it

```rust
use std::ops::Deref;

struct Wrapper<T> {
    value: T,
    reads: std::cell::Cell<u32>,
}

impl<T> Deref for Wrapper<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.reads.set(self.reads.get() + 1);
        &self.value
    }
}

let w = Wrapper { value: vec![1, 2, 3], reads: std::cell::Cell::new(0) };
println!("{}", w.len());       // method of Vec
```

`DerefMut` does the same for `&mut`.

### 2.3 When not to implement it

`Deref` is made for **pointers**. Implementing it on a newtype to "inherit" the methods of the inner type is discouraged: all the methods become accessible, including those that bypass the newtype's invariant.

```rust
struct Email(String);

// Avoid: email.clear() or email.push_str("x") would become possible
// impl Deref for Email { type Target = String; ... }

// Prefer exposing what makes sense
impl Email {
    fn as_str(&self) -> &str { &self.0 }
}
```

## 3. The `Drop` trait

### 3.1 Writing a `Drop`

```rust
struct Connection { name: String }

impl Drop for Connection {
    fn drop(&mut self) {
        println!("closing {}", self.name);
    }
}

fn main() {
    let _c = Connection { name: "database".into() };
    println!("working");
}
// working
// closing database
```

`drop` is called **automatically** when the value goes out of scope, even if the program panics (stack unwinding). It is used to release what Rust does not know about: file, lock, connection, temporary file, counter to decrement.

### 3.2 Order of destruction

```rust
struct Name(&'static str);
impl Drop for Name {
    fn drop(&mut self) { println!("drop {}", self.0); }
}

struct Pair { a: Name, b: Name }

fn main() {
    let _x = Name("x");
    let _y = Name("y");
    let _p = Pair { a: Name("a"), b: Name("b") };
}
// drop a, drop b      (fields: declaration order, after the drop of Pair itself)
// drop y, drop x      (variables: reverse declaration order)
```

| What is dropped | Order |
|---|---|
| variables of a block | reverse of declaration |
| fields of a struct | declaration order |
| elements of a `Vec` or an array | first to last |
| temporaries of a statement | at the end of the statement (`;`) |

### 3.3 Dropping earlier

```rust
let c = Connection { name: "database".into() };
// c.drop();               // ERROR E0040: explicit use of destructor method
drop(c);                   // OK: std::mem::drop takes the value
```

You cannot call the `drop` method yourself (it would be called again at the end of the block, so two drops). `drop(c)` moves `c` into an empty function that frees it.

### 3.4 Lock: the drop matters

```rust
use std::sync::Mutex;

let m = Mutex::new(0);
{
    let mut guard = m.lock().unwrap();   // lock taken
    *guard += 1;
}                                        // guard dropped: lock released

let mut guard = m.lock().unwrap();
*guard += 1;
drop(guard);                             // release before a long operation
// long_operation();
```

The lock of a `Mutex` is a `MutexGuard` whose `Drop` releases the lock. Keeping a guard longer than necessary blocks all the other threads.

### 3.5 Cleanup guard: run code on exit, whatever happens

```rust
struct OnExit<F: FnMut()>(F);

impl<F: FnMut()> Drop for OnExit<F> {
    fn drop(&mut self) { (self.0)(); }
}

fn process(fail: bool) -> Result<(), String> {
    let _cleanup = OnExit(|| println!("temporary file removed"));
    if fail {
        return Err("failure in the middle".into());   // the cleanup runs anyway
    }
    Ok(())
}
```

### 3.6 Rules

- A type that implements `Drop` cannot be `Copy`.
- You cannot move a field out of a struct that implements `Drop` (the struct would be incomplete at the time of the `drop`); use `std::mem::take` or `Option::take`.
- `drop` must not panic: a panic during stack unwinding aborts the program.
- `std::mem::forget(x)` prevents the `drop` (deliberate leak); so does `ManuallyDrop<T>`, field by field.

## 4. `Rc<T>`: several owners

### 4.1 The problem

```rust
let a = String::from("shared");
let list1 = vec![a];
// let list2 = vec![a];      // ERROR: a has already been moved
```

Sometimes a value must belong to **several** structures at once (a node referenced by several parents, a configuration shared by several components).

### 4.2 `Rc`: reference counting

```rust
use std::rc::Rc;

let a = Rc::new(String::from("shared"));
let b = Rc::clone(&a);         // same String, counter at 2
let c = a.clone();             // same thing, shorter to write
println!("{}", Rc::strong_count(&a));   // 3
drop(b);
println!("{}", Rc::strong_count(&a));   // 2
// the String is freed when the last Rc disappears
```

`Rc::clone` does not copy the value: it increments a counter. `Rc::clone(&a)` is preferred to `a.clone()` in most code, so that it is obvious at first glance that this is not an expensive copy.

### 4.3 `Rc` is read-only

```rust
let r = Rc::new(vec![1, 2]);
// r.push(3);                 // ERROR: cannot borrow data in an `Rc` as mutable
```

Several owners that could all modify: this is exactly what the borrowing rules forbid. To modify, you need interior mutability (section 8).

### 4.4 Limits

- `Rc` is **not** `Send`: it cannot move to another thread (the counter is not protected). For several threads: `Arc`.
- Two `Rc` pointing at each other are never freed (section 9).

### 4.5 Getting the value back

```rust
let r = Rc::new(String::from("x"));
match Rc::try_unwrap(r) {
    Ok(s) => println!("sole owner: {s}"),     // possible if the counter is 1
    Err(r) => println!("still shared"),
}
```

## 5. `Arc<T>`: several owners across threads

```rust
use std::sync::Arc;
use std::thread;

let config = Arc::new(vec!["a", "b"]);
let mut handles = Vec::new();
for i in 0..3 {
    let c = Arc::clone(&config);
    handles.push(thread::spawn(move || println!("{i}: {:?}", c)));
}
for h in handles {
    h.join().unwrap();
}
```

`Arc` (*atomic reference counted*) has the same interface as `Rc`, with an atomic counter, so it can be shared between threads. The cost: each `clone` and each drop performs an atomic operation, slightly slower.

| | `Rc<T>` | `Arc<T>` |
|---|---|---|
| Threads | a single one | several (if `T: Send + Sync`) |
| Counter | plain | atomic |
| Modification | with `RefCell` | with `Mutex` or `RwLock` |

The details of sharing between threads are in `concurrence`.

## 6. Interior mutability: `Cell<T>`

A `Cell<T>` lets you modify a value through a **shared** reference (`&`), by replacing the whole value.

```rust
use std::cell::Cell;

struct Counter {
    calls: Cell<u32>,
}

impl Counter {
    fn count(&self) {                        // &self, not &mut self
        self.calls.set(self.calls.get() + 1);
    }
}

let c = Counter { calls: Cell::new(0) };
c.count();
c.count();
println!("{}", c.calls.get());   // 2
```

| Method | Effect |
|---|---|
| `get()` | copy of the value (if `T: Copy`) |
| `set(v)` | replaces |
| `replace(v)` | replaces and returns the old one |
| `take()` | takes the value and leaves `Default` |

`Cell` **never** gives a reference to its content: it is impossible to create two conflicting borrows, so there is no runtime check. Ideal for a counter, a flag, a small `Copy` value.

## 7. Interior mutability: `RefCell<T>`

### 7.1 Borrowing at runtime

```rust
use std::cell::RefCell;

let v = RefCell::new(vec![1, 2]);
v.borrow_mut().push(3);                  // temporary mutable borrow
println!("{:?}", v.borrow());            // temporary shared borrow
```

`RefCell` applies **the same** borrowing rules (one write or several reads), but **at runtime** instead of compile time.

### 7.2 Breaking the rule: panic

```rust
let v = RefCell::new(vec![1]);
let read = v.borrow();
// let write = v.borrow_mut();       // PANIC: already borrowed: BorrowMutError
drop(read);
let write = v.borrow_mut();           // OK
```

### 7.3 Version that does not panic

```rust
match v.try_borrow_mut() {
    Ok(mut w) => w.push(4),
    Err(_) => println!("already borrowed elsewhere"),
}
```

### 7.4 Limiting the duration of borrows

```rust
let data = RefCell::new(vec![1, 2, 3]);

// Risky: the borrow lives for the whole block
{
    let read = data.borrow();
    for x in read.iter() {
        // data.borrow_mut().push(*x);   // panics
    }
}

// Safe: copy what you need, then release
let copy: Vec<i32> = data.borrow().clone();
for x in copy {
    data.borrow_mut().push(x);
}
```

The borrows of `RefCell` (`Ref`, `RefMut`) are released at the end of their scope, like a lock.

### 7.5 `Cell`, `RefCell` or `&mut`?

Always start with `&mut self`: the compiler checks everything. Move to interior mutability only when the structure requires it: value shared through `Rc`, cache in a `&self` method, mock object in a test (course `tests-rust`).

`Cell` and `RefCell` are not `Sync`: for several threads, use `Mutex`, `RwLock` or the atomic types (`AtomicU32`...).

## 8. `Rc<RefCell<T>>`

Combining the two gives a value that is **shared and modifiable** in a single thread.

```rust
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
struct Account { balance: i64 }

let account = Rc::new(RefCell::new(Account { balance: 100 }));
let holder = Rc::clone(&account);
let bank = Rc::clone(&account);

holder.borrow_mut().balance -= 30;
bank.borrow_mut().balance += 5;
println!("{:?}", account.borrow());      // Account { balance: 75 }
```

| Role | Type |
|---|---|
| several owners | `Rc` |
| modification | `RefCell` |
| both, in one thread | `Rc<RefCell<T>>` |
| both, across threads | `Arc<Mutex<T>>` or `Arc<RwLock<T>>` |

If your code fills up with `Rc<RefCell<...>>`, it is often a sign that the architecture needs rethinking: a single owner with indexes (section 13) is generally simpler.

## 9. Cycles and `Weak<T>`

### 9.1 The leak

```rust
use std::cell::RefCell;
use std::rc::Rc;

struct Node {
    next: RefCell<Option<Rc<Node>>>,
}

let a = Rc::new(Node { next: RefCell::new(None) });
let b = Rc::new(Node { next: RefCell::new(Some(Rc::clone(&a))) });
*a.next.borrow_mut() = Some(Rc::clone(&b));    // a -> b -> a
// at the end of the block, each counter stays at 1: neither is ever freed
```

Rust prevents invalid accesses, not leaks: a cycle of `Rc` is safe but never frees its memory.

### 9.2 `Weak`: a reference that does not retain

```rust
use std::rc::{Rc, Weak};

let strong = Rc::new(5);
let weak: Weak<i32> = Rc::downgrade(&strong);

println!("{:?}", weak.upgrade());   // Some(5): the value still exists
drop(strong);
println!("{:?}", weak.upgrade());   // None: freed
```

A `Weak` does not count toward freeing. `upgrade()` returns an `Rc` if the value still exists, `None` otherwise.

### 9.3 Tree with a link to the parent

```rust
use std::cell::RefCell;
use std::rc::{Rc, Weak};

#[derive(Debug)]
struct Node {
    value: i32,
    parent: RefCell<Weak<Node>>,          // weak: the child does not retain the parent
    children: RefCell<Vec<Rc<Node>>>,     // strong: the parent retains its children
}

let leaf = Rc::new(Node { value: 3, parent: RefCell::new(Weak::new()), children: RefCell::new(vec![]) });
let branch = Rc::new(Node {
    value: 5,
    parent: RefCell::new(Weak::new()),
    children: RefCell::new(vec![Rc::clone(&leaf)]),
});
*leaf.parent.borrow_mut() = Rc::downgrade(&branch);

let parent = leaf.parent.borrow().upgrade().map(|p| p.value);   // Some(5)
```

Rule: links that go **downward** (owner → owned) are strong, those that go **back up** are weak.

## 10. One-time initialisation: `OnceCell`, `OnceLock`, `LazyLock`

### 10.1 The same lazy value, written in three ways

```rust
use std::collections::HashMap;
use std::sync::{LazyLock, OnceLock};

// 1. LazyLock (Rust 1.80+): the initialisation function is given at the declaration
static COUNTRIES: LazyLock<HashMap<&str, &str>> = LazyLock::new(|| {
    HashMap::from([("fr", "France"), ("be", "Belgium")])
});

// 2. OnceLock: the value is set later, only once
static CONFIG: OnceLock<String> = OnceLock::new();

fn init(url: String) {
    CONFIG.set(url).expect("configuration already initialised");
}

fn config() -> &'static str {
    CONFIG.get().map(String::as_str).unwrap_or("not configured")
}

// 3. OnceLock with get_or_init: computed on first request, from anywhere
fn email_regex() -> &'static regex::Regex {
    static R: OnceLock<regex::Regex> = OnceLock::new();
    R.get_or_init(|| regex::Regex::new(r"^[^@\s]+@[^@\s]+$").unwrap())
}

fn main() {
    println!("{}", COUNTRIES["fr"]);
    init("postgres://...".into());
    println!("{}", config());
}
```

| Type | Threads | Value provided |
|---|---|---|
| `std::cell::OnceCell<T>` | a single one | later (`set`, `get_or_init`) |
| `std::cell::LazyCell<T>` | a single one | function at the declaration |
| `std::sync::OnceLock<T>` | several | later |
| `std::sync::LazyLock<T>` | several | function at the declaration |

These types replace the `lazy_static` and `once_cell` crates in recent code.

### 10.2 In a struct

```rust
use std::cell::OnceCell;

struct Document {
    text: String,
    word_count: OnceCell<usize>,          // computed on first request, then kept
}

impl Document {
    fn word_count(&self) -> usize {
        *self.word_count.get_or_init(|| self.text.split_whitespace().count())
    }
}
```

## 11. Copy on write: `Rc::make_mut`

```rust
use std::rc::Rc;

let mut a = Rc::new(vec![1, 2, 3]);
let b = Rc::clone(&a);

Rc::make_mut(&mut a).push(4);      // a is shared: the value is COPIED then modified
println!("{:?} {:?}", a, b);       // [1, 2, 3, 4] [1, 2, 3]

Rc::make_mut(&mut a).push(5);      // a is now alone: modified in place, no copy
```

What it is for: sharing a large value as long as nobody modifies it, and paying for the copy only at the time of a modification. `Arc::make_mut` does the same across threads.

## 12. Choosing

| Need | Tool |
|---|---|
| A single owner, value on the heap | `Box<T>` |
| Several owners, read-only, one thread | `Rc<T>` |
| Several owners, read-only, several threads | `Arc<T>` |
| Modify a small `Copy` value behind `&` | `Cell<T>` |
| Modify any value behind `&`, one thread | `RefCell<T>` |
| Shared and modifiable, one thread | `Rc<RefCell<T>>` |
| Shared and modifiable, several threads | `Arc<Mutex<T>>`, `Arc<RwLock<T>>` |
| Counter or flag shared between threads | `AtomicU64`, `AtomicBool` |
| Link that must not retain the value | `Weak<T>` |
| Value computed once | `OnceLock`, `LazyLock` (`OnceCell`, `LazyCell` in one thread) |
| Action guaranteed on leaving a block | type with `Drop` |

## 13. The same structure written in several ways

A graph of users who follow each other.

```rust
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

// 1. Rc + RefCell + Weak: pointers between nodes
struct User1 {
    name: String,
    follows: RefCell<Vec<Weak<User1>>>,
}

// 2. Indexes in a Vec: a single owner, numbers as links
struct Network2 {
    names: Vec<String>,
    follows: Vec<Vec<usize>>,          // follows[i] = indexes of the people followed by i
}

impl Network2 {
    fn add(&mut self, name: &str) -> usize {
        self.names.push(name.to_string());
        self.follows.push(Vec::new());
        self.names.len() - 1
    }
    fn follow(&mut self, who: usize, target: usize) {
        self.follows[who].push(target);
    }
}

// 3. Identifiers in a HashMap: like in a database
struct Network3 {
    users: HashMap<u64, String>,
    follows: HashMap<u64, Vec<u64>>,
}
```

| Version | Advantages | Drawbacks |
|---|---|---|
| 1 | close to an object model | `borrow()` everywhere, risk of cycles, possible runtime panics |
| 2 | simple, fast, no borrows to manage | an index can become invalid after a removal |
| 3 | simple removals, database-like model | one lookup per link |

In most Rust programs, versions 2 and 3 are preferred to version 1.

## 14. Exercises

### Exercise 1: closing log

Write a `Timer` type that prints, when it is dropped, the time elapsed since its creation.

```rust
// Solution:
use std::time::Instant;

struct Timer { name: &'static str, start: Instant }

impl Timer {
    fn new(name: &'static str) -> Self { Self { name, start: Instant::now() } }
}

impl Drop for Timer {
    fn drop(&mut self) {
        println!("{}: {:?}", self.name, self.start.elapsed());
    }
}

fn main() {
    let _t = Timer::new("computation");
    let _s: u64 = (0..1_000_000).sum();
}   // prints the duration, even if the computation panics
```

### Exercise 2: shared list

Two `Team` structs share the same list of members and must both be able to add to it. Choose the type and write it.

```rust
// Solution:
use std::cell::RefCell;
use std::rc::Rc;

struct Team { name: String, members: Rc<RefCell<Vec<String>>> }

fn main() {
    let shared = Rc::new(RefCell::new(Vec::new()));
    let a = Team { name: "A".into(), members: Rc::clone(&shared) };
    let b = Team { name: "B".into(), members: Rc::clone(&shared) };
    a.members.borrow_mut().push("Ada".into());
    b.members.borrow_mut().push("Linus".into());
    assert_eq!(shared.borrow().len(), 2);
}
```

### Exercise 3: cache in a `&self` method

Add to `struct Article { text: String }` a method `summary(&self) -> &str` that computes the first 50 characters only once.

```rust
// Solution:
use std::cell::OnceCell;

struct Article { text: String, summary: OnceCell<String> }

impl Article {
    fn summary(&self) -> &str {
        self.summary.get_or_init(|| self.text.chars().take(50).collect())
    }
}
```

## 15. Cheat sheet

```rust
impl Drop for T { fn drop(&mut self) { .. } }     drop(x)
impl Deref for T { type Target = U; fn deref(&self) -> &U { .. } }

Rc::new(v)   Rc::clone(&r)   Rc::strong_count(&r)   Rc::downgrade(&r)   Rc::try_unwrap(r)   Rc::make_mut(&mut r)
Arc::new(v)  Arc::clone(&a)
weak.upgrade()                                     // Option<Rc<T>>
Cell::new(v)   c.get()   c.set(v)   c.replace(v)   c.take()
RefCell::new(v)   r.borrow()   r.borrow_mut()   r.try_borrow_mut()
OnceLock::new()   o.set(v)   o.get()   o.get_or_init(|| v)
static X: LazyLock<T> = LazyLock::new(|| v);
```

### Key points

- `Deref` makes a pointer transparent; keep it for pointers, not for newtypes.
- `Drop` always runs, even on panic: ideal for releasing a resource.
- Variables are dropped in reverse order, fields in declaration order.
- `Rc` shares within a thread, `Arc` across threads; `clone` only copies the pointer.
- `Cell` for small `Copy` values, `RefCell` for the rest; `RefCell` panics if the rules are broken.
- A cycle of `Rc` leaks: use `Weak` for links that go back up.
- `LazyLock` and `OnceLock` replace `lazy_static` and `once_cell`.
- Before `Rc<RefCell<T>>`, consider a single owner and indexes.
