# Traits en Rust — Les Bases
> Interfaces, implémentations, derives et `impl Trait` — le système de types en action

## Objectifs

- Comprendre ce qu'est un trait et à quoi il sert
- Définir et implémenter ses propres traits
- Utiliser les implémentations par défaut
- Maîtriser `impl Trait` en paramètre et en retour
- Connaître les derives courantes (`Debug`, `Clone`, `Copy`, `PartialEq`, `Hash`)
- Gérer plusieurs implémentations et la règle de cohérence

---

## Table des matières

1. [Qu'est-ce qu'un trait](#1-quest-ce-quun-trait)
2. [Définir un trait](#2-définir-un-trait)
3. [Implémenter un trait sur une struct](#3-implémenter-un-trait-sur-une-struct)
4. [Implémentation par défaut](#4-implémentation-par-défaut)
5. [impl Trait en paramètre et en retour](#5-impl-trait-en-paramètre-et-en-retour)
6. [Les derives communes](#6-les-derives-communes)
7. [Impl multiple et cohérence](#7-impl-multiple-et-cohérence)
8. [Exercices pratiques](#8-exercices-pratiques)
9. [Aide-mémoire](#9-aide-mémoire)

---

## 1. Qu'est-ce qu'un trait

Un **trait** est un contrat : il définit un ensemble de méthodes qu'un type doit implémenter.
C'est l'équivalent des interfaces dans d'autres langages, avec des fonctionnalités en plus.

```rust
// Un trait définit un comportement
trait Saluer {
    fn saluer(&self) -> String;
}

// N'importe quel type peut l'implémenter
struct Francais;
struct Japonais;

impl Saluer for Francais {
    fn saluer(&self) -> String {
        "Bonjour !".to_string()
    }
}

impl Saluer for Japonais {
    fn saluer(&self) -> String {
        "Konnichiwa !".to_string()
    }
}

// Utilisation polymorphique
fn accueillir(personne: &impl Saluer) {
    println!("{}", personne.saluer());
}
```

Les traits permettent de :
- Écrire du code générique réutilisable
- Définir des interfaces sans héritage
- Garantir des comportements à la compilation

---

## 2. Définir un trait

Un trait déclare des signatures de méthodes. Les types qui l'implémentent doivent fournir le corps.

```rust
trait Forme {
    // Méthode requise — pas de corps
    fn aire(&self) -> f64;

    // Méthode requise
    fn perimetre(&self) -> f64;

    // Méthode avec implémentation par défaut
    fn description(&self) -> String {
        format!("Aire : {:.2}, Périmètre : {:.2}", self.aire(), self.perimetre())
    }
}
```

Un trait peut aussi définir des **méthodes associées** (sans `&self`) :

```rust
trait Creable {
    fn nouveau() -> Self;
}

struct Compteur {
    valeur: u32,
}

impl Creable for Compteur {
    fn nouveau() -> Self {
        Compteur { valeur: 0 }
    }
}

let c = Compteur::nouveau();
```

---

## 3. Implémenter un trait sur une struct

La syntaxe est `impl NomTrait for NomType`.

```rust
struct Rectangle {
    largeur: f64,
    hauteur: f64,
}

struct Cercle {
    rayon: f64,
}

impl Forme for Rectangle {
    fn aire(&self) -> f64 {
        self.largeur * self.hauteur
    }

    fn perimetre(&self) -> f64 {
        2.0 * (self.largeur + self.hauteur)
    }
}

impl Forme for Cercle {
    fn aire(&self) -> f64 {
        std::f64::consts::PI * self.rayon * self.rayon
    }

    fn perimetre(&self) -> f64 {
        2.0 * std::f64::consts::PI * self.rayon
    }

    // On peut surcharger la méthode par défaut
    fn description(&self) -> String {
        format!("Cercle r={} — aire {:.2}", self.rayon, self.aire())
    }
}

fn main() {
    let r = Rectangle { largeur: 4.0, hauteur: 3.0 };
    let c = Cercle { rayon: 5.0 };

    println!("{}", r.description()); // méthode par défaut
    println!("{}", c.description()); // méthode surchargée
}
```

---

## 4. Implémentation par défaut

Une implémentation par défaut s'applique automatiquement si le type ne la redéfinit pas.
Elle peut appeler d'autres méthodes du même trait.

```rust
trait Resumable {
    // Méthode à implémenter obligatoirement
    fn auteur(&self) -> &str;

    fn titre(&self) -> &str;

    // Méthode par défaut qui s'appuie sur les deux précédentes
    fn resume(&self) -> String {
        format!("« {} » par {}", self.titre(), self.auteur())
    }
}

struct Article {
    titre: String,
    auteur: String,
    contenu: String,
}

impl Resumable for Article {
    fn auteur(&self) -> &str {
        &self.auteur
    }

    fn titre(&self) -> &str {
        &self.titre
    }

    // resume() non redéfini → utilise la version par défaut
}

struct Tweet {
    utilisateur: String,
    message: String,
}

impl Resumable for Tweet {
    fn auteur(&self) -> &str {
        &self.utilisateur
    }

    fn titre(&self) -> &str {
        &self.message
    }

    // Surcharge la méthode par défaut
    fn resume(&self) -> String {
        format!("@{} : {}", self.utilisateur, self.message)
    }
}

let article = Article {
    titre: "Rust en production".to_string(),
    auteur: "Alice".to_string(),
    contenu: "...".to_string(),
};

println!("{}", article.resume()); // « Rust en production » par Alice
```

---

## 5. `impl Trait` en paramètre et en retour

`impl Trait` est un raccourci syntaxique pour les trait bounds. Il rend le code plus lisible.

### En paramètre

```rust
use std::fmt::Display;

// Syntaxe impl Trait (raccourci)
fn afficher(valeur: impl Display) {
    println!("{valeur}");
}

// Équivalent avec générique explicite
fn afficher_generique<T: Display>(valeur: T) {
    println!("{valeur}");
}

// Plusieurs paramètres — chacun peut être un type différent
fn comparer(a: impl Display, b: impl Display) {
    println!("{a} vs {b}");
}

// Avec plusieurs bounds
fn afficher_debug(valeur: impl Display + std::fmt::Debug) {
    println!("Display: {valeur}  Debug: {valeur:?}");
}
```

### En retour

`impl Trait` en position de retour cache le type concret tout en gardant le dispatch statique.

```rust
// Le type exact de l'itérateur est caché
fn nombres_pairs(limite: u32) -> impl Iterator<Item = u32> {
    (0..limite).filter(|n| n % 2 == 0)
}

// Utile pour retourner des closures
fn multiplicateur(facteur: i32) -> impl Fn(i32) -> i32 {
    move |x| x * facteur
}

let doubler = multiplicateur(2);
println!("{}", doubler(5)); // 10
```

> **Limite :** avec `impl Trait` en retour, tous les chemins de code doivent retourner le **même type concret**. Pour retourner des types différents, utilisez `Box<dyn Trait>`.

```rust
// Ceci ne compile PAS — deux types concrets différents
// fn animal(chien: bool) -> impl Animal {
//     if chien { Chien } else { Chat }
// }

// Solution : Box<dyn Trait>
fn animal(chien: bool) -> Box<dyn Animal> {
    if chien { Box::new(Chien) } else { Box::new(Chat) }
}
```

---

## 6. Les derives communes

L'attribut `#[derive(...)]` génère automatiquement des implémentations de traits standard.

### `Debug`

Permet l'affichage avec `{:?}` et `{:#?}` (pretty-print).

```rust
#[derive(Debug)]
struct Utilisateur {
    nom: String,
    age: u32,
    actif: bool,
}

let u = Utilisateur { nom: "Alice".to_string(), age: 30, actif: true };

println!("{:?}", u);   // Utilisateur { nom: "Alice", age: 30, actif: true }
println!("{:#?}", u);  // version indenté multi-ligne
```

### `Clone` et `Copy`

```rust
// Clone — copie explicite via .clone()
#[derive(Debug, Clone)]
struct Config {
    host: String,
    port: u16,
}

let config1 = Config { host: "localhost".to_string(), port: 8080 };
let config2 = config1.clone(); // copie indépendante

// Copy — copie implicite (types légers, sans heap)
// Copy nécessite Clone
#[derive(Debug, Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
}

let p1 = Point { x: 1.0, y: 2.0 };
let p2 = p1; // copié, pas déplacé

println!("{p1:?}"); // p1 est toujours valide
println!("{p2:?}");
```

> **Règle :** `Copy` ne peut s'appliquer qu'aux types dont tous les champs sont `Copy`. `String`, `Vec`, `Box` ne peuvent pas être `Copy` car ils possèdent de la mémoire sur le tas.

### `PartialEq` et `Eq`

```rust
#[derive(Debug, PartialEq)]
struct Coordonnee {
    x: i32,
    y: i32,
}

let a = Coordonnee { x: 1, y: 2 };
let b = Coordonnee { x: 1, y: 2 };
let c = Coordonnee { x: 3, y: 4 };

assert!(a == b);
assert!(a != c);

// Eq garantit la réflexivité totale (a == a toujours vrai)
// f64 implémente PartialEq mais pas Eq (NaN != NaN)
#[derive(Debug, PartialEq, Eq)]
struct Id(u64);
```

### `Hash`

`Hash` est nécessaire pour utiliser un type comme clé de `HashMap` ou dans un `HashSet`.
Il requiert `PartialEq` (et recommande `Eq`).

```rust
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CleComposee {
    categorie: String,
    identifiant: u32,
}

let mut map: HashMap<CleComposee, String> = HashMap::new();

map.insert(
    CleComposee { categorie: "utilisateur".to_string(), identifiant: 42 },
    "Alice".to_string(),
);

let cle = CleComposee { categorie: "utilisateur".to_string(), identifiant: 42 };
println!("{:?}", map.get(&cle)); // Some("Alice")
```

---

## 7. Impl multiple et cohérence

### Plusieurs traits sur un même type

Un type peut implémenter autant de traits que nécessaire.

```rust
use std::fmt;

#[derive(Clone)]
struct Vecteur2D {
    x: f64,
    y: f64,
}

impl Vecteur2D {
    fn norme(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

impl fmt::Display for Vecteur2D {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl fmt::Debug for Vecteur2D {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Vecteur2D {{ x: {}, y: {} }}", self.x, self.y)
    }
}

impl PartialEq for Vecteur2D {
    fn eq(&self, other: &Self) -> bool {
        (self.x - other.x).abs() < f64::EPSILON
            && (self.y - other.y).abs() < f64::EPSILON
    }
}

impl std::ops::Add for Vecteur2D {
    type Output = Vecteur2D;

    fn add(self, other: Vecteur2D) -> Vecteur2D {
        Vecteur2D {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}
```

### La règle de cohérence (orphan rule)

Rust impose une contrainte : pour implémenter un trait sur un type, **au moins l'un des deux** doit être défini dans le crate courant.

```rust
// OK — MonType est dans notre crate
impl Display for MonType { ... }

// OK — MonTrait est dans notre crate
impl MonTrait for String { ... }

// INTERDIT — ni Display ni Vec ne sont dans notre crate
// impl Display for Vec<i32> { ... }
```

Pour contourner cette règle, on utilise le **newtype pattern** :

```rust
// Wrapper local autour d'un type externe
struct MesNombres(Vec<i32>);

impl fmt::Display for MesNombres {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s: Vec<String> = self.0.iter().map(|n| n.to_string()).collect();
        write!(f, "[{}]", s.join(", "))
    }
}

let liste = MesNombres(vec![1, 2, 3]);
println!("{liste}"); // [1, 2, 3]
```

---

## 8. Exercices pratiques

### Exercice 1 — Trait `Convertible`

Créez un trait `Convertible` avec une méthode `en_chaine(&self) -> String` et implémentez-le
pour `f64`, une struct `Temperature` et une struct `Couleur { r: u8, g: u8, b: u8 }`.

```rust
trait Convertible {
    fn en_chaine(&self) -> String;
}

impl Convertible for f64 {
    fn en_chaine(&self) -> String {
        format!("{:.2}", self)
    }
}

struct Temperature {
    celsius: f64,
}

impl Convertible for Temperature {
    fn en_chaine(&self) -> String {
        format!("{:.1}°C", self.celsius)
    }
}

struct Couleur {
    r: u8,
    g: u8,
    b: u8,
}

impl Convertible for Couleur {
    fn en_chaine(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

// Test
let t = Temperature { celsius: 36.6 };
let c = Couleur { r: 255, g: 128, b: 0 };
println!("{}", t.en_chaine()); // 36.6°C
println!("{}", c.en_chaine()); // #FF8000
```

### Exercice 2 — Tri générique

Écrivez une fonction `plus_grand` générique qui fonctionne avec tout type comparable et affichable.

```rust
use std::fmt::Display;

fn plus_grand<T>(liste: &[T]) -> Option<&T>
where
    T: PartialOrd + Display,
{
    let mut max = liste.first()?;
    for item in liste.iter() {
        if item > max {
            max = item;
        }
    }
    Some(max)
}

let nombres = vec![34, 50, 25, 100, 65];
let lettres = vec!['y', 'm', 'a', 'q'];

println!("{:?}", plus_grand(&nombres)); // Some(100)
println!("{:?}", plus_grand(&lettres)); // Some('y')
```

---

## 9. Aide-mémoire

| Syntaxe | Signification |
|---|---|
| `trait Foo { fn bar(&self); }` | Définir un trait |
| `impl Foo for MaStruct { ... }` | Implémenter un trait |
| `fn f(x: impl Foo)` | Paramètre avec trait bound |
| `fn f() -> impl Foo` | Retour avec type opaque |
| `fn f<T: Foo>(x: T)` | Générique explicite |
| `where T: Foo + Bar` | Clause where (bounds multiples) |
| `#[derive(Debug, Clone)]` | Implémentation automatique |

**Derives et leurs usages :**

| Derive | Permet |
|---|---|
| `Debug` | `{:?}` et `{:#?}` |
| `Clone` | `.clone()` explicite |
| `Copy` | Copie implicite (types légers) |
| `PartialEq` | `==` et `!=` |
| `Eq` | Égalité totale (+ `PartialEq`) |
| `Hash` | Clé de `HashMap` / `HashSet` |
| `Default` | `T::default()` |
| `PartialOrd` / `Ord` | `<`, `>`, tri |

**Points clés à retenir :**

- Un trait = un contrat de comportement
- `impl Trait` en paramètre = syntaxe courte pour un bound
- `impl Trait` en retour = type concret opaque (statique, pas de box)
- `Box<dyn Trait>` = dispatch dynamique (pour types hétérogènes)
- La règle de cohérence protège l'écosystème des conflits
- `Copy` requiert `Clone`, et tous les champs doivent être `Copy`
