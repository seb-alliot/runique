# Closures & Itérateurs
> Fonctions anonymes, traits Fn, chaînes d'itérateurs — le style fonctionnel en Rust

## Objectifs

- Écrire et utiliser des closures
- Comprendre les traits `Fn`, `FnMut`, `FnOnce`
- Maîtriser l'`Iterator` trait et ses méthodes
- Chaîner les itérateurs efficacement
- Créer ses propres itérateurs

---

## Table des matières

1. [Les closures](#1-les-closures)
   - 1.1 [Syntaxe](#11-syntaxe)
   - 1.2 [Capture de l'environnement](#12-capture-de-lenvironnement)
   - 1.3 [Traits Fn, FnMut, FnOnce](#13-traits-fn-fnmut-fnonce)
   - 1.4 [Retourner une closure](#14-retourner-une-closure)
2. [L'Iterator trait](#2-literator-trait)
3. [Méthodes d'itérateurs](#3-méthodes-ditérateurs)
   - 3.1 [Transformation](#31-transformation)
   - 3.2 [Filtrage](#32-filtrage)
   - 3.3 [Réduction](#33-réduction)
   - 3.4 [Collecte](#34-collecte)
4. [Créer un itérateur](#4-créer-un-itérateur)

---

## 1. Les closures

### 1.1 Syntaxe

```rust
// Fonction classique
fn ajouter(x: i32, y: i32) -> i32 { x + y }

// Closure équivalente
let ajouter = |x: i32, y: i32| -> i32 { x + y };

// Types inférés, corps implicite (expression)
let ajouter = |x, y| x + y;

// Sans paramètres
let dire_bonjour = || println!("Bonjour !");

// Corps multi-lignes
let traiter = |x: i32| {
    let double = x * 2;
    double + 1
};
```

### 1.2 Capture de l'environnement

```rust
let seuil = 5;

// Capture par référence (par défaut)
let est_grand = |x| x > seuil;
println!("{}", est_grand(10)); // true
println!("{seuil}");           // seuil toujours accessible

// Capture par valeur avec `move`
let texte = String::from("bonjour");
let afficher = move || println!("{texte}");
afficher();
// println!("{texte}"); // ERREUR : texte a été moved
```

> **Règle :** Utilisez `move` quand la closure doit vivre plus longtemps que son environnement (ex : threads, `async`).

### 1.3 Traits Fn, FnMut, FnOnce

| Trait | Capture | Peut être appelée |
|---|---|---|
| `FnOnce` | par valeur (move) | une seule fois |
| `FnMut` | par référence mutable | plusieurs fois (modifie env) |
| `Fn` | par référence partagée | plusieurs fois (lecture seule) |

```rust
// FnOnce — consomme une valeur capturée
let texte = String::from("adieu");
let consommer = move || drop(texte);
consommer();
// consommer(); // ERREUR : ne peut être appelée qu'une fois

// FnMut — modifie l'environnement
let mut compteur = 0;
let mut incrementer = || { compteur += 1; compteur };
println!("{}", incrementer()); // 1
println!("{}", incrementer()); // 2

// Fn — lit seulement
let nom = String::from("Rust");
let saluer = || println!("Bonjour {nom}");
saluer();
saluer(); // OK, Fn peut être appelée plusieurs fois
```

### 1.4 Retourner une closure

```rust
// Avec Box<dyn Fn>
fn multiplicateur(n: i32) -> Box<dyn Fn(i32) -> i32> {
    Box::new(move |x| x * n)
}

let triple = multiplicateur(3);
println!("{}", triple(5)); // 15

// Avec impl Fn (plus idiomatique)
fn multiplicateur(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x * n
}
```

---

## 2. L'Iterator trait

```rust
pub trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
    // + des dizaines de méthodes par défaut
}
```

**Créer un itérateur :**

```rust
let v = vec![1, 2, 3];

let iter = v.iter();           // itère sur &T
let iter = v.iter_mut();       // itère sur &mut T
let iter = v.into_iter();      // itère sur T (consomme v)

// Plages
let plage = 1..=5;             // 1, 2, 3, 4, 5
let plage = (0..10).step_by(2); // 0, 2, 4, 6, 8
```

---

## 3. Méthodes d'itérateurs

### 3.1 Transformation

```rust
let nombres = vec![1, 2, 3, 4, 5];

// map — transforme chaque élément
let doubles: Vec<i32> = nombres.iter().map(|x| x * 2).collect();
// [2, 4, 6, 8, 10]

// flat_map — map + aplatit
let mots = vec!["bonjour monde", "rust langage"];
let lettres: Vec<&str> = mots.iter()
    .flat_map(|s| s.split_whitespace())
    .collect();
// ["bonjour", "monde", "rust", "langage"]

// enumerate — ajoute l'index
for (i, val) in nombres.iter().enumerate() {
    println!("{i}: {val}");
}

// zip — combine deux itérateurs
let lettres = vec!['a', 'b', 'c'];
let paires: Vec<_> = nombres.iter().zip(lettres.iter()).collect();
// [(1, 'a'), (2, 'b'), (3, 'c')]
```

### 3.2 Filtrage

```rust
// filter — garde les éléments qui satisfont la condition
let pairs: Vec<i32> = nombres.iter()
    .filter(|&&x| x % 2 == 0)
    .copied()
    .collect();
// [2, 4]

// filter_map — filtre et transforme en même temps
let strings = vec!["1", "deux", "3", "quatre"];
let entiers: Vec<i32> = strings.iter()
    .filter_map(|s| s.parse().ok())
    .collect();
// [1, 3]

// take / skip
let premiers_trois: Vec<_> = nombres.iter().take(3).collect();
let sans_deux: Vec<_>      = nombres.iter().skip(2).collect();

// take_while / skip_while
let avant_grand: Vec<_> = nombres.iter()
    .take_while(|&&x| x < 4)
    .collect();
// [1, 2, 3]
```

### 3.3 Réduction

```rust
// fold — accumulateur général
let somme = nombres.iter().fold(0, |acc, &x| acc + x); // 15

// sum et product
let somme: i32   = nombres.iter().sum();     // 15
let produit: i32 = nombres.iter().product(); // 120

// count
let n = nombres.iter().filter(|&&x| x > 2).count(); // 3

// min / max / min_by / max_by
let min = nombres.iter().min(); // Some(1)
let max = nombres.iter().max(); // Some(5)

// any / all
let a_pair  = nombres.iter().any(|&x| x % 2 == 0); // true
let tous_pos = nombres.iter().all(|&x| x > 0);     // true

// find / position
let premier_pair = nombres.iter().find(|&&x| x % 2 == 0); // Some(2)
let pos          = nombres.iter().position(|&x| x == 3);   // Some(2)
```

### 3.4 Collecte

```rust
// Vec
let v: Vec<i32> = (1..=5).collect();

// HashSet
use std::collections::HashSet;
let set: HashSet<i32> = vec![1, 2, 2, 3].into_iter().collect();

// HashMap
use std::collections::HashMap;
let map: HashMap<&str, i32> = vec![("un", 1), ("deux", 2)]
    .into_iter()
    .collect();

// String
let s: String = vec!['R', 'u', 's', 't'].into_iter().collect();
```

---

## 4. Créer un itérateur

```rust
struct Compte {
    valeur: u32,
    max: u32,
}

impl Compte {
    fn new(max: u32) -> Self {
        Compte { valeur: 0, max }
    }
}

impl Iterator for Compte {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        if self.valeur < self.max {
            self.valeur += 1;
            Some(self.valeur)
        } else {
            None
        }
    }
}

// Utilisation
let somme: u32 = Compte::new(5)
    .zip(Compte::new(5).skip(1))
    .map(|(a, b)| a * b)
    .filter(|x| x % 3 == 0)
    .sum();
// Compte a toutes les méthodes d'Iterator gratuitement !
```
