# Advanced borrowing — Cow, AsRef, Borrow, ToOwned

> The traits that convert between owned and borrowed values: write functions that accept several forms of the same type, and allocate only when necessary.

## Prerequisites

The `borrow-bases`, `lifetimes` and `generics` courses.

## Objectives

- Understand the owned / borrowed pairs: `String`/`str`, `Vec<T>`/`[T]`, `PathBuf`/`Path`
- Accept several types as a parameter with `AsRef`, `Borrow`, `Into` and know which one to choose
- Use `ToOwned` to go from borrowed to owned
- Use `Cow` to allocate only when a modification is needed
- Put `Cow` in structs and public APIs
- Know the common techniques to avoid needless copies

## Table of contents

1. The owned / borrowed pairs
2. `AsRef<T>` and `AsMut<T>`
3. `Borrow<T>` and `BorrowMut<T>`
4. `ToOwned`
5. `AsRef`, `Borrow`, `Into`, `Deref`: which one to choose
6. `Cow<'a, B>`: borrow or own as needed
7. All the ways to create and read a `Cow`
8. `Cow` in practice
9. Techniques to avoid copies
10. Examples in Runique
11. The same function written in several ways
12. Exercises
13. Cheat sheet

---

## 1. The owned / borrowed pairs

| Owned (allocates, mutable) | Borrowed (view, no allocation) |
|---|---|
| `String` | `&str` |
| `Vec<T>` | `&[T]` |
| `PathBuf` | `&Path` |
| `OsString` | `&OsStr` |
| `Box<T>` | `&T` |

Each pair follows the same principle: the owned type **contains** the data; the borrowed type is a **view** on data that belongs to someone else. You go from one to the other:

```rust
let owned: String = String::from("abc");
let view: &str = &owned;               // owned -> borrowed: free
let copy: String = view.to_owned();    // borrowed -> owned: allocation
```

The traits in this course generalise these conversions.

---

## 2. `AsRef<T>` and `AsMut<T>`

### 2.1 The trait

```rust
pub trait AsRef<T: ?Sized> {
    fn as_ref(&self) -> &T;
}
```

"I can give you a `&T`, without copying." `String`, `&str`, `Box<str>` and `Cow<str>` all implement `AsRef<str>`.

### 2.2 Accepting several types as a parameter

```rust
fn count_words<S: AsRef<str>>(text: S) -> usize {
    text.as_ref().split_whitespace().count()
}

count_words("one two");                        // &str
count_words(String::from("one two three"));    // String
count_words(&String::from("a"));               // &String
```

The standard library uses it everywhere for paths:

```rust
use std::path::Path;

fn exists<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().exists()
}

exists("config.toml");                    // &str
exists(String::from("config.toml"));      // String
exists(Path::new("config.toml"));         // &Path
exists(std::path::PathBuf::from("a"));    // PathBuf
```

`std::fs::read_to_string`, `File::open` and `Command::new` all accept `impl AsRef<Path>` or `impl AsRef<OsStr>`.

### 2.3 Collections of strings

```rust
fn join_all<S: AsRef<str>>(parts: &[S]) -> String {
    parts.iter().map(|p| p.as_ref()).collect::<Vec<_>>().join(", ")
}

join_all(&["a", "b"]);
join_all(&[String::from("a"), String::from("b")]);
```

Without `AsRef`, you would need two functions, or you would have to convert the `Vec<String>` into a `Vec<&str>` before each call.

### 2.4 `AsMut<T>`

```rust
fn zero_out<B: AsMut<[u8]>>(mut buffer: B) {
    buffer.as_mut().fill(0);
}

let mut v = vec![1u8, 2, 3];
zero_out(&mut v);
let mut t = [9u8; 4];
zero_out(&mut t);
```

### 2.5 Implementing `AsRef` for your own type

```rust
struct Email(String);

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

count_words(Email("ada@ex.com".into()));
```

---

## 3. `Borrow<T>` and `BorrowMut<T>`

### 3.1 The trait

```rust
pub trait Borrow<Borrowed: ?Sized> {
    fn borrow(&self) -> &Borrowed;
}
```

The signature is the same as `AsRef`. The difference is a **promise**: `Borrow` guarantees that the borrowed form compares, hashes and orders **exactly like** the owned form.

### 3.2 Why it is useful: map lookups

```rust
use std::collections::HashMap;

let mut m: HashMap<String, u32> = HashMap::new();
m.insert("ada".to_string(), 36);

let age = m.get("ada");          // we search with a &str, without creating a String
```

The signature of `get`:

```rust
pub fn get<Q: ?Sized>(&self, k: &Q) -> Option<&V>
where
    K: Borrow<Q>,
    Q: Hash + Eq,
```

`String: Borrow<str>` lets you search with a `&str`. The promise of `Borrow` guarantees that `"ada"` hashed as a `&str` gives the same result as `String::from("ada")`: otherwise the lookup would fail silently.

### 3.3 Keeping the promise

```rust
// A case-insensitive key
#[derive(PartialEq, Eq, Hash)]
struct LowerKey(String);

// DO NOT: impl Borrow<str> for LowerKey
// "ADA" and "ada" are equal as LowerKey but different as &str:
// a lookup by &str would give inconsistent results.
```

If the equality or hashing of your type differs from that of the borrowed form, implement `AsRef`, not `Borrow`.

### 3.4 Lookup with a composite type

```rust
use std::collections::HashSet;

let mut s: HashSet<Vec<u8>> = HashSet::new();
s.insert(vec![1, 2]);
s.contains(&[1u8, 2][..]);      // Vec<u8>: Borrow<[u8]>: we search with a slice
```

### 3.5 `BorrowMut<T>`

The mutable equivalent, rarely used directly.

---

## 4. `ToOwned`

### 4.1 The trait

```rust
pub trait ToOwned {
    type Owned: Borrow<Self>;
    fn to_owned(&self) -> Self::Owned;
}
```

`Clone` only works from `&T` to `T`. `ToOwned` generalises it: from a borrowed form to **its** owned form, even if the types differ.

```rust
let s: &str = "abc";
let a: String = s.to_owned();          // str -> String
let t: &[i32] = &[1, 2];
let b: Vec<i32> = t.to_owned();        // [i32] -> Vec<i32>
let p = std::path::Path::new("a");
let c: std::path::PathBuf = p.to_owned();
```

### 4.2 `to_owned`, `to_string`, `String::from`, `into`

```rust
let s = "text";
let a = s.to_owned();        // ToOwned: generic, the most direct
let b = s.to_string();       // ToString (via Display): works for any displayable type
let c = String::from(s);     // From
let d: String = s.into();    // Into
```

All four give the same result for a `&str`, at the same cost. `to_string` is the only one that also works for `42.to_string()`.

---

## 5. `AsRef`, `Borrow`, `Into`, `Deref`: which one to choose

| Trait | Promises | Cost | Use as a parameter |
|---|---|---|---|
| `AsRef<T>` | "I can give a `&T`" | none | the function **reads**: `fn f(p: impl AsRef<Path>)` |
| `Borrow<T>` | same + same equality and hashing | none | collection keys (`get`, `contains`) |
| `Into<T>` | "I can become a `T`" | may allocate | the function **keeps** the value: `fn new(name: impl Into<String>)` |
| `Deref` | "I am a pointer to `T`" | none | not used as a bound: automatic conversion |

### 5.1 The same function with each form

```rust
struct User { name: String }

// &str: simple, accepts &String through automatic conversion (Deref), not String
fn greet_v1(name: &str) -> String { format!("Hello {name}") }

// AsRef: also accepts a String by value
fn greet_v2(name: impl AsRef<str>) -> String { format!("Hello {}", name.as_ref()) }

// Into<String>: the function needs an owned String
fn create(name: impl Into<String>) -> User { User { name: name.into() } }

fn main() {
    let s = String::from("Ada");
    greet_v1(&s);
    greet_v1("Linus");
    greet_v2(s.clone());
    create("Grace");         // allocates a String
    create(s);               // no copy: the String is moved
}
```

Simple rule:
- the function **reads** a string: `&str` (or `impl AsRef<str>` if the API must accept many forms);
- the function **stores** a string: `impl Into<String>` (or `String`);
- the function does a **lookup** in a collection: a `Borrow` bound.

---

## 6. `Cow<'a, B>`: borrow or own as needed

### 6.1 Definition

```rust
pub enum Cow<'a, B: ?Sized + ToOwned + 'a> {
    Borrowed(&'a B),
    Owned(<B as ToOwned>::Owned),
}
```

`Cow` (*clone on write*) holds **either** a reference, **or** an owned value. `Cow<'a, str>` is either a `&'a str` or a `String`.

### 6.2 The problem it solves

```rust
// Always allocate: a String is created even when nothing changes
fn no_spaces_v1(s: &str) -> String {
    s.replace(' ', "_")
}

// Allocate only if necessary
use std::borrow::Cow;

fn no_spaces_v2(s: &str) -> Cow<'_, str> {
    if s.contains(' ') {
        Cow::Owned(s.replace(' ', "_"))   // a change is needed: we allocate
    } else {
        Cow::Borrowed(s)                   // nothing to change: we return the input as is
    }
}
```

If most inputs have no space, the second version almost never allocates.

### 6.3 A `Cow` is used like the value

```rust
let r = no_spaces_v2("hello big world");
println!("{r}");                 // Display
println!("{}", r.len());         // Deref to str: all the str methods
let s: &str = &r;
```

---

## 7. All the ways to create and read a `Cow`

### 7.1 Creating

```rust
use std::borrow::Cow;

let a: Cow<str> = Cow::Borrowed("text");
let b: Cow<str> = Cow::Owned(String::from("text"));
let c: Cow<str> = "text".into();                  // From<&str>: Borrowed
let d: Cow<str> = String::from("text").into();    // From<String>: Owned
let e: Cow<[i32]> = Cow::from(&[1, 2][..]);
let f: Cow<[i32]> = Cow::from(vec![1, 2]);
```

### 7.2 Reading and converting

```rust
let mut c: Cow<str> = Cow::Borrowed("abc");

let length = c.len();                     // via Deref
let view: &str = c.as_ref();
let is_borrowed = matches!(c, Cow::Borrowed(_));

c.to_mut().push('!');                     // if Borrowed: copies first, then modifies; c becomes Owned
let owned: String = c.into_owned();       // String: no copy if already Owned
```

| Method | Effect |
|---|---|
| `Deref` (`*c`, `c.len()`) | read, in every case, without copying |
| `to_mut()` | `&mut` to the owned value; copies the first time if `Borrowed` |
| `into_owned()` | the owned value; copies only if `Borrowed` |
| `clone()` | if `Borrowed`, copies the reference; if `Owned`, copies the value |

### 7.3 Modify only if necessary

```rust
use std::borrow::Cow;

fn normalize(input: &str) -> Cow<'_, str> {
    let mut r = Cow::Borrowed(input);
    if r.starts_with(' ') || r.ends_with(' ') {
        r = Cow::Owned(r.trim().to_string());
    }
    if r.contains("  ") {
        let simplified = r.split_whitespace().collect::<Vec<_>>().join(" ");
        r = Cow::Owned(simplified);
    }
    r
}
```

---

## 8. `Cow` in practice

### 8.1 HTML escaping

```rust
use std::borrow::Cow;

fn escape(s: &str) -> Cow<'_, str> {
    if !s.contains(['<', '>', '&', '"', '\'']) {
        return Cow::Borrowed(s);               // the common case: no special character
    }
    let mut r = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '<' => r.push_str("&lt;"),
            '>' => r.push_str("&gt;"),
            '&' => r.push_str("&amp;"),
            '"' => r.push_str("&quot;"),
            '\'' => r.push_str("&#x27;"),
            _ => r.push(c),
        }
    }
    Cow::Owned(r)
}
```

This is the principle of template engines (Tera escapes every variable printed with `{{ }}`): most texts have nothing to escape.

### 8.2 Messages: `Cow<'static, str>`

```rust
use std::borrow::Cow;

fn message(code: u16, detail: Option<&str>) -> Cow<'static, str> {
    match (code, detail) {
        (404, None) => Cow::Borrowed("Page not found"),
        (404, Some(d)) => Cow::Owned(format!("Page not found: {d}")),
        (_, _) => Cow::Borrowed("Error"),
    }
}
```

`Cow<'static, str>` is either a literal (no allocation, lives for the whole program) or a built `String`. It is an ideal type for messages, field names and configuration keys.

### 8.3 In a struct

```rust
use std::borrow::Cow;

struct Request<'a> {
    path: Cow<'a, str>,
}

impl<'a> Request<'a> {
    fn from_path(path: &'a str) -> Self {
        Self { path: Cow::Borrowed(path) }
    }

    fn normalize(&mut self) {
        if self.path.ends_with('/') && self.path.len() > 1 {
            let trimmed = self.path.trim_end_matches('/').to_string();
            self.path = Cow::Owned(trimmed);
        }
    }

    fn into_static(self) -> Request<'static> {
        Request { path: Cow::Owned(self.path.into_owned()) }   // detach from the source
    }
}
```

### 8.4 `Cow` and serde

```rust
#[derive(serde::Deserialize)]
struct Article<'a> {
    #[serde(borrow)]
    title: Cow<'a, str>,     // borrows from the source JSON when no escape has to be decoded
}
```

With `#[serde(borrow)]`, a `Cow<str>` field borrows the JSON text directly if it contains no escape sequence (`\n`, `\"`), and allocates otherwise.

### 8.5 When not to use `Cow`

`Cow` adds a lifetime and an internal `match`. For a function that **always** modifies its input, return a `String` directly. For a function that **never** modifies it, return a `&str`. `Cow` is only worth it if both cases exist and the unmodified case is frequent.

---

## 9. Techniques to avoid copies

### 9.1 Borrow in loops

```rust
let users = vec![String::from("ada"), String::from("linus")];

// Copies each name
let names: Vec<String> = users.iter().cloned().collect();

// Borrows: no copy, valid as long as users exists
let views: Vec<&str> = users.iter().map(String::as_str).collect();
```

### 9.2 Return a reference rather than a copy

```rust
struct Profile { name: String }

impl Profile {
    fn name(&self) -> &str { &self.name }                 // no clone
    fn name_copy(&self) -> String { self.name.clone() }   // only if the caller must own it
}
```

### 9.3 Write into a provided buffer

```rust
use std::fmt::Write;

fn write_line(output: &mut String, name: &str, age: u32) {
    let _ = writeln!(output, "{name};{age}");     // appends to the existing buffer
}

let mut csv = String::with_capacity(1024);
for (n, a) in [("ada", 36), ("linus", 54)] {
    write_line(&mut csv, n, a);                    // a single allocation for the whole file
}
```

### 9.4 Move instead of clone

```rust
fn register(name: String) {}

let name = String::from("ada");
// register(name.clone());       // useless if name is no longer used
register(name);                   // move: no copy
```

`clippy::redundant_clone` flags clones whose source is not used afterwards.

### 9.5 Take instead of clone

```rust
struct Form { errors: Vec<String> }

impl Form {
    fn drain_errors(&mut self) -> Vec<String> {
        std::mem::take(&mut self.errors)        // takes the Vec, leaves an empty Vec
    }
}
```

---

## 10. Examples in Runique

### 10.1 Translations: `Cow<'static, str>`

```rust
// runique/src/utils/trad/switch_lang.rs
pub fn t(key: &str) -> Cow<'static, str> { /* ... */ }
pub fn tf<T: Display>(key: &str, args: &[T]) -> String { /* ... */ }
```

`t` returns a `Cow<'static, str>`: the translation of a key without parameters is a fixed text, returned without allocation. `tf` fills in parameters (`{}`): the result is necessarily new, hence a `String`. Both signatures follow the rule of section 8.5 exactly.

### 10.2 `effective_key` of the `LoginGuard`

```rust
// runique/src/auth/guard.rs
pub fn effective_key<'a>(username: &'a str, ip: &str) -> std::borrow::Cow<'a, str> {
    let username = username.trim();
    if username.is_empty() {
        std::borrow::Cow::Owned(format!("anonym:{ip}"))
    } else {
        std::borrow::Cow::Owned(username.to_lowercase())
    }
}
```

The key of the login attempt counter is normalised (`trim` then `to_lowercase`): `admin`, `Admin` and `admin ` share a single counter. Without this, changing the case would bypass the lockout. The normalisation allocates, so both branches return `Owned`; the `Cow` signature leaves room to return `Borrowed` (for example for an input already in lowercase) without changing the callers.

### 10.3 Flexible parameters

Runique constructors often accept `impl Into<String>` for names and labels: the user passes a literal or a `String` without explicit conversion, and the value is stored owned (section 5).

---

## 11. The same function written in several ways

Capitalise the first letter of a word.

```rust
use std::borrow::Cow;

// 1. Always a new String
fn capitalize_v1(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

// 2. Cow: no allocation if the first letter is already uppercase
fn capitalize_v2(s: &str) -> Cow<'_, str> {
    match s.chars().next() {
        Some(f) if f.is_uppercase() || !f.is_alphabetic() => Cow::Borrowed(s),
        Some(_) => Cow::Owned(capitalize_v1(s)),
        None => Cow::Borrowed(s),
    }
}

// 3. In place, in the caller's String (ASCII only: same length in bytes)
fn capitalize_v3(s: &mut String) {
    if let Some(first) = s.get_mut(0..1) {
        first.make_ascii_uppercase();
    }
}

// 4. Generic input, owned output
fn capitalize_v4(s: impl AsRef<str>) -> String {
    capitalize_v1(s.as_ref())
}
```

| Version | Allocation | When |
|---|---|---|
| 1 | always | simple, when the result must be stored |
| 2 | only if necessary | called very often on data that is already correct |
| 3 | never | the caller already owns the `String` and no longer needs the original |
| 4 | always | API that accepts `&str`, `String`, `Cow`... |

---

## 12. Exercises

### Exercise 1 — `AsRef`

Write `extension(path: impl AsRef<Path>) -> Option<String>` that returns the extension in lowercase, and call it with a `&str`, a `String` and a `PathBuf`.

```rust
// Solution:
use std::path::{Path, PathBuf};

fn extension(path: impl AsRef<Path>) -> Option<String> {
    path.as_ref().extension()?.to_str().map(str::to_lowercase)
}

fn main() {
    extension("photo.JPG");
    extension(String::from("doc.pdf"));
    extension(PathBuf::from("archive.tar.gz"));
}
```

### Exercise 2 — `Cow`

Write `strip_accents(s: &str) -> Cow<str>` that replaces `é`, `è`, `ê` with `e` and `à` with `a`, without allocating when there is nothing to replace.

```rust
// Solution:
use std::borrow::Cow;

fn strip_accents(s: &str) -> Cow<'_, str> {
    if !s.contains(['é', 'è', 'ê', 'à']) {
        return Cow::Borrowed(s);
    }
    Cow::Owned(
        s.chars()
            .map(|c| match c {
                'é' | 'è' | 'ê' => 'e',
                'à' => 'a',
                other => other,
            })
            .collect(),
    )
}

assert!(matches!(strip_accents("rust"), Cow::Borrowed(_)));
assert_eq!(strip_accents("été"), "ete");
```

### Exercise 3 — lookup key

A `HashMap<String, u32>` is queried thousands of times with `&str` values. Write the lookup without creating a `String`.

```rust
// Solution:
use std::collections::HashMap;

fn age(m: &HashMap<String, u32>, name: &str) -> Option<u32> {
    m.get(name).copied()       // String: Borrow<str>
}
```

## 13. Cheat sheet

```rust
fn f(x: impl AsRef<str>)        // read: accepts &str, String, &String, Cow<str>
fn f(x: impl AsRef<Path>)       // paths
fn f(x: impl Into<String>)      // store: accepts &str (allocates) and String (moves)
map.get(&borrowed_key)          // thanks to K: Borrow<Q>
x.to_owned()                    // borrowed -> owned (&str -> String, &[T] -> Vec<T>)

Cow::Borrowed(&x)   Cow::Owned(v)   "x".into()   String::from("x").into()
c.to_mut()          c.into_owned()  *c / c.len()
fn f(s: &str) -> Cow<'_, str>   // allocates only if necessary
Cow<'static, str>               // literal or built String
```

### Key points

- Every owned type has its borrowed view: `String`/`str`, `Vec<T>`/`[T]`, `PathBuf`/`Path`.
- `AsRef` to read several forms, `Into` to store, `Borrow` for collection lookups.
- `Borrow` promises the same equality and hashing: do not implement it if that is not true.
- `Cow` allocates only when a modification is needed; useful if the unmodified case is frequent.
- `Cow<'static, str>`: fixed or built text, with no lifetime to propagate.
- Moving, borrowing and taking (`mem::take`) avoid most `clone` calls.
