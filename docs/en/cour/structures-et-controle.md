# Control structures in Rust

**Conditions, loops and pattern matching**: every way to write control flow, and when to choose which.

## Course objectives

By the end of this course, you will be able to:

- Write a condition with `if`, `else if`, `else`, and use it as a value
- Choose between `loop`, `while`, `while let` and `for`
- Iterate in every way: by value, by reference, by mutable reference, with an index, backwards, with a step
- Exit a loop with a value, target a specific loop with a label, exit a block
- Use `match`, its guards, `matches!`, `if let`, `let ... else` and the let-chains of the 2024 edition
- Replace a loop with an iterator when it is clearer

## Table of contents

1. Conditions
2. `if` as an expression and its alternatives
3. `loop`
4. `while`
5. `for`
6. `break`, `continue` and labels
7. Loops or iterators: the same function written in several ways
8. `match`
9. `if let`, `while let`, `let ... else`
10. Let-chains (2024 edition)
11. `matches!`
12. Practical examples
13. Exercises
14. Cheat sheet

## 1. Conditions

A condition runs a block depending on whether an expression is `true` or `false`. In Rust, the condition is **always** a `bool`: there is no implicit conversion.

### 1.1 — Simple `if`

```rust
fn main() {
    let number = 7;
    if number < 10 {
        println!("The number is small");
    }
}
```

No parentheses around the condition (the compiler accepts them but `clippy` flags them), and braces are **required** even for a single statement.

### 1.2 — No implicit "truthy" value

```rust
let x = 5;
// if x { }            // ERROR: expected `bool`, found integer
if x != 0 { }          // OK

let name = String::new();
// if name { }         // ERROR too
if !name.is_empty() { } // OK

let value: Option<i32> = None;
// if value { }        // ERROR
if value.is_some() { } // OK
```

What it is for: in JavaScript or Python, `0`, `""` or `null` are "false", which hides bugs (a counter at zero treated as missing). In Rust, you always write the test you really mean.

### 1.3 — `else` and `else if`

```rust
fn main() {
    let number = 7;

    if number % 2 == 0 {
        println!("Even");
    } else {
        println!("Odd");
    }

    if number < 0 {
        println!("Negative");
    } else if number == 0 {
        println!("Zero");
    } else {
        println!("Positive");
    }
}
```

Branches are tested in order and **only the first true one** runs.

### 1.4 — Logical operators and short-circuiting

```rust
let age = 20;
let registered = true;

if age >= 18 && registered { }   // AND: both must be true
if age < 12 || age > 65 { }      // OR: either one is enough
if !registered { }               // NOT
```

`&&` and `||` **short-circuit**: the right-hand side is only evaluated if needed.

```rust
let v: Vec<i32> = vec![];
// Without short-circuiting, v[0] would panic on an empty vector
if !v.is_empty() && v[0] > 10 {
    println!("first element is large");
}
```

What it is for: put the "can I" test on the left and the "should I" test on the right. `&` and `|` also exist on `bool`, but they do not stop evaluation: they are not used for conditions.

### 1.5 — Comparing three cases: chained `if` or `cmp`

```rust
use std::cmp::Ordering;

fn compare(a: i32, b: i32) -> &'static str {
    // Syntax 1: chained if
    if a < b {
        "smaller"
    } else if a > b {
        "greater"
    } else {
        "equal"
    }
}

fn compare_alt(a: i32, b: i32) -> &'static str {
    // Syntax 2: match on Ordering, the compiler checks that all 3 cases are handled
    match a.cmp(&b) {
        Ordering::Less => "smaller",
        Ordering::Greater => "greater",
        Ordering::Equal => "equal",
    }
}
```

The second form is preferable when all three cases matter: it is impossible to forget one, and `a` and `b` are compared only once.

## 2. `if` as an expression and its alternatives

In Rust, `if` **produces a value**. This is what replaces the ternary operator `cond ? a : b`, which does not exist.

### 2.1 — Assigning the result of an `if`

```rust
fn main() {
    let age = 20;

    // Syntax 1: mutable variable then assignment (C style, to avoid)
    let mut status = "";
    if age >= 18 {
        status = "Adult";
    } else {
        status = "Minor";
    }

    // Syntax 2: if expression, the variable stays immutable
    let status = if age >= 18 { "Adult" } else { "Minor" };
    println!("{status}");
}
```

The second syntax is the idiomatic form: the variable is never in an intermediate state and stays immutable.

### 2.2 — Branches must have the same type

```rust
let condition = true;
let x = if condition {
    5
} else {
    // "six"   // ERROR: `if` and `else` have incompatible types
    6
};
```

An `if` **without `else`** is `()` (the unit type). So you cannot write `let x = if c { 5 };`: the value for the false case would be missing.

### 2.3 — A branch that does not return

A branch that does `return`, `panic!`, `break` or `continue` has type `!` ("never"): it is compatible with any type.

```rust
fn divide(a: i32, b: i32) -> Result<i32, String> {
    let divisor = if b != 0 { b } else { return Err("division by zero".into()) };
    Ok(a / divisor)
}
```

### 2.4 — The last element of a block

The value of a block is its last expression **without a semicolon**.

```rust
let x = if true { 5 } else { 6 };    // x = 5
let y = if true { 5; } else { 6; };  // y = (): the ; turns the expression into a statement
```

### 2.5 — `bool::then` and `then_some`: getting an `Option`

```rust
let age = 20;

// Syntax 1: if / else
let discount = if age < 26 { Some(10) } else { None };

// Syntax 2: then_some, the value is computed in every case
let discount = (age < 26).then_some(10);

// Syntax 3: then, the closure is only called if the condition is true
let discount = (age < 26).then(|| expensive_computation());

fn expensive_computation() -> i32 { 10 }
```

What it is for: building an `Option` from a condition, often in an iterator chain (`filter_map(|x| (x > 0).then_some(x * 2))`). Use `then` whenever the value costs something to compute.

## 3. `loop`

`loop` repeats a block **without a condition**. You leave it with `break`, `return` or `?`.

### 3.1 — Infinite loop and exit

```rust
fn main() {
    let mut counter = 0;
    loop {
        counter += 1;
        if counter == 5 {
            break;
        }
    }
    println!("{counter}"); // 5
}
```

### 3.2 — `loop` that returns a value

`break expr` makes the loop an expression. **Only `loop`** allows it.

```rust
fn main() {
    let mut counter = 0;
    let result = loop {
        counter += 1;
        if counter == 10 {
            break counter * 2;
        }
    };
    println!("{result}"); // 20
}
```

Why not `for` or `while`? These loops can end without going through `break` (the condition becomes false, the iterator is empty): there would then be no value. The compiler therefore rejects `break 5` in a `while` (error E0571). `loop`, on the other hand, ends **only** with a `break`.

### 3.3 — What `loop` is for: retrying

```rust
fn connect() -> Result<&'static str, &'static str> { Err("refused") }

fn main() {
    let mut attempts = 0;
    let connection = loop {
        attempts += 1;
        match connect() {
            Ok(c) => break Ok(c),
            Err(_) if attempts < 3 => continue,
            Err(e) => break Err(e),
        }
    };
    println!("{connection:?}"); // Err("refused")
}
```

### 3.4 — What `loop` is for: server or background task

```rust
use std::time::Duration;

async fn periodic_cleanup() {
    let mut interval = tokio::time::interval(Duration::from_secs(60));
    loop {
        interval.tick().await;
        // purge expired sessions, etc.
    }
}
```

A task that runs as long as the program lives is written with `loop`, never with `while true` (which `clippy` flags anyway: `while_true`).

### 3.5 — `loop` with a state: state machine

```rust
enum State { Start, Reading(u32), End }

fn main() {
    let mut state = State::Start;
    loop {
        state = match state {
            State::Start => State::Reading(0),
            State::Reading(n) if n < 3 => State::Reading(n + 1),
            State::Reading(_) => State::End,
            State::End => break,
        };
    }
}
```

## 4. `while`

`while` repeats a block **as long as a condition is true**, tested before each iteration.

### 4.1 — Basic form

```rust
fn main() {
    let mut number = 3;
    while number != 0 {
        println!("{number}!");
        number -= 1;
    }
    println!("Liftoff!");
}
```

### 4.2 — `while` is a `loop` with an `if`

These two loops are equivalent:

```rust
let mut n = 3;
while n > 0 {
    n -= 1;
}

let mut n = 3;
loop {
    if !(n > 0) {
        break;
    }
    n -= 1;
}
```

`while` is always `()`: you cannot exit it with a value.

### 4.3 — When to use `while`

When the number of iterations **depends on a state** you do not know in advance: user input, convergence, a buffer being emptied.

```rust
use std::io;

fn main() {
    let mut input = String::new();
    let mut attempts = 3;
    while attempts > 0 {
        println!("Password ({attempts} attempts left):");
        input.clear();
        io::stdin().read_line(&mut input).expect("read error");
        if input.trim() == "secret" {
            println!("Access granted");
            break;
        }
        attempts -= 1;
    }
}
```

```rust
// Convergence: we do not know how many iterations it will take
fn square_root(x: f64) -> f64 {
    let mut estimate = x / 2.0;
    while (estimate * estimate - x).abs() > 1e-10 {
        estimate = (estimate + x / estimate) / 2.0;
    }
    estimate
}
```

### 4.4 — `while` with an index: the pitfall

```rust
let numbers = [10, 20, 30];

// Syntax 1: while with an index — works but fragile
let mut i = 0;
while i < numbers.len() {
    println!("{}", numbers[i]);
    i += 1; // forgetting it = infinite loop; getting the bound wrong = panic
}

// Syntax 2: for — same result, no bound to manage
for n in numbers {
    println!("{n}");
}
```

Each `numbers[i]` is checked at runtime (panic if out of bounds). `for` over the collection removes both the risk and the check.

## 5. `for`

`for` iterates over **anything that implements `IntoIterator`**: ranges, arrays, `Vec`, `HashMap`, `String` (via `.chars()`), iterators.

### 5.1 — What `for` really does

```rust
for x in collection {
    // body
}

// is transformed by the compiler into:
let mut iter = IntoIterator::into_iter(collection);
loop {
    match iter.next() {
        Some(x) => { /* body */ }
        None => break,
    }
}
```

All of `for`'s behaviour comes from there: what you give after `in` determines whether the collection is consumed, read or modified.

### 5.2 — Ranges

```rust
for i in 0..5 { }       // 0, 1, 2, 3, 4      (end excluded)
for i in 0..=5 { }      // 0, 1, 2, 3, 4, 5   (end included)
for i in (0..5).rev() { }        // 4, 3, 2, 1, 0
for i in (0..10).step_by(3) { }  // 0, 3, 6, 9
for i in (0..=10).rev().step_by(5) { } // 10, 5, 0
for c in 'a'..='e' { }  // a, b, c, d, e
for i in 5..0 { }       // no iteration: empty range, no error
```

To count down, you need `.rev()`: `5..0` does not count backwards, it is empty.

### 5.3 — The three ways to iterate over a collection

```rust
fn main() {
    let mut fruits = vec![String::from("apple"), String::from("pear")];

    // 1. By reference: we read, the collection stays usable
    for fruit in &fruits {          // same as fruits.iter()
        println!("{fruit}");        // fruit: &String
    }

    // 2. By mutable reference: we modify in place
    for fruit in &mut fruits {      // same as fruits.iter_mut()
        fruit.push('!');            // fruit: &mut String
    }

    // 3. By value: we consume, the collection no longer exists afterwards
    for fruit in fruits {           // same as fruits.into_iter()
        println!("{fruit}");        // fruit: String (owned)
    }
    // println!("{:?}", fruits);    // ERROR: value moved
}
```

| Syntax | Equivalent method | Element type | The collection afterwards |
|---|---|---|---|
| `for x in &v` | `v.iter()` | `&T` | intact |
| `for x in &mut v` | `v.iter_mut()` | `&mut T` | modified |
| `for x in v` | `v.into_iter()` | `T` | destroyed |

Rule of thumb: start with `&v`. Move to `&mut v` to modify, and to `v` only when you want to **move** the elements elsewhere (put them in another collection, send them to a thread).

### 5.4 — Modifying elements: three syntaxes

```rust
let mut v = vec![1, 2, 3];

// Syntax 1: mutable reference, dereferencing with *
for x in &mut v {
    *x *= 2;
}

// Syntax 2: explicit iter_mut
for x in v.iter_mut() {
    *x += 1;
}

// Syntax 3: no loop, with an iterator
v.iter_mut().for_each(|x| *x -= 1);

println!("{v:?}"); // [2, 4, 6]
```

### 5.5 — With the index: `enumerate`

```rust
let names = ["Ada", "Linus", "Grace"];

for (i, name) in names.iter().enumerate() {
    println!("{i}: {name}");
}

// The index starts at 0; for human numbering:
for (i, name) in names.iter().enumerate() {
    println!("{}. {name}", i + 1);
}
```

Avoid `for i in 0..names.len() { names[i] }`: it is longer, each access is checked, and `clippy` flags it (`needless_range_loop`). The index is only useful when you need it **in addition to** the element.

### 5.6 — Destructuring in the `for`

```rust
use std::collections::HashMap;

let points = vec![(0, 1), (2, 3)];
for (x, y) in &points {
    println!("{x}, {y}");
}

let mut ages = HashMap::new();
ages.insert("Ada", 36);
for (name, age) in &ages {
    println!("{name} is {age} years old");
}

// &element to copy an integer instead of keeping a reference
let numbers = [1, 2, 3];
for &n in &numbers {
    let double: i32 = n * 2; // n: i32 and not &i32
}
```

### 5.7 — Iterating over several collections

```rust
let names = ["Ada", "Linus"];
let ages = [36, 54];

// In parallel: zip stops at the shortest
for (name, age) in names.iter().zip(ages.iter()) {
    println!("{name}: {age}");
}

// One after the other: chain
for n in names.iter().chain(["Grace"].iter()) {
    println!("{n}");
}

// Nested: all combinations
for name in &names {
    for age in &ages {
        println!("{name} / {age}");
    }
}
```

### 5.8 — Iterating over a string

```rust
let text = "café";

for c in text.chars() { }          // 'c', 'a', 'f', 'é'  (Unicode characters)
for b in text.bytes() { }          // 99, 97, 102, 195, 169 (UTF-8 bytes)
for (i, c) in text.char_indices() { } // byte position + character
for word in "one two three".split_whitespace() { }
for line in "a\nb".lines() { }
```

`for c in text` does not compile: a `&str` is not directly iterable, you must choose between characters and bytes. `é` takes 2 bytes, hence the difference.

### 5.9 — Windows and chunks

```rust
let v = [1, 2, 3, 4, 5];

for pair in v.windows(2) { }   // [1,2], [2,3], [3,4], [4,5]  (sliding)
for chunk in v.chunks(2) { }   // [1,2], [3,4], [5]           (split)
```

What it is for: `windows` to compare each element with the next one (checking order, gaps), `chunks` to process in batches (database insertion in batches of 100).

### 5.10 — Modifying the collection while iterating over it

```rust
let mut v = vec![1, 2, 3];

// for x in &v {
//     v.push(*x);   // ERROR: v is borrowed by the loop
// }

// Solution 1: build another collection
let additions: Vec<i32> = v.iter().map(|x| x * 10).collect();
v.extend(additions);

// Solution 2: remove elements according to a condition
v.retain(|&x| x % 2 == 0);
```

The borrow checker prevents iterator invalidation here, a classic bug in C++ or Java (`ConcurrentModificationException`).

## 6. `break`, `continue` and labels

### 6.1 — `break` and `continue`

```rust
for i in 1..10 {
    if i % 2 == 0 {
        continue;      // moves on to the next iteration
    }
    if i > 7 {
        break;         // leaves the loop
    }
    println!("{i}");   // 1, 3, 5, 7
}
```

### 6.2 — `continue` to avoid nesting

```rust
struct User { active: bool, email: Option<String> }

fn send(users: &[User]) {
    // Syntax 1: nested conditions
    for u in users {
        if u.active {
            if let Some(email) = &u.email {
                println!("sending to {email}");
            }
        }
    }

    // Syntax 2: discard the useless cases first
    for u in users {
        if !u.active {
            continue;
        }
        let Some(email) = &u.email else { continue };
        println!("sending to {email}");
    }
}
```

The second form keeps the useful case at the first indentation level, whatever the number of conditions.

### 6.3 — Labels: targeting a specific loop

Without a label, `break` and `continue` act on the **nearest** loop. A label (`'name:`) lets you act on an outer loop.

```rust
fn main() {
    'rows: for row in 0..3 {
        for column in 0..3 {
            if column == 1 {
                continue 'rows;   // moves on to the next row
            }
            if row == 2 {
                break 'rows;      // leaves both loops
            }
            println!("({row}, {column})");
        }
    }
    // Prints (0, 0) then (1, 0)
}
```

### 6.4 — Label and value

```rust
let grid = [[1, 2], [3, 4]];

let position = 'search: loop {
    for (r, row) in grid.iter().enumerate() {
        for (c, &val) in row.iter().enumerate() {
            if val == 3 {
                break 'search Some((r, c));
            }
        }
    }
    break None;
};
println!("{position:?}"); // Some((1, 0))
```

### 6.5 — Labeled block (Rust 1.65+)

An ordinary block can carry a label and be left with `break 'label value`. It is not a loop: `continue` is forbidden in it.

```rust
fn price(age: u32, student: bool) -> u32 {
    let amount = 'compute: {
        if age < 12 {
            break 'compute 5;
        }
        if student {
            break 'compute 8;
        }
        12
    };
    amount
}
```

What it is for: leaving a computation early without making it a separate function or using `return` (which would leave the whole function). A separate function is often more readable; the labeled block is for when the computation uses many local variables.

### 6.6 — `return` and `?` in a loop

`return` leaves the whole **function**, however deep the loop is. `?` does the same on error.

```rust
fn first_negative(v: &[i32]) -> Option<usize> {
    for (i, &x) in v.iter().enumerate() {
        if x < 0 {
            return Some(i);
        }
    }
    None
}

fn sum_texts(texts: &[&str]) -> Result<i32, std::num::ParseIntError> {
    let mut total = 0;
    for t in texts {
        total += t.parse::<i32>()?; // the first error stops everything and goes up
    }
    Ok(total)
}
```

## 7. Loops or iterators: the same function written in several ways

A `for` loop and an iterator chain produce the same machine code in the vast majority of cases (the compiler optimises both). The choice is a matter of readability.

### 7.1 — Sum from 1 to n

```rust
// Syntax 1: while
fn sum_while(n: u64) -> u64 {
    let mut i = 1;
    let mut total = 0;
    while i <= n {
        total += i;
        i += 1;
    }
    total
}

// Syntax 2: for
fn sum_for(n: u64) -> u64 {
    let mut total = 0;
    for i in 1..=n {
        total += i;
    }
    total
}

// Syntax 3: iterator
fn sum_iter(n: u64) -> u64 {
    (1..=n).sum()
}

// Syntax 4: fold, the general form of any accumulation
fn sum_fold(n: u64) -> u64 {
    (1..=n).fold(0, |acc, i| acc + i)
}

// Syntax 5: formula, no loop at all
fn sum_formula(n: u64) -> u64 {
    n * (n + 1) / 2
}
```

### 7.2 — Searching for an element

```rust
let v = [3, 8, 12, 5];

// Does one exist?
let mut found = false;
for &x in &v {
    if x > 10 { found = true; break; }
}
let found = v.iter().any(|&x| x > 10);
let contains = v.contains(&12);          // simple equality

// Do all of them satisfy it?
let all_positive = v.iter().all(|&x| x > 0);

// Which one?
let first_large: Option<&i32> = v.iter().find(|&&x| x > 10);

// At which position?
let position: Option<usize> = v.iter().position(|&x| x == 12);
```

`any`, `all`, `find` and `position` stop at the first result, like a `break`.

### 7.3 — Filtering and transforming

```rust
let v = vec![1, 2, 3, 4, 5, 6];

// Syntax 1: loop and push
let mut even_squares = Vec::new();
for x in &v {
    if x % 2 == 0 {
        even_squares.push(x * x);
    }
}

// Syntax 2: filter then map
let even_squares: Vec<i32> = v.iter().filter(|&&x| x % 2 == 0).map(|x| x * x).collect();

// Syntax 3: filter_map, filters and transforms in one step
let even_squares: Vec<i32> = v.iter().filter_map(|&x| (x % 2 == 0).then_some(x * x)).collect();
```

### 7.4 — Counting, minimum, maximum

```rust
let v = [4, 9, 1, 7];

let large_count = v.iter().filter(|&&x| x > 5).count(); // 2
let min = v.iter().min();                               // Some(1)
let max = v.iter().max();                               // Some(9)

let words = ["rust", "go", "python"];
let longest = words.iter().max_by_key(|w| w.len());     // Some("python")
```

`min` and `max` return an `Option`: an empty collection has no minimum.

### 7.5 — `for_each` or `for`?

```rust
let v = [1, 2, 3];

for x in &v {
    println!("{x}");
}

v.iter().for_each(|x| println!("{x}"));
```

Prefer `for` as soon as the body is more than one line or uses `break`, `continue`, `return` or `?`: these keywords do not cross a closure (`return` in the closure of `for_each` only leaves the closure). `for_each` is for the end of an iterator chain.

### 7.6 — Which form to choose

| Need | Recommended form |
|---|---|
| Unknown number of iterations, stop on a state | `while` or `loop` |
| Retry, permanent task, value on exit | `loop` |
| Empty a stack, read until exhausted | `while let` |
| Iterate over a collection with side effects (print, write to the database) | `for` |
| Compute a value from a collection | iterator (`sum`, `filter`, `map`, `collect`) |
| Search | `any`, `all`, `find`, `position`, `contains` |

## 8. `match`

`match` compares a value with a series of patterns and runs the branch of the **first** one that matches. The compiler checks that **all** cases are covered.

### 8.1 — Basic form and value

```rust
let number = 2;
let text = match number {
    1 => "one",
    2 => "two",
    _ => "other",   // _: everything else
};
```

### 8.2 — Multi-line branch

```rust
match number {
    0 => println!("zero"),
    n => {
        let double = n * 2;
        println!("double: {double}");
    }
}
```

A branch in braces does not need a comma.

### 8.3 — Several values, ranges, binding

```rust
let n = 42;
match n {
    0 => println!("zero"),
    1 | 2 | 3 => println!("small"),           // OR
    4..=9 => println!("digit"),               // inclusive range
    x @ 10..=99 => println!("{x}: two digits"), // range + name
    _ => println!("large"),
}
```

### 8.4 — Exhaustiveness

```rust
let b = true;
match b {
    true => {}
    false => {}   // the two cases are enough, no need for _
}

let byte: u8 = 200;
match byte {
    0..=127 => println!("ASCII"),
    128..=255 => println!("extended"),   // covers all of u8: exhaustive without _
}
```

What it is for: if you add a variant to an enum, every `match` that does not handle it becomes a compile error. This is the reason to avoid `_` on your own enums: it would silence this alert.

### 8.5 — Guards

A guard adds a condition after the pattern.

```rust
let temperature = Some(-3);
match temperature {
    Some(t) if t < 0 => println!("freezing"),
    Some(t) if t < 25 => println!("normal"),
    Some(_) => println!("hot"),
    None => println!("no measurement"),
}
```

The compiler does not take guards into account for exhaustiveness: the `Some(_)` branch without a guard is still needed.

### 8.6 — Tuples: combining several conditions

```rust
fn fizzbuzz(i: u32) -> String {
    match (i % 3, i % 5) {
        (0, 0) => "FizzBuzz".into(),
        (0, _) => "Fizz".into(),
        (_, 0) => "Buzz".into(),
        _ => i.to_string(),
    }
}
```

### 8.7 — Chained `if` or `match`: the same function

```rust
fn category(age: u32) -> &'static str {
    if age <= 12 {
        "child"
    } else if age <= 17 {
        "teenager"
    } else if age <= 64 {
        "adult"
    } else {
        "senior"
    }
}

fn category_match(age: u32) -> &'static str {
    match age {
        0..=12 => "child",
        13..=17 => "teenager",
        18..=64 => "adult",
        65.. => "senior",   // open range: 65 and above
    }
}
```

With `match`, the compiler flags a range that overlaps another or a gap (for example if you forget 13..=17 without `_`).

## 9. `if let`, `while let`, `let ... else`

These forms are shortcuts for `match` when **a single pattern** interests you.

### 9.1 — `if let`

```rust
let value: Option<i32> = Some(5);

// Syntax 1: match
match value {
    Some(n) => println!("{n}"),
    None => {}
}

// Syntax 2: if let
if let Some(n) = value {
    println!("{n}");
}

// With else
if let Some(n) = value {
    println!("{n}");
} else {
    println!("nothing");
}
```

### 9.2 — Chained `else if let`

```rust
enum Message { Text(String), Number(i32), Empty }

fn handle(m: Message) {
    if let Message::Text(t) = m {
        println!("text: {t}");
    } else if let Message::Number(n) = m {
        println!("number: {n}");
    } else {
        println!("other");
    }
}
```

As soon as you chain several `else if let` on the same value, a `match` is clearer and checks exhaustiveness.

### 9.3 — `while let`

```rust
let mut stack = vec![1, 2, 3];

// Syntax 1: loop + match
loop {
    match stack.pop() {
        Some(x) => println!("{x}"),
        None => break,
    }
}

// Syntax 2: while let
let mut stack = vec![1, 2, 3];
while let Some(x) = stack.pop() {
    println!("{x}"); // 3, 2, 1
}
```

What it is for: emptying a stack or a queue while adding elements during the traversal (impossible with `for`), reading a stream until its end.

```rust
// Breadth-first traversal: we add to the queue while emptying it
use std::collections::VecDeque;

let mut queue = VecDeque::from([1]);
while let Some(n) = queue.pop_front() {
    if n < 8 {
        queue.push_back(n * 2);
    }
    println!("{n}"); // 1, 2, 4, 8
}
```

```rust
// Receive messages until the channel is closed (tokio)
async fn receive(mut rx: tokio::sync::mpsc::Receiver<String>) {
    while let Some(msg) = rx.recv().await {
        println!("{msg}");
    }
}
```

### 9.4 — `let ... else` (Rust 1.65+)

`let ... else` extracts a value or **leaves**: the `else` block must diverge (`return`, `break`, `continue`, `panic!`).

```rust
fn read_id(text: &str) -> Option<u32> {
    // Syntax 1: match
    let id = match text.parse::<u32>() {
        Ok(id) => id,
        Err(_) => return None,
    };

    // Syntax 2: let else, same effect
    let Ok(id) = text.parse::<u32>() else {
        return None;
    };

    // Syntax 3: ok() then ?, the shortest when the exit is None
    let id = text.parse::<u32>().ok()?;

    Some(id)
}
```

What it is for: chaining checks at the start of a function without stacking indentation levels. The extracted variable stays available for the rest of the function, unlike `if let` where it only exists in the block.

## 10. Let-chains (2024 edition)

Since Rust 1.88 in **2024 edition**, you can chain `let` and conditions with `&&` in an `if` or a `while`.

### 10.1 — The same condition written three times

```rust
struct Session { user: Option<u32>, active: bool }

fn check(session: &Session) {
    // Syntax 1: nesting (the only option before 2024)
    if let Some(id) = session.user {
        if session.active {
            if id > 0 {
                println!("user {id}");
            }
        }
    }

    // Syntax 2: let-chain
    if let Some(id) = session.user && session.active && id > 0 {
        println!("user {id}");
    }

    // Syntax 3: match with a guard
    match session.user {
        Some(id) if session.active && id > 0 => println!("user {id}"),
        _ => {}
    }
}
```

### 10.2 — Several `let` in the same chain

```rust
fn city(json: &std::collections::HashMap<String, String>) {
    if let Some(address) = json.get("address")
        && let Some(city) = address.split(',').nth(1)
        && !city.trim().is_empty()
    {
        println!("city: {}", city.trim());
    }
}
```

Each `let` can use the variables bound by the previous ones. The chain stops at the first failure.

### 10.3 — Limits

- `||` is not allowed with a `let` in the chain: only `&&` is.
- The crate must be on `edition = "2024"` in `Cargo.toml` (see the Cargo course).
- `match` guards do not accept `let` yet (`Some(x) if let ...` is unstable): only boolean conditions are written there.

### 10.4 — Scope of temporaries in the 2024 edition

In the 2024 edition, temporary values created in the condition of an `if let` are dropped **before** the `else` block, and no longer at the end of the whole `if`.

```rust
use std::sync::Mutex;

let m = Mutex::new(Some(1));
if let Some(x) = *m.lock().unwrap() {
    println!("{x}");
} else {
    // In the 2021 edition, the lock was still held here: locking again = deadlock.
    // In the 2024 edition, it is already released.
    let _ = m.lock().unwrap();
}
```

## 11. `matches!`

`matches!(value, pattern)` returns a `bool`: true if the value matches the pattern.

### 11.1 — The same function written twice

```rust
enum Role { Admin, Moderator, Member }

fn can_moderate(r: &Role) -> bool {
    // Syntax 1: match
    match r {
        Role::Admin | Role::Moderator => true,
        _ => false,
    }
}

fn can_moderate_alt(r: &Role) -> bool {
    // Syntax 2: matches!
    matches!(r, Role::Admin | Role::Moderator)
}
```

`==` only works if the type implements `PartialEq`, and cannot test a shape (`Some(_)`) or a range. `matches!` handles everything a pattern can.

### 11.2 — With a range and a guard

```rust
let c = 'x';
let is_lowercase = matches!(c, 'a'..='z');

let grade: Option<u32> = Some(15);
let honours = matches!(grade, Some(n) if n >= 14);
```

### 11.3 — In a filter

```rust
let roles = [Role::Admin, Role::Member, Role::Moderator];
let staff_count = roles.iter().filter(|r| matches!(r, Role::Admin | Role::Moderator)).count();
```

## 12. Practical examples

### Example 1 — calculator

```rust
fn calculate(operator: char, a: i32, b: i32) -> Option<i32> {
    match operator {
        '+' => a.checked_add(b),
        '-' => a.checked_sub(b),
        '*' => a.checked_mul(b),
        '/' => a.checked_div(b),
        _ => None,
    }
}

fn calculate_guard(operator: char, a: i32, b: i32) -> Option<i32> {
    match operator {
        '+' => Some(a + b),
        '/' if b != 0 => Some(a / b),   // guard instead of an if in the branch
        _ => None,
    }
}

fn main() {
    match calculate('/', 10, 0) {
        Some(n) => println!("Result: {n}"),
        None => println!("Invalid operation"),
    }
}
```

`checked_*` returns `None` instead of panicking: `checked_div` covers both division by zero and `i32::MIN / -1`, which exceeds the capacity of an `i32`. The guard version shows the syntax, but it forgets this second case.

### Example 2 — prime number, three syntaxes

```rust
// Syntax 1: for with early exit
fn is_prime(n: u32) -> bool {
    if n < 2 {
        return false;
    }
    for i in 2..n {
        if n % i == 0 {
            return false;
        }
    }
    true
}

// Syntax 2: stop at the square root (much faster)
fn is_prime_fast(n: u32) -> bool {
    if n < 2 {
        return false;
    }
    let mut i = 2;
    while i * i <= n {
        if n % i == 0 {
            return false;
        }
        i += 1;
    }
    true
}

// Syntax 3: iterator
fn is_prime_iter(n: u32) -> bool {
    n >= 2 && (2..).take_while(|i| i * i <= n).all(|i| n % i != 0)
}
```

### Example 3 — searching an array, four syntaxes

```rust
fn find_index(array: &[i32], value: i32) -> Option<usize> {
    for (index, &element) in array.iter().enumerate() {
        if element == value {
            return Some(index);
        }
    }
    None
}

fn find_index_iter(array: &[i32], value: i32) -> Option<usize> {
    array.iter().position(|&x| x == value)
}

// On a SORTED array: binary search, O(log n) instead of O(n)
fn find_index_sorted(array: &[i32], value: i32) -> Option<usize> {
    array.binary_search(&value).ok()
}

fn main() {
    let numbers = [10, 20, 30, 40, 50];
    if let Some(index) = find_index_iter(&numbers, 30) {
        println!("Found at index {index}");
    }
}
```

### Example 4 — reading a configuration line by line

```rust
fn read_config(text: &str) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        pairs.push((key.trim().to_string(), value.trim().to_string()));
    }
    pairs
}
```

The cases to ignore are discarded first (`continue`), then `let ... else` extracts the pair.

## 13. Exercises

### Exercise 1 — age classification

Classify an age: 0-12 child, 13-17 teenager, 18-64 adult, 65 and over senior. Write it with `if`, then with `match`.

```rust
// Solution:
fn classify_age(age: u32) -> &'static str {
    match age {
        0..=12 => "Child",
        13..=17 => "Teenager",
        18..=64 => "Adult",
        65.. => "Senior",
    }
}
```

### Exercise 2 — sum of multiples

Compute the sum of the numbers from 1 to n divisible by 3 or by 5, first with `for`, then with an iterator.

```rust
// Solution:
fn sum_for(n: u32) -> u32 {
    let mut total = 0;
    for i in 1..=n {
        if i % 3 == 0 || i % 5 == 0 {
            total += i;
        }
    }
    total
}

fn sum_iter(n: u32) -> u32 {
    (1..=n).filter(|i| i % 3 == 0 || i % 5 == 0).sum()
}
```

### Exercise 3 — first even number

In a grid, return the position of the first even number, leaving both loops as soon as it is found.

```rust
// Solution:
fn first_even(grid: &[Vec<i32>]) -> Option<(usize, usize)> {
    let mut result = None;
    'grid: for (r, row) in grid.iter().enumerate() {
        for (c, &v) in row.iter().enumerate() {
            if v % 2 == 0 {
                result = Some((r, c));
                break 'grid;
            }
        }
    }
    result
}
```

### Exercise 4 — countdown

Print 10, 8, 6, 4, 2, 0 without `while`.

```rust
// Solution:
fn main() {
    for i in (0..=10).rev().step_by(2) {
        println!("{i}");
    }
}
```

## 14. Cheat sheet

### Conditions

```rust
if a { } else if b { } else { }
let x = if a { 1 } else { 2 };          // no ternary operator
let o = cond.then_some(value);           // Option from a bool
let o = cond.then(|| compute());         // same, lazy computation
```

### Loops

```rust
loop { break; }                          // no condition
let v = loop { break 42; };              // only loop returns a value
while cond { }                           // as long as
while let Some(x) = stack.pop() { }      // as long as the pattern matches
for i in 0..10 { }                       // 0 to 9
for i in 0..=10 { }                      // 0 to 10
for i in (0..10).rev() { }               // 9 to 0
for i in (0..10).step_by(2) { }          // 0, 2, 4, 6, 8
for x in &v { }                          // read
for x in &mut v { *x += 1; }             // modify
for x in v { }                           // consume
for (i, x) in v.iter().enumerate() { }   // with index
for (a, b) in x.iter().zip(&y) { }       // in parallel
'outer: for ... { break 'outer; }        // leave an outer loop
'outer: for ... { continue 'outer; }     // next iteration of the outer loop
let r = 'b: { break 'b 1; };             // labeled block
```

### Pattern matching

```rust
match v {
    1 => {}
    2 | 3 => {}
    4..=10 => {}
    x @ 11..=20 => {}
    n if n > 100 => {}
    _ => {}
}
if let Some(x) = opt { }
let Some(x) = opt else { return; };
if let Some(x) = opt && x > 0 { }        // let-chain, 2024 edition
matches!(v, Some(1..=5))
```

### Key points

- Conditions are `bool`, with no implicit conversion.
- `if`, `match`, `loop` and blocks are expressions.
- Only `loop` can exit with a value; `while` and `for` are `()`.
- `for x in v` consumes `v`: use `&v` or `&mut v` to keep it.
- To count backwards: `.rev()`, never `5..0`.
- `match` is exhaustive: avoid `_` on your own enums.
- `let ... else` and `continue` avoid nesting.
- A computation on a collection often reads better as an iterator; a loop with side effects and early exits stays a `for`.

## Going further

- `pattern-matching.md`: all patterns (structs, slices, `ref`, `@`).
- `closures-iterateurs.md`: writing and combining iterators.
- `collections.md`: iterating over `Vec`, `HashMap` and `HashSet`.
- `cargo-dependances.md`: choosing the 2024 edition for let-chains.
