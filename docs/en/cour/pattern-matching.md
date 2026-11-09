# Pattern Matching
> Every pattern in Rust, every place where they can be written, and the different ways to express the same match.

## Objectives

- Know every kind of pattern: literal, range, variable, wildcard, tuple, struct, enum, slice, reference
- Know where a pattern can appear: `match`, `let`, `if let`, `while let`, `let ... else`, `for`, function and closure parameters
- Tell refutable and irrefutable patterns apart
- Use `@`, `|`, `..`, guards and `matches!`
- Understand binding modes: when a pattern moves, copies or borrows
- Choose between `match`, `if let`, `let ... else`, `matches!` and the `Option` methods

---

## Table of contents

1. The `match` expression
2. Where to write a pattern
3. Refutable or irrefutable
4. Literals, constants and ranges
5. Variables, `_` and `..`
6. Tuples
7. Structs
8. Enums
9. Slices
10. References and binding modes
11. `@`: naming what you test
12. `|`: several patterns
13. Guards
14. Strings
15. The same match written in several ways
16. Exercises
17. Cheat sheet

---

## 1. The `match` expression

`match` compares a value to a list of `pattern => expression` arms, from top to bottom, and runs **the first one** that matches.

```rust
let number = 7;

match number {
    1 => println!("one"),
    2 | 3 => println!("two or three"),
    4..=6 => println!("between 4 and 6"),
    n => println!("other: {n}"),     // a variable matches anything
}
```

### Exhaustiveness

The compiler requires that **all** possible values are covered.

```rust
let x: Option<i32> = None;
// match x {
//     Some(n) => println!("{n}"),
// }                               // ERROR: pattern `None` not covered
```

### Arm order

```rust
match number {
    n => println!("everything"),
    1 => println!("one"),          // warning: unreachable pattern
}
```

An arm placed after a more general pattern is never reached: the compiler reports it. Put the **specific cases first**.

### `match` is an expression

```rust
let parity = match number % 2 {
    0 => "even",
    _ => "odd",
};
```

All arms must produce the same type (or diverge: `return`, `panic!`, `continue`).

---

## 2. Where to write a pattern

Patterns are not reserved for `match`.

```rust
// let
let (a, b) = (1, 2);

// if let / else
if let Some(n) = Some(5) { }

// while let
let mut stack = vec![1, 2];
while let Some(x) = stack.pop() { }

// let ... else
let Some(n) = Some(5) else { panic!() };

// for
for (i, c) in "abc".chars().enumerate() { }

// function parameters
fn show(&(x, y): &(i32, i32)) {
    println!("{x} {y}");
}

// closure parameters
let sum = |(a, b): (i32, i32)| a + b;

// matches!
let is_small = matches!(5, 0..=9);
```

---

## 3. Refutable or irrefutable

- An **irrefutable** pattern matches all possible values: `x`, `(a, b)`, `Point { x, y }`.
- A **refutable** pattern can fail: `Some(x)`, `1..=5`, `[first, ..]`.

| Place | Accepted pattern |
|---|---|
| `let`, `for`, function and closure parameters | irrefutable only |
| `if let`, `while let`, `let ... else`, `match` arms | refutable (an irrefutable one triggers a warning) |

```rust
let opt = Some(3);
// let Some(x) = opt;              // ERROR: refutable pattern in local binding
let Some(x) = opt else { return };   // OK: the None case is handled
if let Some(x) = opt { }             // OK
```

---

## 4. Literals, constants and ranges

### 4.1 — Literals

```rust
let value = 0;
let letter = 'a';
let flag = true;

match value {
    0 => {}
    -1 => {}
    _ => {}
}

match letter {
    'a' => {}
    _ => {}
}

match flag {
    true => {}
    false => {}
}
```

### 4.2 — Ranges

```rust
let n = 42;
match n {
    0 => "zero",
    1..=9 => "one digit",           // inclusive
    10..100 => "two digits",        // exclusive (Rust 1.80+): 10 to 99
    100.. => "three or more",       // open at the top
    ..=-1 => "negative",            // open at the bottom, inclusive
};

let c = 'k';
match c {
    'a'..='z' => "lowercase",
    'A'..='Z' => "uppercase",
    '0'..='9' => "digit",
    _ => "other",
};
```

Ranges work on integers and `char`. On floats they are accepted but discouraged (rounding problems).

### 4.3 — Constants

```rust
const ADMIN: u32 = 1;
const ANONYMOUS: u32 = 0;

fn role(id: u32) -> &'static str {
    match id {
        ADMIN => "admin",
        ANONYMOUS => "anonymous",
        _ => "member",
    }
}
```

A constant in `UPPERCASE` is compared to the value. Pitfall: a lowercase name is a **new variable** that catches everything.

```rust
let admin = 1;
match 5 {
    admin => println!("still here!"),   // admin is a new variable, equal to 5
}
```

The compiler warns (`unreachable pattern` on the following arms, unused variable), but only a constant (`const`) or a guard (`x if x == admin`) really compares.

---

## 5. Variables, `_` and `..`

### 5.1 — Binding a variable

```rust
match Some(5) {
    Some(n) => println!("{n}"),     // n receives the value
    None => {}
}
```

### 5.2 — `_`: ignoring a value

```rust
let (x, _) = (1, 2);           // the second element is ignored
match Some(5) {
    Some(_) => println!("there is something"),
    None => {}
}
```

`_` **binds nothing**: the value is neither moved nor copied.

```rust
let s = Some(String::from("a"));
if let Some(_) = s { }          // s is not moved
println!("{s:?}");              // OK

if let Some(_t) = s { }         // _t is a real variable: s is moved
// println!("{s:?}");           // ERROR
```

`_t` only silences the unused variable warning; `_` does not bind at all.

### 5.3 — `..`: ignoring the rest

```rust
struct Point3 { x: i32, y: i32, z: i32 }
let p = Point3 { x: 1, y: 2, z: 3 };

let Point3 { x, .. } = p;                // y and z ignored

let t = (1, 2, 3, 4, 5);
let (first, .., last) = t;               // 1 and 5
let (a, b, ..) = t;                      // 1 and 2
// let (.., m, ..) = t;                  // ERROR: .. can only appear once
```

---

## 6. Tuples

```rust
let point = (0, 5);

match point {
    (0, 0) => println!("origin"),
    (0, y) => println!("on the Y axis at {y}"),
    (x, 0) => println!("on the X axis at {x}"),
    (x, y) => println!("({x}, {y})"),
}
```

### Combining several values in a tuple

Matching on a tuple created for the occasion lets you test several values together.

```rust
fn access(logged_in: bool, admin: bool) -> &'static str {
    match (logged_in, admin) {
        (false, _) => "login required",
        (true, false) => "member access",
        (true, true) => "admin access",
    }
}
```

The compiler checks that all four combinations are covered.

---

## 7. Structs

### 7.1 — All the forms

```rust
struct Point { x: i32, y: i32 }
let p = Point { x: 3, y: 7 };

let Point { x, y } = p;                   // shorthand: variables x and y
let Point { x: abs, y: ord } = p;         // rename
let Point { x, .. } = p;                  // part of it

match p {
    Point { x: 0, y } => println!("on the Y axis at {y}"),   // fixed field + bound field
    Point { x, y: 0 } => println!("on the X axis at {x}"),
    Point { x: 1..=5, y: 1..=5 } => println!("inside the square"),
    Point { x, y } => println!("({x}, {y})"),
}
```

### 7.2 — Tuple struct and nesting

```rust
struct Color(u8, u8, u8);
struct Pixel { pos: Point, color: Color }

let px = Pixel { pos: Point { x: 0, y: 0 }, color: Color(255, 0, 0) };

match px {
    Pixel { color: Color(255, 0, 0), .. } => println!("red"),
    Pixel { pos: Point { x: 0, y: 0 }, color: Color(r, g, b) } => {
        println!("origin in {r},{g},{b}")
    }
    _ => {}
}
```

Patterns nest to any depth.

### 7.3 — In parameters

```rust
fn norm(&Point { x, y }: &Point) -> f64 {
    ((x * x + y * y) as f64).sqrt()
}
```

---

## 8. Enums

```rust
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    Color(u8, u8, u8),
}

fn handle(msg: Message) {
    match msg {
        Message::Quit => println!("quit"),
        Message::Move { x, y } => println!("to {x},{y}"),
        Message::Write(text) => println!("write: {text}"),
        Message::Color(r, _, _) if r > 200 => println!("rather red"),
        Message::Color(r, g, b) => println!("{r},{g},{b}"),
    }
}
```

### Nested enums

```rust
let r: Result<Option<i32>, String> = Ok(Some(3));

match r {
    Ok(Some(n)) if n > 0 => println!("positive {n}"),
    Ok(Some(_)) => println!("negative or zero"),
    Ok(None) => println!("absent"),
    Err(e) => println!("error {e}"),
}
```

A single `match` replaces two levels of `match` or `if let`.

---

## 9. Slices

```rust
fn describe(v: &[i32]) -> String {
    match v {
        [] => "empty".into(),
        [x] => format!("only one: {x}"),
        [a, b] => format!("two: {a}, {b}"),
        [first, .., last] => format!("from {first} to {last}"),
    }
}
```

### Binding the rest with `@ ..`

```rust
fn sum(v: &[i32]) -> i32 {
    match v {
        [] => 0,
        [head, rest @ ..] => head + sum(rest),   // rest: &[i32]
    }
}

fn command(args: &[&str]) {
    match args {
        ["help"] => println!("help"),
        ["add", name] => println!("adding {name}"),
        ["remove", names @ ..] if !names.is_empty() => println!("removing {names:?}"),
        _ => println!("unknown command"),
    }
}
```

### On a `Vec` or an array

```rust
let v = vec![1, 2, 3];
match v.as_slice() {               // a Vec is matched through its slice
    [1, ..] => println!("starts with 1"),
    _ => {}
}

let array = [1, 2, 3];
let [a, b, c] = array;             // known size: irrefutable pattern
```

---

## 10. References and binding modes

### 10.1 — The problem

```rust
let opt = Some(String::from("hello"));

match opt {
    Some(s) => println!("{s}"),    // s: String — the value is MOVED into s
    None => {}
}
// println!("{opt:?}");            // ERROR: opt was partially moved
```

### 10.2 — Three ways not to move

```rust
let opt = Some(String::from("hello"));

// 1. Match on a reference (modern form)
match &opt {
    Some(s) => println!("{s}"),    // s: &String
    None => {}
}

// 2. as_ref: Option<String> -> Option<&String>
match opt.as_ref() {
    Some(s) => println!("{s}"),
    None => {}
}

// 3. ref in the pattern (old form)
match opt {
    Some(ref s) => println!("{s}"),
    None => {}
}

println!("{opt:?}");               // OK in all three cases
```

### 10.3 — Modifying through the pattern

```rust
let mut opt = Some(5);

match &mut opt {
    Some(n) => *n += 1,            // n: &mut i32
    None => {}
}

if let Some(n) = opt.as_mut() {
    *n *= 2;
}

if let Some(ref mut n) = opt {
    *n -= 1;
}
```

### 10.4 — The binding mode rules

When you match a **reference** with a pattern that is not one (`Some(s)` against `&Option<String>`), Rust automatically switches to "borrow" mode: every bound variable becomes a reference.

| You match | Pattern | Variable type |
|---|---|---|
| `Option<String>` | `Some(s)` | `String` (moved) |
| `&Option<String>` | `Some(s)` | `&String` |
| `&mut Option<String>` | `Some(s)` | `&mut String` |
| `Option<String>` | `Some(ref s)` | `&String` |
| `&Option<i32>` | `&Some(n)` | `i32` (copied) |

### 10.5 — The `&` pattern: dereferencing

```rust
let numbers = vec![1, 2, 3];

for n in &numbers {               // n: &i32
    let _: &i32 = n;
}
for &n in &numbers {              // n: i32 (copied)
    let _: i32 = n;
}

let evens: Vec<&i32> = numbers.iter().filter(|&&x| x % 2 == 0).collect();
//                                         ^^^ filter receives &&i32: two & to remove
```

`&n` only works if the pointed value is `Copy`: you cannot move a `String` out of a reference.

### 10.6 — Edition 2024: stricter rules

In edition 2024, you can no longer write `ref`, `ref mut`, `mut` or `&` in a part of a pattern where Rust has already switched to automatic borrow mode.

```rust
let opt = &Some(5);
// if let Some(ref n) = opt { }    // rejected in 2024: n is already a reference
if let Some(n) = opt { }           // n: &i32
if let &Some(n) = opt { }          // n: i32, the & is at the outer level: accepted
```

`cargo fix --edition` automatically rewrites the affected patterns.

---

## 11. `@`: naming what you test

`name @ pattern` tests the pattern **and** binds the whole value.

```rust
let age = 15;
match age {
    n @ 0..=12 => println!("child of {n} years"),
    n @ 13..=17 => println!("teenager of {n} years"),
    n => println!("adult of {n} years"),
}

// With several alternatives: parentheses
match age {
    n @ (1 | 5 | 10) => println!("round age: {n}"),
    _ => {}
}

// In an enum
enum Message { Id { id: i32 } }
let msg = Message::Id { id: 5 };
match msg {
    Message::Id { id: id_variable @ 3..=7 } => println!("id in range: {id_variable}"),
    Message::Id { id } => println!("other id: {id}"),
}
```

Without `@`, you would have to choose between testing the range (`3..=7`) and retrieving the value (`id`). A guard is the other solution: `Message::Id { id } if (3..=7).contains(&id)`.

---

## 12. `|`: several patterns

```rust
let n = 1;
let opt = Some(1);

match n {
    1 | 2 | 3 => "small",
    _ => "other",
};

// Nested (Rust 1.53+)
match opt {
    Some(1 | 2) => "one or two",
    Some(_) => "other number",
    None => "nothing",
};

// With bindings: each alternative must bind the same variables, of the same type
enum Shape { Square(f64), Circle(f64), Rectangle(f64, f64) }
fn dimension(s: &Shape) -> f64 {
    match s {
        Shape::Square(c) | Shape::Circle(c) => *c,
        Shape::Rectangle(w, _) => *w,
    }
}
```

---

## 13. Guards

A guard `if condition` adds a test after the pattern. It can use the bound variables and all visible variables.

```rust
let pair = (2, -2);

match pair {
    (x, y) if x == y => println!("equal"),
    (x, y) if x + y == 0 => println!("opposite"),
    (x, _) if x % 2 == 0 => println!("x even"),
    _ => println!("other"),
}
```

### Comparing to a variable

```rust
let target = 5;
match Some(5) {
    Some(n) if n == target => println!("found"),
    Some(_) => println!("other"),
    None => {}
}
```

### Guard and `|`

```rust
let x = 4;
let y = false;
match x {
    4 | 5 | 6 if y => println!("yes"),     // the guard applies to all THREE values
    _ => println!("no"),
}
```

`4 | 5 | 6 if y` means `(4 | 5 | 6) if y`, not `4 | 5 | (6 if y)`.

### Guards and exhaustiveness

The compiler does **not** take guards into account when checking exhaustiveness:

```rust
match x {
    n if n > 0 => {}
    n if n <= 0 => {}
    _ => {}          // required: the compiler does not know the two guards cover everything
}
```

When possible, prefer ranges (`1..` and `..=0`): they are checked.

---

## 14. Strings

You cannot match a `String` directly against literals: you need a `&str`.

```rust
let command = String::from("stop");

match command.as_str() {
    "start" => println!("starting"),
    "stop" => println!("stopping"),
    other => println!("unknown: {other}"),
}

// Case-insensitive
match command.to_lowercase().as_str() {
    "stop" | "halt" => {}
    _ => {}
}

// By prefix: guard or strip_prefix
match command.as_str() {
    c if c.starts_with("get ") => println!("read"),
    _ => {}
}
if let Some(name) = command.strip_prefix("get ") {
    println!("reading {name}");
}
```

On an `Option<String>`, `as_deref` gives a matchable `Option<&str>`:

```rust
let language: Option<String> = Some("fr".into());
match language.as_deref() {
    Some("fr") => "Bonjour",
    Some("en") => "Hello",
    Some(_) | None => "Hello",
};
```

---

## 15. The same match written in several ways

### 15.1 — Extracting a value or leaving

```rust
// 1. match
fn length_v1(name: Option<&str>) -> usize {
    let n = match name {
        Some(n) => n,
        None => return 0,
    };
    n.len()
}

// 2. let else: same effect, the variable stays at the top level
fn length_v2(name: Option<&str>) -> usize {
    let Some(n) = name else { return 0 };
    n.len()
}

// 3. if let: the rest lives inside the block
fn length_v3(name: Option<&str>) -> usize {
    if let Some(n) = name {
        n.len()
    } else {
        0
    }
}

// 4. Option method
fn length_v4(name: Option<&str>) -> usize {
    name.map_or(0, str::len)
}
```

### 15.2 — Testing a shape

```rust
enum State { Active { since: u32 }, Suspended, Deleted }

fn is_active(s: &State) -> bool {
    // 1. match
    match s {
        State::Active { .. } => true,
        _ => false,
    }
}

fn is_active_v2(s: &State) -> bool {
    // 2. matches!
    matches!(s, State::Active { .. })
}

fn is_long_active(s: &State) -> bool {
    // 3. let-chain (edition 2024) when a condition is added
    if let State::Active { since } = s && *since > 30 {
        true
    } else {
        false
    }
}

fn is_long_active_v2(s: &State) -> bool {
    // 4. matches! with a guard: the same thing in one line
    matches!(s, State::Active { since } if *since > 30)
}
```

### 15.3 — Several levels

```rust
struct Request { user: Option<User> }
struct User { email: Option<String> }

fn email(r: &Request) -> Option<&str> {
    // 1. nested if let
    if let Some(u) = &r.user {
        if let Some(e) = &u.email {
            return Some(e.as_str());
        }
    }
    None
}

fn email_pattern(r: &Request) -> Option<&str> {
    // 2. a single nested pattern
    match r {
        Request { user: Some(User { email: Some(e) }) } => Some(e.as_str()),
        _ => None,
    }
}

fn email_chain(r: &Request) -> Option<&str> {
    // 3. methods
    r.user.as_ref()?.email.as_deref()
}
```

### 15.4 — Choosing

| Need | Form |
|---|---|
| Several cases, each with its own handling | `match` |
| A single case triggers an action | `if let` |
| The missing case stops the function | `let ... else` |
| Getting a `bool` | `matches!` |
| Transforming an `Option` / a `Result` | `map`, `and_then`, `?` |
| A pattern plus conditions | let-chain or guard |

---

## 16. Exercises

### Exercise 1 — classifying a point

With a single `match` on `(x, y)`, return "origin", "X axis", "Y axis", "quadrant 1" to "quadrant 4".

```rust
// Solution:
fn quadrant(p: (i32, i32)) -> &'static str {
    match p {
        (0, 0) => "origin",
        (_, 0) => "X axis",
        (0, _) => "Y axis",
        (1.., 1..) => "quadrant 1",
        (..=-1, 1..) => "quadrant 2",
        (..=-1, ..=-1) => "quadrant 3",
        (1.., ..=-1) => "quadrant 4",
    }
}
```

Open ranges make the `match` exhaustive without `_`: the compiler checks that no point is forgotten.

### Exercise 2 — command parser

Match a `&[&str]` slice: `["go", direction]`, `["take", items @ ..]` with at least one item, `["quit"]`, otherwise "unknown".

```rust
// Solution:
fn parse(words: &[&str]) -> String {
    match words {
        ["go", dir] => format!("go {dir}"),
        ["take", items @ ..] if !items.is_empty() => format!("take {}", items.join(", ")),
        ["quit"] => "goodbye".into(),
        _ => "unknown".into(),
    }
}
```

### Exercise 3 — without moving

Print the content of an `Option<String>` in a `match`, then reuse the variable. Write it in two ways.

```rust
// Solution:
fn main() {
    let name = Some(String::from("Ada"));
    match &name {
        Some(n) => println!("{n}"),
        None => {}
    }
    if let Some(n) = name.as_deref() {
        println!("{n}");
    }
    println!("{name:?}");
}
```

## 17. Cheat sheet

```rust
match v {
    0 => {}                       // literal
    1 | 2 => {}                   // alternatives
    3..=9 => {}                   // inclusive range
    10..20 => {}                  // exclusive range
    100.. => {}                   // open range
    CONSTANT => {}                // comparison with a const
    n @ 20..=29 => {}             // test + binding
    n if n < 0 => {}              // guard
    _ => {}                       // everything else, without binding
}

let (a, _, ..) = tuple;           // ignore
let Point { x, y: ord, .. } = p;  // struct, renaming
match slice { [] => {}, [x] => {}, [first, rest @ ..] => {} }
match &opt { Some(s) => {}, None => {} }        // borrows instead of moving
match s.as_str() { "a" => {}, _ => {} }         // String against literals
if let Some(x) = opt {}
let Some(x) = opt else { return };
matches!(v, Some(1..=5))
```

### Key points

- The first arm that matches wins: specific cases first.
- A lowercase name in a pattern is a new variable, never a comparison.
- `_` binds nothing; `_x` binds (and can move).
- Matching on `&value` avoids moving the content.
- `@` combines test and binding; `|` nests; a guard applies to all the alternatives of the arm.
- Guards do not count for exhaustiveness: prefer ranges when possible.
