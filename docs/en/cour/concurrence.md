# Concurrency & shared state
> Threads, locks, atomics, channels and one-time initialization: share data between several threads without data races, deadlocks or leaks.

## Objectives

- Spawn threads, pass them data and get their result back
- Share a value with `Arc` and protect it with `Mutex` or `RwLock`
- Understand lock poisoning and know how to handle it
- Use atomic types for counters and flags
- Communicate through channels rather than shared memory
- Initialize a global value only once with `LazyLock` and `OnceLock`
- Avoid deadlocks and stores that grow without end
- Pick the right tool for each situation

## Table of contents

1. Why concurrency is hard
2. Threads
3. `Arc<T>`: sharing
4. `Mutex<T>`: mutual exclusion
5. `RwLock<T>`: multiple readers
6. Atomics
7. Channels
8. `LazyLock<T>` and `OnceLock<T>`
9. Deadlocks
10. Parallel computation on data
11. Choosing
12. Examples in Runique
13. Pitfall: a shared store without cleanup
14. The same function written in several ways
15. Practical exercises
16. Cheat sheet

---

## 1. Why concurrency is hard

Three families of bugs appear when several threads touch the same data:

| Bug | Description | What Rust does |
|---|---|---|
| **data race** | two threads access the same memory, one writes, with no synchronization | does not compile (`Send`, `Sync`, course `send-sync`) |
| **logical race condition** | the order of operations changes the result (reading then writing in two steps) | possible: it is up to you to group the operations under a single lock |
| **deadlock** | two threads each wait for a lock the other one holds | possible: discipline on lock ordering (section 9) |

Rust removes the first family at compile time. The other two remain design errors.

```rust
// Logical race condition: compiles, but wrong
use std::sync::Mutex;
let stock = Mutex::new(1);

// Two threads do this at the same time:
let available = *stock.lock().unwrap() > 0;   // lock taken then released
if available {
    *stock.lock().unwrap() -= 1;              // the other thread may have slipped in between: stock = -1
}

// Correct: read and write under the same lock
let mut s = stock.lock().unwrap();
if *s > 0 {
    *s -= 1;
}
```

---

## 2. Threads

### 2.1 Spawn and wait

```rust
use std::thread;

let h = thread::spawn(|| {
    (1..=10).sum::<u32>()
});

let total = h.join().unwrap();       // wait for the end; Err if the thread panicked
```

### 2.2 Passing data: `move`

```rust
let v = vec![1, 2, 3];
let h = thread::spawn(move || v.iter().sum::<i32>());   // v belongs to the thread
let s = h.join().unwrap();
```

### 2.3 Several threads

```rust
let handles: Vec<_> = (0..4)
    .map(|i| thread::spawn(move || i * i))
    .collect();

let results: Vec<i32> = handles.into_iter().map(|h| h.join().unwrap()).collect();
```

### 2.4 Borrowing: `thread::scope`

```rust
let data = vec![1, 2, 3, 4, 5, 6];
let (left, right) = data.split_at(3);

let (a, b) = thread::scope(|s| {
    let ha = s.spawn(|| left.iter().sum::<i32>());   // borrows, with no move or Arc
    let hb = s.spawn(|| right.iter().sum::<i32>());
    (ha.join().unwrap(), hb.join().unwrap())
});
```

A thread spawned in a `scope` cannot outlive the block, so it can **borrow** local variables. This is often the simplest way to parallelize a computation.

### 2.5 Naming a thread

```rust
let h = thread::Builder::new()
    .name("cleanup".into())
    .spawn(|| { /* ... */ })
    .unwrap();
```

The name appears in panic messages and profiling tools.

### 2.6 Threads or async tasks

For thousands of network waits: async tasks (course `async-tokio`). For a few heavy computations in parallel: threads. The primitives in this course (`Arc`, `Mutex`, atomics, `LazyLock`) are useful in both cases.

---

## 3. `Arc<T>`: sharing

```rust
use std::sync::Arc;
use std::thread;

let config = Arc::new(vec!["a", "b"]);
let mut handles = vec![];
for i in 0..3 {
    let c = Arc::clone(&config);         // copy of the pointer, counter + 1
    handles.push(thread::spawn(move || println!("{i} : {:?}", c)));
}
for h in handles {
    h.join().unwrap();
}
println!("{}", Arc::strong_count(&config));   // 1: the threads released their copy
```

`Arc` gives **several owners** read-only access. To modify, you need a lock or an atomic **inside**: `Arc<Mutex<T>>`, `Arc<RwLock<T>>`, `Arc<AtomicU64>`.

| | `Rc<T>` | `Arc<T>` |
|---|---|---|
| Threads | one | several |
| Counter | plain | atomic (slightly slower) |

---

## 4. `Mutex<T>`: mutual exclusion

### 4.1 Taking the lock

```rust
use std::sync::Mutex;

let m = Mutex::new(0);
{
    let mut guard = m.lock().unwrap();   // only one thread at a time here
    *guard += 1;
}                                        // guard dropped: lock released
```

`lock()` returns a `MutexGuard`: it dereferences to the value, and its `Drop` releases the lock. You **cannot** forget to unlock.

### 4.2 Counter shared between threads

```rust
use std::sync::{Arc, Mutex};
use std::thread;

let counter = Arc::new(Mutex::new(0));
let handles: Vec<_> = (0..10)
    .map(|_| {
        let c = Arc::clone(&counter);
        thread::spawn(move || *c.lock().unwrap() += 1)
    })
    .collect();
for h in handles {
    h.join().unwrap();
}
assert_eq!(*counter.lock().unwrap(), 10);
```

### 4.3 Hold the lock as briefly as possible

```rust
// Bad: the lock is held during the whole processing
let list = Mutex::new(vec![1, 2, 3]);
{
    let guard = list.lock().unwrap();
    for x in guard.iter() {
        long_task(*x);                   // the other threads wait
    }
}

// Better: copy, release, process
let copy = list.lock().unwrap().clone();   // lock released at the end of the statement
for x in copy {
    long_task(x);
}

fn long_task(_: i32) {}
```

### 4.4 Poisoning

If a thread **panics** while holding the lock, the `Mutex` is marked "poisoned": the data may be in an inconsistent state. Later `lock()` calls return `Err(PoisonError)`.

```rust
use std::sync::Mutex;

let m = Mutex::new(vec![1]);

// 1. unwrap: propagate the panic (most common default choice)
let g = m.lock().unwrap();
drop(g);

// 2. Recover the data anyway, if it cannot be inconsistent
let g = match m.lock() {
    Ok(g) => g,
    Err(poisoned) => poisoned.into_inner(),
};
drop(g);

// 3. Short form of 2
let g = m.lock().unwrap_or_else(|e| e.into_inner());
```

Form 2 suits a cache or a counter table, where a half-written entry does no harm: continuing is better than stopping the service. This is what Runique does in its limiters (section 12).

### 4.5 Trying without waiting

```rust
match m.try_lock() {
    Ok(mut g) => g.push(2),
    Err(_) => println!("busy: we will try again later"),
}
```

### 4.6 Lock-free access when you are alone

```rust
let mut m = Mutex::new(5);
*m.get_mut().unwrap() += 1;        // &mut m proves no other access exists: no lock
let value = m.into_inner().unwrap();   // take the data out and destroy the Mutex
```

---

## 5. `RwLock<T>`: multiple readers

```rust
use std::sync::{Arc, RwLock};

let config = Arc::new(RwLock::new(String::from("v1")));

let reader = config.read().unwrap();      // several readers at the same time
println!("{}", *reader);
drop(reader);

let mut writer = config.write().unwrap(); // a single writer, no readers
*writer = String::from("v2");
```

| | `Mutex<T>` | `RwLock<T>` |
|---|---|---|
| Simultaneous readers | no | yes |
| Cost | lower | higher |
| For | frequent reads and writes, short sections | many reads, rare writes (configuration, cache) |
| Requirement on `T` to share | `Send` | `Send + Sync` |

An `RwLock` is not always faster: for very short sections, a `Mutex` is often better. And a writer can wait a long time if readers follow one another without a break.

---

## 6. Atomics

### 6.1 Counters and flags without a lock

```rust
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;

let counter = Arc::new(AtomicU64::new(0));
let handles: Vec<_> = (0..10)
    .map(|_| {
        let c = Arc::clone(&counter);
        thread::spawn(move || {
            c.fetch_add(1, Ordering::Relaxed);
        })
    })
    .collect();
for h in handles {
    h.join().unwrap();
}
assert_eq!(counter.load(Ordering::Relaxed), 10);

static STOP: AtomicBool = AtomicBool::new(false);
STOP.store(true, Ordering::Release);
if STOP.load(Ordering::Acquire) { /* stop */ }
```

### 6.2 Operations

| Method | Effect |
|---|---|
| `load`, `store` | read, write |
| `fetch_add`, `fetch_sub` | add, subtract; returns the old value |
| `fetch_max`, `fetch_min` | keep the maximum, the minimum |
| `swap` | replace, returns the old value |
| `compare_exchange(expected, new, ..)` | replace only if the value is the expected one |

```rust
use std::sync::atomic::{AtomicU32, Ordering};

// Reserve a seat only if any are left: read and write in a single operation
fn reserve(seats: &AtomicU32) -> bool {
    seats
        .fetch_update(Ordering::AcqRel, Ordering::Acquire, |p| p.checked_sub(1))
        .is_ok()
}
```

### 6.3 `Ordering` in practice

| Ordering | Use |
|---|---|
| `Relaxed` | counter, statistic: only the number matters |
| `Release` (write) / `Acquire` (read) | a flag that publishes other data ("ready" after filling a buffer) |
| `AcqRel` | read-modify-write that does both |
| `SeqCst` | strict global order; the safest when in doubt |

An atomic protects **only one** value. To keep two related values consistent (a total and a count), you need a `Mutex`.

---

## 7. Channels

### 7.1 `std::sync::mpsc`

```rust
use std::sync::mpsc;
use std::thread;

let (tx, rx) = mpsc::channel();
for i in 0..3 {
    let tx = tx.clone();
    thread::spawn(move || tx.send(i * 10).unwrap());
}
drop(tx);                                   // close it: otherwise rx waits forever

let total: i32 = rx.iter().sum();           // receives until the channel closes
```

### 7.2 Bounded channel

```rust
let (tx, rx) = mpsc::sync_channel::<String>(10);   // beyond 10, send blocks
```

### 7.3 Shared memory or messages

```rust
// 1. Shared memory: several threads modify the same Vec under a lock
// 2. Messages: each thread sends its results; a single owner assembles them
```

Messages avoid locks and deadlocks: a single task owns the data. For async code, tokio provides its own channels (`mpsc`, `oneshot`, `broadcast`, `watch`, see `async-tokio`). The `crossbeam-channel` crate offers richer std-style channels (several receivers, `select!`).

---

## 8. `LazyLock<T>` and `OnceLock<T>`

### 8.1 `LazyLock`: global value computed on first access

```rust
use std::collections::HashMap;
use std::sync::LazyLock;

static CODES: LazyLock<HashMap<u16, &'static str>> = LazyLock::new(|| {
    HashMap::from([(404, "not found"), (500, "internal error")])
});

fn message(code: u16) -> &'static str {
    CODES.get(&code).copied().unwrap_or("unknown")
}
```

- The closure runs **only once**, on first access, even if several threads access it at the same time.
- Available since Rust 1.80: it replaces `lazy_static!` and `once_cell::sync::Lazy`.

### 8.2 Before Rust 1.80

```rust
// lazy_static (crate)
// lazy_static! { static ref CODES: HashMap<u16, &'static str> = { .. }; }

// once_cell (crate)
// static CODES: once_cell::sync::Lazy<HashMap<..>> = Lazy::new(|| ..);

// Today: std only
// static CODES: std::sync::LazyLock<HashMap<..>> = LazyLock::new(|| ..);
```

### 8.3 `OnceLock`: value set later, only once

```rust
use std::sync::OnceLock;

static BASE_URL: OnceLock<String> = OnceLock::new();

fn initialize(url: String) {
    if BASE_URL.set(url).is_err() {
        panic!("BASE_URL already initialized");
    }
}

fn base_url() -> &'static str {
    BASE_URL.get().map(String::as_str).expect("initialize() must be called at startup")
}

// Or: computed on first request, from a function
fn cache() -> &'static Vec<u32> {
    static C: OnceLock<Vec<u32>> = OnceLock::new();
    C.get_or_init(|| (1..=10).collect())
}
```

### 8.4 `LazyLock` or `OnceLock`

| | `LazyLock` | `OnceLock` |
|---|---|---|
| The value is known | at declaration (a function) | later (at startup, from configuration) |
| Initialization | automatic on first access | explicit `set`, or `get_or_init` |
| For | constant tables, regexes, reading the environment | configuration received at startup, value passed by `main` |

### 8.5 The lazy initialization pitfall

A `LazyLock` that reads an environment variable reads it **on first access**, not when the program starts. If the first access happens before `.env` is loaded, the value read is the one from before. And once computed, it does not change until the program ends: changing `.env` requires a restart.

---

## 9. Deadlocks

### 9.1 Two locks taken in a different order

```rust
use std::sync::{Arc, Mutex};
use std::thread;

let a = Arc::new(Mutex::new(0));
let b = Arc::new(Mutex::new(0));

let (a1, b1) = (Arc::clone(&a), Arc::clone(&b));
let t1 = thread::spawn(move || {
    let _ga = a1.lock().unwrap();
    let _gb = b1.lock().unwrap();      // waits for b
});

let (a2, b2) = (Arc::clone(&a), Arc::clone(&b));
let t2 = thread::spawn(move || {
    let _gb = b2.lock().unwrap();
    let _ga = a2.lock().unwrap();      // waits for a: if t1 holds a and waits for b, both wait forever
});
```

### 9.2 The rules

1. **Always the same order**: if some code takes `a` then `b`, all code takes `a` then `b`.
2. **One lock at a time** when possible: group related data in a single `Mutex<Struct>`.
3. **Never re-take a lock you already hold**: a std `Mutex` is not reentrant, a second `lock()` in the same thread blocks forever.
4. **Never call anything unknown** (user closure, a function that might take a lock) while holding a lock.

```rust
let m = std::sync::Mutex::new(0);
let _g = m.lock().unwrap();
// let _g2 = m.lock().unwrap();   // deadlock in the same thread
```

---

## 10. Parallel computation on data

```rust
use rayon::prelude::*;

let v: Vec<u64> = (1..=1_000_000).collect();

let total: u64 = v.iter().sum();                      // one core
let total_par: u64 = v.par_iter().sum();              // all cores

let squares: Vec<u64> = v.par_iter().map(|x| x * x).collect();
let mut to_sort = v.clone();
to_sort.par_sort_unstable();
```

The `rayon` crate replaces `iter()` with `par_iter()`: the work is spread over a pool of threads, with no lock or channel to write. The compiler checks that the closure can run in parallel (`Send + Sync`). Keep it for computations that are big enough: for a few thousand simple elements, the splitting costs more than it brings.

---

## 11. Choosing

| Need | Tool |
|---|---|
| run a computation in parallel and get the result | `thread::spawn` + `join`, or `thread::scope` |
| parallelize a loop over a collection | `rayon` |
| share a value read-only | `Arc<T>` |
| share and modify, short sections | `Arc<Mutex<T>>` |
| many reads, rare writes | `Arc<RwLock<T>>` |
| counter, flag, identifier to increment | `AtomicU64`, `AtomicBool` |
| producers and a consumer | `mpsc` channel |
| constant global value, computed once | `LazyLock` |
| global value set at startup | `OnceLock` |
| shared state in async code with awaiting | `tokio::sync::Mutex` (see `async-tokio`) |

---

## 12. Examples in Runique

### 12.1 `LazyLock` for the environment

```rust
// runique/src/utils/config/env.rs (excerpt)
static DEBUG: LazyLock<bool> = LazyLock::new(|| debug_from(read_after_dotenv("DEBUG").as_deref()));

pub fn is_debug() -> bool {
    *DEBUG
}

// Version token for static files (?v=...), computed only once:
// a fingerprint of the contents of the project's and Runique's .css/.js files.
static CSS_TOKEN: LazyLock<String> = LazyLock::new(|| { /* ... */ });
```

`read_after_dotenv` loads `.env` before reading the variable: this is the answer to the pitfall in section 8.5. And since the value is computed only once, a CSS file changed in production is only picked up when the server restarts.

### 12.2 `Arc<Mutex<HashMap>>` with tolerated poisoning

The request limiter (`RateLimiter`) and the login guard (`LoginGuard`) keep their counters in an `Arc<Mutex<HashMap<...>>>`. Their cleanup:

```rust
// runique/src/middleware/security/rate_limit.rs (excerpt)
pub fn spawn_cleanup(&self, period: tokio::time::Duration) {
    let store = self.store.clone();
    let window = self.window;
    let period = period.max(tokio::time::Duration::from_secs(1));
    tokio::spawn(async move {
        let mut ticker = interval(period);
        loop {
            ticker.tick().await;
            let mut guard = match store.lock() {
                Ok(g) => g,
                Err(p) => p.into_inner(),       // section 4.4: a half-written counter does no harm
            };
            let now = Instant::now();
            guard.retain(|_, (_, start)| now.duration_since(*start) < window);
        }
    });
}
```

You can see: `Arc` cloned before the `move` (section 3), `std::sync::Mutex` taken and released **with no `.await` in between** (the lock is released at the end of the loop iteration, before the next `tick().await`), tolerated poisoning, and `retain` to remove expired entries.

---

## 13. Pitfall: a shared store without cleanup

### 13.1 The problem

```rust
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

static ATTEMPTS: LazyLock<Mutex<HashMap<String, u32>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

fn login_failure(ip: &str) {
    *ATTEMPTS.lock().unwrap().entry(ip.to_string()).or_insert(0) += 1;
}
// Every IP address that gets it wrong adds an entry, never removed.
```

### 13.2 Why it is serious

- No error, no panic: memory slowly climbs.
- In development, the process restarts often: the problem is not visible.
- An attacker who varies addresses or identifiers can **fill the memory on purpose**: this is a denial of service.

### 13.3 The protections

1. **Clean up periodically** the expired entries (section 12.2).
2. **Bound the size**: refuse or evict beyond a maximum (a crate such as `lru` or `moka`).
3. **Expire on read**: an entry that is too old is ignored and removed when it is looked up.

### 13.4 What Runique does

The route extensions and the admin builder start the cleanup themselves when they create a limiter:

```rust
// runique/src/macros/routeur/router_ext.rs (excerpt)
let limiter = Arc::new(limiter);
limiter.spawn_cleanup(limiter.cleanup_interval());
```

For sessions, the builder sets a memory limit and a cleanup period:

```rust
RuniqueAppBuilder::new(config)
    .middleware(|m| {
        m.with_session_memory_limit(5 * 1024 * 1024, 10 * 1024 * 1024)   // low and high thresholds, in bytes
            .with_session_cleanup_interval(300)                           // in seconds
    })
```

At the low threshold, expired anonymous sessions are cleaned without blocking; at the high threshold, an immediate cleanup takes place, and if the limit is still exceeded, the request is refused (503) rather than letting memory grow.

A `RateLimiter` or a `LoginGuard` created by hand (outside these extensions) must call `spawn_cleanup(..)` once at startup: its documentation says so.

> **Rule**: every shared table that receives insertions coming from outside must have a cleanup or a maximum size.

---

## 14. The same function written in several ways

Count the words of several texts in parallel.

```rust
use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

fn count(t: &str) -> HashMap<String, usize> {
    let mut m = HashMap::new();
    for word in t.split_whitespace() {
        *m.entry(word.to_lowercase()).or_insert(0) += 1;
    }
    m
}

fn merge(total: &mut HashMap<String, usize>, partial: HashMap<String, usize>) {
    for (k, v) in partial {
        *total.entry(k).or_insert(0) += v;
    }
}

// 1. Shared memory: each thread writes into a common table under a lock
fn v1(texts: Vec<String>) -> HashMap<String, usize> {
    let total = Arc::new(Mutex::new(HashMap::new()));
    let hs: Vec<_> = texts.into_iter().map(|t| {
        let total = Arc::clone(&total);
        thread::spawn(move || {
            let partial = count(&t);                         // computation outside the lock
            merge(&mut total.lock().unwrap(), partial);      // lock only to merge
        })
    }).collect();
    hs.into_iter().for_each(|h| h.join().unwrap());
    Arc::try_unwrap(total).unwrap().into_inner().unwrap()
}

// 2. Channel: each thread sends its result, the main thread merges
fn v2(texts: Vec<String>) -> HashMap<String, usize> {
    let (tx, rx) = mpsc::channel();
    for t in texts {
        let tx = tx.clone();
        thread::spawn(move || tx.send(count(&t)).unwrap());
    }
    drop(tx);
    let mut total = HashMap::new();
    for partial in rx {
        merge(&mut total, partial);
    }
    total
}

// 3. scope + join: the threads borrow the texts, return their result
fn v3(texts: &[String]) -> HashMap<String, usize> {
    thread::scope(|s| {
        let hs: Vec<_> = texts.iter().map(|t| s.spawn(move || count(t))).collect();
        let mut total = HashMap::new();
        for h in hs {
            merge(&mut total, h.join().unwrap());
        }
        total
    })
}

// 4. rayon: parallel map, then reduction
fn v4(texts: &[String]) -> HashMap<String, usize> {
    use rayon::prelude::*;
    texts
        .par_iter()
        .map(|t| count(t))
        .reduce(HashMap::new, |mut a, b| { merge(&mut a, b); a })
}
```

| Version | Lock | Texts | Simplicity |
|---|---|---|---|
| 1 | yes | moved into the threads | average |
| 2 | no | moved | good |
| 3 | no | borrowed | very good |
| 4 | no | borrowed, thread count adapted to the cores | excellent |

---

## 15. Practical exercises

### Exercise 1 — concurrent counter

Increment a counter from 8 threads, 1,000 times each, first with `Arc<Mutex<u64>>`, then with an atomic. Check that the total is 8,000.

```rust
// Solution:
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

fn with_mutex() -> u64 {
    let c = Arc::new(Mutex::new(0u64));
    let hs: Vec<_> = (0..8).map(|_| {
        let c = Arc::clone(&c);
        thread::spawn(move || for _ in 0..1000 { *c.lock().unwrap() += 1; })
    }).collect();
    hs.into_iter().for_each(|h| h.join().unwrap());
    let total = *c.lock().unwrap();
    total
}

fn with_atomic() -> u64 {
    let c = Arc::new(AtomicU64::new(0));
    thread::scope(|s| {
        for _ in 0..8 {
            s.spawn(|| for _ in 0..1000 { c.fetch_add(1, Ordering::Relaxed); });
        }
    });
    c.load(Ordering::Relaxed)
}
```

### Exercise 2 — cache with `RwLock`

Write a cache `get_or_compute(key, f)`: shared read if the key exists, otherwise compute then write.

```rust
// Solution:
use std::collections::HashMap;
use std::sync::RwLock;

struct Cache { m: RwLock<HashMap<String, String>> }

impl Cache {
    fn get_or_compute(&self, key: &str, f: impl FnOnce() -> String) -> String {
        if let Some(v) = self.m.read().unwrap().get(key) {
            return v.clone();                         // read lock released here
        }
        let mut w = self.m.write().unwrap();
        w.entry(key.to_string()).or_insert_with(f).clone()   // checks again: another thread may have computed it
    }
}
```

`entry` under the write lock avoids computing twice if two threads missed the read at the same time.

### Exercise 3 — single configuration

Make sure a configuration can only be set once at startup, then read everywhere.

```rust
// Solution:
use std::sync::OnceLock;

#[derive(Debug)]
struct Config { port: u16 }

static CONFIG: OnceLock<Config> = OnceLock::new();

fn initialize(c: Config) -> Result<(), Config> {
    CONFIG.set(c)                                     // Err(c) if already set
}

fn config() -> &'static Config {
    CONFIG.get().expect("initialize() must be called in main")
}
```

---

## 16. Cheat sheet

```rust
thread::spawn(move || ..).join()         thread::scope(|s| { s.spawn(|| ..); })
Arc::new(v)   Arc::clone(&a)             Arc::try_unwrap(a)
Mutex::new(v)   m.lock().unwrap()        m.lock().unwrap_or_else(|e| e.into_inner())
m.try_lock()   m.get_mut()   m.into_inner()
RwLock::new(v)   r.read()   r.write()
AtomicU64::new(0)   a.fetch_add(1, Ordering::Relaxed)   a.load(..)   a.fetch_update(..)
mpsc::channel()   mpsc::sync_channel(n)   drop(tx)   rx.iter()
static X: LazyLock<T> = LazyLock::new(|| ..);
static Y: OnceLock<T> = OnceLock::new();   Y.set(v)   Y.get()   Y.get_or_init(|| ..)
v.par_iter().map(f).sum()                // rayon
```

### Key points

- Rust prevents data races; logical race conditions and deadlocks are still yours to avoid.
- Read then write under the **same** lock.
- Hold a lock as briefly as possible; never make a slow call or an `.await` while holding it.
- A poisoned lock can be recovered with `into_inner` when the data cannot be inconsistent.
- Atomics for a single value, `Mutex` for related values.
- Always take several locks in the same order.
- Channels and `thread::scope` often avoid any lock.
- `LazyLock` reads on first access and never changes afterwards; any table fed from outside must be cleaned or bounded.
