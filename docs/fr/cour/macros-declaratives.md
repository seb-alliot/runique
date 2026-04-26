# Macros déclaratives — macro_rules!
> Écrire du code qui génère du code, sans coût à l'exécution

## Objectifs

- Comprendre le rôle des macros déclaratives en Rust
- Maîtriser la syntaxe `macro_rules!` et ses fragments
- Écrire des macros avec répétitions et plusieurs patterns
- Éviter les pièges classiques

---

## Table des matières

1. [Pourquoi les macros ?](#1-pourquoi-les-macros-)
2. [Syntaxe de base](#2-syntaxe-de-base)
3. [Fragments](#3-fragments)
4. [Répétitions](#4-repetitions)
5. [Exemples simples](#5-exemples-simples)
6. [Exemples intermédiaires](#6-exemples-intermediaires)
7. [Bonnes pratiques](#7-bonnes-pratiques)

---

## 1. Pourquoi les macros ?

Les macros déclaratives permettent ce que les fonctions ne peuvent pas faire :

- **Nombre variable d'arguments** : `println!("val: {}", x)` ou `println!("{} {}", a, b)`
- **Générer du code répétitif** : déclarer plusieurs méthodes d'un coup
- **DSL** : syntaxe personnalisée qui ressemble à du Rust mais n'en est pas

> **Important :** Quand une fonction normale peut faire le travail, utilise une fonction. Les macros sont plus difficiles à déboguer.

---

## 2. Syntaxe de base

```rust
macro_rules! nom_macro {
    (motif) => {
        // Code généré
    };
    (autre_motif) => {
        // Autre branche
    };
}
```

Chaque branche est un `(motif) => { expansion }`. Rust essaie les branches dans l'ordre et utilise la première qui correspond.

---

## 3. Fragments

Les fragments définissent ce que chaque variable de pattern peut capturer :

| Désignateur | Description | Exemple |
|---|---|---|
| `$x:expr` | Expression | `2 + 2`, `x.foo()` |
| `$x:ident` | Identificateur | `nom_variable`, `foo` |
| `$x:ty` | Type | `u32`, `Vec<String>` |
| `$x:pat` | Pattern | `Some(x)`, `(a, b)` |
| `$x:stmt` | Statement | `let x = 5;` |
| `$x:block` | Bloc de code | `{ ... }` |
| `$x:item` | Item | `struct`, `fn`, `impl` |
| `$x:tt` | Token tree | N'importe quel token |
| `$x:literal` | Littéral | `42`, `"texte"` |

---

## 4. Répétitions

```rust
$(...)*   // Zéro ou plusieurs
$(...)+   // Une ou plusieurs
$(...)?   // Zéro ou une
```

### Comment ça marche

```rust
macro_rules! afficher_tout {
    ($($val:expr),*) => {
        $(
            println!("{:?}", $val);
        )*
    };
}

fn main() {
    afficher_tout!(1, "hello", true, 3.14);
}
```

Le `$($val:expr),*` capture zéro ou plusieurs expressions séparées par des virgules. Le `$( ... )*` dans l'expansion répète le bloc pour chaque valeur capturée.

---

## 5. Exemples simples

### Macro avec un message préfixé

```rust
macro_rules! info {

    ($msg:expr) => {
        println!("[INFO] {}", $msg);
    };

}

fn main() {
    info!("Démarrage");
    info!("Connexion établie");
}
```

### Plusieurs patterns — macro de log

```rust
macro_rules! log {

    (info $msg:expr)  => { println!("[INFO]  {}", $msg); };

    (warn $msg:expr)  => { println!("[WARN]  {}", $msg); };

    (error $msg:expr) => { eprintln!("[ERROR] {}", $msg); };

}

fn main() {
    log!(info  "Démarrage");
    log!(warn  "Mémoire faible");
    log!(error "Échec de connexion");
}
```

### Calcul à la compilation — le piège des parenthèses

```rust
// ❌ Sans parenthèses — bug subtil
macro_rules! carre_mauvais {
    ($n:expr) => { $n * $n };
}

// ✅ Avec parenthèses — correct
macro_rules! carre {
    ($n:expr) => { ($n) * ($n) };
}

fn main() {
    let a = carre_mauvais!(2 + 3); // Devient 2 + 3 * 2 + 3 = 11, pas 25!
    let b = carre!(2 + 3);         // Devient (2 + 3) * (2 + 3) = 25 ✅
}
```

> **Règle :** Toujours entourer les paramètres `$x:expr` de parenthèses dans l'expansion.

---

## 6. Exemples intermédiaires

### Créer un HashMap avec une syntaxe facilitée

```rust
use std::collections::HashMap;

macro_rules! hashmap {

    () => { HashMap::new() };

    ($($key:expr => $val:expr),+ $(,)?) => {{
        let mut map = HashMap::new();

        $(
            map.insert($key, $val);
        )+

        map
    }};

}

fn main() {
    let scores = hashmap! {
        "Alice" => 100,
        "Bob"   => 85,
    };

    println!("{:?}", scores);
}
```

Le `$(,)?` à la fin accepte une virgule trailing facultative — comme Rust le fait partout.

### Recréer vec!

```rust
macro_rules! mon_vec {

    () => {
        Vec::new()
    };

    ($($e:expr),+ $(,)?) => {{
        let mut v = Vec::new();
        $(v.push($e);)+
        v
    }};

    ($e:expr; $n:expr) => {{
        let mut v = Vec::with_capacity($n);
        for _ in 0..$n { v.push($e); }
        v
    }};

}

fn main() {
    let a = mon_vec![];
    let b = mon_vec![1, 2, 3];
    let c = mon_vec![0; 10];
}
```

### Générateur de getters

```rust
macro_rules! create_getter {
    ($nom:ident, $type:ty, $field:ident) => {
        pub fn $nom(&self) -> &$type {
            &self.$field
        }
    };
}

struct Personne {
    nom:   String,
    age:   u32,
}

impl Personne {
    create_getter!(get_nom, String, nom);
    create_getter!(get_age, u32,    age);
}
```

---

## 7. Bonnes pratiques

### Évaluation multiple — le piège

```rust
// ❌ $a et $b sont évalués deux fois
macro_rules! max_mauvais {
    ($a:expr, $b:expr) => {
        if $a > $b { $a } else { $b }
    };
}

// ✅ Évaluation une seule fois
macro_rules! max_bon {
    ($a:expr, $b:expr) => {{
        let a = $a;
        let b = $b;

        if a > b { a } else { b }
    }};
}
```

Si `$a` est un appel de fonction avec effet de bord, `max_mauvais!` l'appelle deux fois. Toujours lier les arguments à des variables locales.

### Hygiène des variables

```rust
macro_rules! avec_temp {
    ($val:expr) => {{
        let temp = $val;  // 'temp' est locale à la macro, pas de conflit
        println!("temp = {:?}", temp);
    }};
}

fn main() {
    let temp = "extérieur";
    avec_temp!(42);
    println!("temp = {}", temp); // Toujours "extérieur"
}
```

Rust isole les variables introduites par les macros — elles n'entrent pas en conflit avec les variables du code appelant.

### Déboguer avec cargo-expand

```bash
cargo install cargo-expand
cargo expand         # Affiche tout le code après expansion des macros
cargo expand main    # Seulement la fonction main
```

C'est l'outil indispensable pour comprendre ce que ta macro génère réellement.
