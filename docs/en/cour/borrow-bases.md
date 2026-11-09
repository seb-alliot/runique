# Ownership and Borrowing in Rust
> Who owns a value, when it is freed, how to lend it, and every way to fix borrow checker errors.

## Objectives

- Understand the stack, the heap, and why Rust needs ownership
- Apply the three ownership rules
- Know when a value is moved, copied or borrowed
- Lend a value for reading (`&`) or writing (`&mut`) and know the borrowing rules
- Read borrow checker errors and know several fixes for each
- Use slices (`&str`, `&[T]`) to borrow part of a value
- Choose a function signature: by value, `&`, `&mut`

---

## Table of contents

1. Stack, heap and freeing memory
2. The three ownership rules
3. Scope and drop order
4. Moves
5. `Copy` and `Clone`
6. `&` references
7. `&mut` references
8. The borrow checker rules
9. Borrows and structs
10. Slices
11. Choosing a function signature
12. Common errors and their fixes
13. Tools to get out of a dead end
14. Exercises
15. Cheat sheet

---

## 1. Stack, heap and freeing memory

### 1.1 — The stack and the heap

- **The stack**: fast memory, with a size known at compile time. Integers, booleans, fixed-size arrays, and the "headers" of values live there. It is freed automatically when each function exits.
- **The heap**: memory allocated at runtime, with a variable size. The content of a `String`, a `Vec`, a `Box` lives there.

```rust
let x = 5;                       // entirely on the stack
let s = String::from("hello");   // on the stack: pointer, length, capacity
                                 // on the heap : the bytes "hello"
```

### 1.2 — The problem to solve

Memory allocated on the heap must be freed **exactly once**:

- never freed: memory leak;
- freed twice: corruption (double free);
- used after being freed: security hole (use after free).

| Language | Solution |
|---|---|
| C | the programmer calls `free` by hand |
| Java, Python, Go, JavaScript | a garbage collector periodically looks for what is no longer used |
| Rust | the compiler knows, for each value, **who** owns it and **when** to free it |

Ownership has no runtime cost: everything is checked at compile time.

---

## 2. The three ownership rules

1. Each value has **one owner** (a variable, a field, a collection element).
2. There is **only one** owner at a time.
3. When the owner goes out of scope, the value is **freed** (`drop`).

```rust
fn main() {
    let s = String::from("hello");   // s owns the String
    println!("{s}");
}                                     // s goes out of scope: the memory is freed
```

---

## 3. Scope and drop order

### 3.1 — One scope per block

```rust
fn main() {
    let a = String::from("a");
    {
        let b = String::from("b");
        println!("{a} {b}");
    }                              // b is freed here
    // println!("{b}");            // ERROR: b no longer exists
}                                  // a is freed here
```

### 3.2 — Reverse order of declaration

```rust
struct Noisy(&'static str);

impl Drop for Noisy {
    fn drop(&mut self) {
        println!("dropping {}", self.0);
    }
}

fn main() {
    let _a = Noisy("a");
    let _b = Noisy("b");
    println!("end of main");
}
// end of main
// dropping b
// dropping a
```

Variables are freed in the **reverse order** of their declaration: a value declared later can borrow a value declared earlier, so the earlier one is always freed last. The fields of a struct are freed in declaration order.

### 3.3 — Freeing earlier

```rust
let big = vec![0u8; 100_000_000];
process(&big);
drop(big);                // frees now instead of waiting for the end of the block
// long code that no longer needs big

fn process(_: &[u8]) {}
```

`drop(x)` is not magic: it is a function that **takes** the value and does nothing, so the value is freed when the function returns. It is also used to release a lock before the end of a block.

---

## 4. Moves

### 4.1 — On assignment

```rust
let s1 = String::from("hello");
let s2 = s1;                       // the String is MOVED from s1 to s2
// println!("{s1}");               // ERROR E0382: borrow of moved value
println!("{s2}");
```

Only the header (pointer, length, capacity) is copied; the bytes on the heap do not move. If `s1` stayed valid, two variables would own the same block and free it twice. Rust therefore invalidates `s1`.

### 4.2 — When passing to a function

```rust
fn take(s: String) {
    println!("{s}");
}                                   // s is freed here

fn main() {
    let text = String::from("hello");
    take(text);                     // moved into the function
    // println!("{text}");          // ERROR
}
```

### 4.3 — When returning a value

```rust
fn make() -> String {
    let s = String::from("new");
    s                               // moved to the caller: not freed here
}

fn add_dot(mut s: String) -> String {
    s.push('.');
    s                               // taken then given back
}

fn main() {
    let a = make();
    let a = add_dot(a);
}
```

Taking then giving back works, but it is heavy: a reference does the same thing more simply (section 6).

### 4.4 — In a collection, a field, a closure

```rust
let name = String::from("Ada");
let v = vec![name];                 // moved into the Vec
// println!("{name}");              // ERROR

let title = String::from("x");
let show = move || println!("{title}");   // move: the closure owns title
show();
// println!("{title}");             // ERROR
```

### 4.5 — In a loop

```rust
let s = String::from("a");
for _ in 0..3 {
    // take(s);                     // ERROR: moved on the first turn, missing on the second
    take(s.clone());                // one copy per turn
}

fn take(_: String) {}
```

### 4.6 — Partial move

```rust
struct Person { name: String, age: u32 }

let p = Person { name: String::from("Ada"), age: 36 };
let name = p.name;                  // only the name field is moved
println!("{}", p.age);              // OK: age is still there
// println!("{}", p.name);          // ERROR: moved
// let q = p;                       // ERROR: p is incomplete
```

### 4.7 — Conditional move

```rust
let s = String::from("a");
let condition = true;
if condition {
    drop(s);
}
// println!("{s}");                 // ERROR: maybe moved
```

The compiler reasons about **all** possible paths: if a single path moves the value, it is considered unavailable afterwards.

---

## 5. `Copy` and `Clone`

### 5.1 — `Copy`: implicit copy

```rust
let x = 5;
let y = x;            // copy: x stays usable
println!("{x} {y}");
```

A `Copy` type is copied bit by bit instead of being moved. These are `Copy`:

- integers, floats, `bool`, `char`;
- shared references `&T`;
- tuples and arrays whose elements are all `Copy`;
- your own types with `#[derive(Clone, Copy)]`, if all their fields are `Copy`.

These are **not** `Copy`: `String`, `Vec`, `Box`, `HashMap`, `&mut T`, and any type that implements `Drop`.

### 5.2 — `Clone`: explicit copy

```rust
let s1 = String::from("hello");
let s2 = s1.clone();              // new allocation, bytes copied
println!("{s1} {s2}");            // both are valid
```

`.clone()` is visible in the code: this is intended, because copying heap data has a cost.

### 5.3 — Comparison

| | `Copy` | `Clone` |
|---|---|---|
| Trigger | automatic, on every assignment or call | explicit `.clone()` call |
| Cost | a few bytes copied | depends on the content (can be large) |
| Types | small, without resources | any type that implements it |
| Relation | every `Copy` is also `Clone` | |

### 5.4 — Cloning: when it is the right solution

Cloning is not a mistake. It is the right solution when:

- the value is small (an identifier, a short name);
- two parts of the program really need their own copy;
- the alternative would make the code much more complicated.

It is the wrong one when you clone just to silence the compiler in a hot loop, or a large `Vec` on every call when a reference would be enough.

---

## 6. `&` references

### 6.1 — Borrowing instead of taking

```rust
fn length(s: &String) -> usize {
    s.len()
}                                 // s is a reference: nothing is freed

fn main() {
    let text = String::from("hello");
    let n = length(&text);        // we lend text
    println!("{text}: {n}");      // text is still there
}
```

A reference `&T` is a pointer to a value **owned by someone else**. It never frees it.

### 6.2 — Several shared references

```rust
let s = String::from("hello");
let r1 = &s;
let r2 = &s;
let r3 = &s;
println!("{r1} {r2} {r3}");       // as many readers as you want
```

### 6.3 — A reference does not allow modification

```rust
fn modify(s: &String) {
    // s.push('!');               // ERROR: cannot borrow `*s` as mutable
}
```

### 6.4 — Dereferencing

```rust
let x = 5;
let r = &x;
assert_eq!(*r, 5);                // * to read the pointed value
assert_eq!(r.pow(2), 25);         // methods dereference automatically
let total = r + 1;                // operators too, for numbers
```

---

## 7. `&mut` references

### 7.1 — Lending for writing

```rust
fn add_world(s: &mut String) {
    s.push_str(", world");
}

fn main() {
    let mut text = String::from("hello");   // the variable must be mut
    add_world(&mut text);                    // we lend for writing
    println!("{text}");                      // hello, world
}
```

You need `mut` in **two** places: on the variable (`let mut`) and on the borrow (`&mut`).

### 7.2 — Modifying through the reference

```rust
fn double(n: &mut i32) {
    *n *= 2;                    // * is required to assign the pointed value
}

let mut x = 5;
double(&mut x);
assert_eq!(x, 10);
```

### 7.3 — Only one at a time

```rust
let mut s = String::from("a");
let r1 = &mut s;
// let r2 = &mut s;             // ERROR E0499: two mutable borrows
r1.push('b');
```

What it is for: if two references could modify the same value, one could reallocate the content while the other reads it. This is the cause of "data races" between threads; Rust makes them impossible at compile time.

---

## 8. The borrow checker rules

### 8.1 — The rule

At any moment, for a given value:

- either **one** `&mut` reference;
- or **as many** `&` references as you want;
- **never both** at the same time.

And a reference never lives longer than the value it points to.

### 8.2 — A borrow lasts until its last use

```rust
let mut s = String::from("hello");

let r1 = &s;
let r2 = &s;
println!("{r1} {r2}");           // last use of r1 and r2

let r3 = &mut s;                 // OK: r1 and r2 are no longer used
r3.push('!');
```

A borrow does not last until the end of the block but until its **last use** (a rule called NLL, Non-Lexical Lifetimes). Reordering the lines is often enough to fix an error.

### 8.3 — While a value is lent, its owner is restricted

```rust
let mut v = vec![1, 2, 3];
let first = &v[0];               // v is lent for reading
// v.push(4);                    // ERROR E0502: v cannot be modified
println!("{first}");
v.push(4);                       // OK: first is no longer used
```

`push` may reallocate the array: `first` would then point to freed memory.

### 8.4 — Dangling reference

```rust
// fn make() -> &String {
//     let s = String::from("hello");
//     &s                        // ERROR: s is freed at the end of the function
// }

fn make() -> String {            // fix: return the value itself
    String::from("hello")
}
```

### 8.5 — Reborrow

```rust
fn add(v: &mut Vec<i32>) {
    v.push(1);
}

let mut v = vec![];
let r = &mut v;
add(r);                           // temporary reborrow of *r
add(r);                           // r is still usable
r.push(2);
```

Passing a `&mut` to a function does not move it: Rust creates a reborrow (`&mut *r`) that lasts for the duration of the call.

### 8.6 — A handy exception

```rust
let mut v: Vec<usize> = vec![1, 2, 3];
v.push(v.len());                  // OK: v.len() is evaluated before the mutable borrow of push
```

---

## 9. Borrows and structs

### 9.1 — Distinct fields: accepted directly

```rust
struct Game { players: Vec<String>, scores: Vec<u32> }

let mut g = Game { players: vec!["a".into()], scores: vec![0] };
let players = &g.players;         // borrows g.players only
g.scores.push(10);                // OK: different field
println!("{players:?}");
```

The compiler tracks borrows **field by field**.

### 9.2 — Through a method: rejected

```rust
impl Game {
    fn players(&self) -> &Vec<String> { &self.players }
    fn add_score(&mut self, s: u32) { self.scores.push(s); }
}

let players = g.players();        // borrows ALL of g
// g.add_score(10);               // ERROR: g is already borrowed
println!("{players:?}");
```

A `&self` method borrows the whole struct: the compiler does not look at what the method really reads. Three fixes:

```rust
// 1. Access the fields directly (in the module that sees them)
let players = &g.players;
g.scores.push(10);

// 2. Copy what you need beforehand
let count = g.players().len();
g.add_score(10);

// 3. A method that returns both parts at the same time
impl Game {
    fn parts(&mut self) -> (&Vec<String>, &mut Vec<u32>) {
        (&self.players, &mut self.scores)
    }
}
let (players, scores) = g.parts();
scores.push(10);
```

---

## 10. Slices

A slice is a reference to a **contiguous part** of a collection.

### 10.1 — `&str`

```rust
let s = String::from("hello world");
let hello: &str = &s[0..5];
let world: &str = &s[6..];
let all: &str = &s[..];
let literal: &str = "text";       // a literal is already a &str
```

Indexes are in **bytes**: cutting in the middle of an accented character panics.

```rust
let word = "café";
// let x = &word[0..4];           // panics: 'é' takes bytes 3 and 4
let x = word.get(0..4);           // None, no panic
let first: String = word.chars().take(3).collect();   // "caf", in characters
```

### 10.2 — `&[T]`

```rust
let v = vec![1, 2, 3, 4, 5];
let middle: &[i32] = &v[1..4];    // [2, 3, 4]
let t = [10, 20, 30];
let start = &t[..2];

let (a, b) = v.split_at(2);       // two slices: [1, 2] and [3, 4, 5]
```

### 10.3 — The slice protects against invalidation

```rust
fn first_word(s: &str) -> &str {
    s.split(' ').next().unwrap_or("")
}

let mut s = String::from("hello world");
let word = first_word(&s);
// s.clear();                     // ERROR: word still borrows s
println!("{word}");
```

If the function returned an index (`usize`), nothing would prevent clearing `s` and then using the index, which is now wrong. The slice ties the result to the data.

### 10.4 — Range syntax

| Syntax | Elements |
|---|---|
| `&v[a..b]` | from `a` included to `b` excluded |
| `&v[a..=b]` | from `a` to `b` included |
| `&v[..b]` | from the start to `b` excluded |
| `&v[a..]` | from `a` to the end |
| `&v[..]` | everything |

---

## 11. Choosing a function signature

### 11.1 — The same function, four signatures

```rust
// 1. Takes the String: the caller loses it
fn uppercase_v1(s: String) -> String { s.to_uppercase() }

// 2. Borrows a &String: only accepts String
fn uppercase_v2(s: &String) -> String { s.to_uppercase() }

// 3. Borrows a &str: accepts String, &str, literals, slices
fn uppercase_v3(s: &str) -> String { s.to_uppercase() }

// 4. Modifies in place: no new allocation
fn uppercase_v4(s: &mut String) { s.make_ascii_uppercase(); }

fn main() {
    let mut a = String::from("abc");
    uppercase_v3(&a);
    uppercase_v3("literal");
    uppercase_v4(&mut a);
    let b = uppercase_v1(a);       // a is no longer usable
}
```

### 11.2 — Rules for choosing

| The function... | Parameter |
|---|---|
| only reads | `&T`, `&str`, `&[T]` |
| modifies the caller's value | `&mut T` |
| keeps the value (stores it in a struct, sends it to a thread) | `T` by value |
| receives a small `Copy` type | `T` by value |

Taking by value when you store avoids a hidden `clone` inside the function: the caller chooses to clone or to hand over its value.

```rust
struct User { name: String }

// Bad: clone required inside the function, even if the caller no longer needed the name
fn create_v1(name: &str) -> User { User { name: name.to_string() } }

// Good: the caller hands over its String (no copy) or passes a converted literal
fn create_v2(name: String) -> User { User { name } }

// Flexible: accepts &str and String
fn create_v3(name: impl Into<String>) -> User { User { name: name.into() } }
```

---

## 12. Common errors and their fixes

### 12.1 — E0382: use after move

```rust
let s = String::from("a");
let t = s;
// println!("{s}");
```

Fixes:

```rust
let t = s.clone();      // 1. copy
let t = &s;             // 2. borrow
// 3. use s before moving it
println!("{s}");
let t = s;
```

### 12.2 — E0499: two mutable borrows

```rust
let mut v = vec![1, 2];
let a = &mut v[0];
// let b = &mut v[1];
// *a += *b;
```

Fixes:

```rust
// 1. split_at_mut: two mutable slices that do not overlap
let (left, right) = v.split_at_mut(1);
left[0] += right[0];

// 2. Go through the indexes
let b = v[1];
v[0] += b;

// 3. swap to exchange
v.swap(0, 1);
```

### 12.3 — E0502: reading and writing at the same time

```rust
let mut v = vec![1, 2, 3];
// for x in &v {
//     if *x == 2 { v.push(4); }
// }
```

Fixes:

```rust
// 1. Decide first, modify afterwards
let contains = v.contains(&2);
if contains { v.push(4); }

// 2. Collect the additions then apply them
let additions: Vec<i32> = v.iter().filter(|&&x| x == 2).map(|_| 4).collect();
v.extend(additions);

// 3. Loop over the indexes when the change depends on the current element
for i in 0..v.len() {
    if v[i] == 2 { v[i] = 20; }
}
```

### 12.4 — E0505: moving a borrowed value

```rust
let s = String::from("a");
let r = &s;
// drop(s);
println!("{r}");
```

Fix: finish using the borrow before moving (`println!` before `drop`), or clone.

### 12.5 — E0507: moving a value out of a reference

```rust
struct Container { content: String }

fn empty(b: &mut Container) -> String {
    // b.content                  // ERROR: cannot move out of `b.content` which is behind a mutable reference
    std::mem::take(&mut b.content)    // takes the value, leaves String::new()
}

fn read(b: &Container) -> String {
    b.content.clone()             // or return &str: &b.content
}
```

With an `Option`: `opt.take()` takes the value and leaves `None`.

### 12.6 — E0597: the value does not live long enough

```rust
let r;
{
    let x = 5;
    r = &x;
}                                 // x freed
// println!("{r}");               // ERROR: `x` does not live long enough
```

Fix: declare `x` in the same scope as `r` (or before), or store the value itself rather than a reference.

### 12.7 — E0106: missing lifetime

```rust
// fn longest(a: &str, b: &str) -> &str { if a.len() > b.len() { a } else { b } }
fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() > b.len() { a } else { b }
}
```

When a function returns a reference that can come from several parameters, you must say which parameter it depends on. Details are in the `lifetimes` course.

---

## 13. Tools to get out of a dead end

| Situation | Tool |
|---|---|
| Take a value behind `&mut` | `std::mem::take`, `std::mem::replace`, `Option::take` |
| Swap two values | `std::mem::swap(&mut a, &mut b)`, `v.swap(i, j)` |
| Two mutable parts of a slice | `split_at_mut`, `chunks_mut`, `iter_mut` |
| Read or insert in a map in one go | `map.entry(k)` |
| Several owners of the same value | `Rc<T>` (one thread), `Arc<T>` (several threads) |
| Modify a shared value | `RefCell<T>`, `Mutex<T>`, `RwLock<T>` |
| Avoid a copy when you do not know if you will modify | `Cow<str>` |
| Structure with links between elements (graph) | indexes into a `Vec` instead of references |

```rust
use std::mem;

let mut a = String::from("old");
let old = mem::replace(&mut a, String::from("new"));
let empty = mem::take(&mut a);          // a becomes ""
let mut x = 1;
let mut y = 2;
mem::swap(&mut x, &mut y);
```

`Rc`, `RefCell`, `Arc` and `Mutex` are covered in `smart-pointers` and `concurrence`, `Cow` in `borrow-avance`.

---

## 14. Exercises

### Exercise 1 — move or copy?

For each line, say whether the variable stays usable.

```rust
let a = 5;            let b = a;            // a?
let s = String::new(); let t = s;           // s?
let c = 'x';          let d = c;            // c?
let v = vec![1];      let w = &v;           // v?
let p = (1, 2.0);     let q = p;            // p?
let r = (1, String::new()); let u = r;      // r?
```

Answers: `a` yes (`i32` is Copy), `s` no (moved), `c` yes, `v` yes (borrow), `p` yes (tuple of Copy), `r` no (contains a `String`).

### Exercise 2 — fix without cloning

```rust
fn main() {
    let mut list = vec![1, 2, 3];
    let first = &list[0];
    list.push(4);
    println!("{first}");
}
```

```rust
// Solution 1: copy the integer (Copy, free)
let first = list[0];
list.push(4);
println!("{first}");

// Solution 2: reorder
let first = &list[0];
println!("{first}");
list.push(4);
```

### Exercise 3 — empty an option in a struct

```rust
struct Session { token: Option<String> }

// Write `fn log_out(s: &mut Session) -> Option<String>` that returns the token and removes it.
```

```rust
// Solution:
fn log_out(s: &mut Session) -> Option<String> {
    s.token.take()
}
```

### Exercise 4 — signature

Write `count_vowels` that accepts a `String`, a `&String` and a literal without copying anything.

```rust
// Solution:
fn count_vowels(s: &str) -> usize {
    s.chars().filter(|c| "aeiouyAEIOUY".contains(*c)).count()
}
```

---

## 15. Cheat sheet

```rust
let t = s;              // move (non-Copy type)
let t = s.clone();      // explicit copy
let r = &s;             // read borrow (several possible)
let r = &mut s;         // write borrow (only one, nothing else at the same time)
*r = value;             // write through &mut
drop(s);                // free now
std::mem::take(&mut x)  // take, leaving Default
opt.take()              // take, leaving None
v.split_at_mut(i)       // two disjoint &mut
```

### Key points

- One owner only; the value is freed when it goes out of scope.
- Assigning, passing to a function or putting in a collection **moves** a non-`Copy` type.
- `Copy` for small types without resources, explicit `Clone` for the rest.
- One `&mut` or several `&`, never both.
- A borrow lasts until its last use: reordering often fixes the error.
- A `&self` method borrows the whole struct; direct field access borrows field by field.
- As a parameter: `&str` and `&[T]` to read, `&mut T` to modify, `T` to keep.
