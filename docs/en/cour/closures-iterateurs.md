# Closures & Iterators
> Anonymous functions that capture their environment, and iterator chains: every way to write them, pass them, return them, and what they cost.

## Objectives

- Write a closure in every form and understand what it captures
- Choose between `Fn`, `FnMut` and `FnOnce`, and know which one a function should ask for
- Pass, store and return a closure in every way (generic, `impl`, `dyn`, function pointer)
- Understand that an iterator is lazy and does nothing without a consumer
- Know the adapters and consumers of the standard library
- Collect into any collection, including a `Result`
- Write your own iterators
- Choose between a loop and an iterator chain

---

## Table of contents

1. Closures: syntax
2. Capturing the environment
3. `Fn`, `FnMut`, `FnOnce`
4. Accepting a closure as a parameter
5. Storing and returning a closure
6. Async closures
7. The `Iterator` trait
8. Getting an iterator
9. Laziness
10. Adapters
11. Consumers
12. `collect` in detail
13. Writing an iterator
14. The same function written in several ways
15. Common pitfalls
16. Exercises
17. Cheat sheet

---

## 1. Closures: syntax

A closure is a function without a name, written where you need it, that can **use the variables** of the place where it is written.

```rust
// Function
fn add(x: i32, y: i32) -> i32 { x + y }

// The same thing as a closure, from the most explicit to the shortest
let add = |x: i32, y: i32| -> i32 { x + y };
let add = |x: i32, y: i32| x + y;          // return type inferred
let add = |x, y| x + y;                    // types inferred from the first call

let hello = || println!("Hello");          // no parameter

let process = |x: i32| {                   // multi-line body
    let double = x * 2;
    double + 1
};

let pair = |(a, b): (i32, i32)| a * b;     // destructured parameter

println!("{}", add(1, 2));
```

### 1.1 — Types are fixed at first use

```rust
let identity = |x| x;
let a = identity(5);          // x is now i32
// let b = identity("text");  // ERROR: expected integer, found &str
```

A closure is not generic: its types are inferred once and for all.

### 1.2 — Closure or function?

| | `fn` function | Closure |
|---|---|---|
| Captures local variables | no | yes |
| Parameter types | required | inferred |
| Generic | possible | no |
| Can be recursive | yes | no (directly) |
| Typical use | reusable logic | short argument passed to `map`, `filter`, `sort_by`, `spawn` |

A function can always be passed where a closure is expected: `v.iter().map(f)`.

---

## 2. Capturing the environment

### 2.1 — The three capture modes

A closure captures each variable in the **least restrictive** way that its body allows:

```rust
let threshold = 5;
let is_big = |x: i32| x > threshold;     // reads threshold: shared borrow (&)
println!("{}", is_big(10));
println!("{threshold}");                  // threshold is still usable

let mut counter = 0;
let mut increment = || counter += 1;      // modifies counter: mutable borrow (&mut)
increment();
increment();
// as long as increment is used further down, counter cannot be read here
println!("{counter}");                    // 2: OK, increment is no longer used

let name = String::from("Ada");
let consume = || drop(name);              // moves name: capture by value
consume();
// println!("{name}");                    // ERROR: name has been moved
```

### 2.2 — `move`: forcing capture by value

```rust
let text = String::from("hello");
let show = move || println!("{text}");    // text belongs to the closure
show();
// println!("{text}");                    // ERROR
```

What `move` is for: the closure must **live longer** than the function that creates it. This is the case for a thread, an async task or a returned closure.

```rust
use std::thread;

let data = vec![1, 2, 3];
let h = thread::spawn(move || {
    println!("{data:?}");         // without move: ERROR, the thread could outlive data
});
h.join().unwrap();
```

### 2.3 — `move` with a `Copy` type

```rust
let x = 5;
let f = move || x + 1;     // x is COPIED into the closure
println!("{x}");           // OK: x still exists
```

### 2.4 — Keeping the original and giving a copy

```rust
let config = String::from("prod");

// Form 1: clone first, then move
let c = config.clone();
let task = move || println!("{c}");

// Form 2: a block that prepares the captures
let task2 = {
    let config = config.clone();
    move || println!("{config}")
};

println!("{config}");      // the original is intact
```

The second form is common with threads and tokio tasks: it keeps the same variable name.

### 2.5 — Field capture (edition 2021+)

```rust
struct State { name: String, total: u32 }
let mut e = State { name: "a".into(), total: 0 };

let mut add = |n| e.total += n;        // captures only e.total, not all of e
println!("{}", e.name);                // OK: e.name is not captured
add(5);
```

Since the 2021 edition, a closure captures the **field** it uses, not the whole struct.

---

## 3. `Fn`, `FnMut`, `FnOnce`

Every closure automatically implements one or more of these traits, depending on what its body does.

| Trait | The body... | Callable | Example |
|---|---|---|---|
| `Fn` | only reads its captures | as many times as you want, even in parallel | `is_big` above |
| `FnMut` | modifies its captures | several times, one at a time | `increment` above |
| `FnOnce` | moves a capture out of the closure | only once | `consume` above |

### 3.1 — The hierarchy

```
Fn  ⊂  FnMut  ⊂  FnOnce
```

Every `Fn` closure is also `FnMut` and `FnOnce`. A function that asks for `FnOnce` therefore accepts **all** closures; a function that asks for `Fn` only accepts those that modify nothing.

### 3.2 — `move` does not decide the trait

```rust
let name = String::from("Ada");
let f = move || println!("{name}");   // move, but only reads: Fn
f();
f();                                  // callable several times
```

`move` decides **how** the value enters the closure; the trait depends on **what the body does with it**.

### 3.3 — Which trait to ask for

| Your function calls the closure... | Ask for |
|---|---|
| once | `FnOnce` (the most permissive for the caller) |
| several times, one after the other | `FnMut` |
| several times in parallel, or through a shared reference | `Fn` |

```rust
fn run_once<F: FnOnce() -> String>(f: F) -> String { f() }
fn repeat<F: FnMut()>(mut f: F, n: usize) { for _ in 0..n { f(); } }
fn apply<F: Fn(i32) -> i32>(f: F, v: &[i32]) -> Vec<i32> { v.iter().map(|&x| f(x)).collect() }
```

`Option::unwrap_or_else` asks for `FnOnce`, `Iterator::map` asks for `FnMut`, `thread::spawn` asks for `FnOnce + Send + 'static`.

---

## 4. Accepting a closure as a parameter

The same function, which applies an operation to a number, written in five ways.

```rust
// 1. Generic with where: the compiler generates one version per closure (the fastest)
fn apply_v1<F>(f: F, x: i32) -> i32
where
    F: Fn(i32) -> i32,
{
    f(x)
}

// 2. Inline generic
fn apply_v2<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 { f(x) }

// 3. impl Trait: same as 2, without naming the type
fn apply_v3(f: impl Fn(i32) -> i32, x: i32) -> i32 { f(x) }

// 4. Reference to a trait object: a single compiled version, indirect call
fn apply_v4(f: &dyn Fn(i32) -> i32, x: i32) -> i32 { f(x) }

// 5. Function pointer: only accepts closures WITHOUT captures and functions
fn apply_v5(f: fn(i32) -> i32, x: i32) -> i32 { f(x) }

fn main() {
    let factor = 3;
    apply_v1(|x| x * factor, 2);
    apply_v3(|x| x + 1, 2);
    apply_v4(&|x| x * factor, 2);
    apply_v5(|x| x * 2, 2);                    // no capture: accepted
    // apply_v5(|x| x * factor, 2);            // ERROR: captures factor
}
```

| Form | Cost | When to use it |
|---|---|---|
| generic / `impl Fn` | no indirect call, code duplicated per type | default case |
| `&dyn Fn` | indirect call, a single copy of the code | reduce binary size or compile time |
| `fn(..)` | indirect call, no capture | function tables, C interop |

---

## 5. Storing and returning a closure

### 5.1 — Returning a closure

```rust
// 1. impl Fn: only one kind of closure possible, no allocation
fn multiplier(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x * n                 // move required: n disappears at the end of the function
}

// 2. Box<dyn Fn>: several different closures depending on the case
fn operation(name: &str) -> Box<dyn Fn(i32, i32) -> i32> {
    match name {
        "+" => Box::new(|a, b| a + b),
        "*" => Box::new(|a, b| a * b),
        _ => Box::new(|_, _| 0),
    }
}

fn main() {
    let triple = multiplier(3);
    println!("{}", triple(5));          // 15
    println!("{}", operation("*")(2, 4)); // 8
}
```

`impl Fn` does not work for `operation`: each closure has its own type, and `impl Trait` requires a single concrete type.

### 5.2 — Storing in a struct

```rust
// 1. Generic parameter: fast, a single closure type per instance
struct Button<F: Fn()> {
    on_click: F,
}

// 2. Box<dyn Fn>: any closure, and you can put several in a Vec
struct Events {
    listeners: Vec<Box<dyn Fn(&str)>>,
}

impl Events {
    fn add(&mut self, f: impl Fn(&str) + 'static) {
        self.listeners.push(Box::new(f));
    }
    fn emit(&self, msg: &str) {
        for l in &self.listeners {
            l(msg);
        }
    }
}

fn main() {
    let b = Button { on_click: || println!("click") };
    (b.on_click)();                   // parentheses: call the field, not a method

    let mut ev = Events { listeners: vec![] };
    ev.add(|m| println!("log: {m}"));
    ev.add(|m| println!("alert: {m}"));
    ev.emit("login");
}
```

`'static` means the closure captures **no temporary reference**: it can be kept as long as needed. To share it between threads: `Box<dyn Fn(&str) + Send + Sync>`.

### 5.3 — Recursive closure

A closure cannot call itself. Use a function, possibly a nested one:

```rust
fn factorial(n: u64) -> u64 {
    fn fact(n: u64) -> u64 { if n <= 1 { 1 } else { n * fact(n - 1) } }
    fact(n)
}
```

---

## 6. Async closures

Since Rust 1.85, a closure can be `async`:

```rust
async fn process_all<F>(ids: &[u32], f: F)
where
    F: AsyncFn(u32) -> String,
{
    for &id in ids {
        println!("{}", f(id).await);
    }
}

async fn example() {
    let prefix = String::from("id-");
    process_all(&[1, 2], async |id| format!("{prefix}{id}")).await;
}
```

Before 1.85, you wrote `|id| async move { ... }`: a **normal** closure that returns a future. This form cannot borrow its captures inside the future, hence `clone` everywhere. `async ||` and the `AsyncFn`, `AsyncFnMut`, `AsyncFnOnce` traits solve this problem.

---

## 7. The `Iterator` trait

```rust
pub trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
    // + more than 70 provided methods: map, filter, sum, collect...
}
```

An iterator is a value that produces elements one by one: `next()` returns `Some(element)` while some remain, then `None`.

```rust
let v = vec![1, 2, 3];
let mut it = v.iter();
assert_eq!(it.next(), Some(&1));
assert_eq!(it.next(), Some(&2));
assert_eq!(it.next(), Some(&3));
assert_eq!(it.next(), None);
```

You only need to write `next`: all the other methods come for free.

---

## 8. Getting an iterator

### 8.1 — From a collection

```rust
let mut v = vec![1, 2, 3];

let a = v.iter();          // &i32 elements: reading
let b = v.iter_mut();      // &mut i32 elements: modifying
let c = v.into_iter();     // i32 elements: consumes v
```

`for x in &v` calls `iter()`, `for x in &mut v` calls `iter_mut()`, `for x in v` calls `into_iter()`.

### 8.2 — Other sources

```rust
let range = 1..=5;
let by_step = (0..10).step_by(2);
let characters = "abc".chars();
let words = "one two".split_whitespace();
let lines = "a\nb".lines();
let keys = std::collections::HashMap::<i32, i32>::new().into_keys();

use std::iter;
let just_one = iter::once(42);
let empty = iter::empty::<i32>();
let repeated = iter::repeat("x").take(3);             // "x", "x", "x"
let powers = iter::successors(Some(1), |&x| (x < 100).then(|| x * 2)); // 1, 2, 4 ... 128
let mut n = 0;
let counter = iter::from_fn(|| { n += 1; (n <= 3).then_some(n) });     // 1, 2, 3
let squares = (1..).map(|x| x * x);                   // infinite: limit it with take
```

### 8.3 — Iterating over an array

```rust
let t = [1, 2, 3];
for x in t { }             // edition 2021+: x is i32 (by value)
for x in t.iter() { }      // x is &i32
```

---

## 9. Laziness

An adapter (`map`, `filter`...) **does nothing** until a consumer asks for elements.

```rust
let v = vec![1, 2, 3];
v.iter().map(|x| {
    println!("processing {x}");
    x * 2
});
// Nothing is printed. The compiler warns: unused `Map` that must be used
```

```rust
let doubles: Vec<i32> = v.iter().map(|x| x * 2).collect();   // collect triggers the computation
```

### 9.1 — What laziness allows

```rust
// An infinite iterator, usable because only the beginning is taken
let first_evens: Vec<u32> = (1..).filter(|x| x % 2 == 0).take(5).collect();   // [2, 4, 6, 8, 10]

// A single pass, without an intermediate collection
let total: i32 = (1..=1_000_000).filter(|x| x % 3 == 0).map(|x| x * 2).sum();

// Early exit: find stops at the first match, map is only called up to there
let first = (1..).map(|x| x * x).find(|&x| x > 50);    // Some(64)
```

### 9.2 — The order of operations

Each element goes through the **whole chain** before the next one starts:

```rust
let _: Vec<_> = [1, 2].iter()
    .map(|x| { println!("map {x}"); x * 10 })
    .filter(|x| { println!("filter {x}"); true })
    .collect();
// map 1, filter 10, map 2, filter 20
```

Put the `filter`s **before** the expensive `map`s: discarded elements are never transformed.

### 9.3 — Cost

An iterator chain is compiled into a loop as fast as a hand-written one (this is called a zero-cost abstraction). Bounds checks often disappear, which sometimes makes it faster than a loop with an index.

---

## 10. Adapters

An adapter takes an iterator and returns another one.

### 10.1 — Transforming

```rust
let n = vec![1, 2, 3];

n.iter().map(|x| x * 2);                       // 2, 4, 6
n.iter().copied();                             // &i32 -> i32 (Copy types)
n.iter().cloned();                             // &T -> T (Clone types)
n.iter().enumerate();                          // (0, &1), (1, &2), (2, &3)
n.iter().inspect(|x| println!("seen {x}"));    // look without modifying (debugging)
n.iter().scan(0, |acc, &x| { *acc += x; Some(*acc) });   // running sums: 1, 3, 6
```

### 10.2 — Filtering

```rust
n.iter().filter(|&&x| x > 1);                  // 2, 3
n.iter().filter_map(|&x| (x > 1).then_some(x * 10));   // 20, 30
["1", "a", "3"].iter().filter_map(|s| s.parse::<i32>().ok());   // 1, 3
n.iter().take(2);                              // 1, 2
n.iter().skip(1);                              // 2, 3
n.iter().take_while(|&&x| x < 3);              // 1, 2 then stops
n.iter().skip_while(|&&x| x < 2);              // 2, 3
["1", "2", "x", "4"].iter().map_while(|s| s.parse::<i32>().ok());   // 1, 2 then stops
n.iter().step_by(2);                           // 1, 3
```

`filter` keeps going after a rejected element; `take_while` stops for good.

### 10.3 — Combining

```rust
let a = [1, 2];
let b = [10, 20];

a.iter().chain(b.iter());                      // 1, 2, 10, 20
a.iter().zip(b.iter());                        // (1, 10), (2, 20)
a.iter().rev();                                // 2, 1
a.iter().cycle().take(5);                      // 1, 2, 1, 2, 1
vec![vec![1], vec![2, 3]].into_iter().flatten();          // 1, 2, 3
["a b", "c"].iter().flat_map(|s| s.split(' '));            // "a", "b", "c"
vec![Some(1), None, Some(3)].into_iter().flatten();       // 1, 3: flatten on Options
```

### 10.4 — Looking at the next element: `peekable`

```rust
let mut it = [1, 1, 2, 3, 3, 3].into_iter().peekable();
let mut groups = Vec::new();
while let Some(x) = it.next() {
    let mut count = 1;
    while it.peek() == Some(&x) {      // looks without consuming
        it.next();
        count += 1;
    }
    groups.push((x, count));
}
// [(1, 2), (2, 1), (3, 3)]
```

What it is for: parsers (read the next character to know whether a number continues), grouping consecutive elements. `next_if(|x| ...)` consumes only if the condition is true.

### 10.5 — `by_ref`: consuming in several steps

```rust
let mut lines = "title\n---\nbody 1\nbody 2".lines();
let header: Vec<&str> = lines.by_ref().take_while(|l| *l != "---").collect();
let body: Vec<&str> = lines.collect();     // the rest, after the separator
```

`take` or `take_while` consumes the iterator; `by_ref()` lends it, so you can keep using it afterwards.

---

## 11. Consumers

A consumer walks through the iterator and produces a result.

```rust
let n = [3, 1, 4, 1, 5];

n.iter().sum::<i32>();                          // 14
n.iter().product::<i32>();                      // 60
n.iter().count();                               // 5
n.iter().min();                                 // Some(&1)
n.iter().max();                                 // Some(&5)
n.iter().last();                                // Some(&5)
n.iter().nth(2);                                // Some(&4)
n.iter().any(|&x| x > 4);                       // true
n.iter().all(|&x| x > 0);                       // true
n.iter().find(|&&x| x > 3);                     // Some(&4)
n.iter().position(|&x| x == 1);                 // Some(1)
n.iter().rposition(|&x| x == 1);                // Some(3): from the end
["a", "12", "b"].iter().find_map(|s| s.parse::<i32>().ok());   // Some(12)
n.iter().fold(0, |acc, &x| acc + x);            // 14
n.iter().copied().reduce(|a, b| a.max(b));      // Some(5): fold without an initial value
n.iter().for_each(|x| println!("{x}"));

let words = ["rust", "go", "python"];
words.iter().max_by_key(|m| m.len());           // Some("python")
words.iter().min_by(|a, b| a.len().cmp(&b.len()));   // Some("go")
```

### 11.1 — Consumers that can fail

```rust
// try_fold: stops at the first error
let r: Option<i32> = [1, 2, 3].iter().try_fold(0i32, |acc, &x| acc.checked_add(x));   // Some(6)

// try_for_each: a loop with ?
let r: Result<(), String> = ["1", "x"].iter().try_for_each(|s| {
    s.parse::<i32>().map(|_| ()).map_err(|e| e.to_string())
});
```

### 11.2 — `sum` and `max` of empty iterators

```rust
let empty: [i32; 0] = [];
empty.iter().sum::<i32>();     // 0
empty.iter().max();            // None: no maximum
```

### 11.3 — Floats

```rust
let f = [2.5, 1.0, 3.2];
// f.iter().max();                                   // ERROR: f64 is not Ord
f.iter().copied().fold(f64::NEG_INFINITY, f64::max); // 3.2
f.iter().max_by(|a, b| a.total_cmp(b));              // Some(&3.2)
```

---

## 12. `collect` in detail

### 12.1 — Into any collection

```rust
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

let v: Vec<i32> = (1..4).collect();
let s: HashSet<i32> = [1, 2, 2].into_iter().collect();
let d: VecDeque<i32> = (1..4).collect();
let m: HashMap<&str, i32> = [("a", 1), ("b", 2)].into_iter().collect();
let b: BTreeMap<i32, char> = "abc".char_indices().map(|(i, c)| (i as i32, c)).collect();
let t: String = ['a', 'b'].iter().collect();
let t2: String = ["one", "two"].iter().map(|s| s.to_uppercase()).collect();
let t3 = ["one", "two"].join(", ");        // to join with a separator
```

### 12.2 — Specifying the type

```rust
let v: Vec<_> = (1..4).collect();          // annotation on the variable
let v = (1..4).collect::<Vec<_>>();        // turbofish
fn firsts() -> Vec<i32> { (1..4).collect() }   // type inferred from the return type
```

### 12.3 — Collecting `Result`s and `Option`s

```rust
let texts = ["1", "2", "x"];

// A Vec of Results: each element keeps its own result
let all: Vec<Result<i32, _>> = texts.iter().map(|s| s.parse::<i32>()).collect();

// A Result of Vec: Ok if ALL succeed, otherwise the first error
let r: Result<Vec<i32>, _> = texts.iter().map(|s| s.parse::<i32>()).collect();   // Err

// Keep only the successes
let ok: Vec<i32> = texts.iter().filter_map(|s| s.parse().ok()).collect();       // [1, 2]

// Split successes and failures
let (good, bad): (Vec<_>, Vec<_>) = texts.iter().map(|s| s.parse::<i32>()).partition(|r| r.is_ok());

// Same thing with Option
let o: Option<Vec<i32>> = [Some(1), None].into_iter().collect();   // None
```

`collect::<Result<Vec<_>, _>>()` stops at the first error: this is how you validate a whole list.

### 12.4 — Splitting in two

```rust
let (evens, odds): (Vec<i32>, Vec<i32>) = (1..=6).partition(|x| x % 2 == 0);
let (names, ages): (Vec<&str>, Vec<u32>) = [("a", 1), ("b", 2)].into_iter().unzip();
```

### 12.5 — Adding to an existing collection

```rust
let mut v = vec![1];
v.extend((2..4).map(|x| x * 10));          // [1, 20, 30]
```

---

## 13. Writing an iterator

### 13.1 — Implementing `Iterator`

```rust
struct Count { value: u32, max: u32 }

impl Count {
    fn new(max: u32) -> Self { Count { value: 0, max } }
}

impl Iterator for Count {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        if self.value < self.max {
            self.value += 1;
            Some(self.value)
        } else {
            None
        }
    }
}

let total: u32 = Count::new(5)
    .zip(Count::new(5).skip(1))
    .map(|(a, b)| a * b)
    .filter(|x| x % 3 == 0)
    .sum();
// Count gets all the Iterator methods for free
```

### 13.2 — Without a struct: `from_fn` and `successors`

```rust
use std::iter;

// Fibonacci with an implicit struct
fn fibonacci() -> impl Iterator<Item = u64> {
    let (mut a, mut b) = (0u64, 1u64);
    iter::from_fn(move || {
        let r = a;
        (a, b) = (b, a.checked_add(b)?);    // stops before overflowing
        Some(r)
    })
}

// Each element computed from the previous one
fn collatz(n: u64) -> impl Iterator<Item = u64> {
    iter::successors(Some(n), |&x| match x {
        1 => None,
        x if x % 2 == 0 => Some(x / 2),
        x => Some(3 * x + 1),
    })
}

let f: Vec<u64> = fibonacci().take(8).collect();   // [0, 1, 1, 2, 3, 5, 8, 13]
let c: Vec<u64> = collatz(6).collect();            // [6, 3, 10, 5, 16, 8, 4, 2, 1]
```

### 13.3 — Returning an iterator

```rust
// 1. impl Iterator: the exact type stays hidden, no cost
fn evens(v: &[i32]) -> impl Iterator<Item = &i32> {
    v.iter().filter(|x| *x % 2 == 0)
}

// 2. Box<dyn Iterator>: when several different chains can be returned
fn selection(v: &[i32], reverse: bool) -> Box<dyn Iterator<Item = &i32> + '_> {
    if reverse { Box::new(v.iter().rev()) } else { Box::new(v.iter()) }
}

// 3. Collecting: simple, but allocates a Vec even if the caller only wants the first one
fn evens_vec(v: &[i32]) -> Vec<i32> {
    v.iter().copied().filter(|x| x % 2 == 0).collect()
}
```

### 13.4 — Making your type usable in a `for`

```rust
struct Basket { items: Vec<String> }

impl<'a> IntoIterator for &'a Basket {
    type Item = &'a String;
    type IntoIter = std::slice::Iter<'a, String>;
    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

let p = Basket { items: vec!["apple".into()] };
for a in &p {
    println!("{a}");
}
```

### 13.5 — Additional traits

| Trait | What it adds | Example |
|---|---|---|
| `DoubleEndedIterator` | `next_back`, hence `rev()` | `Vec` iterators, ranges |
| `ExactSizeIterator` | exact `len()` | `v.iter()` |
| `FusedIterator` | guarantees `None` forever after the first `None` | most standard iterators |

---

## 14. The same function written in several ways

### 14.1 — Valid emails in lowercase

```rust
struct User { email: String, active: bool }

// 1. Loop
fn emails_v1(u: &[User]) -> Vec<String> {
    let mut r = Vec::new();
    for x in u {
        if x.active && x.email.contains('@') {
            r.push(x.email.to_lowercase());
        }
    }
    r
}

// 2. filter + map
fn emails_v2(u: &[User]) -> Vec<String> {
    u.iter()
        .filter(|x| x.active && x.email.contains('@'))
        .map(|x| x.email.to_lowercase())
        .collect()
}

// 3. filter_map
fn emails_v3(u: &[User]) -> Vec<String> {
    u.iter()
        .filter_map(|x| (x.active && x.email.contains('@')).then(|| x.email.to_lowercase()))
        .collect()
}
```

### 14.2 — Counting words

```rust
use std::collections::HashMap;

// 1. Loop + entry
fn count_v1(t: &str) -> HashMap<&str, usize> {
    let mut m = HashMap::new();
    for word in t.split_whitespace() {
        *m.entry(word).or_insert(0) += 1;
    }
    m
}

// 2. fold
fn count_v2(t: &str) -> HashMap<&str, usize> {
    t.split_whitespace().fold(HashMap::new(), |mut m, word| {
        *m.entry(word).or_insert(0) += 1;
        m
    })
}
```

Here the loop is more readable: `fold` with a mutable accumulator adds nothing.

### 14.3 — Choosing

- A data **transformation** (filtering, converting, computing a total) reads better as an iterator chain.
- A loop with **side effects** (printing, writing to the database), several early exits, `?`, `break` or `continue` stays clearer as a `for`.
- Beyond five or six steps, a chain is better split into named variables or functions.

---

## 15. Common pitfalls

### 15.1 — Too many or too few `&`

```rust
let v = vec![1, 2, 3];

v.iter().filter(|x| **x > 1);          // filter receives &&i32
v.iter().filter(|&&x| x > 1);          // destructuring both &
v.iter().copied().filter(|&x| x > 1);  // copied first: a single &
v.iter().map(|x| x * 2);               // map receives &i32; * works on &i32
```

`filter` receives **a reference to the element** of the iterator; if the iterator already produces `&i32`, you get `&&i32`.

### 15.2 — Using after `into_iter`

```rust
let v = vec![String::from("a")];
let m: Vec<String> = v.into_iter().map(|s| s + "!").collect();
// println!("{v:?}");                  // ERROR: v consumed
```

### 15.3 — Modifying the source during iteration

```rust
let mut v = vec![1, 2, 3];
// for x in v.iter() { v.push(*x); }  // ERROR: v is borrowed
let copy: Vec<i32> = v.clone();
v.extend(copy);
```

### 15.4 — `collect` without a type

```rust
// let v = (1..4).collect();           // ERROR: type annotations needed
let v: Vec<i32> = (1..4).collect();
```

### 15.5 — `return` inside a closure

```rust
fn first_negative(v: &[i32]) -> Option<i32> {
    v.iter().for_each(|&x| {
        if x < 0 {
            return;                    // leaves the CLOSURE, not the function
        }
    });
    v.iter().copied().find(|&x| x < 0) // the right way
}
```

---

## 16. Exercises

### Exercise 1 — counter

Write a function that returns a closure counting its calls (`FnMut() -> u32`).

```rust
// Solution:
fn counter() -> impl FnMut() -> u32 {
    let mut n = 0;
    move || {
        n += 1;
        n
    }
}

let mut c = counter();
assert_eq!(c(), 1);
assert_eq!(c(), 2);
```

### Exercise 2 — validating a list

Convert `&[&str]` into `Vec<u32>`, returning the first error if an element is not a number.

```rust
// Solution:
fn convert(t: &[&str]) -> Result<Vec<u32>, std::num::ParseIntError> {
    t.iter().map(|s| s.trim().parse::<u32>()).collect()
}
```

### Exercise 3 — moving average

For `[1.0, 2.0, 3.0, 4.0]` and a window of 2, return `[1.5, 2.5, 3.5]`.

```rust
// Solution:
fn moving_average(v: &[f64], n: usize) -> Vec<f64> {
    v.windows(n).map(|w| w.iter().sum::<f64>() / n as f64).collect()
}
```

### Exercise 4 — date iterator

Write an iterator that produces the day numbers 1 to 31, except Sundays (every 7 days starting from day 7).

```rust
// Solution:
fn working_days() -> impl Iterator<Item = u32> {
    (1..=31).filter(|d| d % 7 != 0)
}
```

---

## 17. Cheat sheet

### Closures

```rust
|x| x + 1                   |x: i32| -> i32 { x + 1 }       || println!()
move || uses(value)         async |x| f(x).await
fn f(g: impl Fn(i32) -> i32)      fn f<F: FnMut()>(g: F)      fn f(g: &dyn Fn())
fn f() -> impl Fn(i32) -> i32     Box<dyn Fn(i32) -> i32>     fn(i32) -> i32
```

### Iterators

```rust
v.iter()  v.iter_mut()  v.into_iter()  (a..b)  iter::once  iter::repeat  iter::from_fn  iter::successors
.map  .filter  .filter_map  .flat_map  .flatten  .enumerate  .zip  .chain  .rev  .take  .skip
.take_while  .skip_while  .map_while  .step_by  .peekable  .scan  .inspect  .copied  .cloned  .by_ref
.collect  .sum  .product  .count  .min  .max  .min_by_key  .max_by  .fold  .reduce  .any  .all
.find  .find_map  .position  .last  .nth  .partition  .unzip  .for_each  .try_fold  .try_for_each
```

### Key points

- A closure captures in the least restrictive way; `move` forces capture by value.
- The trait (`Fn`, `FnMut`, `FnOnce`) depends on what the body does, not on `move`.
- Ask for `FnOnce` if you call once, `FnMut` several times, `Fn` in parallel.
- `impl Fn` for one closure, `Box<dyn Fn>` for several kinds or to store a list of them.
- An iterator does nothing without a consumer.
- `collect::<Result<Vec<_>, _>>()` validates a whole list and stops at the first error.
- An iterator chain costs as much as a hand-written loop.
