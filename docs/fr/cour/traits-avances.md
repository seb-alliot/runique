## Traits Avancés en Rust

## Maîtriser les Traits et le Polymorphisme 

Associated Types, Trait Objects et Plus 

### Objectifs du cours

À la fin de ce cours, tu sauras : 

- Maîtriser les traits standard (Debug, Display, Clone) 

- Utiliser les associated types 

- Comprendre les trait objects (dyn Trait) 

- Appliquer les trait bounds avancés 

- Créer des APIs flexibles avec traits 

## Table des matières

## 1. Traits de la stdlib 

- 1.1 - Debug et Display 

- 1.2 - Clone et Copy 

- 1.3 - Default 

- 1.4 - PartialEq et Eq 

2. Associated Types 

- 2.1 - Différence avec génériques 

- 2.2 - Exemples pratiques 

3. Trait Objects (dyn Trait) 

- 3.1 - Box 

- 3.2 - Object safety 

4. Trait Bounds Avancés 

- 4.1 - Where clauses 

- 4.2 - Bounds multiples 

5. Default Implementations 

6. Supertraits 

7. Patterns avancés 

8. Exercices 

## 1. Traits de la stdlib

**1.1 - Debug et Display** 

```
use std::fmt;
#[derive(Debug)]  // Dérive automatiquement Debug
struct Point {
    x: i32,
    y: i32,
}
// Implémenter Display manuellement
impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}
fn main() {
    let p = Point { x: 3, y: 4 };
    println!("{:?}", p);   // Debug : Point { x: 3, y: 4 }
    println!("{:#?}", p);  // Pretty Debug (multiline)
    println!("{}", p);     // Display : (3, 4)
}
// Debug pour enums
#[derive(Debug)]
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
}
```

### Debug vs Display :

• `Debug` : Pour les développeurs (débogage) • `Display` : Pour les utilisateurs (affichage) 

## 1.2 - Clone et Copy

```
// Clone : copie explicite
#[derive(Clone)]
struct User {
    nom: String,
    age: u32,
}
let user1 = User {
    nom: String::from("Alice"),
    age: 25,
};
let user2 = user1.clone();  // Copie explicite
// Copy : copie implicite (types simples)
#[derive(Copy, Clone)]  // Copy requiert Clone
struct Point {
    x: i32,
    y: i32,
}
```

```
let p1 = Point { x: 1, y: 2 };
let p2 = p1;  // Copié automatiquement
println!("{}, {}", p1.x, p2.x);  // p1 toujours valide !
```

**`//`** II **`Copy seulement pour types sans heap allocation // String ne peut pas être Copy (contient des données heap)`** 

## 1.3 - Default

```
#[derive(Default)]
struct Config {
    host: String,     // Default = ""
    port: u16,        // Default = 0
    debug: bool,      // Default = false
}
fn main() {
    let config = Config::default();
    println!("{}", config.port);  // 0
    // Avec valeurs personnalisées
    let config = Config {
        port: 8080,
        ..Default::default()
    };
}
// Implémentation manuelle
impl Default for Config {
    fn default() -> Self {
        Config {
            host: String::from("localhost"),
            port: 3000,
            debug: false,
        }
    }
}
```

## 1.4 - PartialEq et Eq

```
// PartialEq : égalité partielle
#[derive(PartialEq)]
struct Point {
    x: i32,
    y: i32,
}
let p1 = Point { x: 1, y: 2 };
let p2 = Point { x: 1, y: 2 };
assert!(p1 == p2);
```

```
// Eq : égalité totale (réflexive)
// Pour les types sans NaN
#[derive(PartialEq, Eq)]
struct User {
    id: u32,
    nom: String,
}
```

```
// f32 et f64 sont seulement PartialEq (à cause de NaN)
let x = f64::NAN;
assert!(x != x);  // NaN != NaN !
```

```
// Implémentation manuelle
impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
```

```
}
```

## 2. Associated Types

## 2.1 - Différence avec génériques

```
// Avec générique (peut avoir plusieurs implémentations)
trait Converter<T> {
    fn convert(&self, input: T) -> String;
}
struct IntConverter;
impl Converter<i32> for IntConverter {
    fn convert(&self, input: i32) -> String {
        input.to_string()
    }
}
impl Converter<f64> for IntConverter {
    fn convert(&self, input: f64) -> String {
        input.to_string()
    }
}
// Avec associated type (une seule implémentation)
trait Converter {
    type Output;
    fn convert(&self, input: Self::Output) -> String;
}
struct IntConverter;
impl Converter for IntConverter {
    type Output = i32;
```

```
    fn convert(&self, input: i32) -> String {
        input.to_string()
    }
}
```

## Quand utiliser quoi ?

• **Générique** : Plusieurs implémentations possibles pour un type 

• **Associated type** : Une seule implémentation logique 

## 2.2 - Exemples pratiques

```
// Iterator utilise associated types
trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}
struct Counter {
    count: u32,
}
impl Iterator for Counter {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        self.count += 1;
        Some(self.count)
    }
}
// Utiliser le associated type
fn print_iter<I: Iterator>(iter: &mut I)
where
    I::Item: std::fmt::Display,  // Contraindre l'associated type
{
    while let Some(item) = iter.next() {
        println!("{}", item);
    }
}
// Autre exemple : Add trait
use std::ops::Add;
impl Add for Point {
    type Output = Point;
    fn add(self, other: Point) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}
```

## 3. Trait Objects (dyn Trait)

Les **trait objects** permettent le polymorphisme dynamique : stocker différents types qui implémentent le même trait. 

## 3.1 - Box

```
trait Animal {
    fn faire_bruit(&self) -> String;
}
struct Chien;
impl Animal for Chien {
    fn faire_bruit(&self) -> String {
        "Woof!".to_string()
    }
}
struct Chat;
impl Animal for Chat {
    fn faire_bruit(&self) -> String {
        "Miaou!".to_string()
    }
}
// Vecteur de trait objects
fn main() {
    let animaux: Vec<Box<dyn Animal>> = vec![
        Box::new(Chien),
        Box::new(Chat),
        Box::new(Chien),
    ];
    for animal in &animaux {
        println!("{}", animal.faire_bruit());
    }
}
// Fonction qui accepte n'importe quel Animal
fn faire_parler(animal: &dyn Animal) {
    println!("{}", animal.faire_bruit());
}
```

## II **Static vs Dynamic dispatch :** 

• Génériques ( `<T: Trait>` ) : Static dispatch (plus rapide) 

- Trait objects ( `dyn Trait` ) : Dynamic dispatch (plus flexible) 

## 3.2 - Object safety

**`//`** I **`Object-safe (peut être dyn) trait Draw { fn draw(&self); }`** 

**`//`** I **`Pas object-safe (ne peut pas être dyn) trait Clone { fn clone(&self) -> Self;  // Retourne Self }`** 

```
trait Generic {
    fn method<T>(&self, x: T);  // Méthode générique
}
```

```
// Règles d'object safety :
// 1. Pas de méthodes retournant Self
// 2. Pas de méthodes génériques
// 3. Pas de associated functions (sans &self)
```

```
// Solution : diviser le trait
trait Draw {
    fn draw(&self);
}
```

```
trait Clone {
    fn clone_box(&self) -> Box<dyn Draw>;
}
```

```
// Maintenant on peut avoir :
let shapes: Vec<Box<dyn Draw>> = vec![
    Box::new(Circle),
    Box::new(Square),
];
```

## 4. Trait Bounds Avancés

## 4.1 - Where clauses

```
// Sans where (devient illisible)
fn fonction<T: Display + Clone, U: Clone + Debug>(t: T, u: U) -> i32 {
    // ...
}
```

```
// Avec where (plus lisible)
fn fonction<T, U>(t: T, u: U) -> i32
where
    T: Display + Clone,
    U: Clone + Debug,
{
    // ...
}
// Where avec associated types
fn print_collection<C>(collection: C)
where
    C: IntoIterator,
    C::Item: Display,
{
    for item in collection {
        println!("{}", item);
    }
}
// Contraintes complexes
fn compare<T, U>(t: &T, u: &U) -> bool
where
    T: PartialEq<U>,
    U: PartialEq<T>,
{
    t == u
}
```

## 4.2 - Bounds multiples

```
use std::fmt::Display;
// Multiple bounds avec +
fn notify<T: Display + Clone>(item: T) {
    println!("{}", item);
    let copy = item.clone();
}
// Bounds sur lifetime
fn longest<'a, T>(x: &'a T, y: &'a T) -> &'a T
where
    T: PartialOrd,
{
    if x > y { x } else { y }
}
// impl Trait (raccourci)
fn retourne_closure() -> impl Fn(i32) -> i32 {
    |x| x + 1
}
// Équivalent à :
fn retourne_closure() -> Box<dyn Fn(i32) -> i32> {
    Box::new(|x| x + 1)
}
```

## 5. Default Implementations

```
trait Summary {
    fn summarize_author(&self) -> String;
    // Implémentation par défaut
    fn summarize(&self) -> String {
        format!("(Lire plus de {}...)", self.summarize_author())
    }
}
struct Article {
    author: String,
    content: String,
}
impl Summary for Article {
    fn summarize_author(&self) -> String {
        self.author.clone()
    }
    // Peut override summarize() si nécessaire
    fn summarize(&self) -> String {
        format!("{} : {}...", self.author, &self.content[..50])
    }
}
struct Tweet {
    username: String,
}
impl Summary for Tweet {
    fn summarize_author(&self) -> String {
        format!("@{}", self.username)
    }
    // Utilise l'implémentation par défaut de summarize()
}
```

## 6. Supertraits

```
use std::fmt::Display;
```

```
// OutlinePrint nécessite Display
trait OutlinePrint: Display {
    fn outline_print(&self) {
        let output = self.to_string();
        let len = output.len();
        println!("{}", "*".repeat(len + 4));
        println!("*{}*", " ".repeat(len + 2));
        println!("* {} *", output);
        println!("*{}*", " ".repeat(len + 2));
        println!("{}", "*".repeat(len + 4));
    }
}
struct Point {
    x: i32,
    y: i32,
}
// Doit implémenter Display avant OutlinePrint
impl Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}
impl OutlinePrint for Point {}
```

```
// Utilisation
let p = Point { x: 1, y: 3 };
p.outline_print();
```

## 7. Patterns avancés

```
// 1. Newtype pattern
struct Wrapper(Vec<String>);
```

```
impl Display for Wrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[{}]", self.0.join(", "))
    }
}
```

```
// 2. Extension methods
trait VecExt<T> {
    fn first_or_default(&self) -> T
    where
        T: Default;
}
```

```
impl<T: Clone> VecExt<T> for Vec<T> {
    fn first_or_default(&self) -> T
    where
        T: Default,
    {
        self.first().cloned().unwrap_or_default()
    }
}
// 3. Blanket implementations
trait MyTrait {
    fn method(&self);
}
```

```
impl<T: Display> MyTrait for T {
    fn method(&self) {
        println!("{}", self);
    }
}
```

## 8. Exercices pratiques

### Exercice 1 : Créer un trait Shape

```
// Crée un trait Shape avec:
// - area() -> f64
// - perimeter() -> f64
// Implémente pour Rectangle et Circle
trait Shape {
    fn area(&self) -> f64;
    fn perimeter(&self) -> f64;
}
struct Rectangle {
    largeur: f64,
    hauteur: f64,
}
impl Shape for Rectangle {
    fn area(&self) -> f64 {
        self.largeur * self.hauteur
    }
    fn perimeter(&self) -> f64 {
        2.0 * (self.largeur + self.hauteur)
    }
}
struct Circle {
    rayon: f64,
}
impl Shape for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.rayon * self.rayon
    }
    fn perimeter(&self) -> f64 {
        2.0 * std::f64::consts::PI * self.rayon
    }
}
```

## Aide-mémoire

|**Trait**|**Usage**|
|---|---|
|**`Debug`**|println!("{:?}", x)|
|**`Display`**|println!("{}", x)|
|**`Clone`**|x.clone()|
|**`Copy`**|Copie implicite|
|**`Default`**|T::default()|
|**`PartialEq`**|x == y|

- **Traits** = interfaces de Rust 

- **Associated types** = un type par implémentation 

- **dyn Trait** = polymorphisme dynamique 

- **Where clauses** = contraintes lisibles 

- **Default impl** = comportement par défaut 

- **Supertraits** = dépendances entre traits 

### Bravo !

Tu maîtrises les traits avancés ! 

Tu peux maintenant créer des APIs flexibles et réutilisables. C'est la clé du polymorphisme en Rust ! 

I **Expert Rust en approche !** I
