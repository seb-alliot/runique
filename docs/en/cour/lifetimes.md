# Lifetimes

> Lifetimes tell the compiler how long a reference stays valid. This course shows when you have to write them, when the compiler guesses them, and every way to avoid them.

## Objectives

- Understand what a lifetime is and why the compiler needs one
- Read and write `'a` in a function, a struct, an `impl` block, a trait
- Know the three elision rules that spare you from writing them most of the time
- Tell `&'static T` and the `T: 'static` bound apart
- Use several lifetimes and the `'a: 'b` relations
- Fix the common errors (E0106, E0515, E0597, E0716)
- Know when to return an owned value rather than a reference

## Table of contents

1. The problem
2. What a lifetime is
3. Syntax
4. Lifetimes in functions
5. The elision rules
6. Lifetimes in structs
7. Lifetimes in `impl` blocks
8. `'static`
9. Several lifetimes and relations
10. Lifetimes, traits and closures
11. Common errors and fixes
12. The same function written in several ways
13. Exercises
14. Cheat sheet

---

## 1. The problem

```rust
fn main() {
    let r;
    {
        let x = 5;
        r = &x;
    }                       // x is freed here
    // println!("{r}");     // ERROR E0597: `x` does not live long enough
}
```

`r` would point to freed memory. The compiler compares the **lifetime** of `x` (the inner block) with that of `r` (the whole function) and refuses.

---

## 2. What a lifetime is

A lifetime is the **portion of the program** during which a reference is valid. It only exists at compile time: no trace in the binary, no cost.

```rust
fn main() {
    let x = 5;              // ----------+-- 'b (lifetime of x)
    let r = &x;             // --+-- 'a  |
    println!("{r}");        //   |       |
                            // --+       |
}                           // ----------+
```

The reference `r` (lifetime `'a`) lives for a shorter time than `x` (lifetime `'b`): this is valid.

In the vast majority of cases, the compiler **infers** lifetimes. You only write them when it cannot guess a relation between several references.

---

## 3. Syntax

```rust
&i32            // a reference
&'a i32         // a reference with lifetime 'a
&'a mut i32     // a mutable reference with lifetime 'a
```

- The name starts with an apostrophe, in lowercase, often short: `'a`, `'b`, or descriptive: `'req`, `'conn`.
- It is **declared** between angle brackets, like a generic parameter: `fn f<'a>(...)`, `struct S<'a>`, `impl<'a>`.
- `'_` is an anonymous lifetime: "infer it".
- `'static` is the only predefined named lifetime.

An annotation **does not change** the real lifetime of a value. It describes a relation that the compiler checks.

---

## 4. Lifetimes in functions

### 4.1 — The case that needs an annotation

```rust
// fn longest(a: &str, b: &str) -> &str {
//     if a.len() > b.len() { a } else { b }
// }
// ERROR E0106: missing lifetime specifier
```

The result comes from `a` **or** from `b`: the compiler does not know how long it stays valid.

```rust
fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() > b.len() { a } else { b }
}
```

Reading: "for a lifetime `'a`, if `a` and `b` live at least `'a`, the result also lives `'a`". In practice, the result is valid as long as **both** arguments are.

### 4.2 — What the annotation allows and forbids

```rust
fn main() {
    let a = String::from("long string");
    let result;
    {
        let b = String::from("short");
        result = longest(&a, &b);
        println!("{result}");        // OK: a and b are still alive
    }
    // println!("{result}");         // ERROR: b is freed, result could come from b
}
```

The compiler does not look at **which** of the two is returned at run time: it applies the signature.

### 4.3 — When the result depends on a single argument

```rust
fn first_word<'a>(text: &'a str, _separator: &str) -> &'a str {
    text.split(' ').next().unwrap_or("")
}
```

`_separator` does not need `'a`: the result never comes from it. Linking only what is really linked makes the function easier to call.

### 4.4 — You cannot return a reference to a local variable

```rust
// fn create<'a>() -> &'a str {
//     let s = String::from("hello");
//     &s                       // ERROR E0515: returns a reference to data owned by the current function
// }

fn create() -> String {          // return the value itself
    String::from("hello")
}

fn create_constant() -> &'static str {   // or a literal, which lives for the whole program
    "hello"
}
```

No annotation can extend the life of a local value: it is freed at the end of the function.

---

## 5. The elision rules

The compiler applies three rules to infer the lifetimes of a signature. If they are enough, you write nothing.

1. Each reference parameter gets **its own** lifetime.
2. If there is **exactly one** reference parameter, its lifetime is given to all the references in the result.
3. If there is `&self` or `&mut self`, the lifetime of `self` is given to all the references in the result.

### Examples

```rust
fn length(s: &str) -> usize                   // no reference returned: nothing to infer
fn first(s: &str) -> &str                     // rule 2: fn first<'a>(s: &'a str) -> &'a str
fn name(&self) -> &str                        // rule 3: tied to self
fn choose(a: &str, b: &str) -> &str           // two parameters, no self: ANNOTATION REQUIRED
fn get(&self, key: &str) -> &str              // rule 3: tied to self, not to key
```

The last line is a trap: if the method really returns part of `key`, rule 3 is wrong and you have to write it out:

```rust
struct Dict;
impl Dict {
    fn or_key<'a>(&self, key: &'a str) -> &'a str {
        key
    }
}
```

---

## 6. Lifetimes in structs

### 6.1 — A struct that borrows

```rust
struct Excerpt<'a> {
    text: &'a str,
}

fn main() {
    let novel = String::from("Once upon a time. The rest.");
    let first = novel.split('.').next().unwrap();
    let e = Excerpt { text: first };
    println!("{}", e.text);
}   // e must go away before novel: guaranteed by 'a
```

`Excerpt<'a>` means: "an `Excerpt` cannot live longer than the text it borrows".

### 6.2 — Owned or borrowed: the same struct twice

```rust
// 1. Borrows: no copy, but tied to the lifetime of the source
struct LineRef<'a> {
    name: &'a str,
    value: &'a str,
}

// 2. Owns: independent, can be stored, returned, sent to a thread
struct Line {
    name: String,
    value: String,
}

fn parse<'a>(text: &'a str) -> Vec<LineRef<'a>> {
    text
        .lines()
        .filter_map(|l| l.split_once('='))
        .map(|(name, value)| LineRef { name, value })
        .collect()
}
```

| | Struct that borrows | Struct that owns |
|---|---|---|
| Copy of the data | none | one allocation per field |
| Can outlive the source | no | yes |
| Usable in a thread, an async task, a cache | with difficulty | yes |
| Complexity | lifetimes to propagate everywhere | none |

Rule of thumb: start with **owned** fields. Switch to references when a measurement shows that copies cost too much, or for temporary views (parser, iterator).

### 6.3 — Propagation

A struct that contains a struct with a lifetime has one too:

```rust
struct Document<'a> {
    lines: Vec<LineRef<'a>>,
}
```

---

## 7. Lifetimes in `impl` blocks

### 7.1 — Declare, then use

```rust
impl<'a> Excerpt<'a> {
    fn text(&self) -> &str {            // elision rule 3: tied to self
        self.text
    }

    fn source_text(&self) -> &'a str {  // tied to the source, not to self
        self.text
    }
}
```

The difference matters: the result of `source_text` stays valid after the `Excerpt` is gone, as long as the source exists.

```rust
fn extract(novel: &str) -> &str {
    let e = Excerpt { text: novel };
    e.source_text()                      // OK: tied to novel
    // e.text()                          // ERROR: tied to e, which goes away here
}
```

### 7.2 — `'_` in an `impl`

```rust
impl Excerpt<'_> {
    fn length(&self) -> usize {
        self.text.len()
    }
}
```

`'_` avoids declaring `'a` when the block does not use it.

---

## 8. `'static`

`'static` has two meanings depending on where it appears.

### 8.1 — `&'static T`: a reference valid until the end of the program

```rust
let s: &'static str = "literal";      // literals are stored in the binary
static NAME: &str = "Runique";         // a static is 'static
```

You can also create one by deliberately "leaking" memory:

```rust
let config: &'static str = Box::leak(String::from("read at startup").into_boxed_str());
```

Keep this for values created once at startup and used until the end.

### 8.2 — `T: 'static`: a type that contains no temporary reference

```rust
fn launch<T: Send + 'static>(value: T) {
    std::thread::spawn(move || {
        let _v = value;
    });
}

launch(String::from("ok"));          // String: owns its data, so 'static
launch(42);                          // OK
let local = String::from("x");
// launch(&local);                   // ERROR: &local is not 'static
```

`T: 'static` does **not** mean the value lives for the whole program. It means it **could**: it depends on no borrow. A `String`, a `Vec<u8>`, an `Arc<T>` are `'static`. This is the bound of `thread::spawn` and `tokio::spawn`, because the task can outlive the function that launches it.

### 8.3 — Fixing "borrowed value does not live long enough" with `spawn`

```rust
use std::sync::Arc;

let data = vec![1, 2, 3];

// 1. Move the value into the task
std::thread::spawn(move || println!("{data:?}"));

// 2. Share with Arc: each task owns a copy of the pointer
let shared = Arc::new(vec![1, 2, 3]);
let p = Arc::clone(&shared);
std::thread::spawn(move || println!("{p:?}"));

// 3. Scoped threads: borrowing is allowed
let v = vec![1, 2, 3];
std::thread::scope(|s| {
    s.spawn(|| println!("{v:?}"));     // no 'static: the scope waits for the thread to finish
});
```

---

## 9. Several lifetimes and relations

### 9.1 — Two independent lifetimes

```rust
struct Parser<'text, 'pattern> {
    text: &'text str,
    pattern: &'pattern str,
}

impl<'text, 'pattern> Parser<'text, 'pattern> {
    fn first_match(&self) -> Option<&'text str> {
        let i = self.text.find(self.pattern)?;
        Some(&self.text[i..i + self.pattern.len()])
    }
}
```

The result only depends on `'text`: the caller can free the pattern without losing the result. With a single lifetime `'a` for both, it could not.

### 9.2 — `'a: 'b`: "`'a` lives at least as long as `'b`"

```rust
fn choose<'long: 'short, 'short>(a: &'long str, _b: &'short str) -> &'short str {
    a                       // OK: a longer reference can stand in for a shorter one
}
```

### 9.3 — `T: 'a`: "`T` contains no reference shorter than `'a`"

```rust
struct Cache<'a, T: 'a> {
    values: &'a [T],
}
```

The compiler infers this bound itself in most cases; you mostly see it in error messages.

---

## 10. Lifetimes, traits and closures

### 10.1 — Trait objects

```rust
trait Rule { fn ok(&self, s: &str) -> bool; }

struct Contains<'a>(&'a str);
impl Rule for Contains<'_> {
    fn ok(&self, s: &str) -> bool { s.contains(self.0) }
}

// Box<dyn Rule> implies Box<dyn Rule + 'static>: rejects rules that borrow
fn static_rules() -> Vec<Box<dyn Rule>> { vec![] }

// With an explicit lifetime, a rule can borrow
fn rules<'a>(word: &'a str) -> Vec<Box<dyn Rule + 'a>> {
    vec![Box::new(Contains(word))]
}
```

### 10.2 — `impl Trait` in return position

```rust
fn words(text: &str) -> impl Iterator<Item = &str> {
    text.split_whitespace()
}
```

In the 2024 edition, an `impl Trait` in return position **automatically captures** all the lifetimes of the parameters: the returned iterator is tied to `text`, without writing anything. In the 2021 edition, you often had to add `+ '_`. To restrict what is captured: `impl Iterator<Item = u32> + use<>`.

### 10.3 — Closures that receive a reference

```rust
fn apply<F>(f: F) -> usize
where
    F: Fn(&str) -> usize,          // in reality: for<'a> Fn(&'a str) -> usize
{
    let local = String::from("local text");
    f(&local)
}

apply(|s| s.len());
```

`F: Fn(&str) -> usize` means "a closure that accepts a `&str` of **any** lifetime". The long form `for<'a> Fn(&'a str)` is called a higher-ranked bound (HRTB); you rarely see it written out, but it appears in some error messages.

---

## 11. Common errors and fixes

### 11.1 — E0106: missing lifetime

```rust
// struct User { name: &str }
struct User1 { name: String }            // 1. own it
struct User2<'a> { name: &'a str }       // 2. declare the lifetime
```

### 11.2 — E0515: returning a reference to a local

```rust
// fn full_name(f: &str, l: &str) -> &str { &format!("{f} {l}") }
fn full_name(f: &str, l: &str) -> String { format!("{f} {l}") }
```

A value **created** in the function is returned owned.

### 11.3 — E0597: does not live long enough

```rust
let result;
let a = String::from("abc");
{
    let b = String::from("de");
    result = longest(&a, &b);
}
// println!("{result}");
```

Fixes: use the result inside the block; declare `b` in the same scope as `a`; or make a copy (`longest(&a, &b).to_string()`).

### 11.4 — E0716: temporary value freed too early

```rust
// let s: &str = String::from("abc").as_str();   // the temporary String dies at the end of the line
// println!("{s}");

let owned = String::from("abc");                 // 1. name the value
let s: &str = owned.as_str();

let s2: &str = &String::from("abc");             // 2. a direct & extends the temporary to the end of the block
```

### 11.5 — Struct that contains a reference to itself

```rust
// struct Analysis {
//     text: String,
//     first_word: &str,      // would like to point into text: impossible
// }

struct Analysis {
    text: String,
    first_word: std::ops::Range<usize>,   // store positions rather than a reference
}

impl Analysis {
    fn first_word(&self) -> &str {
        &self.text[self.first_word.clone()]
    }
}
```

A struct cannot contain a reference to one of its own fields (it would be invalidated when the struct is moved). You store indexes, or you split the owner and the view into two structs.

---

## 12. The same function written in several ways

Return the longer of two strings.

```rust
use std::borrow::Cow;

// 1. Tied references: no copy, the result depends on both arguments
fn longest_v1<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() >= b.len() { a } else { b }
}

// 2. Owned value: independent, but one allocation
fn longest_v2(a: &str, b: &str) -> String {
    if a.len() >= b.len() { a.to_string() } else { b.to_string() }
}

// 3. An index: no reference to return
fn longest_v3(a: &str, b: &str) -> usize {
    if a.len() >= b.len() { 0 } else { 1 }
}

// 4. Cow: borrows when possible, owns when it has to transform
fn longest_v4<'a>(a: &'a str, b: &'a str, uppercase: bool) -> Cow<'a, str> {
    let chosen = if a.len() >= b.len() { a } else { b };
    if uppercase {
        Cow::Owned(chosen.to_uppercase())
    } else {
        Cow::Borrowed(chosen)
    }
}
```

| Version | When |
|---|---|
| 1 | the result is used right away, while the sources exist |
| 2 | the result must be stored or outlive the sources |
| 3 | the caller already has both values and just wants to know which one |
| 4 | the result is sometimes a transformed copy (see `borrow-avance`) |

---

## 13. Exercises

### Exercise 1 — annotate

Add the lifetimes that are needed, and only those.

```rust
fn shortest(a: &str, b: &str) -> &str { if a.len() < b.len() { a } else { b } }
fn prefix(text: &str, n: usize) -> &str { &text[..n] }
fn find(text: &str, pattern: &str) -> Option<&str> { text.find(pattern).map(|i| &text[i..]) }
```

```rust
// Solution:
fn shortest<'a>(a: &'a str, b: &'a str) -> &'a str { if a.len() < b.len() { a } else { b } }
fn prefix(text: &str, n: usize) -> &str { &text[..n] }                 // rule 2: nothing to write
fn find<'a>(text: &'a str, pattern: &str) -> Option<&'a str> {         // tied to text only
    text.find(pattern).map(|i| &text[i..])
}
```

### Exercise 2 — struct that borrows

Write `Words<'a>` that keeps a `&'a str` and a method `longest(&self) -> &'a str`.

```rust
// Solution:
struct Words<'a> {
    text: &'a str,
}

impl<'a> Words<'a> {
    fn longest(&self) -> &'a str {
        self.text.split_whitespace().max_by_key(|w| w.chars().count()).unwrap_or("")
    }
}
```

### Exercise 3 — fix without an annotation

This function does not compile. Fix it in two ways without writing any lifetime.

```rust
// fn greet(name: &str) -> &str { &format!("Hello {name}") }
```

```rust
// Solution 1: return a String
fn greet(name: &str) -> String { format!("Hello {name}") }

// Solution 2: write into a buffer provided by the caller
fn greet_into(name: &str, output: &mut String) {
    use std::fmt::Write;
    let _ = write!(output, "Hello {name}");
}
```

---

## 14. Cheat sheet

```rust
fn f<'a>(x: &'a str, y: &'a str) -> &'a str    // result tied to both
fn f<'a>(x: &'a str, y: &str) -> &'a str       // tied to x only
fn f(x: &str) -> &str                          // elision: tied to x
fn m(&self, k: &str) -> &str                   // elision: tied to self
struct S<'a> { r: &'a str }
impl<'a> S<'a> { fn get(&self) -> &'a str { self.r } }
impl S<'_> { }
&'static str                                    // valid until the end of the program
T: 'static                                      // T contains no temporary borrow
'a: 'b                                          // 'a lasts at least as long as 'b
Box<dyn Trait + 'a>                             // trait object that can borrow
std::thread::scope(|s| { s.spawn(|| ..); })     // threads that borrow
```

### Key points

- A lifetime describes a relation; it never extends the life of a value.
- The three elision rules cover most signatures.
- Annotate only the references that are really tied to the result.
- A value created in a function is returned owned, never by reference.
- `T: 'static` = "contains no borrow", not "lives forever".
- Start with structs that own; borrow when measured as useful.
- A struct cannot point into its own fields: store indexes.
