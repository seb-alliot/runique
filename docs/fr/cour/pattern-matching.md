# Pattern Matching
> match, if let, while let, destructuring — maîtriser la correspondance de motifs en Rust

## Objectifs

- Maîtriser l'expression `match` et ses motifs
- Utiliser `if let` et `while let`
- Destructurer structs, enums, tuples et slices
- Appliquer les gardes (`if` dans `match`)
- Connaître les patterns avancés (`@`, `..`, `|`)

---

## Table des matières

1. [L'expression match](#1-lexpression-match)
2. [Motifs courants](#2-motifs-courants)
   - 2.1 [Littéraux et plages](#21-littéraux-et-plages)
   - 2.2 [Destructurer les enums](#22-destructurer-les-enums)
   - 2.3 [Destructurer les structs](#23-destructurer-les-structs)
   - 2.4 [Destructurer les tuples](#24-destructurer-les-tuples)
   - 2.5 [Slices](#25-slices)
3. [Gardes de motif](#3-gardes-de-motif)
4. [if let et while let](#4-if-let-et-while-let)
5. [Patterns avancés](#5-patterns-avancés)

---

## 1. L'expression match

`match` compare une valeur contre une liste de motifs et exécute la branche correspondante.

```rust
let nombre = 7;

match nombre {
    1       => println!("un"),
    2 | 3   => println!("deux ou trois"),
    4..=6   => println!("entre 4 et 6"),
    n       => println!("autre : {n}"),  // variable de capture
}
```

> **Important :** `match` est **exhaustif** — toutes les variantes doivent être couvertes. Le compilateur l'impose.

```rust
// Utiliser _ pour ignorer les cas restants
match nombre {
    1 => println!("un"),
    _ => println!("autre"),
}
```

---

## 2. Motifs courants

### 2.1 Littéraux et plages

```rust
let c = 'a';

match c {
    'a'..='z' => println!("minuscule"),
    'A'..='Z' => println!("majuscule"),
    '0'..='9' => println!("chiffre"),
    _         => println!("autre caractère"),
}
```

### 2.2 Destructurer les enums

```rust
enum Message {
    Quitter,
    Deplacer { x: i32, y: i32 },
    Ecrire(String),
    ChangerCouleur(u8, u8, u8),
}

let msg = Message::Deplacer { x: 10, y: 20 };

match msg {
    Message::Quitter => println!("quitter"),
    Message::Deplacer { x, y } => println!("déplacer vers {x},{y}"),
    Message::Ecrire(texte) => println!("écrire : {texte}"),
    Message::ChangerCouleur(r, g, b) => println!("couleur : {r},{g},{b}"),
}
```

### 2.3 Destructurer les structs

```rust
struct Point { x: i32, y: i32 }

let p = Point { x: 3, y: 7 };

// Destructuration complète
let Point { x, y } = p;
println!("{x}, {y}");

// Dans un match
match p {
    Point { x: 0, y } => println!("sur l'axe Y à {y}"),
    Point { x, y: 0 } => println!("sur l'axe X à {x}"),
    Point { x, y }    => println!("({x}, {y})"),
}
```

### 2.4 Destructurer les tuples

```rust
let tuple = (1, true, "bonjour");

match tuple {
    (1, true, msg) => println!("un, vrai, {msg}"),
    (n, false, _)  => println!("n={n}, faux"),
    _              => println!("autre"),
}

// Destructuration directe
let (a, b, c) = tuple;
```

### 2.5 Slices

```rust
let nombres = vec![1, 2, 3, 4, 5];

match nombres.as_slice() {
    []         => println!("vide"),
    [seul]     => println!("un seul : {seul}"),
    [premier, .., dernier] => println!("de {premier} à {dernier}"),
}
```

---

## 3. Gardes de motif

Une garde (`if condition`) ajoute un test supplémentaire après le motif.

```rust
let pair = (2, -3);

match pair {
    (x, y) if x == y       => println!("égaux"),
    (x, y) if x + y == 0   => println!("opposés"),
    (x, _) if x % 2 == 0   => println!("x est pair"),
    _                       => println!("autre"),
}
```

> **Attention :** La garde ne participe pas à l'exhaustivité — le compilateur ne peut pas vérifier les conditions arbitraires.

---

## 4. if let et while let

### if let — pattern unique sans exhaustivité

```rust
let valeur: Option<i32> = Some(42);

// Verbeux avec match
match valeur {
    Some(n) => println!("got {n}"),
    None    => {},
}

// Concis avec if let
if let Some(n) = valeur {
    println!("got {n}");
}

// Avec else
if let Some(n) = valeur {
    println!("got {n}");
} else {
    println!("rien");
}
```

### while let — boucle tant que le motif correspond

```rust
let mut pile = vec![1, 2, 3];

while let Some(sommet) = pile.pop() {
    println!("{sommet}");
}
// Affiche : 3, 2, 1
```

---

## 5. Patterns avancés

### `@` — capturer et tester

```rust
let n = 15;

match n {
    x @ 1..=12 => println!("mois {x}"),
    x @ 13..=19 => println!("ado {x}"),
    _ => println!("autre"),
}
```

### `..` — ignorer les champs restants

```rust
struct Point3D { x: i32, y: i32, z: i32 }

let p = Point3D { x: 1, y: 2, z: 3 };

match p {
    Point3D { x, .. } => println!("x = {x}"),  // y et z ignorés
}

// Dans un tuple
let tuple = (1, 2, 3, 4, 5);
let (premier, .., dernier) = tuple;
```

### `|` — plusieurs motifs

```rust
match valeur {
    1 | 2 | 3 => println!("petit"),
    4 | 5 | 6 => println!("moyen"),
    _          => println!("grand"),
}
```

### `ref` et `ref mut` — emprunter dans un motif

```rust
let texte = String::from("bonjour");

match texte {
    ref s => println!("longueur : {}", s.len()),  // s est &String
}

// texte est toujours valide ici
println!("{texte}");
```

### let-else (Rust 1.65+)

```rust
fn parse_id(s: &str) -> u32 {
    let Ok(id) = s.parse::<u32>() else {
        panic!("id invalide : {s}");
    };
    id
}
```
