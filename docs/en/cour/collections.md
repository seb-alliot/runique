# Collections in Rust

**Vec, VecDeque, HashMap, BTreeMap, HashSet, BTreeSet, BinaryHeap**: create, fill, read, modify and iterate over each collection, with the different ways of doing the same operation and the cost of each.

## Course objectives

By the end of this course, you will know how to:

- Create a collection in every way (`new`, macro, `from`, `collect`, reserved capacity)
- Add, remove, search and sort the elements of a `Vec`
- Read an element without risking a panic
- Use the `entry` API of a `HashMap` to count, group or initialize
- Perform set operations with `HashSet`
- Choose the right collection based on the most frequent operation
- Understand what is moved or borrowed when you put a value in a collection

## Table of contents

1. Overview
2. `Vec<T>`: create
3. `Vec<T>`: add and remove
4. `Vec<T>`: read
5. `Vec<T>`: sort, search, deduplicate
6. `Vec<T>`: iterate and transform
7. Slices and `Vec`
8. `VecDeque<T>`
9. `HashMap<K, V>`: create and fill
10. `HashMap<K, V>`: read and modify
11. The `entry` API
12. `BTreeMap<K, V>`
13. `HashSet<T>` and `BTreeSet<T>`
14. `BinaryHeap<T>`
15. Ownership and collections
16. Choosing the right collection
17. The same task written in several ways
18. Exercises
19. Cheat sheet

## 1. Overview

| Collection | Content | Order | Typical use |
|---|---|---|---|
| `Vec<T>` | sequence of elements | insertion order | the default collection |
| `VecDeque<T>` | sequence, add/remove at both ends | insertion order | queue |
| `HashMap<K, V>` | key → value pairs | none | lookup by key |
| `BTreeMap<K, V>` | key → value pairs | sorted by key | lookup by key + sorted iteration, ranges |
| `HashSet<T>` | unique values | none | membership, deduplication |
| `BTreeSet<T>` | unique values | sorted | sorted set |
| `BinaryHeap<T>` | priority queue | largest first | process the highest priority |

All of them are in `std::collections`, except `Vec` (imported by default).

```rust
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, VecDeque};
```

## 2. `Vec<T>`: create

### 2.1 — The different ways

```rust
let v1: Vec<i32> = Vec::new();             // empty, type annotated
let mut v2 = Vec::new();                    // empty, type inferred later
v2.push(1);

let v3 = vec![1, 2, 3];                     // macro with elements
let v4 = vec![0; 10];                       // ten zeros
let v5 = Vec::from([1, 2, 3]);              // from an array
let v6: Vec<i32> = (1..=5).collect();       // from an iterator
let v7 = vec!["a"; 3];                      // ["a", "a", "a"]
let v8: Vec<char> = "abc".chars().collect();
let v9 = [1, 2].to_vec();                   // copy of a slice
```

### 2.2 — Capacity and length

```rust
let mut v = Vec::with_capacity(100);   // reserves room for 100 elements, length 0
println!("{} {}", v.len(), v.capacity());  // 0 100
for i in 0..100 {
    v.push(i);                          // no reallocation
}
v.reserve(50);                          // guarantees room for 50 more
v.shrink_to_fit();                      // gives back unused memory
```

- `len()`: number of elements.
- `capacity()`: allocated space.

When `len` reaches `capacity`, `push` allocates a block about twice as large and copies everything into it. What `with_capacity` is for: when you know the final size, it avoids these successive copies. `collect` does it on its own when the iterator knows its size.

### 2.3 — `vec![x; n]` and cloning

```rust
let rows = vec![Vec::<i32>::new(); 3];   // three independent Vecs (each one cloned)
```

`vec![x; n]` **clones** `x`: the type must implement `Clone`.

## 3. `Vec<T>`: add and remove

### 3.1 — At the end (fast)

```rust
let mut v = vec![1, 2, 3];
v.push(4);                     // [1, 2, 3, 4]
let last = v.pop();            // Some(4); None if empty
```

### 3.2 — In the middle (slow: shifts the elements)

```rust
let mut v = vec![1, 2, 3];
v.insert(1, 10);               // [1, 10, 2, 3]
let removed = v.remove(0);     // 1; [10, 2, 3] — panics if the index does not exist
```

### 3.3 — Removing without keeping the order (fast)

```rust
let mut v = vec![1, 2, 3, 4];
let x = v.swap_remove(0);      // 1; the last one takes its place: [4, 2, 3]
```

`remove` shifts all the following elements (cost proportional to the size), `swap_remove` moves only one. Use it when the order does not matter.

### 3.4 — Several elements

```rust
let mut v = vec![1, 2, 3];
v.extend([4, 5]);                        // adds from any iterator
v.extend_from_slice(&[6, 7]);            // copies from a slice

let mut other = vec![8, 9];
v.append(&mut other);                    // moves everything: other becomes empty

let tail = v.split_off(5);               // v keeps [0..5), tail receives the rest
v.truncate(2);                           // keeps the first 2
let extracted: Vec<i32> = v.drain(..1).collect();   // removes a range and returns it
v.clear();                               // empties (keeps the capacity)
```

### 3.5 — Removing based on a condition

```rust
let mut v = vec![1, 2, 3, 4, 5, 6];

// 1. retain: keeps those that satisfy the condition (in place)
v.retain(|&x| x % 2 == 0);              // [2, 4, 6]

// 2. filter + collect: new collection, the original is untouched
let v = vec![1, 2, 3, 4];
let evens: Vec<i32> = v.iter().copied().filter(|x| x % 2 == 0).collect();

// 3. extract_if (Rust 1.87+): removes AND returns what is removed
let mut v = vec![1, 2, 3, 4];
let odds: Vec<i32> = v.extract_if(.., |x| *x % 2 == 1).collect();   // v = [2, 4]
```

Never remove elements by index in a `for i in 0..v.len()` loop: the indexes shift and the end of the loop goes out of bounds.

## 4. `Vec<T>`: read

### 4.1 — By index or with `get`

```rust
let v = vec![10, 20, 30];

let a = v[0];                 // 10 — panics if the index does not exist
let b = v.get(0);             // Some(&10)
let c = v.get(10);            // None, no panic

match v.get(5) {
    Some(x) => println!("{x}"),
    None => println!("no element"),
}
let x = v.get(5).copied().unwrap_or(0);
```

| Syntax | Missing index | When to use it |
|---|---|---|
| `v[i]` | panics | the index is guaranteed valid (computed from `len`) |
| `v.get(i)` | `None` | the index comes from outside (user, file, URL) |

### 4.2 — First, last, pieces

```rust
let v = vec![1, 2, 3, 4];
v.first();               // Some(&1)
v.last();                // Some(&4)
v.split_first();         // Some((&1, &[2, 3, 4]))
v.get(1..3);             // Some(&[2, 3])
v.is_empty();
v.len();
```

### 4.3 — Modifying an element

```rust
let mut v = vec![1, 2, 3];
v[0] = 10;

if let Some(x) = v.get_mut(1) {
    *x += 5;
}

if let Some(last) = v.last_mut() {
    *last = 0;
}

v.swap(0, 2);            // swaps two elements
v.fill(7);               // replaces everything
```

### 4.4 — Borrowing and modifying

```rust
let mut v = vec![1, 2, 3];
let first = &v[0];
// v.push(4);            // ERROR: v is borrowed by `first`
println!("{first}");
v.push(4);               // OK: `first` is no longer used
```

`push` may reallocate the memory block, which would make `first` invalid: the compiler therefore forbids modifying the `Vec` while a reference to an element is in use.

## 5. `Vec<T>`: sort, search, deduplicate

### 5.1 — Sorting

```rust
let mut v = vec![3, 1, 2];
v.sort();                                  // [1, 2, 3], stable
v.sort_unstable();                         // faster, does not preserve the order of equal elements
v.reverse();                               // [3, 2, 1]
v.sort_by(|a, b| b.cmp(a));                // descending

struct P { name: String, age: u32 }
let mut people = vec![P { name: "B".into(), age: 30 }, P { name: "A".into(), age: 20 }];
people.sort_by_key(|p| p.age);                          // by age
people.sort_by(|a, b| a.name.cmp(&b.name));             // by name
people.sort_by(|a, b| b.age.cmp(&a.age).then_with(|| a.name.cmp(&b.name))); // age descending then name
people.sort_by_key(|p| std::cmp::Reverse(p.age));       // descending, other syntax

let mut f = vec![2.5, 1.0, 3.2];
f.sort_by(|a, b| a.total_cmp(b));          // floats do not have Ord
```

`sort_by_key` with an expensive key (a `String` created at each comparison): prefer `sort_by_cached_key`, which computes it once per element.

### 5.2 — Searching

```rust
let v = vec![10, 20, 30, 40];

v.contains(&20);                          // true
v.iter().position(|&x| x == 30);          // Some(2)
v.iter().find(|&&x| x > 15);              // Some(&20)
v.iter().any(|&x| x > 35);                // true
v.binary_search(&30);                     // Ok(2) — v must be SORTED
v.binary_search(&25);                     // Err(2): position where to insert it to keep the order
```

`contains` and `position` go through everything (cost proportional to the size). `binary_search` halves the range at each step (1,000,000 elements: 20 comparisons), but requires a sorted `Vec`.

### 5.3 — Deduplicating

```rust
let mut v = vec![1, 1, 2, 3, 3, 3, 1];
v.dedup();                     // [1, 2, 3, 1]: removes CONSECUTIVE duplicates

let mut v = vec![3, 1, 3, 2, 1];
v.sort();
v.dedup();                     // [1, 2, 3]: sort first to deduplicate everything

// Keeping the original order
use std::collections::HashSet;
let v = vec![3, 1, 3, 2, 1];
let mut seen = HashSet::new();
let uniques: Vec<i32> = v.into_iter().filter(|x| seen.insert(*x)).collect();   // [3, 1, 2]
```

## 6. `Vec<T>`: iterate and transform

### 6.1 — The three ways to iterate

```rust
let mut v = vec![1, 2, 3];

for x in &v { }              // &i32: read
for x in &mut v { *x += 1; } // &mut i32: modify
for x in v { }               // i32: consumes v
```

The details of these three forms are in the `structures-et-controle` course.

### 6.2 — Transforming

```rust
let v = vec![1, 2, 3];

let doubled: Vec<i32> = v.iter().map(|x| x * 2).collect();
let texts: Vec<String> = v.iter().map(|x| x.to_string()).collect();
let total: i32 = v.iter().sum();
let joined = v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", ");   // "1, 2, 3"
let (evens, odds): (Vec<i32>, Vec<i32>) = v.iter().copied().partition(|x| x % 2 == 0);
```

Iterators are covered in the `closures-iterateurs` course.

### 6.3 — Flattening and concatenating

```rust
let matrix = vec![vec![1, 2], vec![3]];
let flat: Vec<i32> = matrix.concat();                          // [1, 2, 3]
let flat2: Vec<i32> = matrix.into_iter().flatten().collect();  // same result

let words = vec!["a", "b", "c"];
let sentence = words.join("-");                                // "a-b-c"
```

## 7. Slices and `Vec`

### 7.1 — Accepting a slice

```rust
// Version 1: &Vec<i32> — only accepts a Vec
fn sum_vec(v: &Vec<i32>) -> i32 { v.iter().sum() }

// Version 2: &[i32] — accepts Vec, array, part of either
fn sum(v: &[i32]) -> i32 { v.iter().sum() }

fn main() {
    let v = vec![1, 2, 3];
    let t = [4, 5];
    sum(&v);           // &Vec<i32> becomes &[i32] automatically
    sum(&t);
    sum(&v[1..]);
}
```

`clippy` flags `&Vec<T>` as a parameter (`ptr_arg`): `&[T]` does the same thing while accepting more types. Likewise, prefer `&str` to `&String`.

### 7.2 — Slice methods

All the read, sort and search methods of `Vec` actually come from slices: `sort`, `contains`, `iter`, `first`, `binary_search`, `windows`, `chunks`, `split_at`, `starts_with`, `concat`, `join`... They therefore also work on an array or a piece of a `Vec`.

```rust
let v = [1, 2, 3, 4, 5, 6];
let (left, right) = v.split_at(2);        // [1, 2] and [3, 4, 5, 6]
for pair in v.windows(2) { }              // sliding windows
for batch in v.chunks(4) { }              // [1, 2, 3, 4], [5, 6]
v.starts_with(&[1, 2]);                   // true
v.iter().rev();                           // iterate backwards
```

## 8. `VecDeque<T>`

A double-ended queue: fast add and remove at **both ends**.

```rust
use std::collections::VecDeque;

let mut queue = VecDeque::new();
queue.push_back(1);            // at the end
queue.push_back(2);
queue.push_front(0);           // at the start
let first = queue.pop_front();    // Some(0)
let last = queue.pop_back();      // Some(2)
queue.front();                 // Some(&1)
```

What it is for: a waiting queue (first come, first served). With a `Vec`, removing at the start (`v.remove(0)`) shifts all the elements every time; with `VecDeque`, it is immediate.

```rust
// Task queue limited to the last 3 entries
let mut history = VecDeque::with_capacity(3);
for event in ["a", "b", "c", "d"] {
    if history.len() == 3 {
        history.pop_front();
    }
    history.push_back(event);
}
// ["b", "c", "d"]
```

## 9. `HashMap<K, V>`: create and fill

### 9.1 — Creating

```rust
use std::collections::HashMap;

let mut scores: HashMap<String, u32> = HashMap::new();
let scores2 = HashMap::from([("Alice", 10), ("Bob", 7)]);
let scores3: HashMap<&str, u32> = vec![("Alice", 10), ("Bob", 7)].into_iter().collect();
let names = ["a", "b"];
let lengths: HashMap<&str, usize> = names.iter().map(|n| (*n, n.len())).collect();
let reserved: HashMap<u32, String> = HashMap::with_capacity(1000);
```

### 9.2 — Conditions on the key

The key must implement `Eq` and `Hash`. Consequences:

- `String`, `&str`, integers, `char`, `bool`, tuples and structs deriving `Eq + Hash`: OK;
- `f64`: **impossible** (no `Eq` because of `NaN`); store floats as integers (cents, thousandths) or use a crate such as `ordered-float`.

```rust
#[derive(PartialEq, Eq, Hash)]
struct Coord { x: i32, y: i32 }

let mut grid: HashMap<Coord, char> = HashMap::new();
grid.insert(Coord { x: 0, y: 0 }, '#');
```

### 9.3 — Inserting

```rust
let mut scores = HashMap::new();
let old = scores.insert("Alice", 10);   // None: new key
let old = scores.insert("Alice", 15);   // Some(10): the value is replaced
```

`insert` **always replaces**. To insert only if the key is absent: `entry(...).or_insert(...)` (section 11).

### 9.4 — The order is not guaranteed

The iteration order of a `HashMap` is **random** and can change from one run to the next (the hashing is deliberately unpredictable, which protects against collision attacks). For a stable order: `BTreeMap` (sorted) or sort the keys before displaying.

## 10. `HashMap<K, V>`: read and modify

### 10.1 — Reading

```rust
let mut scores = HashMap::from([("Alice".to_string(), 10)]);

let s = scores.get("Alice");                 // Some(&10) — a &str is enough for a String key
let s = scores.get("Zoe").copied().unwrap_or(0);
let exists = scores.contains_key("Alice");
let s = scores["Alice"];                     // panics if the key is absent
```

`get` accepts a borrowed form of the key: with `String` keys, you search with a `&str`, without creating a `String`.

### 10.2 — Modifying

```rust
if let Some(s) = scores.get_mut("Alice") {
    *s += 5;
}

let removed = scores.remove("Alice");                // Some(15)
let pair = scores.remove_entry("Bob");               // Some(("Bob", v)) or None
scores.retain(|_name, score| *score > 5);            // keeps based on a condition
```

### 10.3 — Iterating

```rust
let scores = HashMap::from([("Alice", 10), ("Bob", 7)]);

for (name, score) in &scores { }         // (&K, &V)
for name in scores.keys() { }
for score in scores.values() { }

let mut scores = scores;
for score in scores.values_mut() {
    *score *= 2;
}
for (name, score) in scores { }          // consumes the map

// In a stable order
let mut pairs: Vec<_> = HashMap::from([("b", 2), ("a", 1)]).into_iter().collect();
pairs.sort();
```

## 11. The `entry` API

`entry(key)` gives access to the slot of a key, **present or not**, in a single lookup.

### 11.1 — Counting: four ways

```rust
use std::collections::HashMap;

let text = "the cat and the dog";

// 1. get then insert: two lookups, longer
let mut c1: HashMap<&str, u32> = HashMap::new();
for word in text.split_whitespace() {
    let n = c1.get(word).copied().unwrap_or(0);
    c1.insert(word, n + 1);
}

// 2. match on get_mut
let mut c2: HashMap<&str, u32> = HashMap::new();
for word in text.split_whitespace() {
    match c2.get_mut(word) {
        Some(n) => *n += 1,
        None => { c2.insert(word, 1); }
    }
}

// 3. entry + or_insert: a single lookup (idiomatic form)
let mut c3: HashMap<&str, u32> = HashMap::new();
for word in text.split_whitespace() {
    *c3.entry(word).or_insert(0) += 1;
}

// 4. entry + or_default
let mut c4: HashMap<&str, u32> = HashMap::new();
for word in text.split_whitespace() {
    *c4.entry(word).or_default() += 1;
}
```

### 11.2 — All the `entry` methods

```rust
let mut m: HashMap<&str, Vec<u32>> = HashMap::new();

m.entry("a").or_insert(Vec::new());          // value created even if the key exists
m.entry("b").or_insert_with(Vec::new);       // created only if absent
m.entry("c").or_default();                   // Default of the type
m.entry("a").or_default().push(1);           // group: returns &mut V

m.entry("a")
    .and_modify(|v| v.push(2))               // if present: modify
    .or_insert_with(|| vec![2]);             // if absent: insert

let key = "d";
m.entry(key).or_insert_with_key(|k| vec![k.len() as u32]);   // the value depends on the key
```

### 11.3 — Grouping

```rust
let words = ["tree", "tent", "boat", "bus", "car"];
let mut by_initial: HashMap<char, Vec<&str>> = HashMap::new();
for word in words {
    let initial = word.chars().next().unwrap();
    by_initial.entry(initial).or_default().push(word);
}
// {'t': ["tree", "tent"], 'b': ["boat", "bus"], 'c': ["car"]}
```

### 11.4 — Matching on the entry

```rust
use std::collections::hash_map::Entry;

let mut stock: HashMap<&str, u32> = HashMap::new();
match stock.entry("apple") {
    Entry::Occupied(mut e) => { *e.get_mut() += 1; }
    Entry::Vacant(e) => { e.insert(1); }
}
```

What it is for: when the two cases need different and lengthy handling, or to remove the entry (`e.remove()`) depending on its value.

## 12. `BTreeMap<K, V>`

Same API as a `HashMap`, but the keys stay **sorted**. The key must implement `Ord` (not `Hash`).

```rust
use std::collections::BTreeMap;

let mut grades = BTreeMap::new();
grades.insert("Charlie", 12);
grades.insert("Alice", 15);
grades.insert("Bob", 9);

for (name, grade) in &grades {
    println!("{name}: {grade}");         // Alice, Bob, Charlie: always in order
}

grades.first_key_value();                // Some((&"Alice", &15))
grades.last_key_value();                 // Some((&"Charlie", &12))

let mut events = BTreeMap::new();
events.insert(1990, "a");
events.insert(2005, "b");
events.insert(2020, "c");
for (year, e) in events.range(2000..2021) {
    println!("{year} {e}");              // 2005, 2020
}
```

| | `HashMap` | `BTreeMap` |
|---|---|---|
| Lookup, insertion | very fast (constant time on average) | fast (logarithmic) |
| Iteration | random order | sorted |
| Ranges (`range`), min, max | no | yes |
| Key | `Eq + Hash` | `Ord` |

Choose `BTreeMap` when order matters (display, stable output for tests, export) or for range queries.

## 13. `HashSet<T>` and `BTreeSet<T>`

A set: **unique** values, with no associated value.

### 13.1 — Creating and adding

```rust
use std::collections::HashSet;

let mut s = HashSet::new();
let is_new = s.insert("rust");       // true
let is_new = s.insert("rust");       // false: already present, nothing changes
s.remove("rust");

let s2: HashSet<i32> = [1, 2, 2, 3].into_iter().collect();   // {1, 2, 3}
let s3 = HashSet::from([1, 2, 3]);
```

`insert` returns `false` for a duplicate: handy for detecting duplicates along the way.

### 13.2 — Membership: `Vec` or `HashSet`

```rust
let forbidden_vec = vec!["admin", "root", "system"];
let forbidden_set: HashSet<&str> = forbidden_vec.iter().copied().collect();

let name = "root";
forbidden_vec.contains(&name);     // goes through the whole Vec
forbidden_set.contains(name);      // one hash lookup
```

For a few elements, there is no difference. For thousands of checks on thousands of elements, the `HashSet` is vastly faster.

### 13.3 — Set operations

```rust
let a: HashSet<i32> = [1, 2, 3, 4].into();
let b: HashSet<i32> = [3, 4, 5].into();

let union: HashSet<_> = a.union(&b).copied().collect();                 // {1, 2, 3, 4, 5}
let inter: HashSet<_> = a.intersection(&b).copied().collect();          // {3, 4}
let diff: HashSet<_> = a.difference(&b).copied().collect();             // {1, 2}
let sym: HashSet<_> = a.symmetric_difference(&b).copied().collect();    // {1, 2, 5}

let inter2 = &a & &b;      // same operations with operators
let union2 = &a | &b;
let diff2 = &a - &b;

a.is_subset(&b);           // false
a.is_disjoint(&b);         // false
```

### 13.4 — `BTreeSet`

```rust
use std::collections::BTreeSet;

let s: BTreeSet<i32> = [5, 1, 3].into();
for x in &s { }            // 1, 3, 5
s.first();                 // Some(&1)
s.range(2..).next();       // Some(&3): smallest value >= 2
```

## 14. `BinaryHeap<T>`

A priority queue: `pop` always returns the **largest** element.

```rust
use std::collections::BinaryHeap;
use std::cmp::Reverse;

let mut heap = BinaryHeap::new();
heap.push(3);
heap.push(10);
heap.push(1);
heap.pop();            // Some(10)
heap.peek();           // Some(&3)

// The SMALLEST first: wrap in Reverse
let mut min_heap = BinaryHeap::new();
min_heap.push(Reverse(3));
min_heap.push(Reverse(1));
let Some(Reverse(smallest)) = min_heap.pop() else { return };   // 1

// Priority + data: the tuple is compared element by element
let mut tasks = BinaryHeap::new();
tasks.push((2, "medium"));
tasks.push((9, "urgent"));
tasks.pop();           // Some((9, "urgent"))
```

What it is for: processing tasks by priority, keeping the N largest values of a stream, Dijkstra's algorithm.

## 15. Ownership and collections

### 15.1 — Inserting moves the value

```rust
let name = String::from("Alice");
let mut names = Vec::new();
names.push(name);
// println!("{name}");    // ERROR: name was moved into the Vec

let key = String::from("k");
let mut m = std::collections::HashMap::new();
m.insert(key, 1);
// println!("{key}");     // ERROR too
```

Three solutions depending on the need:

```rust
let name = String::from("Alice");
let mut a = Vec::new();
a.push(name.clone());     // 1. copy: name stays usable

let mut b: Vec<&str> = Vec::new();
b.push(&name);            // 2. store a reference: b must not outlive name

let mut c = Vec::new();
c.push(name);             // 3. move, and access the value through the collection afterwards
println!("{}", c[0]);
```

### 15.2 — Taking a value out of a collection

```rust
let mut v = vec![String::from("a"), String::from("b")];

// let s = v[0];                    // ERROR: cannot move out of index
let s = &v[0];                      // borrow
let s = v[0].clone();               // copy
let s = v.remove(0);                // remove (and shift)
let s = std::mem::take(&mut v[0]);  // take the value, leave String::new() in its place
let s = std::mem::replace(&mut v[0], String::from("z"));   // swap
```

## 16. Choosing the right collection

| Main need | Collection |
|---|---|
| List, insertion order, access by position | `Vec` |
| Queue, add and remove at both ends | `VecDeque` |
| Find a value by a key | `HashMap` |
| Same, with sorted iteration or key ranges | `BTreeMap` |
| Know whether a value is present, deduplicate | `HashSet` |
| Sorted set | `BTreeSet` |
| Always take the highest priority | `BinaryHeap` |

### Cost of operations

| Operation | `Vec` | `VecDeque` | `HashMap`/`HashSet` | `BTreeMap`/`BTreeSet` |
|---|---|---|---|---|
| Access by index | immediate | immediate | — | — |
| Add at the end | immediate (amortized) | immediate | — | — |
| Add at the start | proportional to n | immediate | — | — |
| Search for a value | proportional to n | proportional to n | immediate on average | logarithmic |
| Insert by key | — | — | immediate on average | logarithmic |
| Sorted iteration | sort first | sort first | sort first | free |

When in doubt, start with a `Vec`: it is the simplest and, for small sizes, often the fastest (contiguous memory).

## 17. The same task written in several ways

### 17.1 — Statistics on a `Vec`

```rust
fn stats_loop(v: &[f64]) -> Option<(f64, f64, f64)> {
    if v.is_empty() {
        return None;
    }
    let mut min = v[0];
    let mut max = v[0];
    let mut total = 0.0;
    for &x in v {
        if x < min { min = x; }
        if x > max { max = x; }
        total += x;
    }
    Some((min, max, total / v.len() as f64))
}

fn stats_iter(v: &[f64]) -> Option<(f64, f64, f64)> {
    let first = *v.first()?;
    let min = v.iter().copied().fold(first, f64::min);
    let max = v.iter().copied().fold(first, f64::max);
    let average = v.iter().sum::<f64>() / v.len() as f64;
    Some((min, max, average))
}
```

`f64` does not have `Ord`, so there is no `.min()` on the iterator: use `fold` with `f64::min`.

### 17.2 — Removing duplicates

```rust
use std::collections::{BTreeSet, HashSet};

let v = vec![3, 1, 3, 2, 1];

// 1. Sort + dedup: sorted order
let mut a = v.clone();
a.sort();
a.dedup();                                             // [1, 2, 3]

// 2. Go through a BTreeSet: sorted order
let b: Vec<i32> = v.iter().copied().collect::<BTreeSet<_>>().into_iter().collect();

// 3. HashSet as a filter: original order kept
let mut seen = HashSet::new();
let c: Vec<i32> = v.iter().copied().filter(|x| seen.insert(*x)).collect();   // [3, 1, 2]
```

### 17.3 — The most frequent word

```rust
use std::collections::HashMap;

fn most_frequent(text: &str) -> Option<(&str, u32)> {
    let mut count: HashMap<&str, u32> = HashMap::new();
    for word in text.split_whitespace() {
        *count.entry(word).or_default() += 1;
    }
    count.into_iter().max_by_key(|&(_, n)| n)
}
```

In case of a tie, the result depends on the order of the `HashMap`, so it can change from one run to the next. For a stable result, use a `BTreeMap` or break the tie on the word: `max_by(|a, b| a.1.cmp(&b.1).then(b.0.cmp(a.0)))`.

## 18. Exercises

### Exercise 1 — statistics

Write `average`, `median` and `mode` (most frequent value) for a `&[i32]`, returning `None` on an empty list.

```rust
// Solution:
use std::collections::HashMap;

fn average(v: &[i32]) -> Option<f64> {
    if v.is_empty() {
        return None;
    }
    Some(v.iter().sum::<i32>() as f64 / v.len() as f64)
}

fn median(v: &[i32]) -> Option<f64> {
    if v.is_empty() {
        return None;
    }
    let mut sorted = v.to_vec();
    sorted.sort();
    let m = sorted.len() / 2;
    Some(if sorted.len() % 2 == 0 {
        (sorted[m - 1] + sorted[m]) as f64 / 2.0
    } else {
        sorted[m] as f64
    })
}

fn mode(v: &[i32]) -> Option<i32> {
    let mut count = HashMap::new();
    for &x in v {
        *count.entry(x).or_insert(0) += 1;
    }
    count.into_iter().max_by_key(|&(_, n)| n).map(|(x, _)| x)
}
```

### Exercise 2 — anagrams

Group words by anagrams (`["listen", "silent", "cat"]` → `[["listen", "silent"], ["cat"]]`).

```rust
// Solution:
use std::collections::HashMap;

fn anagrams<'a>(words: &[&'a str]) -> Vec<Vec<&'a str>> {
    let mut groups: HashMap<Vec<char>, Vec<&str>> = HashMap::new();
    for &word in words {
        let mut key: Vec<char> = word.chars().collect();
        key.sort_unstable();
        groups.entry(key).or_default().push(word);
    }
    groups.into_values().collect()
}
```

### Exercise 3 — intersection of lists

Return the elements common to two `Vec<String>`, without duplicates and sorted.

```rust
// Solution:
use std::collections::BTreeSet;

fn common(a: &[String], b: &[String]) -> Vec<String> {
    let a: BTreeSet<&String> = a.iter().collect();
    let b: BTreeSet<&String> = b.iter().collect();
    a.intersection(&b).map(|s| (*s).clone()).collect()
}
```

## 19. Cheat sheet

### Vec

```rust
vec![1, 2]   Vec::with_capacity(n)   iter.collect::<Vec<_>>()
v.push(x)   v.pop()   v.insert(i, x)   v.remove(i)   v.swap_remove(i)
v.extend(it)   v.append(&mut w)   v.truncate(n)   v.clear()   v.drain(a..b)
v.retain(|x| ..)   v.dedup()   v.sort()   v.sort_by_key(|x| ..)   v.reverse()
v[i]   v.get(i)   v.first()   v.last()   v.get_mut(i)   v.contains(&x)
v.binary_search(&x)   v.iter().position(|x| ..)   v.windows(2)   v.chunks(n)
```

### HashMap

```rust
HashMap::from([(k, v)])   m.insert(k, v)   m.get(&k)   m.get_mut(&k)
m.remove(&k)   m.contains_key(&k)   m.keys()   m.values()   m.values_mut()
*m.entry(k).or_insert(0) += 1
m.entry(k).or_default().push(x)
m.entry(k).and_modify(|v| ..).or_insert(v)
```

### HashSet

```rust
s.insert(x)  // false if already present
s.contains(&x)   a.union(&b)   a.intersection(&b)   a.difference(&b)
&a | &b   &a & &b   &a - &b
```

### Key points

- `Vec` is the default collection; use `with_capacity` when the size is known.
- `v[i]` panics, `v.get(i)` returns `None`: use `get` for any index coming from outside.
- `remove` shifts, `swap_remove` does not; to remove at the start of a queue: `VecDeque`.
- `HashMap` guarantees no order; `BTreeMap` is sorted.
- `entry` does a single lookup to read and write.
- `HashMap` keys must be `Eq + Hash`: no `f64`.
- Inserting into a collection moves the value.
- As a parameter, `&[T]` rather than `&Vec<T>`.
