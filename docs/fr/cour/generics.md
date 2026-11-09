# Generics
> Écrire une fonction, une struct ou une enum une seule fois pour plusieurs types, en disant précisément ce que ces types doivent savoir faire.

## Objectifs

- Écrire des fonctions, structs, enums et blocs `impl` génériques
- Exprimer des contraintes de toutes les façons : en ligne, `where`, `impl Trait`
- Écrire des méthodes réservées à certains types
- Comprendre la monomorphisation et son coût
- Choisir entre générique et `dyn Trait`
- Utiliser les const generics, les paramètres par défaut et `PhantomData`
- Lire et corriger les erreurs de bornes manquantes

## Table des matières

1. Pourquoi des génériques
2. Fonctions génériques
3. Contraintes (trait bounds)
4. Les trois façons d'écrire une contrainte
5. Structs et enums génériques
6. Blocs `impl` génériques
7. Méthodes conditionnelles et implémentations globales
8. Monomorphisation
9. Générique ou `dyn Trait`
10. Const generics
11. Paramètres par défaut et `PhantomData`
12. `?Sized`
13. La même fonction écrite de plusieurs façons
14. Erreurs fréquentes
15. Exercices
16. Aide-mémoire

---

## 1. Pourquoi des génériques

```rust
fn plus_grand_i32(liste: &[i32]) -> i32 {
    let mut max = liste[0];
    for &x in liste {
        if x > max { max = x; }
    }
    max
}

fn plus_grand_char(liste: &[char]) -> char {
    let mut max = liste[0];
    for &x in liste {
        if x > max { max = x; }
    }
    max
}
```

Les deux fonctions sont identiques à un type près. Un générique les remplace par une seule :

```rust
fn plus_grand<T: PartialOrd + Copy>(liste: &[T]) -> T {
    let mut max = liste[0];
    for &x in liste {
        if x > max { max = x; }
    }
    max
}

let a = plus_grand(&[3, 7, 2]);       // T = i32
let b = plus_grand(&['a', 'z']);      // T = char
```

`T` est un **paramètre de type**. `PartialOrd + Copy` dit ce que `T` doit savoir faire : se comparer (`>`) et se copier (`let mut max = liste[0]`).

---

## 2. Fonctions génériques

### 2.1 — Syntaxe

```rust
fn identite<T>(x: T) -> T {
    x
}

fn paire<A, B>(a: A, b: B) -> (A, B) {
    (a, b)
}

fn premier<T>(v: &[T]) -> Option<&T> {
    v.first()
}
```

Sans contrainte, on ne peut presque rien faire avec un `T` : le déplacer, le renvoyer, le stocker. Pas l'afficher, ni le comparer, ni le copier.

### 2.2 — Préciser le type à l'appel

```rust
let a = identite(5);                  // déduit : i32
let b = identite::<u8>(5);            // turbofish
let c: u64 = identite(5);             // déduit du type attendu

let v = Vec::<String>::new();         // turbofish sur un type
let n = "42".parse::<i32>();
```

Le turbofish (`::<T>`) sert quand rien ne permet au compilateur de deviner le type.

---

## 3. Contraintes (trait bounds)

Une contrainte dit : « `T` doit implémenter tel trait ». Elle **ouvre** les opérations de ce trait dans le corps de la fonction.

```rust
use std::fmt::Display;

fn afficher<T: Display>(x: T) {
    println!("{x}");              // permis grâce à Display
}

fn afficher_deux<T: Display + Clone>(x: T) {
    let copie = x.clone();        // permis grâce à Clone
    println!("{x} {copie}");
}
```

### Contraintes courantes et ce qu'elles permettent

| Contrainte | Permet |
|---|---|
| `Clone` | `.clone()` |
| `Copy` | copier en affectant (`let a = x; let b = x;`) |
| `Debug` | `{:?}` |
| `Display` | `{}`, `.to_string()` |
| `PartialEq` | `==`, `!=` |
| `PartialOrd` | `<`, `>`, `<=`, `>=` |
| `Ord` | `.cmp()`, `.max()`, trier |
| `Hash + Eq` | clé de `HashMap` |
| `Default` | `T::default()` |
| `Add<Output = T>` | `a + b` |
| `Into<String>` | `x.into()` vers une `String` |
| `AsRef<str>` | `x.as_ref()` vers une `&str` |
| `IntoIterator<Item = U>` | `for x in t` |
| `Send`, `Sync` | passer ou partager entre threads |
| `'static` | ne contient aucune référence temporaire |

---

## 4. Les trois façons d'écrire une contrainte

### 4.1 — La même fonction écrite trois fois

```rust
use std::fmt::{Debug, Display};

// 1. En ligne
fn resume_v1<T: Display + Clone, U: Debug>(a: T, b: U) -> String {
    format!("{a} / {b:?}")
}

// 2. Clause where : plus lisible quand les contraintes sont longues
fn resume_v2<T, U>(a: T, b: U) -> String
where
    T: Display + Clone,
    U: Debug,
{
    format!("{a} / {b:?}")
}

// 3. impl Trait en argument : sans nommer le type
fn resume_v3(a: impl Display + Clone, b: impl Debug) -> String {
    format!("{a} / {b:?}")
}
```

### 4.2 — Les différences

| | En ligne / `where` | `impl Trait` en argument |
|---|---|---|
| Nommer le type dans le corps ou ailleurs | oui (`T`) | non |
| Deux arguments du **même** type | oui (`a: T, b: T`) | non : chaque `impl` est un type à part |
| Turbofish à l'appel | possible | impossible |
| Contraintes portant sur autre chose que les paramètres | oui avec `where` (`Vec<T>: Debug`) | non |

```rust
// Deux paramètres DOIVENT avoir le même type : seule la forme nommée le permet
fn egaux<T: PartialEq>(a: T, b: T) -> bool { a == b }

// Ici a et b peuvent être de types différents
fn deux(a: impl Display, b: impl Display) {}
```

### 4.3 — `where` sur des types construits

```rust
use std::collections::HashMap;
use std::hash::Hash;

fn compter<T>(elements: &[T]) -> HashMap<&T, usize>
where
    T: Hash + Eq,
{
    let mut m = HashMap::new();
    for e in elements {
        *m.entry(e).or_insert(0) += 1;
    }
    m
}

fn afficher_tout<I>(it: I)
where
    I: IntoIterator,
    I::Item: std::fmt::Display,       // contrainte sur le type des éléments
{
    for x in it {
        println!("{x}");
    }
}
```

### 4.4 — `impl Trait` en retour

```rust
fn pairs(limite: u32) -> impl Iterator<Item = u32> {
    (0..limite).filter(|x| x % 2 == 0)
}
```

En retour, `impl Trait` signifie « un type précis, que je ne nomme pas ». Ce n'est **pas** un générique choisi par l'appelant : c'est la fonction qui choisit, et elle doit toujours renvoyer **le même** type.

---

## 5. Structs et enums génériques

### 5.1 — Struct

```rust
struct Point<T> {
    x: T,
    y: T,
}

let entiers = Point { x: 5, y: 10 };
let flottants = Point { x: 1.0, y: 4.0 };
// let melange = Point { x: 5, y: 4.0 };   // ERREUR : x et y doivent être du même T

struct Point2<T, U> {
    x: T,
    y: U,
}
let melange = Point2 { x: 5, y: 4.0 };     // OK
```

### 5.2 — Enum

Les deux enums les plus utilisées de Rust sont génériques :

```rust
enum Option<T> { Some(T), None }
enum Result<T, E> { Ok(T), Err(E) }
```

```rust
enum Arbre<T> {
    Feuille(T),
    Noeud(Box<Arbre<T>>, Box<Arbre<T>>),
}
```

### 5.3 — Où mettre la contrainte

```rust
// 1. Sur la struct : toute utilisation de Trie<T> exige Ord, même celles qui ne trient pas
struct Trie1<T: Ord> { v: Vec<T> }

// 2. Sur le bloc impl seulement (conseillé) : la struct reste utilisable librement
struct Trie2<T> { v: Vec<T> }

impl<T: Ord> Trie2<T> {
    fn inserer(&mut self, x: T) {
        let pos = self.v.binary_search(&x).unwrap_or_else(|p| p);
        self.v.insert(pos, x);
    }
}
```

Mettre la contrainte sur la struct oblige à la répéter **partout** où le type apparaît (chaque `impl`, chaque fonction qui le reçoit). Ne la mets sur la struct que si le type n'a aucun sens sans elle.

---

## 6. Blocs `impl` génériques

### 6.1 — Pour tous les `T`

```rust
impl<T> Point<T> {
    fn x(&self) -> &T {
        &self.x
    }
}
```

`impl<T>` déclare `T`, `Point<T>` l'utilise. Sans le `<T>` après `impl`, Rust chercherait un type qui s'appelle réellement `T`.

### 6.2 — Pour un type précis

```rust
impl Point<f64> {
    fn distance_origine(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

let p = Point { x: 3.0, y: 4.0 };
p.distance_origine();            // 5.0
let q = Point { x: 3, y: 4 };
// q.distance_origine();         // ERREUR : n'existe que pour Point<f64>
```

### 6.3 — Une méthode avec ses propres paramètres de type

```rust
impl<T> Point2<T, T> {
    fn melanger<V>(self, autre: Point2<V, V>) -> Point2<T, V> {
        Point2 { x: self.x, y: autre.y }
    }
}
```

`T` vient du type, `V` est propre à la méthode et choisi à chaque appel.

---

## 7. Méthodes conditionnelles et implémentations globales

### 7.1 — Méthodes qui n'existent que si `T` le permet

```rust
use std::fmt::Display;

struct Paire<T> { a: T, b: T }

impl<T> Paire<T> {
    fn new(a: T, b: T) -> Self { Self { a, b } }       // pour tous les T
}

impl<T: Display + PartialOrd> Paire<T> {
    fn afficher_le_plus_grand(&self) {                  // seulement si T est comparable et affichable
        if self.a >= self.b {
            println!("{}", self.a);
        } else {
            println!("{}", self.b);
        }
    }
}
```

`Paire<Vec<i32>>` existe, mais n'a pas `afficher_le_plus_grand` (`Vec` n'est pas `Display`).

### 7.2 — Implémenter un trait pour tous les types qui en ont un autre

```rust
trait Resume {
    fn resume(&self) -> String;
}

impl<T: std::fmt::Display> Resume for T {
    fn resume(&self) -> String {
        let s = self.to_string();
        if s.chars().count() > 10 {
            format!("{}…", s.chars().take(10).collect::<String>())   // en caractères, pas en octets
        } else {
            s
        }
    }
}

println!("{}", 12345.resume());
println!("{}", "un texte assez long".resume());
```

C'est une **implémentation globale** (blanket impl). La bibliothèque standard en fait autant : `impl<T: Display> ToString for T` donne `.to_string()` à tout type affichable. Une seule par trait et par forme : elle empêche ensuite d'implémenter le même trait à la main pour un type `Display`.

---

## 8. Monomorphisation

### 8.1 — Ce que fait le compilateur

```rust
fn double<T: std::ops::Add<Output = T> + Copy>(x: T) -> T { x + x }

double(2_i32);
double(2.5_f64);
```

Le compilateur génère **une copie** de la fonction par type utilisé, comme si tu avais écrit :

```rust
fn double_i32(x: i32) -> i32 { x + x }
fn double_f64(x: f64) -> f64 { x + x }
```

### 8.2 — Conséquences

| Avantage | Coût |
|---|---|
| aucun coût à l'exécution : chaque copie est optimisée pour son type | binaire plus gros (une copie par type) |
| appels directs, souvent intégrés (inlining) | compilation plus longue |

### 8.3 — Réduire la duplication

```rust
// Toute la fonction est dupliquée pour chaque type qui implémente AsRef<Path>
pub fn lire_v1<P: AsRef<std::path::Path>>(chemin: P) -> std::io::Result<String> {
    let chemin = chemin.as_ref();
    // ... cinquante lignes ...
    std::fs::read_to_string(chemin)
}

// Seule la conversion est générique ; le corps n'est compilé qu'une fois
pub fn lire_v2<P: AsRef<std::path::Path>>(chemin: P) -> std::io::Result<String> {
    fn interne(chemin: &std::path::Path) -> std::io::Result<String> {
        // ... cinquante lignes ...
        std::fs::read_to_string(chemin)
    }
    interne(chemin.as_ref())
}
```

La bibliothèque standard utilise cette technique (`std::fs::read_to_string` elle-même).

---

## 9. Générique ou `dyn Trait`

La même fonction, qui affiche une liste de formes, écrite des deux façons.

```rust
trait Forme {
    fn aire(&self) -> f64;
}

struct Carre(f64);
struct Cercle(f64);
impl Forme for Carre { fn aire(&self) -> f64 { self.0 * self.0 } }
impl Forme for Cercle { fn aire(&self) -> f64 { 3.14159 * self.0 * self.0 } }

// 1. Générique : tous les éléments ont LE MÊME type
fn total_generique<F: Forme>(formes: &[F]) -> f64 {
    formes.iter().map(|f| f.aire()).sum()
}

// 2. dyn Trait : chaque élément peut avoir un type différent
fn total_dyn(formes: &[Box<dyn Forme>]) -> f64 {
    formes.iter().map(|f| f.aire()).sum()
}

fn main() {
    total_generique(&[Carre(1.0), Carre(2.0)]);
    // total_generique(&[Carre(1.0), Cercle(2.0)]);   // ERREUR : types différents
    total_dyn(&[Box::new(Carre(1.0)), Box::new(Cercle(2.0))]);
}
```

| | Générique `<T: Trait>` | `dyn Trait` |
|---|---|---|
| Choix du type | à la compilation | à l'exécution |
| Collection de types mélangés | non | oui |
| Coût d'appel | direct, intégrable | indirect (table de méthodes) |
| Taille du binaire | une copie par type | une seule copie |
| Contraintes sur le trait | aucune | le trait doit être « dyn-compatible » (voir `box-dynamiques`) |

Par défaut : générique. `dyn` quand les types sont mélangés, choisis à l'exécution, ou quand la taille du binaire compte.

---

## 10. Const generics

Un paramètre générique peut être une **valeur constante** (entier, `bool`, `char`) au lieu d'un type.

```rust
fn somme<const N: usize>(t: [i32; N]) -> i32 {
    t.iter().sum()
}

somme([1, 2, 3]);           // N = 3
somme([1, 2, 3, 4, 5]);     // N = 5

struct Tampon<const TAILLE: usize> {
    donnees: [u8; TAILLE],
    longueur: usize,
}

impl<const TAILLE: usize> Tampon<TAILLE> {
    fn new() -> Self {
        Self { donnees: [0; TAILLE], longueur: 0 }
    }
    fn capacite(&self) -> usize { TAILLE }
}

let t = Tampon::<64>::new();
```

À quoi ça sert : des tableaux de taille fixe sans allocation, des matrices dont les dimensions sont vérifiées à la compilation, des tampons de taille connue en embarqué. Avant les const generics, la bibliothèque standard n'implémentait certains traits que pour les tableaux jusqu'à 32 éléments.

---

## 11. Paramètres par défaut et `PhantomData`

### 11.1 — Type par défaut

```rust
struct Compteur<T = u32> {
    valeur: T,
}

let a: Compteur = Compteur { valeur: 5 };          // T = u32
let b: Compteur<u64> = Compteur { valeur: 5 };
```

C'est ainsi que `HashMap<K, V, S = RandomState>` laisse choisir l'algorithme de hachage sans l'imposer à tout le monde.

### 11.2 — `PhantomData` : un paramètre sans donnée

```rust
use std::marker::PhantomData;

struct Id<T> {
    valeur: i64,
    _type: PhantomData<T>,       // Id<T> « porte » T sans en stocker
}

struct Utilisateur;
struct Commande;

impl<T> Id<T> {
    fn new(valeur: i64) -> Self { Self { valeur, _type: PhantomData } }
}

fn annuler(_id: Id<Commande>) {}

fn main() {
    let u: Id<Utilisateur> = Id::new(1);
    // annuler(u);                   // ERREUR : Id<Utilisateur> n'est pas Id<Commande>
    annuler(Id::new(7));
}
```

Rust refuse un paramètre de type inutilisé (`parameter T is never used`). `PhantomData<T>` l'utilise sans occuper de mémoire. À quoi ça sert : distinguer à la compilation des valeurs de même représentation (identifiants, unités de mesure, états d'un builder).

---

## 12. `?Sized`

Par défaut, tout paramètre de type doit avoir une **taille connue** à la compilation (`T: Sized` implicite). `str`, `[T]` et `dyn Trait` n'en ont pas.

```rust
fn longueur_v1<T: AsRef<str>>(x: T) -> usize { x.as_ref().len() }

// Accepter &str directement en tant que &T avec T = str
fn afficher<T: std::fmt::Display + ?Sized>(x: &T) {
    println!("{x}");
}

afficher("texte");          // T = str : possible grâce à ?Sized
afficher(&5);
```

`?Sized` lève la contrainte implicite. On ne peut alors manipuler `T` que derrière une référence ou un pointeur (`&T`, `Box<T>`).

---

## 13. La même fonction écrite de plusieurs façons

Trouver le plus grand élément d'une liste.

```rust
// 1. Copy : renvoie une copie, réservé aux petits types
fn max_v1<T: PartialOrd + Copy>(v: &[T]) -> Option<T> {
    let mut iter = v.iter().copied();
    let mut max = iter.next()?;
    for x in iter {
        if x > max { max = x; }
    }
    Some(max)
}

// 2. Référence : fonctionne aussi avec String, sans copie
fn max_v2<T: PartialOrd>(v: &[T]) -> Option<&T> {
    let mut max = v.first()?;
    for x in v {
        if x > max { max = x; }
    }
    Some(max)
}

// 3. Ord + bibliothèque standard
fn max_v3<T: Ord>(v: &[T]) -> Option<&T> {
    v.iter().max()
}

// 4. N'importe quel itérateur, pas seulement une tranche
fn max_v4<I>(it: I) -> Option<I::Item>
where
    I: IntoIterator,
    I::Item: Ord,
{
    it.into_iter().max()
}

fn main() {
    max_v1(&[3, 9, 2]);                     // Some(9)
    max_v2(&[String::from("a"), String::from("b")]);
    max_v3(&["pomme", "kiwi"]);
    max_v4(vec![1, 5, 3]);
    max_v4((1..10).map(|x| x * x % 7));
    max_v2(&[1.5, 0.2]);                    // f64 : PartialOrd suffit pour max_v2
    // max_v3(&[1.5, 0.2]);                 // ERREUR : f64 n'est pas Ord
}
```

Toutes renvoient une `Option` : une liste vide n'a pas de maximum, alors que la version de la section 1 paniquait sur `liste[0]`.

---

## 14. Erreurs fréquentes

### 14.1 — Opération sans contrainte

```rust
// fn somme<T>(a: T, b: T) -> T { a + b }
// ERREUR E0369 : cannot add `T` to `T`
use std::ops::Add;
fn somme<T: Add<Output = T>>(a: T, b: T) -> T { a + b }
```

Le compilateur propose souvent la contrainte manquante dans son message (`consider restricting type parameter T`).

### 14.2 — Méthode introuvable

```rust
// fn afficher<T>(x: T) { println!("{}", x); }
// ERREUR E0277 : `T` doesn't implement `std::fmt::Display`
fn afficher<T: std::fmt::Display>(x: T) { println!("{x}"); }
```

### 14.3 — Type impossible à deviner

```rust
// let v = Vec::new();            // ERREUR E0282 : type annotations needed
let v: Vec<i32> = Vec::new();
let w = Vec::<i32>::new();
```

### 14.4 — Déplacer hors d'une tranche

```rust
// fn premier<T>(v: &[T]) -> T { v[0] }   // ERREUR : cannot move out of index
fn premier_ref<T>(v: &[T]) -> &T { &v[0] }
fn premier_copie<T: Copy>(v: &[T]) -> T { v[0] }
fn premier_clone<T: Clone>(v: &[T]) -> T { v[0].clone() }
```

---

## 15. Exercices

### Exercice 1 — pile générique

Écris `Pile<T>` avec `new`, `empiler`, `depiler -> Option<T>`, `sommet -> Option<&T>` et `est_vide`.

```rust
// Solution :
struct Pile<T> {
    elements: Vec<T>,
}

impl<T> Pile<T> {
    fn new() -> Self { Self { elements: Vec::new() } }
    fn empiler(&mut self, x: T) { self.elements.push(x); }
    fn depiler(&mut self) -> Option<T> { self.elements.pop() }
    fn sommet(&self) -> Option<&T> { self.elements.last() }
    fn est_vide(&self) -> bool { self.elements.is_empty() }
}
```

### Exercice 2 — méthode conditionnelle

Ajoute à `Pile<T>` une méthode `afficher` qui n'existe que si `T: Debug`.

```rust
// Solution :
impl<T: std::fmt::Debug> Pile<T> {
    fn afficher(&self) {
        println!("{:?}", self.elements);
    }
}
```

### Exercice 3 — moyenne générique

Écris une moyenne pour tout type convertible en `f64` sans perte (`i32`, `u8`, `f32`...).

```rust
// Solution :
fn moyenne<T: Copy + Into<f64>>(v: &[T]) -> Option<f64> {
    if v.is_empty() {
        return None;
    }
    let somme: f64 = v.iter().map(|&x| -> f64 { x.into() }).sum();   // type cible écrit : sum aurait deux choix
    Some(somme / v.len() as f64)
}

moyenne(&[1_i32, 2, 3]);      // Some(2.0)
moyenne(&[1.5_f32, 2.5]);     // Some(2.0)
// moyenne(&[1_i64]);         // ERREUR : i64 -> f64 peut perdre de la précision, pas de From
```

## 16. Aide-mémoire

```rust
fn f<T>(x: T) -> T                         // générique
fn f<T: A + B>(x: T)                       // contraintes en ligne
fn f<T>(x: T) where T: A + B               // where
fn f(x: impl A + B)                        // impl Trait en argument
fn f() -> impl Iterator<Item = u32>        // impl Trait en retour
struct S<T> { x: T }                       // struct générique
impl<T> S<T> { }                           // pour tous les T
impl S<f64> { }                            // pour un type précis
impl<T: Display> S<T> { }                  // méthodes conditionnelles
impl<T: Display> MonTrait for T { }        // implémentation globale
fn f<const N: usize>(t: [u8; N])           // const generic
struct S<T = u32>                          // type par défaut
PhantomData<T>                             // paramètre sans donnée
fn f<T: ?Sized>(x: &T)                     // accepte str, [T], dyn Trait
f::<u8>(5)                                 // turbofish
```

### Points clés

- Une contrainte ouvre des opérations : sans elle, un `T` ne sait presque rien faire.
- `where` pour la lisibilité, `impl Trait` quand on n'a pas besoin de nommer le type.
- Mets les contraintes sur le bloc `impl`, pas sur la struct.
- Un générique est copié pour chaque type : rapide à l'exécution, plus lourd à compiler.
- Générique par défaut, `dyn Trait` pour mélanger des types à l'exécution.
- `PhantomData` distingue des types sans coût en mémoire.
