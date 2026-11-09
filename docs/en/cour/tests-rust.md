# Tests in Rust
> Unit, integration, documentation and asynchronous tests: every way to write a check, organize tests and run them.

## Objectives

- Write a test and choose the right assertion macro
- Test a failure: expected panic, `Result`, error pattern
- Put tests in the right place: `tests` module, `tests/` folder, documentation
- Share setup code between tests
- Test asynchronous code, code that touches a database, code that depends on time or the network
- Run exactly the tests you want with `cargo test`
- Know whether the tests really check something (coverage, mutants)

---

## Table of contents

1. The first test
2. Assertion macros
3. Testing a failure
4. Tests that return a `Result`
5. Ignoring a test
6. The unit test module
7. Integration tests
8. Documentation tests
9. Setup and cleanup
10. Parameterized tests
11. Asynchronous tests
12. Isolating dependencies: database, time, network
13. Tests and features
14. `cargo test` commands
15. Checking the quality of tests
16. Good practices
17. Exercises
18. Cheat sheet

---

## 1. The first test

```rust
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_two_positives() {
        assert_eq!(add(2, 3), 5);
    }
}
```

- `#[test]` marks a function as a test: `cargo test` finds it and runs it.
- A test **passes** if it ends without panicking, and **fails** if it panics.
- `#[cfg(test)]`: the module is only compiled for `cargo test`, so it does not bloat the final binary.
- `use super::*;` imports the whole parent module, private items included.

Each test runs in **its own thread**, in parallel with the others.

---

## 2. Assertion macros

### 2.1 — The three basic macros

```rust
#[test]
fn assertions() {
    let x = 5;
    assert!(x > 0);                 // true condition
    assert_eq!(x, 5);               // equality: prints both values on failure
    assert_ne!(x, 0);               // difference
}
```

`assert_eq!` and `assert_ne!` require the values to implement `PartialEq` and `Debug`.

### 2.2 — Adding a message

```rust
#[test]
fn with_message() {
    let age = 17;
    assert!(age >= 18, "age refused: {age}, at least 18 required");
    assert_eq!(age, 18, "the age read from the form is {age}");
}
```

The message is printed **in addition** to the compared values. It says what the test was checking, especially in a loop.

### 2.3 — Why prefer `assert_eq!` to `assert!(a == b)`

```rust
assert!(result == 42);         // failure: "assertion failed: result == 42"
assert_eq!(result, 42);        // failure: "left: 41, right: 42"
```

The second form shows the value you got, which saves rerunning the test with `println!`.

### 2.4 — Testing a shape: `matches!`

```rust
#[derive(Debug)]
enum Error { NotFound, Forbidden(String) }

fn load(id: u32) -> Result<u32, Error> {
    if id == 0 { Err(Error::Forbidden("anonymous".into())) } else { Ok(id) }
}

#[test]
fn refuses_anonymous() {
    // Option 1: matches! — no need for PartialEq on Error
    assert!(matches!(load(0), Err(Error::Forbidden(_))));

    // Option 2: explicit match, to check the content too
    match load(0) {
        Err(Error::Forbidden(reason)) => assert_eq!(reason, "anonymous"),
        other => panic!("expected Forbidden, got {other:?}"),
    }

    // Option 3: let else
    let Err(Error::Forbidden(reason)) = load(0) else {
        panic!("expected Forbidden");
    };
    assert_eq!(reason, "anonymous");
}
```

### 2.5 — Comparing floats

```rust
#[test]
fn floats() {
    let r = 0.1 + 0.2;
    // assert_eq!(r, 0.3);                        // fails: 0.30000000000000004
    assert!((r - 0.3_f64).abs() < 1e-10, "got {r}");
}
```

### 2.6 — Comparing collections

```rust
use std::collections::HashSet;

#[test]
fn collections() {
    let mut actual = vec![3, 1, 2];
    actual.sort();
    assert_eq!(actual, [1, 2, 3]);       // Vec compared to an array: accepted

    let a: HashSet<_> = [1, 2].into();
    let b: HashSet<_> = [2, 1].into();
    assert_eq!(a, b);                    // order does not matter in a set
}
```

If the order of a result is not guaranteed (iterating over a `HashMap`), sort it or compare sets: otherwise the test will fail at random.

---

## 3. Testing a failure

### 3.1 — `#[should_panic]`

```rust
pub fn divide(a: i32, b: i32) -> i32 {
    if b == 0 {
        panic!("division by zero");
    }
    a / b
}

#[test]
#[should_panic]
fn panics_on_zero() {
    divide(1, 0);
}

#[test]
#[should_panic(expected = "division by zero")]
fn panics_with_the_right_message() {
    divide(1, 0);
}
```

Without `expected`, the test passes on **any** panic, including an unrelated bug (an out-of-bounds index elsewhere). Always set `expected`: it is a substring of the panic message.

### 3.2 — Testing an error `Result`

Most functions return an error rather than panicking. You then check the value:

```rust
fn parse_age(t: &str) -> Result<u8, String> {
    t.parse().map_err(|_| format!("invalid age: {t}"))
}

#[test]
fn refuses_text() {
    // 1. is_err
    assert!(parse_age("abc").is_err());

    // 2. unwrap_err: get the error to check it
    let e = parse_age("abc").unwrap_err();
    assert_eq!(e, "invalid age: abc");

    // 3. compare the whole Result
    assert_eq!(parse_age("12"), Ok(12));
}
```

`is_err` alone checks that there is an error, not that it is **the right one**. Check the content when several errors are possible.

### 3.3 — Catching a panic inside the test

```rust
#[test]
fn caught_panic() {
    let r = std::panic::catch_unwind(|| divide(1, 0));
    assert!(r.is_err());
}
```

Useful to check several panics in a single test, or to keep going afterwards.

---

## 4. Tests that return a `Result`

```rust
#[test]
fn reads_the_config() -> Result<(), Box<dyn std::error::Error>> {
    let n: i32 = "42".parse()?;          // ? instead of unwrap
    assert_eq!(n, 42);
    Ok(())
}
```

The test fails if the function returns `Err`. What it is for: chaining steps that can fail without writing `.unwrap()` everywhere; the failure message shows the error.

A test that returns `Result` cannot use `#[should_panic]`. To test an error, check `is_err()` (section 3.2).

| Syntax | Message on failure |
|---|---|
| `.unwrap()` | `called Result::unwrap() on an Err value: ...` + line |
| `.expect("reading the config")` | your message + the error |
| `?` in a `-> Result` test | the error (without the exact line) |

---

## 5. Ignoring a test

```rust
#[test]
#[ignore = "slow: needs a Postgres database"]
fn full_import() {}
```

```bash
cargo test                       # full_import is skipped
cargo test -- --ignored          # only the ignored ones
cargo test -- --include-ignored  # everything
```

What it is for: slow tests, or tests that need an external service, which you do not run on every change. To skip a test depending on the environment, check inside the test:

```rust
#[test]
fn with_postgres() {
    let Ok(url) = std::env::var("DATABASE_URL_PG") else {
        eprintln!("DATABASE_URL_PG missing: test skipped");
        return;
    };
    // ...
}
```

Careful: a test skipped this way shows up as **passed**. Keep a trace (message) and make sure the CI provides the variable.

---

## 6. The unit test module

### 6.1 — In the same file

```rust
// src/compute.rs
pub fn mean(v: &[f64]) -> Option<f64> {
    if v.is_empty() {
        return None;
    }
    Some(sum(v) / v.len() as f64)
}

fn sum(v: &[f64]) -> f64 {
    v.iter().sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mean_of_nothing_is_none() {
        assert_eq!(mean(&[]), None);
    }

    #[test]
    fn private_sum() {
        assert_eq!(sum(&[1.0, 2.0]), 3.0);   // unit tests see private items
    }
}
```

### 6.2 — In a separate file

```rust
// src/compute.rs
#[cfg(test)]
mod tests;              // content in src/compute/tests.rs
```

```rust
// src/compute/tests.rs
use super::*;

#[test]
fn simple_mean() {
    assert_eq!(mean(&[2.0, 4.0]), Some(3.0));
}
```

What it is for: keeping the code file readable when the tests get long, while keeping access to private items.

### 6.3 — Helper functions in the test module

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn data() -> Vec<f64> {      // no #[test]: plain helper function
        vec![1.0, 2.0, 3.0]
    }

    #[test]
    fn uses_the_helper() {
        assert_eq!(mean(&data()), Some(2.0));
    }
}
```

---

## 7. Integration tests

### 7.1 — The `tests/` folder

```
my_project/
├── src/
│   └── lib.rs
└── tests/
    ├── api.rs          ← one test binary
    ├── forms.rs        ← another test binary
    └── common/
        └── mod.rs      ← shared code (not a test)
```

```rust
// tests/api.rs
use my_project::Calculator;      // only the public API, like a user

#[test]
fn full_computation() {
    let mut c = Calculator::new(10.0);
    c.add(5.0);
    assert_eq!(c.result(), 15.0);
}
```

- Each file in `tests/` is compiled as a **separate crate** that depends on your library.
- No need for `#[cfg(test)]`: the whole folder is only compiled for tests.
- They only see what is `pub`: they check the API as users will see it.
- A package that only has a `main.rs` (no `lib.rs`) cannot have integration tests on its logic: one more reason to put the logic in `lib.rs`.

### 7.2 — Code shared between test files

```rust
// tests/common/mod.rs
pub fn initialized_calculator() -> my_project::Calculator {
    my_project::Calculator::new(100.0)
}
```

```rust
// tests/forms.rs
mod common;

#[test]
fn with_helper() {
    let c = common::initialized_calculator();
    assert_eq!(c.result(), 100.0);
}
```

Why `common/mod.rs` and not `common.rs`: a `tests/common.rs` file would be taken for a test binary (shown as "running 0 tests"). A subfolder is not treated as a test.

### 7.3 — A single binary for all tests

```
tests/
├── main.rs        ← mod api; mod forms; mod common;
├── api.rs
└── forms.rs
```

Each file in `tests/` is linked separately: with many files, compilation and linking take time. Grouping them into one binary (a `main.rs` that declares the others as modules) speeds up large projects noticeably.

### 7.4 — Unit or integration

| | Unit (`mod tests`) | Integration (`tests/`) |
|---|---|---|
| Sees private items | yes | no |
| Tests | a function, a detail | a full flow through the public API |
| Breaks when | the internal detail changes | the public API changes |
| Compilation | with the crate | separate binary per file |

---

## 8. Documentation tests

### 8.1 — Examples are tests

```rust
/// Adds two numbers.
///
/// # Examples
///
/// ```
/// let r = my_project::add(2, 3);
/// assert_eq!(r, 5);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

`cargo test` compiles and runs every code block in the documentation. What it is for: the documentation cannot become wrong without the tests failing.

### 8.2 — Hidden lines

In a documentation example, a line starting with `# ` is executed but not shown in the doc:

```rust
/// ```
/// # use my_project::add;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let n: i32 = "2".parse()?;
/// assert_eq!(add(n, 1), 3);
/// # Ok(())
/// # }
/// ```
```

They are used to hide the imports and the wrapper needed by `?`.

### 8.3 — Block annotations

| Annotation | Effect |
|---|---|
| ` ``` ` | compiled and run |
| ` ```no_run ` | compiled, not run (network access, server loop) |
| ` ```should_panic ` | must panic |
| ` ```compile_fail ` | must **refuse** to compile |
| ` ```ignore ` | neither compiled nor run (avoid: the example can go stale) |
| ` ```text ` | not Rust: simply displayed |

`compile_fail` is used to prove that an API prevents a mistake: for example that you cannot build an authorization proof outside the intended module.

```rust
/// ```compile_fail
/// let a = my_project::Authorized { };   // private constructor: must not compile
/// ```
pub struct Authorized { _private: () }
```

---

## 9. Setup and cleanup

Rust has no built-in `setUp`/`tearDown`. Three ways to get the same effect:

### 9.1 — A helper function

```rust
fn basket_with_two_items() -> Vec<(String, u32)> {
    vec![("apple".into(), 2), ("pear".into(), 1)]
}

#[test]
fn total() {
    let p = basket_with_two_items();
    assert_eq!(p.iter().map(|(_, n)| n).sum::<u32>(), 3);
}
```

### 9.2 — Automatic cleanup with `Drop`

```rust
struct TempFolder(std::path::PathBuf);

impl TempFolder {
    fn new(name: &str) -> Self {
        let p = std::env::temp_dir().join(name);
        std::fs::create_dir_all(&p).unwrap();
        Self(p)
    }
}

impl Drop for TempFolder {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);   // runs even if the test panics
    }
}

#[test]
fn writes_a_file() {
    let folder = TempFolder::new("test_writing");
    std::fs::write(folder.0.join("a.txt"), "x").unwrap();
    assert!(folder.0.join("a.txt").exists());
}   // folder removed here
```

`Drop` also runs when the test fails: cleanup is guaranteed, unlike a call at the end of the function that a panic would skip. The `tempfile` crate provides this type ready-made.

### 9.3 — Shared and expensive setup

```rust
use std::sync::LazyLock;

static DATA: LazyLock<Vec<u32>> = LazyLock::new(|| {
    (0..1_000_000).collect()     // computed once for all tests
});

#[test]
fn uses_the_data() {
    assert_eq!(DATA.len(), 1_000_000);
}
```

Keep it for **read-only** data: tests run in parallel and must not modify each other.

---

## 10. Parameterized tests

### 10.1 — A loop over cases

```rust
fn is_even(n: i32) -> bool { n % 2 == 0 }

#[test]
fn is_even_multiple_cases() {
    let cases = [(0, true), (1, false), (2, true), (-3, false), (i32::MAX, false)];
    for (input, expected) in cases {
        assert_eq!(is_even(input), expected, "is_even({input})");
    }
}
```

Simple, but the test stops at the first failure and only shows up as **one** test.

### 10.2 — A macro that generates one test per case

```rust
macro_rules! even_cases {
    ($($name:ident: $input:expr => $expected:expr,)*) => {
        $(
            #[test]
            fn $name() {
                assert_eq!(is_even($input), $expected);
            }
        )*
    };
}

even_cases! {
    zero: 0 => true,
    one: 1 => false,
    negative_odd: -3 => false,
    max: i32::MAX => false,
}
```

Each case is a separate test, with its name in the report. The `rstest` and `test-case` crates offer the same thing with an attribute (`#[case(0, true)]`).

### 10.3 — Testing both sides of a condition

For each condition in the code, you need at least one case where it is true, one where it is false, and the **boundary**:

```rust
fn can_vote(age: u32) -> bool { age >= 18 }

#[test]
fn vote_at_the_boundaries() {
    assert!(!can_vote(17));   // just below
    assert!(can_vote(18));    // the exact boundary
    assert!(can_vote(19));    // above
}
```

A test that only checks `can_vote(30)` would also pass with `age > 18` or `age >= 0`: it does not protect the boundary.

---

## 11. Asynchronous tests

### 11.1 — `#[tokio::test]`

```rust
async fn fetch(id: u32) -> Option<String> {
    tokio::time::sleep(std::time::Duration::from_millis(1)).await;
    (id > 0).then(|| format!("item {id}"))
}

#[tokio::test]
async fn fetches_an_item() {
    assert_eq!(fetch(1).await, Some("item 1".into()));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn with_several_threads() {}
```

`#[test]` cannot run an `async` function: `#[tokio::test]` creates a runtime for the test. By default, it is single-threaded.

### 11.2 — Controlling time

```rust
#[tokio::test(start_paused = true)]
async fn expiration() {
    let start = tokio::time::Instant::now();
    tokio::time::sleep(std::time::Duration::from_secs(3600)).await;   // instant
    assert!(start.elapsed() >= std::time::Duration::from_secs(3600));
}
```

With `start_paused` (tokio's `test-util` feature), time advances on its own when all tasks are waiting: a one-hour expiration test takes a few microseconds.

---

## 12. Isolating dependencies: database, time, network

### 12.1 — Pass the dependency as a parameter

```rust
// Hard to test: the time is read inside
fn is_expired_v1(expires_at: u64) -> bool {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    now > expires_at
}

// Easy to test: the time is a parameter
fn is_expired(expires_at: u64, now: u64) -> bool {
    now > expires_at
}

#[test]
fn expiration_boundary() {
    assert!(!is_expired(100, 100));
    assert!(is_expired(100, 101));
}
```

### 12.2 — A trait to replace a service

```rust
trait Sender {
    fn send(&self, to: &str, text: &str) -> Result<(), String>;
}

struct Signup<S: Sender> { mail: S }

impl<S: Sender> Signup<S> {
    fn register(&self, email: &str) -> Result<(), String> {
        self.mail.send(email, "Welcome")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct FakeSender { sent: RefCell<Vec<String>> }

    impl Sender for FakeSender {
        fn send(&self, to: &str, _text: &str) -> Result<(), String> {
            self.sent.borrow_mut().push(to.to_string());
            Ok(())
        }
    }

    #[test]
    fn sends_the_welcome_mail() {
        let s = Signup { mail: FakeSender { sent: RefCell::new(vec![]) } };
        s.register("ada@ex.com").unwrap();
        assert_eq!(*s.mail.sent.borrow(), ["ada@ex.com"]);
    }
}
```

Production code uses a real SMTP sender; the test uses a fake one that records the calls. The `mockall` crate generates such fakes automatically.

### 12.3 — Database: one transaction per test

Against a real database, each test must start from a known state. The safest technique: open a transaction at the start of the test and **roll it back** at the end, whatever happens. No test sees the writes of another, even in parallel.

In Runique, this is what the `test-utils` feature provides (to declare in `[dev-dependencies]`) with `runique_test`: each test receives a connection inside a transaction that is rolled back at the end.

Alternatives:

- in-memory SQLite database, recreated per test (fast, but not the same engine as in production);
- one database per test (total isolation, slower);
- serial tests (`serial_test`, or `--test-threads=1`) if isolation is impossible.

### 12.4 — Tests that share global state

Environment variables, fixed files, network port: parallel tests step on each other.

```rust
#[test]
#[serial_test::serial]          // these tests never run at the same time
fn modifies_the_environment() {
    unsafe { std::env::set_var("MODE", "test"); }
}
```

Since the 2024 edition, `set_var` is `unsafe`: modifying the environment while other threads read it is undefined behavior. Prefer passing the configuration as a parameter.

---

## 13. Tests and features

```rust
#[cfg(test)]
mod tests {
    #[test]
    #[cfg(feature = "admin")]          // compiled only with the feature
    fn admin_available() {}

    #[test]
    #[cfg(not(feature = "admin"))]
    fn admin_absent() {}
}
```

```bash
cargo test --features admin
cargo test --no-default-features
```

A test under `#[cfg(feature = "x")]` disappears **silently** without the feature: `cargo test` shows "0 tests" for that module, with no error. With exclusive features, you need one command per combination (see `cargo-dependances`).

---

## 14. `cargo test` commands

```bash
cargo test                          # all tests: unit, integration, doc
cargo test mean                     # tests whose path contains "mean"
cargo test compute::tests::         # one module
cargo test mean -- --exact          # exact name (full path)

cargo test --lib                    # only the library's unit tests
cargo test --test api               # only tests/api.rs
cargo test --doc                    # only the documentation examples
cargo test --bins                   # only the binaries' tests

cargo test -- --nocapture           # show println! output (hidden when the test passes)
cargo test -- --test-threads=1      # one test at a time
cargo test -- --ignored             # only the #[ignore] ones
cargo test -- --include-ignored
cargo test -- --list                # list without running
cargo test --no-fail-fast           # keep going after the first failing file

cargo test -p runique               # one workspace member
cargo test --features admin,test-utils test_name
cargo test --release                # tests with optimizations (integer overflows not detected)
```

Options **before** `--` are for Cargo, those **after** are for the test program.

---

## 15. Checking the quality of tests

### 15.1 — Coverage

```bash
cargo install cargo-llvm-cov
cargo llvm-cov --html             # line-by-line report
cargo llvm-cov --features admin
```

Coverage tells you which lines were **executed**, not whether they were **checked**: a test with no assertion covers code without proving anything.

### 15.2 — Mutation tests

```bash
cargo install cargo-mutants
cargo mutants                     # changes the code (>= becomes >, true becomes false...) and reruns the tests
cargo mutants -f src/auth.rs      # a single file
```

Each mutation that **survives** (the tests still pass) points to a line whose behavior no test checks. It is the best way to find forgotten boundaries (section 10.3).

### 15.3 — Property tests

```rust
// with the proptest crate
proptest::proptest! {
    #[test]
    fn reversing_twice_gives_the_original(v: Vec<i32>) {
        let mut w = v.clone();
        w.reverse();
        w.reverse();
        proptest::prop_assert_eq!(v, w);
    }
}
```

Instead of writing cases by hand, you write a property that is true for **all** inputs; the crate generates hundreds of inputs and shrinks any counterexample to the smallest one.

---

## 16. Good practices

**Name the behavior:**

```rust
#[test] fn test1() {}                                    // vague
#[test] fn dividing_by_zero_returns_an_error() {}        // says what is guaranteed
```

**Arrange, act, assert:**

```rust
#[test]
fn adding_to_the_basket() {
    // Arrange
    let mut basket = Vec::new();
    // Act
    basket.push(("apple", 2));
    // Assert
    assert_eq!(basket.len(), 1);
}
```

**One test, one behavior**: if the name contains "and", it is often two tests.

**Test what must be refused** as much as what must work, especially for security: wrong password, user without permissions, expired token, input too long.

**Deterministic tests**: no real time, no unseeded randomness, no `HashMap` ordering, no dependence on the execution order of tests.

---

## 17. Exercises

### Exercise 1 — password validator

Test `is_valid(pwd: &str) -> bool` (at least 8 characters and a digit) on both sides of each condition.

```rust
// Solution:
fn is_valid(pwd: &str) -> bool {
    pwd.chars().count() >= 8 && pwd.chars().any(|c| c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn length_boundary() {
        assert!(!is_valid("abcdef1"));      // 7 characters
        assert!(is_valid("abcdefg1"));      // 8 characters
    }

    #[test]
    fn digit_required() {
        assert!(!is_valid("abcdefgh"));
        assert!(is_valid("abcdefg9"));
    }

    #[test]
    fn length_in_characters_not_bytes() {
        assert!(!is_valid("ééééé1"));       // 6 characters but 11 bytes
    }
}
```

### Exercise 2 — precise error

`parse_port(t: &str) -> Result<u16, String>` rejects non-numeric text and port 0. Check the message of each error.

```rust
// Solution:
fn parse_port(t: &str) -> Result<u16, String> {
    let p: u16 = t.parse().map_err(|_| "non-numeric port".to_string())?;
    if p == 0 {
        return Err("port 0 forbidden".into());
    }
    Ok(p)
}

#[test]
fn port_errors() {
    assert_eq!(parse_port("abc").unwrap_err(), "non-numeric port");
    assert_eq!(parse_port("0").unwrap_err(), "port 0 forbidden");
    assert_eq!(parse_port("70000").unwrap_err(), "non-numeric port");   // exceeds u16
    assert_eq!(parse_port("1"), Ok(1));
}
```

### Exercise 3 — injected clock

Make testable a function that tells whether a 30-minute session has expired.

```rust
// Solution:
fn session_expired(created_at: u64, now: u64) -> bool {
    now.saturating_sub(created_at) > 30 * 60
}

#[test]
fn session_expiration() {
    assert!(!session_expired(0, 1800));
    assert!(session_expired(0, 1801));
    assert!(!session_expired(100, 50));    // clock going backwards: no overflow
}
```

---

## 18. Cheat sheet

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn a() { assert!(c); assert_eq!(x, y); assert_ne!(x, z); }
    #[test] fn b() -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
    #[test] #[should_panic(expected = "message")] fn c() {}
    #[test] #[ignore = "reason"] fn d() {}
    #[tokio::test] async fn e() {}
}

assert!(matches!(r, Err(Error::X)));
assert_eq!(f().unwrap_err(), "message");
```

```bash
cargo test filter                 cargo test -- --nocapture
cargo test --lib / --test name / --doc
cargo test -- --test-threads=1    cargo test -- --ignored
cargo test --features a,b         cargo llvm-cov    cargo mutants
```

### Key points

- A test fails by panicking; `assert_eq!` shows the values.
- `#[should_panic]` always with `expected`.
- Check the content of an error, not just `is_err()`.
- Unit tests in `mod tests` (see private items), integration tests in `tests/` (public API).
- Documentation examples are run: `no_run`, `compile_fail`, `should_panic`.
- `Drop` guarantees cleanup, even when the test fails.
- Test each condition on both sides, boundary included.
- Inject time, network and database to make code testable.
- A test under `#[cfg(feature)]` silently disappears without the feature.
- Coverage measures execution, mutants measure checking.
