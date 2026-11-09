# Rust fundamentals

**Variables, types and functions** — every way to declare, convert, print and organize code into functions, and what each one brings.

## Objectives

By the end of this course, you will know how to:

- Declare variables, constants and statics, and choose between them
- Use shadowing instead of mutability when it is clearer
- Know each scalar type, its limits and its behavior on overflow
- Convert between types in every way (`as`, `From`, `TryFrom`, `parse`)
- Write functions: parameters, return values, multiple returns, early exits
- Replace what does not exist in Rust: default parameters, overloading, variable number of arguments
- Print values with every `println!` format
- Understand the basics of ownership and references

## Table of contents

1. Variables
2. Constants and statics
3. Shadowing
4. Scalar types
5. Integer overflow
6. Compound types
7. Type inference and annotation
8. Conversions between types
9. Functions
10. Expressions and statements
11. What does not exist in Rust and how to do without
12. Printing values
13. Utility macros
14. Ownership and references (introduction)
15. Exercises
16. Cheat sheet

## 1. Variables

### 1.1 — Declaration

```rust
fn main() {
    let x = 5;                 // inferred type: i32
    let y: f64 = 3.14;         // annotated type
    let name = "Alice";        // &str
    let active = true;         // bool
    println!("{x} {y} {name} {active}");
}
```

### 1.2 — Immutable by default

```rust
let x = 5;
// x = 6;   // ERROR: cannot assign twice to immutable variable
```

What it is for: when a variable does not change, you know it by reading its declaration, without scanning the whole function. The compiler can also optimize better.

### 1.3 — `mut`

```rust
let mut counter = 0;
counter += 1;
counter = counter * 2;
```

`mut` tells the reader the value will change. The compiler warns (`unused_mut`) if you declare a variable `mut` that is never modified.

### 1.4 — Declare now, initialize later

```rust
let result: i32;             // declared, not initialized
let condition = true;
if condition {
    result = 1;
} else {
    result = 2;
}
println!("{result}");        // OK: initialized in all branches
```

No need for `mut`: the variable is assigned only **once**. The compiler refuses to read it before it is initialized on every path. In practice, `let result = if condition { 1 } else { 2 };` is more direct.

### 1.5 — Destructuring at declaration

```rust
let (a, b) = (1, 2);
let (x, _) = (10, 20);            // _ ignores a value
let [first, .., last] = [1, 2, 3, 4];
let (mut c, d) = (0, 0);          // mut on a single variable
c += 1;

struct Point { x: i32, y: i32 }
let Point { x, y } = Point { x: 1, y: 2 };
```

### 1.6 — Unused variables

```rust
let _unused = 5;         // the _ prefix suppresses the warning
let _ = compute();       // _ alone: the value is dropped immediately

fn compute() -> i32 { 1 }
```

Subtle difference: `let _x = lock()` keeps the value until the end of the block, `let _ = lock()` releases it right away. With a lock or a connection, the difference matters.

## 2. Constants and statics

### 2.1 — `const`

```rust
const MAX_POINTS: u32 = 100_000;
const DELAY_SECONDS: u64 = 60 * 60 * 24;    // computed at compile time

fn main() {
    println!("{MAX_POINTS}");
}
```

- Type **required**.
- Value computable at compile time.
- Name in `UPPER_SNAKE_CASE`.
- Can be declared anywhere, including outside functions.
- The value is **copied** at each use: a constant has no fixed address.

### 2.2 — `static`

```rust
static APP_NAME: &str = "Runique";
static mut COUNTER: u32 = 0;    // avoid: every access requires unsafe

fn main() {
    println!("{APP_NAME}");
}
```

A `static` has **a single address** in memory for the whole duration of the program. A `static mut` is almost always a mistake: since the 2024 edition, even taking a reference to it is rejected. Use a safe interior-mutability type instead.

```rust
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::LazyLock;
use std::collections::HashMap;

static COUNTER: AtomicU32 = AtomicU32::new(0);

// Value computed on first access (Rust 1.80+)
static CONFIG: LazyLock<HashMap<&str, &str>> = LazyLock::new(|| {
    HashMap::from([("language", "en")])
});

fn main() {
    COUNTER.fetch_add(1, Ordering::Relaxed);
    println!("{}", CONFIG["language"]);
}
```

### 2.3 — Choosing

| Need | Choice |
|---|---|
| Fixed value known at compile time (size, limit, text) | `const` |
| Single global value with an address (large array, shared reference) | `static` |
| Global value computed at startup (regex, configuration) | `static` + `LazyLock` |
| Modifiable global counter | `static` + atomic or `Mutex` |
| Value that depends on runtime inside a function | `let` |

### 2.4 — `const fn`

```rust
const fn kilobytes(n: usize) -> usize {
    n * 1024
}

const MAX_SIZE: usize = kilobytes(512);   // evaluated at compile time

fn main() {
    let a = kilobytes(3);                  // also usable at runtime
}
```

A `const fn` can be called in a constant. Its body is limited (no allocation, no non-const trait), but `if`, `match`, `loop` and `while` are allowed.

## 3. Shadowing

Redeclaring a variable with `let` creates a **new** variable that hides the old one.

### 3.1 — Transforming a value in steps

```rust
let input = "  42  ";
let input = input.trim();                   // &str
let input: u32 = input.parse().unwrap();    // u32: the type changes
println!("{input}");
```

### 3.2 — `mut` or shadowing: the same thing written twice

```rust
// Version 1: mut — the type cannot change
let mut price = 100;
price = price * 2;
price = price + 5;

// Version 2: shadowing — each step is immutable
let price = 100;
let price = price * 2;
let price = price + 5;
```

| | `mut` | shadowing |
|---|---|---|
| Change the type | no | yes |
| The variable stays modifiable afterwards | yes | no (except `let mut`) |
| Suited to a loop | yes | no |
| Suited to a series of transformations | less | yes |

### 3.3 — Shadowing in a block

```rust
let x = 5;
{
    let x = x * 2;
    println!("in the block: {x}");   // 10
}
println!("after: {x}");              // 5: the old variable reappears
```

### 3.4 — Making immutable after preparation

```rust
let mut list = vec![3, 1, 2];
list.sort();
let list = list;   // from here on, no more modification possible
```

## 4. Scalar types

### 4.1 — Integers

| Signed | Unsigned | Size | Range (signed) |
|---|---|---|---|
| `i8` | `u8` | 8 bits | -128 to 127 / 0 to 255 |
| `i16` | `u16` | 16 bits | -32,768 to 32,767 |
| `i32` | `u32` | 32 bits | about ±2.1 billion |
| `i64` | `u64` | 64 bits | about ±9.2 × 10¹⁸ |
| `i128` | `u128` | 128 bits | very large |
| `isize` | `usize` | pointer size (64 bits on a PC) | depends on the machine |

- `i32`: default type of an integer literal.
- `usize`: type of **indexes** and **lengths** (`v.len()`, `v[i]`).
- `u8`: bytes.
- `u64`/`i64`: identifiers, timestamps, file sizes.

```rust
println!("{} {}", i32::MIN, i32::MAX);
println!("{}", u8::MAX);   // 255
```

### 4.2 — Writing a literal

```rust
let million = 1_000_000;      // _ for readability
let hex = 0xff;               // 255
let octal = 0o77;             // 63
let binary = 0b1111_0000;     // 240
let byte = b'A';              // u8: 65
let with_suffix = 42u8;       // type suffix
let big = 5_000_000_000i64;
let float = 1e6;              // 1,000,000.0
```

### 4.3 — Floats

```rust
let a = 2.0;        // f64 by default
let b: f32 = 3.0;   // less precise, half the memory

println!("{}", 0.1 + 0.2);            // 0.30000000000000004
println!("{}", 0.1 + 0.2 == 0.3);     // false

// Compare with a tolerance
let equal = (0.1_f64 + 0.2 - 0.3).abs() < f64::EPSILON;

println!("{}", 1.0 / 0.0);            // inf
println!("{}", (0.0_f64 / 0.0).is_nan()); // true
```

A float is **not** exact: never use `f64` for money. Amounts are stored in cents (`i64`) or as a decimal (`rust_decimal::Decimal`, which is what Runique's `decimal` field does).

`f64` implements neither `Eq` nor `Ord` (because of `NaN`): you cannot use it as a `HashMap` key or sort a `Vec<f64>` with `.sort()`. Use `v.sort_by(|a, b| a.total_cmp(b))`.

### 4.4 — Operations

```rust
let sum = 5 + 10;
let difference = 95.5 - 4.3;
let product = 4 * 30;
let quotient = 7 / 2;        // 3: integer division, truncated toward zero
let quotient_f = 7.0 / 2.0;  // 3.5
let remainder = 7 % 2;       // 1
let negative = -7 / 2;       // -3
let modulo = (-7_i32).rem_euclid(2); // 1: always positive

let power = 2_i32.pow(10);   // 1024
let root = 16.0_f64.sqrt();  // 4.0
let absolute = (-5_i32).abs();
let max = 3.max(7);
let bounded = 15.clamp(0, 10);   // 10
```

There is no `++` or `--` operator: write `x += 1`.

Both operands must have **the same type**: `5_i32 + 2_i64` does not compile, you must convert (section 8).

### 4.5 — Boolean and character

```rust
let yes: bool = true;
let c: char = 'é';        // 4 bytes: one Unicode character
let kanji = '日';
let digit = '7'.to_digit(10);   // Some(7)
let is_letter = 'a'.is_alphabetic();
let upper = 'a'.to_ascii_uppercase();
```

`char` (single quotes) is a Unicode character; `&str` (double quotes) is a UTF-8 string. `'a'` and `"a"` are two different types.

## 5. Integer overflow

### 5.1 — The default behavior

```rust
fn add(a: u8, b: u8) -> u8 {
    a + b
}

fn main() {
    let r = add(250, 10);
    // cargo run           : panics "attempt to add with overflow"
    // cargo run --release : r is 4 (260 - 256), with no error
}
```

In **debug**, an overflow panics. In **release**, it silently **wraps around**. Never count on one or the other: choose explicitly.

### 5.2 — The four ways to handle an overflow

```rust
let a: u8 = 250;

// 1. checked: None on overflow
let r1 = a.checked_add(10);        // None
let r2 = a.checked_add(5);         // Some(255)

// 2. saturating: stops at the limit
let r3 = a.saturating_add(10);     // 255
let r4 = 5_u8.saturating_sub(10);  // 0

// 3. wrapping: intended wrap-around
let r5 = a.wrapping_add(10);       // 4

// 4. overflowing: wrapped result + indicator
let (r6, overflowed) = a.overflowing_add(10); // (4, true)
```

| Method | What it is for |
|---|---|
| `checked_*` | user input, size computations: refuse cleanly |
| `saturating_*` | counters, gauges, remaining time: stay within bounds |
| `wrapping_*` | hashing, random numbers, sequence numbers: wrap-around is intended |
| `overflowing_*` | multi-word arithmetic (big numbers) |

### 5.3 — The unsigned subtraction pitfall

```rust
let length: usize = 3;
let index: usize = 5;
// let gap = length - index;               // panics in debug, huge number in release
let gap = length.saturating_sub(index);    // 0
let gap = length.checked_sub(index);       // None
let gap = length.abs_diff(index);          // 2
```

## 6. Compound types

### 6.1 — Tuples

```rust
let point: (i32, f64, &str) = (10, 3.5, "a");

let first = point.0;                // access by position
let (x, y, name) = point;           // destructuring

let empty = ();                     // unit: Rust's "nothing"
let single = (5,);                  // one-element tuple: the comma is required
let not_a_tuple = (5);              // simply 5 in parentheses
```

### 6.2 — Arrays

```rust
let numbers: [i32; 5] = [1, 2, 3, 4, 5];  // fixed size, known at compile time
let zeros = [0; 100];                      // 100 times 0
let first = numbers[0];
let length = numbers.len();

// let x = numbers[10];          // compile error: constant index out of bounds
let i = 10;
let x = numbers.get(i);          // None, no panic
```

An array cannot grow. For a variable size: `Vec<T>` (course `collections`).

### 6.3 — Slices

```rust
let numbers = [1, 2, 3, 4, 5];
let middle: &[i32] = &numbers[1..4];   // [2, 3, 4]
let start = &numbers[..2];             // [1, 2]
let end = &numbers[3..];               // [4, 5]
let all = &numbers[..];

let text = String::from("hello");
let word: &str = &text[0..3];          // "hel"
```

A slice is a **view** onto part of an array, a `Vec` or a string, without copying. A function that takes `&[T]` accepts an array, a `Vec` or a part of either.

Careful: `&text[0..3]` cuts in **bytes**. Cutting in the middle of an accented character panics.

### 6.4 — `String` and `&str`

```rust
let fixed: &str = "literal";          // text baked into the binary, not modifiable
let mut owned = String::from("a");    // allocated text, modifiable
owned.push_str("bc");

let view: &str = &owned;              // a String can be lent as &str
let copy: String = fixed.to_string(); // a &str is copied into a String
let copy2: String = fixed.to_owned();
let copy3 = String::from(fixed);
let copy4: String = fixed.into();
```

The last four lines do the same thing. Rule for parameters: accept `&str`, return `String`.

## 7. Type inference and annotation

### 7.1 — Where the annotation is needed

```rust
let x = 5;                  // inferred: i32
let v = Vec::new();         // ERROR if nothing lets the compiler guess the element type...
let mut v2 = Vec::new();
v2.push(1u8);               // ...here it can: Vec<u8>

let n = "42".parse().unwrap();      // ERROR: parse to which type?
```

### 7.2 — Three ways to specify the type

```rust
// 1. Annotate the variable
let n: u32 = "42".parse().unwrap();

// 2. Turbofish ::<> on the function
let n = "42".parse::<u32>().unwrap();

// 3. Suffix on a literal
let n = 42u32;

// With collect
let v: Vec<i32> = (1..4).collect();
let v = (1..4).collect::<Vec<i32>>();
let v = (1..4).collect::<Vec<_>>();     // _ : lets the compiler guess the element type
```

The turbofish is useful when there is no variable to annotate: in the middle of a chain, or in a function argument.

## 8. Conversions between types

Rust **never** converts implicitly between numeric types.

### 8.1 — `as`: the raw conversion

```rust
let a: i32 = 300;
let b = a as u8;          // 44: truncated (300 - 256), no error
let c = -1_i32 as u32;    // 4294967295
let d = 3.99_f64 as i32;  // 3: truncated toward zero
let e = 1e20_f64 as i32;  // 2147483647: saturated at the limit
let f = 'A' as u8;        // 65
let g = 65_u8 as char;    // 'A'
let h = true as i32;      // 1
```

`as` cannot fail: it **silently loses** information. Keep it for cases where the loss is intended or impossible.

### 8.2 — `From` / `Into`: the lossless conversion

```rust
let small: u8 = 200;
let big: u32 = u32::from(small);     // always possible
let big: u32 = small.into();         // same thing, type inferred from the variable
let f: f64 = f64::from(3_i32);

// u32 -> u8 does not exist with From: the loss would be possible
// let x: u8 = u8::from(300_u32);   // compile ERROR
```

### 8.3 — `TryFrom` / `TryInto`: the conversion that can fail

```rust
use std::convert::TryFrom;

let big: i64 = 300;
let r = u8::try_from(big);            // Err(TryFromIntError)
let ok: Result<u8, _> = 200_i64.try_into(); // Ok(200)

fn valid_index(i: i64) -> Option<usize> {
    usize::try_from(i).ok()           // negative -> None
}
```

### 8.4 — `parse` and `to_string`: from and to text

```rust
let n: i32 = "42".parse().unwrap();
let r = "abc".parse::<i32>();         // Err(ParseIntError)
let f: f64 = "3.14".parse().unwrap();
let b: bool = "true".parse().unwrap();

let t = 42.to_string();
let t = format!("{:.2}", 3.14159);    // "3.14"
```

### 8.5 — Choosing

| Situation | Tool |
|---|---|
| Conversion always possible (`u8` to `u32`, `i32` to `f64`) | `From` / `into()` |
| Conversion that can fail (`i64` to `u8`, negative to `usize`) | `TryFrom` / `try_into()` |
| Text to number | `parse` |
| Number to text | `to_string`, `format!` |
| Intended loss (truncation, byte) | `as` |

## 9. Functions

### 9.1 — Declaration

```rust
fn greet() {
    println!("Hello!");
}

fn main() {
    greet();
}
```

Names in `snake_case`. The order of declaration does not matter: a function can be called before it is written in the file.

### 9.2 — Parameters

```rust
fn introduce(name: &str, age: u32) {
    println!("{name} is {age} years old");
}

// Destructured parameter
fn distance((x, y): (f64, f64)) -> f64 {
    (x * x + y * y).sqrt()
}

// Locally modifiable parameter (the copy, not the original)
fn countdown(mut n: u32) {
    while n > 0 {
        n -= 1;
    }
}

// Ignored parameter
fn callback(_event: &str) {}
```

The type of each parameter is **required**: there is no inference on the signature.

### 9.3 — Return value: three ways to write it

```rust
// 1. Last expression, without a semicolon (idiomatic form)
fn square(x: i32) -> i32 {
    x * x
}

// 2. Explicit return
fn square_again(x: i32) -> i32 {
    return x * x;
}

// 3. Early return then final expression
fn absolute_value(x: i32) -> i32 {
    if x >= 0 {
        return x;
    }
    -x
}
```

Keep `return` for **early** exits. An extra semicolon turns the final expression into a statement:

```rust
fn square_broken(x: i32) -> i32 {
    x * x;   // ERROR: mismatched types, expected i32, found ()
}
```

### 9.4 — Returning several values

```rust
// 1. Tuple
fn min_max(v: &[i32]) -> (i32, i32) {
    let mut min = v[0];
    let mut max = v[0];
    for &x in v {
        min = min.min(x);
        max = max.max(x);
    }
    (min, max)
}

// 2. Named struct: more readable beyond two values
struct Stats { min: i32, max: i32, mean: f64 }

fn stats(v: &[i32]) -> Stats {
    let (min, max) = min_max(v);
    let mean = v.iter().sum::<i32>() as f64 / v.len() as f64;
    Stats { min, max, mean }
}

fn main() {
    let (min, max) = min_max(&[3, 1, 4]);
    let s = stats(&[3, 1, 4]);
    println!("{} {}", s.min, s.mean);
}
```

### 9.5 — Returning "nothing" or "maybe"

```rust
fn log(msg: &str) {}                       // returns () implicitly
fn log_explicit(msg: &str) -> () {}        // identical

fn first_even(v: &[i32]) -> Option<i32> {  // a value or nothing
    v.iter().copied().find(|x| x % 2 == 0)
}

fn divide(a: i32, b: i32) -> Result<i32, String> {  // a value or an error
    if b == 0 {
        return Err("division by zero".to_string());
    }
    Ok(a / b)
}
```

`first_even` with an empty `v` does not panic, unlike `min_max` above, which reads `v[0]`. Returning an `Option` is the way to make an impossible case visible in the signature.

### 9.6 — A function that never returns

```rust
fn abort(message: &str) -> ! {
    panic!("{message}");
}

fn server_loop() -> ! {
    loop {
        // accept connections
    }
}
```

The type `!` signals that the function never gives control back.

### 9.7 — Parameters by value or by reference

```rust
fn length(s: &String) -> usize { s.len() }         // borrows: the caller keeps the String
fn length_better(s: &str) -> usize { s.len() }     // accepts &String and "literal"
fn append(s: &mut String) { s.push('!'); }         // borrows for modification
fn consume(s: String) -> String { s + "!" }        // takes the String, returns a new one

fn main() {
    let mut t = String::from("a");
    length(&t);
    length_better(&t);
    length_better("literal");
    append(&mut t);
    let t = consume(t);
}
```

| Signature | When to use it |
|---|---|
| `x: T` (`Copy` type: numbers, `bool`, `char`) | always by value, the copy is free |
| `x: &T`, `&str`, `&[T]` | read without taking |
| `x: &mut T` | modify the caller's value |
| `x: T` (owned type: `String`, `Vec`) | the function keeps or transforms the value |

### 9.8 — Nested function

```rust
fn process(v: &[i32]) -> i32 {
    fn double(x: i32) -> i32 {
        x * 2
    }
    v.iter().map(|&x| double(x)).sum()
}
```

A nested function is only visible inside the function that contains it, but it does **not capture** its variables. To capture, use a closure (`let double = |x| x * factor;`).

### 9.9 — Methods and associated functions

```rust
struct Rectangle { width: u32, height: u32 }

impl Rectangle {
    fn new(width: u32, height: u32) -> Self {        // associated function (no self)
        Self { width, height }
    }
    fn area(&self) -> u32 {                          // method that reads
        self.width * self.height
    }
    fn grow(&mut self, n: u32) {                     // method that modifies
        self.width += n;
    }
    fn into_square(self) -> Rectangle {              // method that consumes
        Rectangle { height: self.width, ..self }
    }
}

fn main() {
    let mut r = Rectangle::new(2, 3);   // call with ::
    r.grow(1);                          // call with .
    let a = Rectangle::area(&r);        // same as r.area()
}
```

Details in the course `structures-enums`.

### 9.10 — Functions as values

```rust
fn double(x: i32) -> i32 { x * 2 }

fn apply(f: fn(i32) -> i32, x: i32) -> i32 {
    f(x)
}

fn main() {
    let f = double;                  // a function is a value
    println!("{}", apply(f, 5));     // 10
    let v: Vec<i32> = [1, 2].iter().map(|&x| double(x)).collect();
    let v: Vec<i32> = [1, 2].into_iter().map(double).collect(); // passed directly
}
```

### 9.11 — Documenting and marking a function

```rust
/// Computes the area of a circle.
///
/// # Example
///
/// ```
/// assert_eq!(area(1.0).round(), 3.0);
/// ```
#[must_use]
pub fn area(radius: f64) -> f64 {
    std::f64::consts::PI * radius * radius
}

fn main() {
    area(2.0);   // warning: the returned value is not used
}
```

`///` generates the documentation (`cargo doc`) and its examples are run as tests. `#[must_use]` warns the caller who ignores the result: useful on any function without side effects.

## 10. Expressions and statements

### 10.1 — The difference

- An **expression** produces a value: `5`, `x + 1`, `if`, `match`, a `{ }` block, a function call.
- A **statement** does not: `let x = 5;`, an expression followed by `;`.

```rust
let y = {
    let x = 3;
    x + 1           // last expression of the block: its value
};                  // y = 4

let z = {
    let x = 3;
    x + 1;          // ; : the block is worth ()
};                  // z = ()
```

### 10.2 — What follows from this

```rust
// let x = (let y = 6);     // ERROR: let is not an expression
// let a = b = 5;           // ERROR: assignment is worth (), not 5

let value = match 3 { 1 => "one", _ => "other" };
let status = if true { "ok" } else { "ko" };
let n = loop { break 7; };
```

Almost everything is an expression in Rust: this is what lets you write immutable variables without a temporary value.

## 11. What does not exist in Rust and how to do without

### 11.1 — Default parameters

There is no `fn f(x: i32 = 5)`. Four ways to get the same effect:

```rust
// 1. Option: None means "default value"
fn connect(port: Option<u16>) -> u16 {
    port.unwrap_or(8080)
}

// 2. Two functions
fn connect_default() -> u16 { connect_on(8080) }
fn connect_on(port: u16) -> u16 { port }

// 3. Configuration struct with Default
#[derive(Default)]
struct Options { port: u16, verbose: bool }

fn start(opts: Options) {}

// 4. Builder: each option is a method
struct Server { port: u16, verbose: bool }
impl Server {
    fn new() -> Self { Self { port: 8080, verbose: false } }
    fn port(mut self, p: u16) -> Self { self.port = p; self }
    fn verbose(mut self) -> Self { self.verbose = true; self }
}

fn main() {
    connect(None);
    connect(Some(3000));
    start(Options { port: 3000, ..Default::default() });
    let s = Server::new().port(3000).verbose();
}
```

The builder is the form Runique uses (`RuniqueAppBuilder::new(config).with_database()...`): it stays readable with many options and adding one breaks no existing call.

### 11.2 — Overloading (same name, different parameters)

```rust
// Forbidden: two functions cannot have the same name
// fn show(x: i32) {}
// fn show(x: &str) {}

// 1. Different names
fn show_number(x: i32) {}
fn show_text(x: &str) {}

// 2. A generic with a trait
fn show<T: std::fmt::Display>(x: T) {
    println!("{x}");
}

// 3. Into to accept several types
fn title(t: impl Into<String>) -> String {
    t.into()
}

fn main() {
    show(5);
    show("a");
    title("literal");
    title(String::from("owned"));
}
```

### 11.3 — Variable number of arguments

```rust
// 1. A slice
fn total(numbers: &[i32]) -> i32 {
    numbers.iter().sum()
}

// 2. An iterator: accepts a Vec, an array, a range
fn total_iter(numbers: impl IntoIterator<Item = i32>) -> i32 {
    numbers.into_iter().sum()
}

fn main() {
    total(&[1, 2, 3]);
    total_iter(vec![1, 2]);
    total_iter(1..=10);
}
```

Only macros (`println!`, `vec!`) really accept a variable number of arguments.

### 11.4 — Null value

There is no `null`. An absent value is written `Option<T>`: `None` or `Some(value)`. The compiler forces you to handle the `None` case before using the value.

### 11.5 — Exceptions

There is no `try`/`catch`. An expected error is a `Result<T, E>` value, propagated with `?`. A programming error does `panic!`. See the course `gestion-des-erreurs`.

## 12. Printing values

### 12.1 — The printing macros

```rust
println!("with a newline");
print!("without a newline");
eprintln!("to the error output");
let s: String = format!("into a String: {}", 5);
```

### 12.2 — Placing values: four ways to write it

```rust
let name = "Ada";
let age = 36;

println!("{} is {} years old", name, age);        // 1. by position
println!("{0} is {1} years old, {0}!", name, age); // 2. by index, reusable
println!("{n} is {a} years old", n = name, a = age); // 3. by name
println!("{name} is {age} years old");            // 4. captured variable (Rust 1.58+)
```

Form 4 only works with a **variable name**, not an expression: `{person.name}` or `{x + 1}` are rejected, so use form 1.

### 12.3 — `Display` and `Debug`

```rust
let v = vec![1, 2, 3];
println!("{v:?}");    // [1, 2, 3]        Debug: for the developer
println!("{v:#?}");   // one value per line, indented

#[derive(Debug)]
struct Point { x: i32, y: i32 }
println!("{:?}", Point { x: 1, y: 2 });   // Point { x: 1, y: 2 }
```

`{}` requires the `Display` trait (output for the user, written by hand). `{:?}` requires `Debug` (often derived).

### 12.4 — Width, alignment, precision

```rust
let x = 3.14159;
println!("[{x:.2}]");       // [3.14]          2 decimals
println!("[{x:8.2}]");      // [    3.14]      width 8
println!("[{x:<8.2}]");     // [3.14    ]      left-aligned
println!("[{x:^8.2}]");     // [  3.14  ]      centered
println!("[{x:08.2}]");     // [00003.14]      padded with zeros
println!("[{:*>6}]", "ab"); // [****ab]        fill character
println!("[{:+}]", 5);      // [+5]            sign always shown

let width = 10;
println!("[{:>width$}]", "a");   // width in a variable
println!("[{:.*}]", 3, x);       // precision passed as an argument: 3.142
```

### 12.5 — Bases and notations

```rust
let n = 255;
println!("{n:x}");    // ff
println!("{n:X}");    // FF
println!("{n:#x}");   // 0xff
println!("{n:b}");    // 11111111
println!("{n:#010b}");// 0b11111111
println!("{n:o}");    // 377
println!("{:e}", 1234.5);  // 1.2345e3
println!("{{}}");     // {} : literal braces
```

## 13. Utility macros

### 13.1 — `dbg!`

```rust
let a = 2;
let b = dbg!(a * 3) + 1;   // prints [src/main.rs:2:9] a * 3 = 6 on stderr, returns 6
dbg!(&b);                  // pass a reference so as not to move the value
```

Prints the file, the line, the expression and its value, then **returns** the value: you can insert it in the middle of a computation. Remove it before shipping (`clippy::dbg_macro` flags it).

### 13.2 — `todo!`, `unimplemented!`, `unreachable!`

```rust
fn export_pdf() -> Vec<u8> {
    todo!()                          // not written yet: compiles, panics when called
}

fn export_xml() -> Vec<u8> {
    unimplemented!("no XML on this platform")   // will not be done here
}

fn sign(x: i32) -> &'static str {
    match x.signum() {
        1 => "positive",
        -1 => "negative",
        0 => "zero",
        _ => unreachable!("signum only returns -1, 0 or 1"),   // impossible
    }
}
```

All three panic; their name tells the reader **why**. They have the type `!` and so satisfy any return type.

### 13.3 — `assert!` and `debug_assert!`

```rust
fn mean(v: &[f64]) -> f64 {
    assert!(!v.is_empty(), "mean of an empty list");         // checked in debug AND release
    debug_assert!(v.iter().all(|x| x.is_finite()));          // checked in debug only
    v.iter().sum::<f64>() / v.len() as f64
}
```

## 14. Ownership and references (introduction)

**Ownership** lets Rust free memory without a garbage collector, by checking everything at compile time. This chapter gives the basics; the course `borrow-bases` goes into detail.

### 14.1 — The three rules

1. Each value has an **owner**.
2. There is only **one** owner at a time.
3. When the owner goes out of scope, the value is **dropped**.

```rust
fn main() {
    let s1 = String::from("hello");
    let s2 = s1;              // s1 is moved to s2
    // println!("{s1}");      // ERROR: value borrowed here after move
    println!("{s2}");
}
```

### 14.2 — Move, copy, clone

```rust
let a = 5;
let b = a;                     // copy: i32 is Copy, a stays usable

let s = String::from("x");
let t = s.clone();             // explicit copy of the content: s stays usable
let u = s;                     // move: s is no longer usable
```

These are `Copy`: integers, floats, `bool`, `char`, shared references `&T`, and tuples or arrays of `Copy` types. These are not `Copy`: `String`, `Vec`, `Box`, and anything that owns memory.

### 14.3 — Passing to a function: three ways

```rust
fn take(s: String) {}               // moves: the caller loses the value
fn read(s: &String) {}              // borrows for reading
fn modify(s: &mut String) { s.push('!'); } // borrows for writing

fn main() {
    let mut text = String::from("a");
    read(&text);
    modify(&mut text);
    take(text);
    // read(&text);   // ERROR: text was moved
}
```

### 14.4 — The borrowing rules

At any given moment, either **one** mutable reference, or **as many** shared references as you like, never both.

```rust
let mut s = String::from("a");
let r1 = &s;
let r2 = &s;
println!("{r1} {r2}");     // last use of r1 and r2
let r3 = &mut s;           // OK: r1 and r2 are no longer used
r3.push('b');
```

A borrow lasts until its **last use**, not until the end of the block.

## 15. Exercises

### Exercise 1 — temperature conversion

Write `celsius_to_fahrenheit(c: f64) -> f64` (`F = C × 9/5 + 32`), then print the result with one decimal.

```rust
// Solution:
fn celsius_to_fahrenheit(c: f64) -> f64 {
    c * 9.0 / 5.0 + 32.0
}

fn main() {
    println!("{:.1}", celsius_to_fahrenheit(37.0)); // 98.6
}
```

### Exercise 2 — age in days, without overflow

Compute a number of days from an age in `u32`, returning `None` on overflow.

```rust
// Solution:
fn days(age: u32) -> Option<u32> {
    age.checked_mul(365)
}
```

### Exercise 3 — reading a number

Read a text, strip the spaces, convert it to `u8` and return `None` if that is not possible. Write it with `match`, then in one line.

```rust
// Solution:
fn read_u8(text: &str) -> Option<u8> {
    match text.trim().parse::<u8>() {
        Ok(n) => Some(n),
        Err(_) => None,
    }
}

fn read_u8_short(text: &str) -> Option<u8> {
    text.trim().parse().ok()
}
```

### Exercise 4 — default parameter

Write `greet(name: Option<&str>) -> String` that returns `"Hello, guest"` when the name is absent.

```rust
// Solution:
fn greet(name: Option<&str>) -> String {
    format!("Hello, {}", name.unwrap_or("guest"))
}
```

## 16. Cheat sheet

### Variables

```rust
let x = 5;                       // immutable
let mut y = 5;                   // modifiable
let z: u64 = 5;                  // annotated
let (a, b) = (1, 2);             // destructured
let x = x + 1;                   // shadowing
const MAX: u32 = 100;            // compile-time constant
static NAME: &str = "app";       // global, single address
```

### Conversions

```rust
let a = x as u8;                 // raw, may lose
let b: u32 = small.into();       // lossless
let c = u8::try_from(big)?;      // may fail
let d: i32 = "42".parse()?;      // from text
let e = 42.to_string();          // to text
```

### Safe arithmetic

```rust
a.checked_add(b)     // Option
a.saturating_sub(b)  // stops at the bounds
a.wrapping_mul(b)    // intended wrap-around
a.abs_diff(b)        // unsigned gap
```

### Functions

```rust
fn f(x: i32) -> i32 { x * 2 }              // return by last expression
fn g(x: i32) -> Option<i32> { None }       // maybe
fn h(x: i32) -> Result<i32, String> { Ok(x) } // or an error
fn i() -> ! { panic!() }                   // never returns
fn j(t: impl Into<String>) {}              // accepts several types
fn k(v: &[i32]) {}                         // array, Vec or slice
```

### Formats

```rust
"{x}"  "{:?}"  "{:#?}"  "{:.2}"  "{:>8}"  "{:<8}"  "{:^8}"  "{:08.2}"  "{:x}"  "{:b}"  "{:e}"
```

### Key points

- Immutable by default: `mut` signals a change to come.
- `const` for compile-time values, `static` + `LazyLock` for computed globals.
- Shadowing lets you change the type of a value as you transform it.
- An overflow panics in debug and wraps in release: use `checked_`, `saturating_` or `wrapping_`.
- `as` loses silently; `From` never loses; `TryFrom` reports failure.
- No default parameters, no overloading: `Option`, builder, generics, `impl Into`.
- The last expression of a block is its value; an extra `;` turns it into `()`.
