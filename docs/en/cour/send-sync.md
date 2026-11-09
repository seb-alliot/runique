# Send & Sync

> The two traits that let the compiler guarantee the absence of data races: what they mean, which types have them, and how to fix the "cannot be sent between threads safely" errors.

## Objectives

- Understand what `Send` and `Sync` mean, and the link between the two
- Know which standard types implement them and why
- Read and fix `Send`/`Sync` errors with `thread::spawn` and `tokio::spawn`
- Avoid holding a non-`Send` value across an `.await`
- Declare trait objects and shared state correctly for a server
- Know the cases where you implement `Send` or `Sync` by hand, and the danger

## Table of contents

1. The problem: the data race
2. `Send`
3. `Sync`
4. Automatic traits
5. Which types are `Send` and `Sync`
6. `Send` and `Sync` in signatures
7. Threads: fixing errors
8. Async: values held across an `.await`
9. Trait objects and shared server state
10. Sharing a value between threads: all the ways
11. Implementing `Send` or `Sync` by hand
12. Exercises
13. Cheat sheet

## 1. The problem: the data race

A **data race** happens when two threads access the same memory at the same time, at least one of them writes, and nothing synchronizes the accesses. The result is unpredictable: corrupted values, random crashes, vulnerabilities.

```rust
// In C or Java, this compiles and randomly gives a wrong total:
// let mut total = 0;
// for _ in 0..10 { spawn(|| total += 1); }
```

In Rust, this code **does not compile**. The compiler relies on two marker traits, `Send` and `Sync`, to know what can cross a boundary between threads.

## 2. `Send`

> A type is `Send` if a value of that type can be **moved** to another thread.

```rust
use std::thread;

let v = vec![1, 2, 3];              // Vec<i32> is Send
let h = thread::spawn(move || {
    println!("{v:?}");              // v was moved into the new thread
});
h.join().unwrap();
```

```rust
use std::rc::Rc;

let r = Rc::new(5);
// thread::spawn(move || println!("{r}"));
// ERROR E0277: `Rc<i32>` cannot be sent between threads safely
```

`Rc` is not `Send`: its reference counter is not protected. If two threads each cloned or dropped their copy of the same `Rc` at the same time, the counter would be wrong and the value would be freed twice, or never.

## 3. `Sync`

> A type `T` is `Sync` if a **reference** `&T` can be shared between several threads.

In other words: `T: Sync` ⟺ `&T: Send`.

```rust
use std::sync::Arc;
use std::thread;

let counter = Arc::new(std::sync::atomic::AtomicU32::new(0));   // AtomicU32 is Sync
for _ in 0..4 {
    let c = Arc::clone(&counter);
    thread::spawn(move || {
        c.fetch_add(1, std::sync::atomic::Ordering::Relaxed);    // several threads, same value
    });
}
```

```rust
use std::cell::RefCell;

let r = RefCell::new(0);
// sharing &r between threads is rejected: RefCell is not Sync
```

`RefCell` counts its borrows without synchronization: two threads could each get a `borrow_mut()` at the same time.

## 4. Automatic traits

You almost never implement `Send` or `Sync` yourself. They are **automatic traits**: a type has them if **all its fields** have them.

```rust
struct User {
    name: String,     // Send + Sync
    age: u32,         // Send + Sync
}
// User is automatically Send + Sync

struct Session {
    data: std::rc::Rc<String>,   // neither Send nor Sync
}
// Session is neither Send nor Sync
```

A single non-`Send` field is enough to remove `Send` from the whole struct, and from any struct that contains it.

## 5. Which types are `Send` and `Sync`

| Type | `Send` | `Sync` | Why |
|---|---|---|---|
| `i32`, `bool`, `String`, `Vec<T>`, `Box<T>` | yes | yes | (if `T` is) owned data, no hidden state |
| `&T` | if `T: Sync` | if `T: Sync` | |
| `&mut T` | if `T: Send` | if `T: Sync` | |
| `Rc<T>` | **no** | **no** | non-atomic counter |
| `Arc<T>` | if `T: Send + Sync` | if `T: Send + Sync` | atomic counter |
| `Cell<T>`, `RefCell<T>` | if `T: Send` | **no** | modification without synchronization |
| `Mutex<T>` | if `T: Send` | if `T: Send` | the lock serializes accesses |
| `RwLock<T>` | if `T: Send` | if `T: Send + Sync` | several simultaneous readers |
| `MutexGuard<'_, T>` (std) | **no** | if `T: Sync` | must be released by the thread that took it |
| `AtomicU32`, `AtomicBool`... | yes | yes | atomic operations |
| `*const T`, `*mut T` | **no** | **no** | the compiler knows nothing about what they point to |
| `tokio::sync::Mutex<T>` and its guard | yes (if `T: Send`) | yes | designed for async |

Two combinations to remember:

- `Arc<Mutex<T>>`: share **and** modify between threads. `Mutex` makes `T` `Sync`, `Arc` shares it.
- `Arc<RefCell<T>>`: rejected by `spawn`, because `RefCell` is not `Sync`.

## 6. `Send` and `Sync` in signatures

### 6.1 `thread::spawn`

```rust
pub fn spawn<F, T>(f: F) -> JoinHandle<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
```

- the closure is **sent** to the new thread: `Send`;
- it can outlive the caller: `'static` (no temporary borrow);
- the result comes back to the thread that calls `join`: `T: Send`.

### 6.2 `tokio::spawn`

```rust
pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
```

Tokio's multithreaded runtime can move a task from one thread to another at each `.await`: everything the future holds between two `.await` points must be `Send`.

### 6.3 Requiring `Send` in your own functions

```rust
fn in_background<T: Send + 'static>(value: T, process: fn(T)) {
    std::thread::spawn(move || process(value));
}
```

## 7. Threads: fixing errors

### 7.1 "cannot be sent between threads safely"

```rust
use std::rc::Rc;
use std::sync::Arc;

// Rejected: Rc
// let r = Rc::new(vec![1]);
// std::thread::spawn(move || println!("{r:?}"));

// Fix 1: Arc instead of Rc
let a = Arc::new(vec![1]);
let a2 = Arc::clone(&a);
std::thread::spawn(move || println!("{a2:?}"));

// Fix 2: move an owned value
let v = vec![1];
std::thread::spawn(move || println!("{v:?}"));
```

### 7.2 "cannot be shared between threads safely"

```rust
use std::sync::{Arc, Mutex};

// Rejected: Arc<RefCell<i32>> (RefCell is not Sync)
// let c = Arc::new(std::cell::RefCell::new(0));

let c = Arc::new(Mutex::new(0));                 // Mutex instead of RefCell
let mut handles = vec![];
for _ in 0..4 {
    let c = Arc::clone(&c);
    handles.push(std::thread::spawn(move || {
        *c.lock().unwrap() += 1;
    }));
}
for h in handles {
    h.join().unwrap();
}
assert_eq!(*c.lock().unwrap(), 4);
```

### 7.3 "borrowed value does not live long enough" / `'static`

```rust
let data = vec![1, 2, 3];

// Rejected: the closure borrows data, which could disappear before the thread ends
// std::thread::spawn(|| println!("{data:?}"));

// 1. move: the thread owns the value
// 2. Arc: each thread owns a shared pointer
// 3. thread::scope: the threads cannot outlive the block, so borrowing is allowed
std::thread::scope(|s| {
    s.spawn(|| println!("{data:?}"));
    s.spawn(|| println!("{}", data.len()));
});
println!("{data:?}");          // still available
```

`thread::scope` waits for all its threads to finish before returning: they can therefore borrow local variables, without `Arc` or `'static`.

### 7.4 Reading the error message

```text
error[E0277]: `Rc<String>` cannot be sent between threads safely
   |
   = help: within `Session`, the trait `Send` is not implemented for `Rc<String>`
note: required because it appears within the type `Session`
note: required by a bound in `spawn`
```

The compiler follows the chain back: it names the faulty **field** (`Rc<String>`), the struct that contains it (`Session`) and the function that requires `Send` (`spawn`). You fix the field, not the call.

## 8. Async: values held across an `.await`

### 8.1 The std lock held during an `.await`

```rust
use std::sync::{Arc, Mutex};

async fn update(state: Arc<Mutex<Vec<String>>>) {
    let mut v = state.lock().unwrap();        // MutexGuard (std): not Send
    v.push("a".into());
    // send_notification().await;             // ERROR with tokio::spawn: future is not `Send`
}
```

The guard is held **across** the `.await`: it would be part of the future's state, and the future would no longer be `Send`. Also, holding a blocking lock during a wait blocks the other tasks.

### 8.2 Three fixes

```rust
use std::sync::{Arc, Mutex};

async fn send_notification() {}

// 1. Release the lock before the .await, in a block
async fn version1(state: Arc<Mutex<Vec<String>>>) {
    {
        let mut v = state.lock().unwrap();
        v.push("a".into());
    }                                         // guard released here
    send_notification().await;
}

// 2. Copy what you need, then release
async fn version2(state: Arc<Mutex<Vec<String>>>) {
    let count = state.lock().unwrap().len();  // the guard is released at the end of the statement
    if count > 10 {
        send_notification().await;
    }
}

// 3. Async lock when it MUST be held during the wait
async fn version3(state: Arc<tokio::sync::Mutex<Vec<String>>>) {
    let mut v = state.lock().await;           // Send guard
    v.push("a".into());
    send_notification().await;                // allowed, but the other tasks wait
}
```

Prefer versions 1 and 2: a `std::sync::Mutex` is faster than an async lock as long as it is never held during an `.await`.

### 8.3 `Rc` in an async function

```rust
async fn count() {
    let r = std::rc::Rc::new(5);
    // something().await;                     // r is still alive: the future is not Send
    println!("{r}");
}

async fn count_ok() {
    {
        let r = std::rc::Rc::new(5);
        println!("{r}");
    }                                          // r dropped before the wait
    // something().await;                     // OK
}
```

A non-`Send` variable is a problem only if it is **still alive** at the time of an `.await`.

### 8.4 Running a non-`Send` task

```rust
// A task tied to a single thread: LocalSet + spawn_local
async fn local() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            tokio::task::spawn_local(async {
                let r = std::rc::Rc::new(5);
                tokio::task::yield_now().await;      // allowed: the task never changes thread
                println!("{r}");
            })
            .await
            .unwrap();
        })
        .await;
}
```

## 9. Trait objects and shared server state

### 9.1 Adding `Send + Sync` to trait objects

```rust
trait Notifier {
    fn send(&self, msg: &str);
}

struct State {
    // notifier: Box<dyn Notifier>,                  // neither Send nor Sync: rejected by axum and tokio
    notifier: Box<dyn Notifier + Send + Sync>,        // shareable between tasks
}
```

A `dyn Trait` says nothing about its concrete type: the compiler cannot assume it is `Send`. You have to ask for it explicitly. Another way: make `Send + Sync` supertraits.

```rust
trait Notifier2: Send + Sync {
    fn send(&self, msg: &str);
}
struct State2 { notifier: Box<dyn Notifier2> }   // Send + Sync already included
```

This is what Runique does: `FormField` and `PasswordHandler` have `Send + Sync` in their supertraits.

### 9.2 The state of an axum application

```rust
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    config: Arc<Config>,               // read-only, shared
    counter: Arc<std::sync::atomic::AtomicU64>,
    cache: Arc<tokio::sync::RwLock<std::collections::HashMap<String, String>>>,
}

struct Config { name: String }
```

axum requires the state to be `Clone + Send + Sync + 'static`: each request receives a clone, handled on any thread. With `Arc`, cloning the state only costs a few counters. Runique follows the same principle with `AEngine = Arc<RuniqueEngine>`.

## 10. Sharing a value between threads: all the ways

Counting events from several threads.

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

// 1. Arc<Mutex<T>>: general, for any data
fn v1() -> u64 {
    let total = Arc::new(Mutex::new(0u64));
    let hs: Vec<_> = (0..4).map(|_| {
        let t = Arc::clone(&total);
        thread::spawn(move || *t.lock().unwrap() += 1)
    }).collect();
    hs.into_iter().for_each(|h| h.join().unwrap());
    let r = *total.lock().unwrap();
    r
}

// 2. Atomic: for a simple number or a flag, no lock
fn v2() -> u64 {
    let total = Arc::new(AtomicU64::new(0));
    let hs: Vec<_> = (0..4).map(|_| {
        let t = Arc::clone(&total);
        thread::spawn(move || { t.fetch_add(1, Ordering::Relaxed); })
    }).collect();
    hs.into_iter().for_each(|h| h.join().unwrap());
    total.load(Ordering::Relaxed)
}

// 3. Channel: the threads send, a single owner adds up (no sharing at all)
fn v3() -> u64 {
    let (tx, rx) = mpsc::channel();
    for _ in 0..4 {
        let tx = tx.clone();
        thread::spawn(move || tx.send(1u64).unwrap());
    }
    drop(tx);                              // closes the channel when all senders are done
    rx.iter().sum()
}

// 4. Scoped threads: each thread computes its share, the results are added
fn v4() -> u64 {
    thread::scope(|s| {
        let hs: Vec<_> = (0..4).map(|_| s.spawn(|| 1u64)).collect();
        hs.into_iter().map(|h| h.join().unwrap()).sum()
    })
}
```

| Version | When |
|---|---|
| 1 `Arc<Mutex<T>>` | complex shared state, modified by several threads |
| 2 atomic | counter, flag, identifier to increment |
| 3 channel | producers and consumer, no common state |
| 4 scope + return | split a computation, gather the results |

"Do not communicate by sharing memory; share memory by communicating": versions 3 and 4 avoid any lock.

## 11. Implementing `Send` or `Sync` by hand

### 11.1 Removing `Send` on purpose

```rust
use std::marker::PhantomData;

struct ThreadBound {
    _not_send: PhantomData<*const ()>,    // a raw pointer is neither Send nor Sync
}
```

What it is for: a type that represents a resource tied to the thread that created it (graphics context, handle of a non-thread-safe C library).

### 11.2 Adding `Send`: `unsafe impl`

```rust
struct Buffer {
    ptr: *mut u8,          // raw pointer: makes the type neither Send nor Sync
    len: usize,
}

// SAFETY: Buffer alone owns the memory pointed to by ptr, no other alias exists.
unsafe impl Send for Buffer {}
```

`unsafe impl` means: "I guarantee what the compiler cannot check". A mistake here creates exactly the data races that Rust prevents elsewhere. You write it only to wrap a low-level resource whose **every** access you control, with a `SAFETY` comment explaining why it is safe.

In ordinary application code, you **never** need `unsafe impl Send`: if the compiler refuses, the fix is to change the type (`Rc` → `Arc`, `RefCell` → `Mutex`).

## 12. Exercises

### Exercise 1 — fix it

This code does not compile. Fix it in two ways.

```rust
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    let list = Rc::new(RefCell::new(Vec::new()));
    let l = Rc::clone(&list);
    std::thread::spawn(move || l.borrow_mut().push(1)).join().unwrap();
}
```

```rust
// Solution 1: Arc + Mutex
use std::sync::{Arc, Mutex};
fn main() {
    let list = Arc::new(Mutex::new(Vec::new()));
    let l = Arc::clone(&list);
    std::thread::spawn(move || l.lock().unwrap().push(1)).join().unwrap();
}

// Solution 2: the thread returns its value, no sharing
fn main2() {
    let mut list = Vec::new();
    let x = std::thread::spawn(|| 1).join().unwrap();
    list.push(x);
}
```

### Exercise 2 — non-`Send` future

Why can this function not be passed to `tokio::spawn`? Fix it.

```rust
async fn add(state: std::sync::Arc<std::sync::Mutex<Vec<u32>>>) {
    let mut v = state.lock().unwrap();
    v.push(1);
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    v.push(2);
}
```

```rust
// Solution: the std MutexGuard (not Send) is held across the .await.
async fn add(state: std::sync::Arc<std::sync::Mutex<Vec<u32>>>) {
    state.lock().unwrap().push(1);
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    state.lock().unwrap().push(2);
}
```

Warning: between the two `push` calls, another task can modify the `Vec`. If both additions must be atomic together, you need a `tokio::sync::Mutex` held during the wait.

### Exercise 3 — shared trait object

Declare an application state that contains a `Box<dyn Storage>` usable from several tokio tasks.

```rust
// Solution:
trait Storage: Send + Sync {
    fn read(&self, key: &str) -> Option<String>;
}

#[derive(Clone)]
struct State {
    storage: std::sync::Arc<dyn Storage>,
}
```

`Arc` rather than `Box`: the state must be `Clone`, and a `Box<dyn Storage>` is not.

## 13. Cheat sheet

```text
T: Send   → a value T can be moved to another thread
T: Sync   → &T can be shared between threads   (T: Sync ⟺ &T: Send)

Rc        → neither Send nor Sync     → Arc
RefCell   → not Sync                  → Mutex / RwLock
Cell<u32> → not Sync                  → AtomicU32
MutexGuard (std) across .await        → release before, or tokio::sync::Mutex
Box<dyn Trait>                        → Box<dyn Trait + Send + Sync>
borrow in thread::spawn               → move, Arc, or thread::scope
```

### Key points

- `Send`: movable to another thread. `Sync`: shareable by reference.
- They are automatic traits: a type has them if all its fields have them.
- `Rc`, `RefCell`, `Cell` and raw pointers block; `Arc`, `Mutex`, `RwLock` and atomics unblock.
- A future must be `Send` for `tokio::spawn`: nothing non-`Send` may be alive at the time of an `.await`.
- Shared trait objects must say `+ Send + Sync`.
- The error message names the faulty field: that is what you change.
- `unsafe impl Send` is reserved for low-level resources whose every access you control.
