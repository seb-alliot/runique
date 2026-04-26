# Generics
> Types paramétriques, where clauses, monomorphisation — écrire du code réutilisable sans coût runtime

## Objectifs

- Écrire des fonctions et structs génériques
- Utiliser les trait bounds (`T: Trait`)
- Maîtriser les clauses `where`
- Comprendre la monomorphisation
- Distinguer génériques et `dyn Trait`

---

## Table des matières

1. [Fonctions génériques](#1-fonctions-génériques)
2. [Structs et enums génériques](#2-structs-et-enums-génériques)
3. [Trait bounds](#3-trait-bounds)
   - 3.1 [Syntaxe inline](#31-syntaxe-inline)
   - 3.2 [Clauses where](#32-clauses-where)
   - 3.3 [Plusieurs bounds](#33-plusieurs-bounds)
4. [Implémentations conditionnelles](#4-implémentations-conditionnelles)
5. [Generics dans les traits](#5-generics-dans-les-traits)
6. [Monomorphisation](#6-monomorphisation)

---

## 1. Fonctions génériques

```rust
// Sans génériques — dupliqué
fn plus_grand_i32(a: i32, b: i32) -> i32 {
    if a > b { a } else { b }
}
fn plus_grand_f64(a: f64, b: f64) -> f64 {
    if a > b { a } else { b }
}

// Avec génériques — un seul code
fn plus_grand<T: PartialOrd>(a: T, b: T) -> T {
    if a > b { a } else { b }
}

println!("{}", plus_grand(5, 10));       // 10
println!("{}", plus_grand(3.14, 2.71)); // 3.14
println!("{}", plus_grand('a', 'z'));   // z
```

---

## 2. Structs et enums génériques

```rust
// Struct générique
struct Paire<T> {
    premier: T,
    second: T,
}

impl<T> Paire<T> {
    fn new(premier: T, second: T) -> Self {
        Paire { premier, second }
    }
}

// Plusieurs paramètres de type
struct Couple<T, U> {
    gauche: T,
    droite: U,
}

let c = Couple { gauche: 42, droite: "bonjour" };

// Enums génériques — tu les connais déjà !
enum Option<T> {
    Some(T),
    None,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

---

## 3. Trait bounds

### 3.1 Syntaxe inline

```rust
// T doit implémenter Display
fn afficher<T: std::fmt::Display>(valeur: T) {
    println!("{valeur}");
}

// T doit implémenter Display + Debug
fn afficher_debug<T: std::fmt::Display + std::fmt::Debug>(val: T) {
    println!("Display: {val}  Debug: {val:?}");
}
```

### 3.2 Clauses where

La clause `where` rend le code plus lisible quand les bounds sont nombreux.

```rust
// Inline — difficile à lire
fn comparer<T: PartialOrd + std::fmt::Display, U: std::fmt::Debug>(a: T, b: T, extra: U) -> bool {
    println!("extra: {extra:?}");
    a > b
}

// Avec where — beaucoup plus clair
fn comparer<T, U>(a: T, b: T, extra: U) -> bool
where
    T: PartialOrd + std::fmt::Display,
    U: std::fmt::Debug,
{
    println!("extra: {extra:?}");
    a > b
}
```

### 3.3 Plusieurs bounds

```rust
use std::fmt::{Display, Debug};

fn traiter<T>(valeur: T)
where
    T: Display + Debug + Clone + PartialEq,
{
    let copie = valeur.clone();
    println!("Display: {valeur}");
    println!("Debug:   {valeur:?}");
    println!("Égaux:   {}", valeur == copie);
}
```

---

## 4. Implémentations conditionnelles

```rust
use std::fmt::Display;

struct Wrapper<T>(T);

// Méthode disponible pour tous les T
impl<T> Wrapper<T> {
    fn valeur(&self) -> &T {
        &self.0
    }
}

// Méthode disponible uniquement si T: Display
impl<T: Display> Wrapper<T> {
    fn afficher(&self) {
        println!("{}", self.0);
    }
}

// Implémenter un trait conditionnellement (blanket impl)
impl<T: Display> std::fmt::Debug for Wrapper<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Wrapper({})", self.0)
    }
}

let w = Wrapper(42);
w.afficher();       // disponible car i32: Display
```

---

## 5. Generics dans les traits

### Types associés vs paramètres génériques

```rust
// Type associé — une seule implémentation par type
trait Convertir {
    type Sortie;
    fn convertir(self) -> Self::Sortie;
}

impl Convertir for i32 {
    type Sortie = f64;
    fn convertir(self) -> f64 { self as f64 }
}

// Paramètre générique — plusieurs implémentations possibles
trait ConvertirEn<T> {
    fn convertir_en(self) -> T;
}

impl ConvertirEn<f64> for i32 {
    fn convertir_en(self) -> f64 { self as f64 }
}
impl ConvertirEn<String> for i32 {
    fn convertir_en(self) -> String { self.to_string() }
}
```

### `impl Trait` en paramètre et en retour

```rust
// En paramètre : syntaxe courte pour un trait bound
fn afficher(val: impl Display) {
    println!("{val}");
}
// Équivalent à : fn afficher<T: Display>(val: T)

// En retour : type concret opaque
fn creer_iterateur() -> impl Iterator<Item = i32> {
    vec![1, 2, 3].into_iter()
}
// Le type exact est caché, seul Iterator est exposé
```

---

## 6. Monomorphisation

Rust génère **une version spécialisée** du code pour chaque type concret utilisé.

```rust
fn identite<T>(x: T) -> T { x }

identite(5i32);     // génère : fn identite_i32(x: i32) -> i32
identite(3.14f64);  // génère : fn identite_f64(x: f64) -> f64
identite("texte");  // génère : fn identite_str(x: &str) -> &str
```

**Conséquences :**
- ✅ Zéro coût à l'exécution (pas d'appel indirect, pas de boxing)
- ✅ Le compilateur peut optimiser chaque version
- ⚠️ Binaire plus grand si beaucoup de types différents utilisés

**Comparaison avec `dyn Trait` (dispatch dynamique) :**

```rust
// Generics (statique) — résolu à la compilation
fn appeler_generique<T: Parler>(animal: &T) {
    animal.parler();
}

// dyn Trait (dynamique) — résolu à l'exécution via vtable
fn appeler_dynamique(animal: &dyn Parler) {
    animal.parler();
}

// dyn Trait utile pour des collections hétérogènes
let animaux: Vec<Box<dyn Parler>> = vec![
    Box::new(Chien),
    Box::new(Chat),
];
```

> **Règle :** Préférez les génériques pour la performance. Utilisez `dyn Trait` quand vous avez besoin de types hétérogènes à l'exécution.
