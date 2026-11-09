# Traits avancés

**Types associés, surcharge d'opérateurs, supertraits, implémentations globales, traits scellés, traits asynchrones** — les outils qui permettent de concevoir des API sûres et expressives, avec les différentes façons d'arriver au même résultat.

## Objectifs du cours

À la fin de ce cours, tu sauras :

- Choisir entre un type associé et un paramètre générique de trait
- Surcharger les opérateurs (`+`, `-`, `*`, `[]`, `+=`, `-x`)
- Exiger un autre trait avec un supertrait
- Écrire des implémentations globales et des traits marqueurs
- Sceller un trait pour que personne d'autre ne puisse l'implémenter
- Encoder des états dans les types (typestate) et améliorer les messages d'erreur avec `#[diagnostic::on_unimplemented]`
- Écrire des méthodes `async` dans un trait et connaître leurs limites
- Utiliser la syntaxe complète `<T as Trait>::` pour lever une ambiguïté

Prérequis : `traits-basics`, `generics`, `box-dynamiques`.

## Table des matières

1. Types associés
2. Type associé ou paramètre générique
3. Paramètres de trait par défaut
4. Surcharge d'opérateurs
5. Supertraits
6. Implémentations globales
7. Traits marqueurs
8. Traits scellés
9. Typestate : des états vérifiés à la compilation
10. Messages d'erreur sur mesure
11. Méthodes `async` dans un trait
12. Syntaxe complète et ambiguïtés
13. Implémenter un trait pour `&T`, `Box<T>`, `Vec<T>`
14. Types associés génériques (GAT)
15. La même API écrite de plusieurs façons
16. Exercices pratiques
17. Aide-mémoire

## 1. Types associés

### 1.1 Définition

```rust
pub trait Iterator {
    type Item;                                   // type associé : choisi par chaque implémentation
    fn next(&mut self) -> Option<Self::Item>;
}
```

```rust
struct Compteur { n: u32 }

impl Iterator for Compteur {
    type Item = u32;                             // ici, Item vaut u32
    fn next(&mut self) -> Option<u32> {
        self.n += 1;
        (self.n <= 3).then_some(self.n)
    }
}
```

Un type associé est un « trou » dans le trait, rempli **une fois** par chaque implémentation.

### 1.2 L'utiliser dans une contrainte

```rust
fn somme<I>(it: I) -> u32
where
    I: Iterator<Item = u32>,          // contrainte sur le type associé
{
    it.sum()
}

fn afficher_tout<I>(it: I)
where
    I: Iterator,
    I::Item: std::fmt::Debug,         // contrainte sur le type associé lui-même
{
    for x in it {
        println!("{x:?}");
    }
}
```

### 1.3 Un type associé avec contrainte

```rust
trait Depot {
    type Cle: std::hash::Hash + Eq + Clone;      // toute implémentation doit choisir une clé hachable
    type Valeur;
    type Erreur: std::error::Error;

    fn lire(&self, cle: &Self::Cle) -> Result<Option<Self::Valeur>, Self::Erreur>;
}
```

## 2. Type associé ou paramètre générique

### 2.1 La même idée écrite deux fois

```rust
// 1. Paramètre générique : un type peut implémenter le trait PLUSIEURS fois
trait ConvertirEn<T> {
    fn convertir(&self) -> T;
}

struct Metres(f64);
impl ConvertirEn<f64> for Metres { fn convertir(&self) -> f64 { self.0 } }
impl ConvertirEn<String> for Metres { fn convertir(&self) -> String { format!("{} m", self.0) } }

// 2. Type associé : UNE seule implémentation par type
trait Contenant {
    type Element;
    fn premier(&self) -> Option<&Self::Element>;
}

struct Pile(Vec<i32>);
impl Contenant for Pile {
    type Element = i32;
    fn premier(&self) -> Option<&i32> { self.0.first() }
}
```

### 2.2 Conséquences pour l'appelant

```rust
let m = Metres(3.0);
// let x = m.convertir();               // ERREUR : vers quel type ?
let x: f64 = m.convertir();             // l'appelant doit préciser
let y = ConvertirEn::<String>::convertir(&m);

let p = Pile(vec![1]);
let z = p.premier();                     // aucune ambiguïté : un seul Element possible
```

| | Paramètre générique `Trait<T>` | Type associé `type T` |
|---|---|---|
| Implémentations par type | plusieurs | une seule |
| L'appelant précise le type | souvent | jamais |
| Exemples standard | `From<T>`, `Add<Rhs>`, `PartialEq<Rhs>` | `Iterator::Item`, `Deref::Target`, `FromStr::Err` |
| Question à se poser | « un type peut-il avoir plusieurs réponses ? » | « la réponse est-elle déterminée par le type ? » |

`String` peut être construite depuis `&str`, `char`, `Box<str>`... : `From<T>` est générique. Un itérateur ne produit qu'**une** sorte d'élément : `Item` est associé.

## 3. Paramètres de trait par défaut

```rust
pub trait Add<Rhs = Self> {        // par défaut, on additionne deux valeurs du même type
    type Output;
    fn add(self, rhs: Rhs) -> Self::Output;
}
```

Le défaut `Rhs = Self` rend le cas courant simple (`impl Add for Point`) tout en permettant des combinaisons (`impl Add<Duree> for Instant`).

## 4. Surcharge d'opérateurs

Chaque opérateur correspond à un trait de `std::ops`.

| Opérateur | Trait | Opérateur | Trait |
|---|---|---|---|
| `a + b` | `Add` | `a += b` | `AddAssign` |
| `a - b` | `Sub` | `a -= b` | `SubAssign` |
| `a * b` | `Mul` | `a *= b` | `MulAssign` |
| `a / b` | `Div` | `a % b` | `Rem` |
| `-a` | `Neg` | `!a` | `Not` |
| `a[i]` | `Index` | `a[i] = v` | `IndexMut` |
| `a & b`, `a ^ b` (et le ou binaire) | `BitAnd`, `BitXor`, `BitOr` | `a << b` | `Shl` |
| `a == b` | `PartialEq` | `a < b` | `PartialOrd` |

### 4.1 Additionner deux valeurs du même type

```rust
use std::ops::{Add, AddAssign, Neg, Sub};

#[derive(Debug, Clone, Copy, PartialEq)]
struct Vecteur { x: f64, y: f64 }

impl Add for Vecteur {
    type Output = Vecteur;
    fn add(self, o: Vecteur) -> Vecteur { Vecteur { x: self.x + o.x, y: self.y + o.y } }
}

impl Sub for Vecteur {
    type Output = Vecteur;
    fn sub(self, o: Vecteur) -> Vecteur { Vecteur { x: self.x - o.x, y: self.y - o.y } }
}

impl Neg for Vecteur {
    type Output = Vecteur;
    fn neg(self) -> Vecteur { Vecteur { x: -self.x, y: -self.y } }
}

impl AddAssign for Vecteur {
    fn add_assign(&mut self, o: Vecteur) { self.x += o.x; self.y += o.y; }
}

let a = Vecteur { x: 1.0, y: 2.0 };
let b = Vecteur { x: 3.0, y: 4.0 };
let mut c = a + b - Vecteur { x: 1.0, y: 1.0 };
c += a;
let d = -c;
```

### 4.2 Combiner deux types différents

```rust
use std::ops::Mul;

impl Mul<f64> for Vecteur {                  // Vecteur * f64
    type Output = Vecteur;
    fn mul(self, k: f64) -> Vecteur { Vecteur { x: self.x * k, y: self.y * k } }
}

impl Mul<Vecteur> for f64 {                  // f64 * Vecteur : permis, Vecteur est local
    type Output = Vecteur;
    fn mul(self, v: Vecteur) -> Vecteur { v * self }
}

let e = a * 2.0;
let f = 2.0 * a;
```

### 4.3 Opérateurs sur des références

```rust
use std::ops::Add;

#[derive(Debug, Clone)]
struct Grand(Vec<u64>);                          // pas Copy : a + b consommerait a et b

impl Add<&Grand> for &Grand {
    type Output = Grand;
    fn add(self, o: &Grand) -> Grand {
        Grand(self.0.iter().zip(&o.0).map(|(x, y)| x + y).collect())
    }
}

let a = Grand(vec![1, 2]);
let b = Grand(vec![3, 4]);
let c = &a + &b;                                 // a et b restent utilisables
```

### 4.4 Indexation

```rust
use std::ops::{Index, IndexMut};

struct Grille { largeur: usize, cases: Vec<char> }

impl Index<(usize, usize)> for Grille {
    type Output = char;
    fn index(&self, (x, y): (usize, usize)) -> &char {
        &self.cases[y * self.largeur + x]
    }
}

impl IndexMut<(usize, usize)> for Grille {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut char {
        &mut self.cases[y * self.largeur + x]
    }
}

let mut g = Grille { largeur: 3, cases: vec!['.'; 9] };
g[(1, 1)] = '#';
println!("{}", g[(1, 1)]);
```

### 4.5 Quand ne pas surcharger

Un opérateur doit faire ce que tout le monde attend (`+` additionne, sans effet de bord, sans erreur possible). Pour une opération qui peut échouer ou qui a un nom métier, une méthode nommée est plus claire (`compte.debiter(montant)` plutôt que `compte - montant`).

## 5. Supertraits

### 5.1 Exiger un autre trait

```rust
use std::fmt::Display;

trait Etiquette: Display {                       // tout type Etiquette doit aussi être Display
    fn etiquette(&self) -> String {
        format!("[{}]", self)                    // permis : Display est garanti
    }
}

struct Code(u32);
impl Display for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "C{}", self.0) }
}
impl Etiquette for Code {}

// struct Brut;
// impl Etiquette for Brut {}                    // ERREUR : Brut n'implémente pas Display
```

### 5.2 Plusieurs supertraits

```rust
trait Entite: Clone + std::fmt::Debug + Send + Sync + 'static {
    fn id(&self) -> u64;
}
```

Un supertrait n'est pas de l'héritage : `Entite` n'hérite d'aucun code, elle **exige** que ses implémenteurs aient aussi `Clone`, `Debug`, etc. Les méthodes par défaut d'`Entite` peuvent alors s'en servir.

### 5.3 Supertrait ou contrainte sur la fonction

```rust
// 1. Supertrait : la contrainte vaut partout où le trait est utilisé
trait Affichable1: Display { fn titre(&self) -> String; }

// 2. Contrainte locale : seule cette fonction exige Display
trait Affichable2 { fn titre(&self) -> String; }
fn montrer<T: Affichable2 + Display>(x: &T) { println!("{x} {}", x.titre()); }
```

Mets un supertrait quand le trait n'a **pas de sens** sans l'autre (une erreur sans `Display`, un trait object partagé sans `Send + Sync`). Sinon, une contrainte locale laisse plus de liberté.

## 6. Implémentations globales

```rust
trait Crier {
    fn crier(&self) -> String;
}

impl<T: std::fmt::Display> Crier for T {          // pour TOUS les types affichables
    fn crier(&self) -> String {
        self.to_string().to_uppercase()
    }
}

println!("{}", "bonjour".crier());
println!("{}", 42.crier());
```

La bibliothèque standard en contient beaucoup :

```rust
// impl<T: Display + ?Sized> ToString for T          // .to_string() sur tout Display
// impl<T, U: From<T>> Into<U> for T                  // Into gratuit pour tout From
// impl<T: ?Sized> Borrow<T> for T
// impl<I: Iterator> IntoIterator for I               // un itérateur est utilisable dans for
```

Limite : une implémentation globale **occupe** tous les types concernés. Après `impl<T: Display> Crier for T`, on ne peut plus écrire `impl Crier for i32` (conflit, erreur E0119). Seule la crate qui définit le trait peut écrire une implémentation globale.

## 7. Traits marqueurs

Un trait sans méthode, qui sert seulement d'**étiquette** vérifiée à la compilation.

```rust
trait Valide {}                                   // « cette valeur a été vérifiée »

struct Brut(String);
struct Verifie(String);
impl Valide for Verifie {}

fn enregistrer<T: Valide>(_v: T) {}

// enregistrer(Brut("x".into()));                 // ERREUR : Brut n'est pas Valide
enregistrer(Verifie("x".into()));
```

Exemples standard : `Copy`, `Eq`, `Send`, `Sync`, `Sized`, `Unpin`. `Eq` n'a aucune méthode : c'est une promesse (l'égalité est totale).

## 8. Traits scellés

### 8.1 Le problème

Un trait public peut être implémenté par **n'importe quelle** crate. Si ta bibliothèque compte sur une liste fermée d'implémentations (pour une garantie de sécurité, ou pour pouvoir ajouter une méthode sans casser personne), il faut l'empêcher.

### 8.2 La technique

```rust
mod prive {
    pub trait Sceau {}                            // public, mais dans un module privé : inaccessible
}

pub trait Format: prive::Sceau {                  // le supertrait est impossible à implémenter ailleurs
    fn extension(&self) -> &'static str;
}

pub struct Json;
pub struct Csv;

impl prive::Sceau for Json {}
impl prive::Sceau for Csv {}
impl Format for Json { fn extension(&self) -> &'static str { "json" } }
impl Format for Csv { fn extension(&self) -> &'static str { "csv" } }
```

Hors de la crate, on peut **utiliser** `Format` (`fn f(x: impl Format)`), mais pas l'**implémenter** : `prive::Sceau` n'est pas nommable.

| Avantage | Pour |
|---|---|
| ajouter une méthode obligatoire n'est plus une rupture | l'évolution d'une bibliothèque |
| la liste des implémentations est connue | une garantie (sécurité, invariant) |

## 9. Typestate : des états vérifiés à la compilation

L'état d'un objet est encodé dans son **type** : une méthode n'existe que dans l'état où elle a du sens.

### 9.1 La même règle écrite deux fois

```rust
// 1. Vérification à l'exécution : un booléen et une erreur possible
struct Connexion1 { authentifiee: bool }
impl Connexion1 {
    fn envoyer(&self, _msg: &str) -> Result<(), String> {
        if !self.authentifiee {
            return Err("non authentifiée".into());
        }
        Ok(())
    }
}

// 2. Vérification à la compilation : deux types d'état
use std::marker::PhantomData;

struct Anonyme;
struct Authentifiee;

struct Connexion<Etat> { _etat: PhantomData<Etat> }

impl Connexion<Anonyme> {
    fn new() -> Self { Connexion { _etat: PhantomData } }
    fn login(self, _mdp: &str) -> Connexion<Authentifiee> { Connexion { _etat: PhantomData } }
}

impl Connexion<Authentifiee> {
    fn envoyer(&self, _msg: &str) {}              // n'existe qu'une fois connecté
}

fn main() {
    let c = Connexion::new();
    // c.envoyer("x");                            // ERREUR : pas de méthode envoyer pour Connexion<Anonyme>
    let c = c.login("secret");
    c.envoyer("x");
}
```

### 9.2 Dans Runique : réglages uniques du builder

Le builder de Runique empêche d'appeler deux fois un réglage qui n'a de sens qu'une fois (`routes()`, `with_log()`, `with_public_url()`...). Chaque réglage a un paramètre de type qui vaut `No` puis `Yes` :

```rust
// runique/src/app/builder/mod.rs (extrait)
pub mod state {
    pub struct No;
    pub struct Yes;
    pub type Fresh = (No, No, No, No, No, No, No);

    mod sealed {
        pub trait Sealed {}
        impl Sealed for super::No {}
    }

    // Pour chaque réglage, un trait scellé implémenté seulement par No
    // pub trait RoutesUnset: sealed::Sealed {}
    // impl RoutesUnset for No {}
}
```

La méthode `routes()` n'est disponible que si le paramètre correspondant implémente `RoutesUnset`, c'est-à-dire s'il vaut `No`. Après l'appel, il vaut `Yes`, et un second `routes()` ne compile plus. Le trait est **scellé** (section 8) : aucun code extérieur ne peut l'implémenter pour `Yes` et contourner la règle.

## 10. Messages d'erreur sur mesure

Un typestate ou un trait marqueur produit des erreurs génériques (« the trait bound `Yes: RoutesUnset` is not satisfied »). L'attribut `#[diagnostic::on_unimplemented]` (Rust 1.78+) les remplace par un message choisi.

```rust
#[diagnostic::on_unimplemented(
    message = "`{Self}` ne peut pas être enregistré : il n'a pas été validé",
    label = "valeur non validée",
    note = "passe d'abord la valeur à `valider()`"
)]
trait Valide {}
```

Runique l'utilise pour son builder :

```rust
// runique/src/app/builder/mod.rs (extrait, généré par une macro)
#[diagnostic::on_unimplemented(
    message = "`routes()` has already been called on this builder",
    label = "a second `routes()` would replace every route of the first",
    note = "build a single `Router` (merge or nest the others into it) and pass it once"
)]
pub trait RoutesUnset: sealed::Sealed {}
```

Le développeur qui appelle deux fois `routes()` lit directement la cause et la correction, au lieu d'une erreur de trait incompréhensible.

## 11. Méthodes `async` dans un trait

### 11.1 Depuis Rust 1.75

```rust
trait Stockage {
    async fn lire(&self, cle: &str) -> Option<String>;
}

struct Memoire(std::collections::HashMap<String, String>);

impl Stockage for Memoire {
    async fn lire(&self, cle: &str) -> Option<String> {
        self.0.get(cle).cloned()
    }
}
```

### 11.2 La limite : `Send` et `dyn`

```rust
async fn utiliser<S: Stockage>(s: &S) {
    let _ = s.lire("a").await;
    // tokio::spawn(async move { s.lire("a").await });   // le futur renvoyé est-il Send ? inconnu
}
```

Le futur renvoyé par `lire` n'a pas de contrainte `Send` : il ne peut pas être passé à `tokio::spawn` dans du code générique, et le trait n'est pas utilisable en `dyn Stockage`.

### 11.3 Trois façons d'écrire un trait asynchrone

```rust
use std::future::Future;
use std::pin::Pin;

// 1. async fn natif : simple, pas de dyn, pas de garantie Send
trait Stockage1 {
    async fn lire(&self, cle: &str) -> Option<String>;
}

// 2. Renvoyer impl Future + Send : garantit Send, toujours pas de dyn
trait Stockage2 {
    fn lire(&self, cle: &str) -> impl Future<Output = Option<String>> + Send;
}

// 3. Futur en Box : compatible dyn et Send, une allocation par appel
trait Stockage3: Send + Sync {
    fn lire<'a>(&'a self, cle: &'a str) -> Pin<Box<dyn Future<Output = Option<String>> + Send + 'a>>;
}
```

La crate `async-trait` génère la forme 3 à partir d'une écriture `async fn` :

```rust
#[async_trait::async_trait]
trait Stockage4: Send + Sync {
    async fn lire(&self, cle: &str) -> Option<String>;
}
```

| Forme | `Send` garanti | `dyn` possible | Allocation |
|---|---|---|---|
| `async fn` natif | non | non | non |
| `-> impl Future + Send` | oui | non | non |
| `Pin<Box<dyn Future>>` / `async-trait` | oui | oui | une par appel |

Runique utilise `async-trait` pour `FormField` (`forms/base.rs`) : les champs d'un formulaire sont des `Box<dyn FormField>` (voir `box-dynamiques`) et certaines de leurs méthodes sont asynchrones, ce qui exige la forme 3. Dans `db/adb.rs`, il sert à implémenter `ConnectionTrait` et `TransactionTrait`, des traits de SeaORM déclarés eux-mêmes avec `async-trait`.

## 12. Syntaxe complète et ambiguïtés

### 12.1 Fonction associée de même nom dans deux traits

```rust
trait Animal { fn nom() -> String; }

struct Chien;
impl Chien { fn nom() -> String { "Médor".into() } }
impl Animal for Chien { fn nom() -> String { "chien".into() } }

fn main() {
    Chien::nom();                     // "Médor" : la fonction propre au type
    // Animal::nom();                 // ERREUR : pour quel type ?
    <Chien as Animal>::nom();         // "chien"
}
```

### 12.2 Dans un générique

```rust
fn description<T: Animal>() -> String {
    <T as Animal>::nom()              // ou simplement T::nom() si aucune ambiguïté
}
```

### 12.3 Type associé d'un trait précis

```rust
fn premier<I: Iterator>(mut it: I) -> Option<<I as Iterator>::Item> {
    it.next()
}
```

`<I as Iterator>::Item` s'écrit d'habitude `I::Item` ; la forme complète sert quand `I` implémente deux traits ayant chacun un type `Item`.

## 13. Implémenter un trait pour `&T`, `Box<T>`, `Vec<T>`

### 13.1 Transmettre l'implémentation

```rust
trait Forme { fn aire(&self) -> f64; }

impl<T: Forme + ?Sized> Forme for &T {
    fn aire(&self) -> f64 { (**self).aire() }
}

impl<T: Forme + ?Sized> Forme for Box<T> {
    fn aire(&self) -> f64 { (**self).aire() }
}

fn total(formes: &[impl Forme]) -> f64 {
    formes.iter().map(|f| f.aire()).sum()
}
```

Avec ces deux implémentations, `total` accepte un `Vec<Carre>`, un `Vec<&Carre>` et un `Vec<Box<dyn Forme>>`.

### 13.2 Pour une collection

```rust
impl<T: Forme> Forme for Vec<T> {
    fn aire(&self) -> f64 { self.iter().map(|f| f.aire()).sum() }   // une liste de formes est une forme
}
```

Possible parce que `Forme` est local (règle de l'orphelin, voir `traits-basics`).

## 14. Types associés génériques (GAT)

Depuis Rust 1.65, un type associé peut avoir ses propres paramètres, notamment une durée de vie.

```rust
trait Preteur {
    type Vue<'a> where Self: 'a;                   // le type dépend de la durée de l'emprunt
    fn voir<'a>(&'a self) -> Self::Vue<'a>;
}

struct Texte(String);

impl Preteur for Texte {
    type Vue<'a> = &'a str;
    fn voir<'a>(&'a self) -> &'a str { &self.0 }
}
```

À quoi ça sert : des traits qui renvoient des vues empruntées dans `self` (itérateurs qui prêtent leurs éléments, collections génériques). C'est un outil de bibliothèque ; dans du code applicatif, on en a rarement besoin.

## 15. La même API écrite de plusieurs façons

Une file de tâches qui accepte plusieurs sortes de tâches.

```rust
// 1. Enum : les sortes de tâches sont connues
enum Tache { Email(String), Nettoyage }
fn executer1(t: &Tache) {
    match t {
        Tache::Email(a) => println!("email à {a}"),
        Tache::Nettoyage => println!("nettoyage"),
    }
}

// 2. Trait + générique : chaque file contient une seule sorte
trait Executable { fn executer(&self); }
struct File<T: Executable> { taches: Vec<T> }
impl<T: Executable> File<T> {
    fn tout(&self) { self.taches.iter().for_each(|t| t.executer()); }
}

// 3. Trait object : sortes mélangées, extensible par l'utilisateur
struct FileDyn { taches: Vec<Box<dyn Executable + Send>> }

// 4. Trait scellé + trait object : sortes mélangées, liste FERMÉE garantie
mod scelle { pub trait Sceau {} }
pub trait TacheInterne: scelle::Sceau + Send { fn executer(&self); }
```

| Version | Types mélangés | Extensible hors de la crate | Exhaustivité vérifiée |
|---|---|---|---|
| enum | oui | non | oui (`match`) |
| générique | non | oui | — |
| trait object | oui | oui | — |
| trait scellé | oui | non | — |

## 16. Exercices pratiques

### Exercice 1 : une forme avec type associé

Crée un trait `Forme` avec un type associé `Mesure` (le type de l'aire : `f64` pour les formes réelles, `u64` pour une grille de pixels) et deux implémentations.

```rust
// Solution :
trait Forme {
    type Mesure;
    fn aire(&self) -> Self::Mesure;
}

struct Cercle(f64);
struct RectanglePixels { l: u64, h: u64 }

impl Forme for Cercle {
    type Mesure = f64;
    fn aire(&self) -> f64 { std::f64::consts::PI * self.0 * self.0 }
}

impl Forme for RectanglePixels {
    type Mesure = u64;
    fn aire(&self) -> u64 { self.l * self.h }
}
```

### Exercice 2 : opérateurs sur un montant

Écris `Montant(i64)` (en centimes) avec `+`, `-`, `+=`, `Montant * i64`, et un `Display` « 12,50 € ».

```rust
// Solution :
use std::fmt;
use std::ops::{Add, AddAssign, Mul, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Montant(i64);

impl Add for Montant { type Output = Montant; fn add(self, o: Montant) -> Montant { Montant(self.0 + o.0) } }
impl Sub for Montant { type Output = Montant; fn sub(self, o: Montant) -> Montant { Montant(self.0 - o.0) } }
impl AddAssign for Montant { fn add_assign(&mut self, o: Montant) { self.0 += o.0; } }
impl Mul<i64> for Montant { type Output = Montant; fn mul(self, k: i64) -> Montant { Montant(self.0 * k) } }

impl fmt::Display for Montant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let signe = if self.0 < 0 { "-" } else { "" };
        let a = self.0.abs();
        write!(f, "{signe}{},{:02} €", a / 100, a % 100)
    }
}
```

### Exercice 3 : typestate

Écris un `Formulaire<Etat>` avec deux états `NonValide` et `Valide` : `valider(self) -> Result<Formulaire<Valide>, String>` n'existe que pour `NonValide`, `enregistrer(&self)` que pour `Valide`.

```rust
// Solution :
use std::marker::PhantomData;

struct NonValide;
struct Valide;

struct Formulaire<E> { email: String, _e: PhantomData<E> }

impl Formulaire<NonValide> {
    fn new(email: &str) -> Self { Self { email: email.into(), _e: PhantomData } }
    fn valider(self) -> Result<Formulaire<Valide>, String> {
        if self.email.contains('@') {
            Ok(Formulaire { email: self.email, _e: PhantomData })
        } else {
            Err("email invalide".into())
        }
    }
}

impl Formulaire<Valide> {
    fn enregistrer(&self) { println!("enregistré : {}", self.email); }
}
```

## 17. Aide-mémoire

```rust
trait T { type A; fn f(&self) -> Self::A; }          // type associé
fn g<I: Iterator<Item = u32>>(i: I)                    // contrainte sur type associé
trait Add<Rhs = Self> { type Output; .. }              // paramètre par défaut
impl Add for X { type Output = X; fn add(self, o: X) -> X { .. } }
impl Index<usize> for X { type Output = Y; fn index(&self, i: usize) -> &Y { .. } }
trait A: Display + Send {}                             // supertraits
impl<T: Display> MonTrait for T {}                     // implémentation globale
trait Marqueur {}                                      // trait marqueur
mod prive { pub trait Sceau {} } pub trait S: prive::Sceau {}   // trait scellé
struct X<Etat> { _e: PhantomData<Etat> }               // typestate
#[diagnostic::on_unimplemented(message = "..")]
trait A { async fn f(&self); }                         // async natif
fn f(&self) -> impl Future<Output = T> + Send;        // async + Send
<Type as Trait>::fonction()                            // syntaxe complète
type Vue<'a> where Self: 'a;                           // GAT
```

### Points clés

- Type associé si le type est déterminé par l'implémentation, paramètre générique si plusieurs implémentations ont un sens.
- Les opérateurs sont des traits de `std::ops` ; ne surcharge que ce qui se comporte comme l'opérateur attendu.
- Un supertrait exige un autre trait ; ce n'est pas de l'héritage de code.
- Une implémentation globale occupe tous les types concernés.
- Un trait scellé se laisse utiliser mais pas implémenter hors de sa crate.
- Le typestate transforme une erreur d'exécution en erreur de compilation ; `on_unimplemented` rend le message lisible.
- `async fn` dans un trait : natif sans `Send` ni `dyn` ; `impl Future + Send` pour `Send` ; `async-trait` pour `dyn`.
