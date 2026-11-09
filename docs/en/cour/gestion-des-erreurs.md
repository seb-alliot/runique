# Error handling

**`panic!`, `Result`, `Option` and `?`**: tell an expected error from a bug, propagate, convert and define your own errors, with the different ways to write each case.

## Course objectives

By the end of this course, you will be able to:

- Choose between `panic!` and `Result`
- Handle a `Result` in every way: `match`, `if let`, `let ... else`, methods, `?`
- Understand exactly what `?` does, and make it work between different error types
- Convert an `Option` to a `Result` and the other way around
- Write your own error type with `Display`, `Error` and `From`
- Use `Box<dyn Error>` and make `main` return an error
- Know when to move to `thiserror` and `anyhow` (next course)

## Table of contents

1. Two kinds of errors
2. `panic!`
3. `Result<T, E>`
4. Handling a `Result`: every syntax
5. `unwrap` and `expect`
6. The `?` operator
7. `Option` and `Result` together
8. Errors from the standard library
9. Defining your own error type
10. Converting errors for `?`
11. `Box<dyn Error>`
12. `main` returning an error
13. Where to handle an error
14. The same function written in several ways
15. Exercises
16. Cheat sheet

## 1. Two kinds of errors

| | **Recoverable** error | **Unrecoverable** error (bug) |
|---|---|---|
| Example | missing file, invalid input, network down, user not found | index out of bounds, broken invariant, impossible state |
| Predictable? | yes: it happens in normal operation | no: the program is wrong |
| Tool | `Result<T, E>` (or `Option<T>`) | `panic!` |
| Who decides | the caller | nobody: the thread stops |

Rust has no exceptions: a recoverable error is a **value** returned by the function, visible in its signature.

## 2. `panic!`

### 2.1 — Triggering a panic

```rust
fn main() {
    panic!("impossible state: {}", 42);
}
// thread 'main' panicked at src/main.rs:2:5:
// impossible state: 42
```

Other operations panic on their own:

```rust
let v = vec![1, 2, 3];
// v[10];                          // index out of bounds
// let x: u8 = 255; x + 1;         // overflow (in debug)
// 1 / 0;                          // division by zero
// None::<i32>.unwrap();           // called `Option::unwrap()` on a `None` value
```

### 2.2 — What happens

A panic **unwinds** the thread's stack: each value is freed (`Drop` is called), then the thread stops. If it is the main thread, the program stops with exit code 101.

```bash
RUST_BACKTRACE=1 cargo run      # prints the call stack that led to the panic
```

In a web server, a panic in a handler only stops the request's task: axum returns a 500 error and keeps serving the others.

### 2.3 — When to panic

- In **tests** (`assert!`, `unwrap`): a failure must stop the test.
- In a **prototype**, until you handle the errors.
- When a program **invariant** is broken: continuing would give a wrong result.
- When the error is **impossible** and you can explain why (`expect` with the reason).

```rust
fn average(v: &[f64]) -> f64 {
    assert!(!v.is_empty(), "average called on an empty list: caller bug");
    v.iter().sum::<f64>() / v.len() as f64
}
```

**Never** panic on input that comes from outside (user, file, network, database): it is a recoverable error.

## 3. `Result<T, E>`

```rust
enum Result<T, E> {
    Ok(T),     // the value
    Err(E),    // the reason for the failure
}
```

```rust
use std::fs::File;

fn main() {
    let f = File::open("config.toml");    // Result<File, std::io::Error>
    match f {
        Ok(file) => println!("opened: {file:?}"),
        Err(e) => println!("cannot open: {e}"),
    }
}
```

`Result` is marked `#[must_use]`: ignoring a `Result` triggers a warning.

```rust
std::fs::remove_file("tmp.txt");          // warning: unused `Result` that must be used
let _ = std::fs::remove_file("tmp.txt");  // ignore on purpose, and say so
```

## 4. Handling a `Result`: every syntax

The same function, which reads an age from a text with 0 as a fallback value.

```rust
fn age_v1(t: &str) -> u32 {
    // 1. match: all cases, access to the error
    match t.trim().parse::<u32>() {
        Ok(n) => n,
        Err(e) => {
            eprintln!("invalid age ({e}): 0 by default");
            0
        }
    }
}

fn age_v2(t: &str) -> u32 {
    // 2. if let: only the success matters
    if let Ok(n) = t.trim().parse::<u32>() {
        n
    } else {
        0
    }
}

fn age_v3(t: &str) -> u32 {
    // 3. let else: the error case leaves the function
    let Ok(n) = t.trim().parse::<u32>() else {
        return 0;
    };
    n
}

fn age_v4(t: &str) -> u32 {
    // 4. method: the shortest
    t.trim().parse().unwrap_or(0)
}

fn age_v5(t: &str) -> u32 {
    // 5. unwrap_or_else: access to the error, value computed only on failure
    t.trim().parse().unwrap_or_else(|e| {
        eprintln!("invalid age ({e})");
        0
    })
}
```

### 4.1 — The methods of `Result`

```rust
let r: Result<i32, String> = Ok(2);

// Extract
r.clone().unwrap_or(0);
r.clone().unwrap_or_default();
r.clone().unwrap_or_else(|_e| 0);

// Transform
r.clone().map(|n| n * 10);                       // Ok(20)
r.clone().map_err(|e| format!("error: {e}"));    // changes the error type
r.clone().and_then(|n| if n > 0 { Ok(n) } else { Err("negative".to_string()) });
r.clone().or_else(|_e| Ok::<i32, String>(0));    // recover from an error
r.clone().map_or(0, |n| n * 2);                  // transform or fallback value

// Test
r.is_ok();
r.is_err();
r.as_ref().is_ok_and(|&n| n > 1);
r.as_ref().is_err_and(|e| e.is_empty());

// Convert
r.clone().ok();                                  // Option<i32>: the error is lost
r.clone().err();                                 // Option<String>

// Inspect without modifying
let _ = r.clone().inspect_err(|e| eprintln!("{e}"));
```

## 5. `unwrap` and `expect`

```rust
let n: i32 = "42".parse().unwrap();                    // panics with the error on failure
let n: i32 = "42".parse().expect("valid constant");    // panics with YOUR message + the error
```

| Syntax | Panic message |
|---|---|
| `.unwrap()` | `called Result::unwrap() on an Err value: ParseIntError { kind: InvalidDigit }` |
| `.expect("the port comes from the validated config")` | `the port comes from the validated config: ParseIntError { kind: InvalidDigit }` |

`expect` documents **why** the failure is impossible. Word the message as an expectation ("the port is a number"), not as a complaint ("error!").

### Where `unwrap` is acceptable

```rust
// 1. Tests
#[test]
fn reads() { assert_eq!("1".parse::<i32>().unwrap(), 1); }

// 2. Constant value, checkable at a glance
let ip: std::net::IpAddr = "127.0.0.1".parse().unwrap();

// 3. Right after a check
let v = vec![1];
if !v.is_empty() {
    let first = v.first().unwrap();      // but `if let Some(p) = v.first()` is safer
}

// 4. Poisoned lock: another thread has already panicked
let m = std::sync::Mutex::new(0);
let guard = m.lock().unwrap();
```

Everywhere else, `unwrap` in production code turns a recoverable error into a hard stop.

## 6. The `?` operator

### 6.1 — What it does

```rust
use std::fs;
use std::io;

fn read_config() -> Result<String, io::Error> {
    let content = fs::read_to_string("config.toml")?;
    Ok(content.trim().to_string())
}
```

`expression?` is equivalent to:

```rust
let content = match fs::read_to_string("config.toml") {
    Ok(value) => value,
    Err(e) => return Err(From::from(e)),   // leaves the function with the error, converted
};
```

Three effects:

1. on success, gives the value;
2. on error, **leaves the function** immediately;
3. **converts** the error with `From`, to the function's error type.

### 6.2 — Chaining

```rust
fn read_port() -> Result<u16, Box<dyn std::error::Error>> {
    let port = std::fs::read_to_string("port.txt")?.trim().parse::<u16>()?;
    Ok(port)
}
```

Each `?` can make the function exit. The line reads from left to right: read, then trim, then convert.

### 6.3 — `?` on an `Option`

```rust
fn name_initial(text: &str) -> Option<char> {
    let name = text.split_whitespace().nth(1)?;   // None: the function returns None
    name.chars().next()
}
```

`?` works on `Option` in a function that returns `Option`, and on `Result` in a function that returns `Result`. You do not mix the two directly (section 7).

### 6.4 — Where `?` does not work

```rust
fn main() {
    // let n: i32 = "x".parse()?;    // ERROR: main returns () (see section 12)
}

let v = vec!["1", "x"];
// v.iter().map(|s| s.parse::<i32>()?);   // ERROR: ? would exit the closure, which does not return Result
```

In a closure, `?` exits the **closure**, not the enclosing function. For a list: `collect::<Result<Vec<_>, _>>()?`.

```rust
fn all(v: &[&str]) -> Result<Vec<i32>, std::num::ParseIntError> {
    let numbers = v.iter().map(|s| s.parse::<i32>()).collect::<Result<Vec<_>, _>>()?;
    Ok(numbers)
}
```

## 7. `Option` and `Result` together

### 7.1 — `Option` to `Result`

```rust
fn find(id: u32) -> Option<String> { (id == 1).then(|| "Ada".to_string()) }

fn name(id: u32) -> Result<String, String> {
    // 1. ok_or: the error is built in every case
    let n = find(id).ok_or("user not found".to_string())?;

    // 2. ok_or_else: the error is built only when the value is missing
    let n = find(id).ok_or_else(|| format!("user {id} not found"))?;

    // 3. let else
    let Some(n) = find(id) else {
        return Err(format!("user {id} not found"));
    };

    Ok(n)
}
```

### 7.2 — `Result` to `Option`

```rust
fn port(t: &str) -> Option<u16> {
    t.parse().ok()            // the error is thrown away: keep for cases where nobody cares about its cause
}
```

### 7.3 — Swapping the nesting

```rust
let a: Option<Result<i32, String>> = Some(Ok(1));
let b: Result<Option<i32>, String> = a.transpose();   // Ok(Some(1))

// Useful for an optional field that must be valid if present
fn optional_age(t: Option<&str>) -> Result<Option<u32>, std::num::ParseIntError> {
    t.map(|s| s.parse::<u32>()).transpose()
}
```

## 8. Errors from the standard library

| Type | Produced by | Useful content |
|---|---|---|
| `std::io::Error` | files, network, input/output | `e.kind()`: `NotFound`, `PermissionDenied`... |
| `std::num::ParseIntError` | `"x".parse::<i32>()` | `e.kind()`: `Empty`, `InvalidDigit`, `PosOverflow`... |
| `std::num::ParseFloatError` | `parse::<f64>()` | |
| `std::str::Utf8Error` | `std::str::from_utf8` | position of the invalid byte |
| `std::string::FromUtf8Error` | `String::from_utf8` | gives the bytes back: `e.into_bytes()` |
| `std::num::TryFromIntError` | `u8::try_from(300)` | |

### Reacting according to the cause

```rust
use std::io::ErrorKind;

fn read_or_create(path: &str) -> std::io::Result<String> {
    match std::fs::read_to_string(path) {
        Ok(c) => Ok(c),
        Err(e) if e.kind() == ErrorKind::NotFound => {
            std::fs::write(path, "")?;
            Ok(String::new())
        }
        Err(e) => Err(e),
    }
}
```

`std::io::Result<T>` is an alias for `Result<T, std::io::Error>`. Many crates define their own (`fmt::Result`, `sea_orm::DbErr`, `AppResult` in Runique).

## 9. Defining your own error type

### 9.1 — One enum per subsystem

```rust
#[derive(Debug)]
pub enum SignupError {
    InvalidEmail(String),
    PasswordTooShort { length: usize, minimum: usize },
    AlreadyRegistered,
}
```

An enum lets the caller **react** to each case (show a message under the right field, return a 409...), which a plain `String` does not allow.

### 9.2 — `Display`: the message

```rust
use std::fmt;

impl fmt::Display for SignupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidEmail(e) => write!(f, "invalid email address: {e}"),
            Self::PasswordTooShort { length, minimum } => {
                write!(f, "password too short ({length} characters, minimum {minimum})")
            }
            Self::AlreadyRegistered => write!(f, "this address is already registered"),
        }
    }
}
```

### 9.3 — `Error`: the common trait

```rust
impl std::error::Error for SignupError {}
```

`Error` requires `Debug` and `Display`. It makes your error compatible with `Box<dyn Error>`, `anyhow` and every tool that prints a chain of causes.

### 9.4 — Keeping the cause: `source`

```rust
#[derive(Debug)]
pub enum ConfigError {
    Read(std::io::Error),
    Format(std::num::ParseIntError),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read(_) => write!(f, "cannot read the configuration"),
            Self::Format(_) => write!(f, "malformed configuration"),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Read(e) => Some(e),
            Self::Format(e) => Some(e),
        }
    }
}
```

`source` links the error to the one that caused it. A display tool can then write "cannot read the configuration: file not found". Rule: the `Display` describes **your** level, the cause is in `source`; do not repeat the cause's message in both.

### 9.5 — Printing the whole chain

```rust
fn print_chain(e: &dyn std::error::Error) {
    eprintln!("error: {e}");
    let mut cause = e.source();
    while let Some(c) = cause {
        eprintln!("  cause: {c}");
        cause = c.source();
    }
}
```

### 9.6 — Do not leak information

An error message often ends up in front of a user or in an HTTP response. Do not put the following in a public `Display`: server file path, SQL query, password, token, internal trace. Keep these details in the logs (`Debug`, `source`), and show the visitor a generic message.

## 10. Converting errors for `?`

### 10.1 — The problem

```rust
fn read_port() -> Result<u16, ConfigError> {
    let t = std::fs::read_to_string("port")?;      // io::Error, not ConfigError
    let p = t.trim().parse::<u16>()?;               // ParseIntError
    Ok(p)
}
// ERROR: `?` couldn't convert the error to `ConfigError`
```

### 10.2 — Three solutions

```rust
// 1. map_err on each call
fn read_port_v1() -> Result<u16, ConfigError> {
    let t = std::fs::read_to_string("port").map_err(ConfigError::Read)?;
    let p = t.trim().parse::<u16>().map_err(ConfigError::Format)?;
    Ok(p)
}

// 2. Implement From once: ? converts on its own
impl From<std::io::Error> for ConfigError {
    fn from(e: std::io::Error) -> Self { Self::Read(e) }
}
impl From<std::num::ParseIntError> for ConfigError {
    fn from(e: std::num::ParseIntError) -> Self { Self::Format(e) }
}

fn read_port_v2() -> Result<u16, ConfigError> {
    let t = std::fs::read_to_string("port")?;
    let p = t.trim().parse::<u16>()?;
    Ok(p)
}

// 3. Box<dyn Error>: everything converts, but the caller can no longer tell the cases apart
fn read_port_v3() -> Result<u16, Box<dyn std::error::Error>> {
    let t = std::fs::read_to_string("port")?;
    Ok(t.trim().parse::<u16>()?)
}
```

`map_err` is for when the same source error must give different variants depending on the place (two different files, two different fields). `From` is for when a source error always maps to the same variant.

`thiserror` generates `Display`, `Error`, `source` and `From` from attributes: see `gestion-erreurs-avancee`.

## 11. `Box<dyn Error>`

```rust
use std::error::Error;

fn load() -> Result<u32, Box<dyn Error>> {
    let t = std::fs::read_to_string("n.txt")?;       // io::Error converted
    let n: u32 = t.trim().parse()?;                   // ParseIntError converted
    if n == 0 {
        return Err("zero not allowed".into());        // a &str also becomes an error
    }
    Ok(n)
}
```

- Accepts any error that implements `Error`.
- `"text".into()` and `format!(..).into()` create an error from a message.
- The caller can no longer `match` on the cases, only print or try `downcast_ref`.

```rust
fn call() {
    if let Err(e) = load() {
        if let Some(io) = e.downcast_ref::<std::io::Error>() {
            eprintln!("file problem: {}", io.kind());
        } else {
            eprintln!("other error: {e}");
        }
    }
}
```

For threads or async tasks: `Box<dyn Error + Send + Sync>`.

## 12. `main` returning an error

```rust
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let config = std::fs::read_to_string("config.toml")?;
    println!("{config}");
    Ok(())
}
```

If `main` returns `Err`, the program prints the error (with `Debug`) and stops with exit code 1. For a cleaner message and a chosen exit code:

```rust
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
```

`std::process::exit(code)` also stops the program, but **without** running pending `Drop`s (unflushed files, unclosed connections): prefer returning from `main`.

## 13. Where to handle an error

An error is **propagated** (`?`) as long as the current function does not know what to do with it, and **handled** where a decision is possible.

```
read_file()           → does not know if the absence is serious        → ?
load_config()         → knows a missing config = default values        → handles
start_server()        → an unreadable config prevents starting         → ? up to main
main()                → prints the error and exits with a code         → handles
```

In a web handler, the error is handled as an **HTTP response**: 404 for a missing resource, 400 for invalid input, 500 for an internal failure (with the detail in the logs, not in the page).

### Do not swallow errors

```rust
fn save(content: &str) {
    // Bad: the failure disappears without a trace
    let _ = std::fs::write("data.txt", content);

    // Better: at the very least, log it
    if let Err(e) = std::fs::write("data.txt", content) {
        eprintln!("save failed: {e}");
    }
}
```

## 14. The same function written in several ways

Read a file of numbers (one per line) and return their sum.

```rust
use std::fs;

// 1. Prototype: panics on the slightest error
fn sum_v1(path: &str) -> i64 {
    fs::read_to_string(path).unwrap()
        .lines()
        .map(|l| l.trim().parse::<i64>().unwrap())
        .sum()
}

// 2. Box<dyn Error>: all errors bubble up, without distinction
fn sum_v2(path: &str) -> Result<i64, Box<dyn std::error::Error>> {
    let mut total = 0;
    for line in fs::read_to_string(path)?.lines() {
        total += line.trim().parse::<i64>()?;
    }
    Ok(total)
}

// 3. Iterator + collect into a Result
fn sum_v3(path: &str) -> Result<i64, Box<dyn std::error::Error>> {
    let text = fs::read_to_string(path)?;
    let numbers = text
        .lines()
        .map(|l| l.trim().parse::<i64>())
        .collect::<Result<Vec<i64>, _>>()?;
    Ok(numbers.iter().sum())
}

// 4. Typed error: the caller knows which line is wrong
#[derive(Debug)]
enum SumError {
    Read(std::io::Error),
    Line { number: usize, text: String },
}

fn sum_v4(path: &str) -> Result<i64, SumError> {
    let text = fs::read_to_string(path).map_err(SumError::Read)?;
    let mut total = 0;
    for (i, line) in text.lines().enumerate() {
        let n: i64 = line.trim().parse().map_err(|_| SumError::Line {
            number: i + 1,
            text: line.to_string(),
        })?;
        total += n;
    }
    Ok(total)
}

// 5. Tolerant: ignores invalid lines
fn sum_v5(path: &str) -> std::io::Result<i64> {
    Ok(fs::read_to_string(path)?
        .lines()
        .filter_map(|l| l.trim().parse::<i64>().ok())
        .sum())
}
```

| Version | For whom |
|---|---|
| 1 | throwaway script |
| 2, 3 | application: the error is printed, not analysed |
| 4 | library, or interface that must say precisely what is wrong |
| 5 | data known to be dirty, where a bad line does not matter |

## 15. Exercises

### Exercise 1 — safe division

Write `divide(a: i32, b: i32) -> Result<i32, String>` that rejects division by zero **and** `i32::MIN / -1`.

```rust
// Solution:
fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        return Err("division by zero".into());
    }
    a.checked_div(b).ok_or_else(|| format!("{a} / {b} exceeds the capacity of an i32"))
}
```

### Exercise 2 — signup error

Validate an email (contains `@`) and a password (8 characters minimum) with the `SignupError` enum from section 9, then print the message.

```rust
// Solution:
fn validate(email: &str, password: &str) -> Result<(), SignupError> {
    if !email.contains('@') {
        return Err(SignupError::InvalidEmail(email.to_string()));
    }
    let length = password.chars().count();
    if length < 8 {
        return Err(SignupError::PasswordTooShort { length, minimum: 8 });
    }
    Ok(())
}

fn main() {
    if let Err(e) = validate("ada", "x") {
        println!("{e}");     // invalid email address: ada
    }
}
```

### Exercise 3 — conversion with `From`

Make `read_port_v2` (section 10) compile by writing only the two `impl From`, then check that a missing file gives `ConfigError::Read`.

```rust
// Solution:
#[test]
fn missing_file() {
    assert!(matches!(read_port_v2(), Err(ConfigError::Read(_))));
}
```

## 16. Cheat sheet

```rust
panic!("msg")                         // bug: stops the thread
Result<T, E>   Ok(v)   Err(e)         // expected error
r?                                    // propagates, converts with From
r.unwrap()   r.expect("why it is impossible")
r.unwrap_or(v)   r.unwrap_or_else(|e| ..)   r.unwrap_or_default()
r.map(f)   r.map_err(f)   r.and_then(f)   r.ok()   r.is_ok_and(p)
opt.ok_or(e)   opt.ok_or_else(|| e)   opt?   opt_res.transpose()
iter.collect::<Result<Vec<_>, _>>()?
impl fmt::Display for MyError { .. }
impl std::error::Error for MyError { fn source(&self) -> .. }
impl From<io::Error> for MyError { .. }
fn main() -> Result<(), Box<dyn Error>> { .. }
```

### Key points

- `panic!` for bugs, `Result` for what can happen normally.
- Never panic on outside input.
- `?` = return the error converted by `From`, or continue with the value.
- In a closure, `?` only exits the closure.
- `expect` documents why a failure is impossible.
- An error type: an enum, `Display`, `Error`, `source`, `From`.
- `Box<dyn Error>` for an application, a typed enum for a library.
- A public error message contains no path, no query, no secret.
