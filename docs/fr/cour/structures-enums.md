# Structures et Enums en Rust

**Modéliser ses données** — structs, enums, `Option`, `Result`, méthodes et traits dérivés, avec les différentes façons de construire, transformer et lire chaque valeur.

## Objectifs du cours

À la fin de ce cours, tu sauras :

- Définir les trois formes de struct (à champs nommés, tuple, unité) et savoir laquelle choisir
- Construire une valeur de toutes les façons : littéral, raccourcis, mise à jour, `new`, `Default`, builder
- Écrire des méthodes qui lisent, modifient ou consomment
- Dériver les traits standard et savoir ce que chacun permet
- Définir des enums avec ou sans données, avec discriminants, récursifs
- Manipuler `Option` et `Result` avec `match`, `if let`, `?` et leurs méthodes
- Choisir entre struct, enum et booléens pour représenter un état

## Table des matières

1. Les structs à champs nommés
2. Construire une struct
3. Méthodes et fonctions associées
4. Tuple structs et newtype
5. Unit structs
6. Traits dérivés
7. Les enums
8. Enums avec données
9. `Option<T>`
10. `Result<T, E>`
11. Lire une enum : toutes les écritures
12. Bien modéliser
13. Exemples pratiques
14. Exercices
15. Aide-mémoire

## 1. Les structs à champs nommés

### 1.1 — Définition

```rust
struct Utilisateur {
    nom: String,
    email: String,
    age: u32,
    actif: bool,
}
```

Noms de type en `PascalCase`, champs en `snake_case`. Une struct regroupe des valeurs qui vont **ensemble** : elle se passe, se renvoie et se stocke comme un tout.

### 1.2 — Accès aux champs

```rust
let mut u = Utilisateur {
    nom: String::from("Alice"),
    email: String::from("alice@example.com"),
    age: 30,
    actif: true,
};

println!("{}", u.nom);
u.age += 1;          // il faut que toute la variable soit mut
```

La mutabilité porte sur **toute** la variable : on ne peut pas rendre un seul champ modifiable. Pour cela, il faut une méthode qui ne modifie que ce champ, ou la mutabilité interne (`Cell`, `RefCell`, voir `smart-pointers`).

### 1.3 — Visibilité des champs

```rust
pub struct Compte {
    pub nom: String,     // lisible et modifiable hors du module
    solde: i64,          // privé : seul le module qui définit Compte y accède
}

impl Compte {
    pub fn new(nom: String) -> Self {
        Self { nom, solde: 0 }
    }
    pub fn solde(&self) -> i64 {
        self.solde
    }
}
```

À quoi ça sert : un champ privé garantit un invariant. Ici, personne ne peut fixer un solde arbitraire : il faut passer par les méthodes. Conséquence : un seul champ privé empêche de construire la struct avec un littéral hors du module ; il faut un constructeur.

### 1.4 — Struct qui contient des références

```rust
struct Extrait<'a> {
    texte: &'a str,
}
```

Une struct qui emprunte doit déclarer une durée de vie : elle ne peut pas vivre plus longtemps que ce qu'elle emprunte. Voir le cours `lifetimes`. Pour débuter, préfère des champs possédés (`String`, `Vec`).

## 2. Construire une struct

### 2.1 — Le littéral complet

```rust
let u = Utilisateur {
    nom: String::from("Alice"),
    email: String::from("a@ex.com"),
    age: 30,
    actif: true,
};
```

Tous les champs sont obligatoires, dans n'importe quel ordre.

### 2.2 — Raccourci quand la variable porte le nom du champ

```rust
fn creer(nom: String, email: String) -> Utilisateur {
    // Écriture longue
    Utilisateur { nom: nom, email: email, age: 0, actif: true }
}

fn creer_court(nom: String, email: String) -> Utilisateur {
    // Raccourci
    Utilisateur { nom, email, age: 0, actif: true }
}
```

### 2.3 — Mise à jour à partir d'une autre valeur

```rust
let u1 = Utilisateur { nom: "A".into(), email: "a@ex.com".into(), age: 30, actif: true };

let u2 = Utilisateur {
    email: String::from("b@ex.com"),
    ..u1                          // les autres champs viennent de u1
};

// println!("{}", u1.nom);        // ERREUR : nom a été DÉPLACÉ vers u2
println!("{}", u1.age);           // OK : age est Copy, il a été copié
```

`..autre` déplace les champs non `Copy`. Pour garder `u1` entier, écris `..u1.clone()`.

### 2.4 — Constructeur `new`

```rust
impl Utilisateur {
    pub fn new(nom: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            nom: nom.into(),
            email: email.into(),
            age: 0,
            actif: true,
        }
    }
}

let u = Utilisateur::new("Alice", "a@ex.com");
```

`new` n'est pas un mot-clé, seulement une convention. On écrit d'autres constructeurs avec des noms parlants : `Utilisateur::invite()`, `Couleur::from_hex("#fff")`.

### 2.5 — Constructeur qui peut échouer

```rust
pub struct Email(String);

impl Email {
    pub fn parse(texte: &str) -> Result<Self, String> {
        if texte.contains('@') {
            Ok(Email(texte.to_string()))
        } else {
            Err(format!("adresse invalide : {texte}"))
        }
    }
}
```

À quoi ça sert : une fois construit, un `Email` est **forcément** valide. Toute fonction qui reçoit un `Email` n'a plus à vérifier. C'est le principe « parse, don't validate ».

### 2.6 — `Default`

```rust
#[derive(Default, Debug)]
struct Options {
    port: u16,        // 0
    hote: String,     // ""
    verbeux: bool,    // false
}

let o = Options::default();
let o2 = Options { port: 8080, ..Default::default() };   // seulement ce qui change
```

Écrire `Default` à la main quand les valeurs par défaut ne sont pas les « zéros » :

```rust
struct Config { port: u16, delai_s: u64 }

impl Default for Config {
    fn default() -> Self {
        Self { port: 8080, delai_s: 30 }
    }
}
```

Dériver `Default` sur une struct de limites donne 0 partout, ce qui peut tout bloquer (une taille maximale de 0). Dans ce cas, `Default` s'écrit toujours à la main.

### 2.7 — Builder

```rust
pub struct Requete { url: String, delai: u64, essais: u32 }

pub struct RequeteBuilder { url: String, delai: u64, essais: u32 }

impl RequeteBuilder {
    pub fn new(url: &str) -> Self {
        Self { url: url.to_string(), delai: 30, essais: 1 }
    }
    pub fn delai(mut self, s: u64) -> Self { self.delai = s; self }
    pub fn essais(mut self, n: u32) -> Self { self.essais = n; self }
    pub fn build(self) -> Requete {
        Requete { url: self.url, delai: self.delai, essais: self.essais }
    }
}

let r = RequeteBuilder::new("https://ex.com").delai(5).essais(3).build();
```

### 2.8 — Choisir

| Situation | Construction |
|---|---|
| Peu de champs, tous significatifs | littéral ou `new` |
| Un invariant à garantir | champs privés + `new` ou `parse` qui renvoie `Result` |
| Beaucoup de champs, la plupart par défaut | `Default` + `..Default::default()` |
| Beaucoup d'options, API publique qui va évoluer | builder |

## 3. Méthodes et fonctions associées

### 3.1 — Les quatre récepteurs

```rust
struct Rectangle { largeur: u32, hauteur: u32 }

impl Rectangle {
    // Fonction associée : pas de self, appelée avec ::
    fn carre(cote: u32) -> Self {
        Self { largeur: cote, hauteur: cote }
    }

    // &self : lit
    fn aire(&self) -> u32 {
        self.largeur * self.hauteur
    }

    // &mut self : modifie
    fn agrandir(&mut self, n: u32) {
        self.largeur += n;
        self.hauteur += n;
    }

    // self : consomme et renvoie autre chose
    fn en_tuple(self) -> (u32, u32) {
        (self.largeur, self.hauteur)
    }
}

fn main() {
    let mut r = Rectangle::carre(3);
    let a = r.aire();
    r.agrandir(1);
    let (l, h) = r.en_tuple();
    // r.aire();        // ERREUR : r a été consommé
}
```

| Récepteur | L'appelant après l'appel | Usage typique |
|---|---|---|
| aucun | — | constructeurs, utilitaires |
| `&self` | garde la valeur intacte | getters, calculs |
| `&mut self` | garde la valeur, modifiée | setters, ajout d'éléments |
| `self` | perd la valeur | conversions (`into_*`), builders, fermeture d'une ressource |

### 3.2 — Appel par méthode ou par chemin

```rust
let r = Rectangle::carre(2);
let a1 = r.aire();               // syntaxe méthode
let a2 = Rectangle::aire(&r);    // syntaxe de chemin : la même fonction
```

La syntaxe méthode prend automatiquement la référence (`&r` ou `&mut r`) selon ce que demande la méthode. La syntaxe de chemin sert à lever une ambiguïté quand deux traits ont une méthode du même nom.

### 3.3 — Plusieurs blocs `impl`

```rust
impl Rectangle {
    fn largeur(&self) -> u32 { self.largeur }
}

impl Rectangle {
    fn hauteur(&self) -> u32 { self.hauteur }
}
```

Autorisé : utile pour séparer des méthodes par thème, ou mettre certaines sous `#[cfg(feature = "...")]`.

### 3.4 — Constantes associées

```rust
impl Rectangle {
    const MAX_COTE: u32 = 10_000;

    fn valide(&self) -> bool {
        self.largeur <= Self::MAX_COTE && self.hauteur <= Self::MAX_COTE
    }
}
```

### 3.5 — `Self`

Dans un bloc `impl`, `Self` désigne le type courant. `Self { .. }`, `-> Self` et `Self::CONSTANTE` évitent de répéter le nom, et restent justes si on renomme le type.

### 3.6 — Getter : par référence ou par copie

```rust
pub struct Profil { nom: String, age: u32 }

impl Profil {
    pub fn nom(&self) -> &str { &self.nom }   // référence : pas de copie
    pub fn age(&self) -> u32 { self.age }     // Copy : renvoyer la valeur
    pub fn nom_owned(&self) -> String { self.nom.clone() }  // seulement si l'appelant doit posséder
}
```

En Rust, un getter s'appelle comme le champ (`nom()`, pas `get_nom()`).

## 4. Tuple structs et newtype

### 4.1 — Définition

```rust
struct Couleur(u8, u8, u8);
struct Point(i32, i32);

let noir = Couleur(0, 0, 0);
let rouge = noir.0;
let Point(x, y) = Point(1, 2);
```

`Couleur` et `Point` sont deux types **différents** même si leurs champs ont des types proches.

### 4.2 — Le newtype : un type pour un sens

```rust
struct IdUtilisateur(u32);
struct IdCommande(u32);

fn annuler(commande: IdCommande) {}

fn main() {
    let u = IdUtilisateur(5);
    // annuler(u);          // ERREUR : expected IdCommande, found IdUtilisateur
    annuler(IdCommande(12));
}
```

À quoi ça sert :

- empêcher de mélanger deux valeurs de même type brut (un identifiant d'utilisateur passé à la place d'un identifiant de commande) ;
- attacher une validation (`Email(String)` construit par `parse`) ;
- implémenter un trait étranger sur un type étranger (règle de l'orphelin, voir `traits-avances`) ;
- masquer une représentation : `struct Mdp(String)` sans `Debug` dérivé ne s'affiche pas par accident dans les logs.

Le newtype ne coûte rien à l'exécution : `IdUtilisateur` occupe exactement la place d'un `u32`.

### 4.3 — Accéder à la valeur d'un newtype

```rust
struct Metres(f64);

impl Metres {
    fn valeur(&self) -> f64 { self.0 }
}

impl From<Metres> for f64 {
    fn from(m: Metres) -> f64 { m.0 }
}

let m = Metres(3.0);
let a = m.0;            // champ direct (si visible)
let b = m.valeur();     // méthode
let c: f64 = m.into();  // conversion
```

## 5. Unit structs

```rust
struct Marqueur;

struct ModeTest;
struct ModeProd;

trait Mode { const NOM: &'static str; }
impl Mode for ModeTest { const NOM: &'static str = "test"; }
impl Mode for ModeProd { const NOM: &'static str = "prod"; }
```

Une struct sans champ n'occupe aucune place. Elle sert à porter un trait ou un type sans donnée : marqueur de type, implémentation d'un comportement (un `struct Argon2Hasher;` qui implémente un trait de hachage), état d'un builder vérifié à la compilation.

## 6. Traits dérivés

### 6.1 — Ce que permet chaque `derive`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
struct Version {
    majeur: u32,
    mineur: u32,
}
```

| Trait | Ce qu'il permet | Condition |
|---|---|---|
| `Debug` | `{:?}`, `dbg!`, `assert_eq!` | tous les champs `Debug` |
| `Clone` | `.clone()` : copie explicite | tous les champs `Clone` |
| `Copy` | copie implicite à l'affectation | `Clone` aussi, champs `Copy` (pas de `String`, `Vec`) |
| `PartialEq` | `==`, `!=` | champs `PartialEq` |
| `Eq` | égalité totale (exigée par `HashMap`) | pas de `f32`/`f64` |
| `Hash` | clé de `HashMap`, élément de `HashSet` | avec `Eq` |
| `PartialOrd` | `<`, `>`, `<=`, `>=` | |
| `Ord` | `.sort()`, `.max()`, clé de `BTreeMap` | avec `Eq` et `PartialOrd` |
| `Default` | `::default()` | champs `Default` |

### 6.2 — Comment la comparaison dérivée fonctionne

```rust
#[derive(PartialEq, PartialOrd)]
struct Version { majeur: u32, mineur: u32 }

let a = Version { majeur: 1, mineur: 9 };
let b = Version { majeur: 2, mineur: 0 };
assert!(a < b);     // compare majeur d'abord, puis mineur : ordre des champs
```

L'ordre dérivé suit **l'ordre de déclaration** des champs, et pour une enum, l'ordre des variantes.

### 6.3 — Égalité à la main

```rust
struct Utilisateur { id: u32, nom: String }

impl PartialEq for Utilisateur {
    fn eq(&self, autre: &Self) -> bool {
        self.id == autre.id       // deux utilisateurs sont égaux s'ils ont le même id
    }
}
```

### 6.4 — `Display` : l'affichage pour l'utilisateur

`Display` ne se dérive pas : il s'écrit.

```rust
use std::fmt;

struct Version { majeur: u32, mineur: u32 }

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.majeur, self.mineur)
    }
}

let v = Version { majeur: 4, mineur: 0 };
println!("{v}");             // 4.0
let texte = v.to_string();   // to_string vient avec Display
```

### 6.5 — `Clone` ou `Copy` ?

`Copy` est réservé aux petites valeurs sans ressource (nombres, coordonnées, identifiants). Dès qu'un champ alloue de la mémoire, seul `Clone` est possible, et l'appel `.clone()` reste visible dans le code : c'est voulu, car une copie a un coût.

## 7. Les enums

### 7.1 — Définition simple

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
enum Direction {
    Nord,
    Sud,
    Est,
    Ouest,
}

let d = Direction::Nord;
```

Une enum est un type dont la valeur est **exactement une** des variantes.

### 7.2 — Importer les variantes

```rust
use Direction::*;          // à éviter hors d'un petit bloc local

fn opposee(d: Direction) -> Direction {
    match d {
        Nord => Sud,
        Sud => Nord,
        Est => Ouest,
        Ouest => Est,
    }
}
```

Piège : si tu écris une variante mal orthographiée dans un `match` avec `use Direction::*`, elle devient une **variable** qui attrape tout. Le compilateur avertit, mais il vaut mieux écrire `Direction::Nord` ou `Self::Nord`.

### 7.3 — Discriminants

```rust
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum Statut {
    Brouillon = 0,
    Publie = 1,
    Archive = 9,
}

let code = Statut::Archive as u8;   // 9
```

Le sens inverse (`u8` vers `Statut`) n'existe pas automatiquement : une valeur comme `5` n'aurait pas de variante. On l'écrit avec `TryFrom` :

```rust
impl TryFrom<u8> for Statut {
    type Error = String;
    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Statut::Brouillon),
            1 => Ok(Statut::Publie),
            9 => Ok(Statut::Archive),
            _ => Err(format!("statut inconnu : {v}")),
        }
    }
}

let s = Statut::try_from(1);    // Ok(Publie)
let s: Result<Statut, _> = 5_u8.try_into();   // Err ; le suffixe u8 dit quelle conversion utiliser
```

### 7.4 — Méthodes sur une enum

```rust
impl Direction {
    fn est_verticale(self) -> bool {
        matches!(self, Direction::Nord | Direction::Sud)
    }

    const TOUTES: [Direction; 4] = [Direction::Nord, Direction::Sud, Direction::Est, Direction::Ouest];
}

for d in Direction::TOUTES {
    println!("{d:?} {}", d.est_verticale());
}
```

Rust ne fournit pas de moyen standard d'itérer sur les variantes : une constante comme `TOUTES` est la solution simple (ou la crate `strum`).

## 8. Enums avec données

### 8.1 — Trois formes de variantes

```rust
enum Message {
    Quitter,                           // sans donnée
    Deplacer { x: i32, y: i32 },       // champs nommés
    Ecrire(String),                    // tuple
    Couleur(u8, u8, u8),
}

let m1 = Message::Quitter;
let m2 = Message::Deplacer { x: 10, y: 20 };
let m3 = Message::Ecrire(String::from("salut"));
```

Chaque variante porte ses propres données. Le compilateur garantit qu'on ne lit `x` que si la valeur est bien un `Deplacer`.

### 8.2 — Taille en mémoire

Une enum occupe la taille de **sa plus grande variante**, plus de quoi savoir laquelle est active. Une variante très grosse grossit toutes les valeurs de l'enum : on la met alors dans une `Box`.

```rust
enum Evenement {
    Clic(u16, u16),
    Rapport(Box<[u8; 4096]>),   // sans Box, chaque Clic pèserait 4 Ko
}
```

### 8.3 — Enum récursive

```rust
enum Expr {
    Nombre(f64),
    Somme(Box<Expr>, Box<Expr>),
    Produit(Box<Expr>, Box<Expr>),
}

fn evaluer(e: &Expr) -> f64 {
    match e {
        Expr::Nombre(n) => *n,
        Expr::Somme(a, b) => evaluer(a) + evaluer(b),
        Expr::Produit(a, b) => evaluer(a) * evaluer(b),
    }
}

let e = Expr::Somme(Box::new(Expr::Nombre(2.0)), Box::new(Expr::Nombre(3.0)));
println!("{}", evaluer(&e)); // 5
```

Sans `Box`, la taille de `Expr` serait infinie : `Box` stocke la sous-expression ailleurs et ne garde qu'un pointeur.

### 8.4 — Conversion vers une enum

```rust
enum Valeur { Entier(i64), Texte(String) }

impl From<i64> for Valeur {
    fn from(n: i64) -> Self { Valeur::Entier(n) }
}
impl From<&str> for Valeur {
    fn from(s: &str) -> Self { Valeur::Texte(s.to_string()) }
}

let v: Valeur = 5.into();
let t: Valeur = "abc".into();
```

### 8.5 — `#[non_exhaustive]`

```rust
#[non_exhaustive]
pub enum ErreurApi {
    Introuvable,
    Interdit,
}
```

Hors de la crate qui la définit, un `match` sur `ErreurApi` doit avoir un bras `_`. À quoi ça sert : une bibliothèque peut ajouter une variante plus tard sans que ce soit une rupture de compatibilité. À l'intérieur de la crate, le `match` reste exhaustif.

## 9. `Option<T>`

### 9.1 — Définition

```rust
enum Option<T> {
    None,
    Some(T),
}
```

`Option` remplace `null` : une valeur qui peut être absente a le type `Option<T>`, et on ne peut pas utiliser le `T` sans traiter le `None`.

### 9.2 — Extraire la valeur : toutes les façons

```rust
let x: Option<i32> = Some(5);

// 1. match : tous les cas, le plus explicite
let v = match x {
    Some(n) => n,
    None => 0,
};

// 2. if let : un seul cas t'intéresse
if let Some(n) = x {
    println!("{n}");
}

// 3. let else : extraire ou sortir de la fonction
// let Some(n) = x else { return; };

// 4. Valeur de secours
let v = x.unwrap_or(0);                  // valeur calculée dans tous les cas
let v = x.unwrap_or_else(|| calcul());   // calculée seulement si None
let v = x.unwrap_or_default();           // Default du type (0, "", vec vide)

// 5. Propagation avec ? (dans une fonction qui renvoie Option)
// let n = x?;

// 6. Panique si None (tests, prototypes, cas réellement impossibles)
let v = x.unwrap();
let v = x.expect("x est toujours défini après l'initialisation");

fn calcul() -> i32 { 0 }
```

| Forme | Quand l'utiliser |
|---|---|
| `match` | les deux cas font des choses différentes et longues |
| `if let` | seul `Some` déclenche une action |
| `let ... else` | `None` doit arrêter la fonction |
| `unwrap_or*` | une valeur de secours a du sens |
| `?` | l'appelant doit décider |
| `expect` | `None` serait un bug : le message dit pourquoi c'est impossible |
| `unwrap` | tests et prototypes |

### 9.3 — Transformer sans extraire

```rust
let nom: Option<&str> = Some("alice");

let longueur = nom.map(|n| n.len());                     // Some(5)
let majuscule = nom.map(str::to_uppercase);              // Some("ALICE")
let court = nom.filter(|n| n.len() < 3);                 // None
let id = nom.and_then(|n| n.parse::<u32>().ok());        // None : and_then pour une étape qui renvoie Option
let defaut = nom.or(Some("invité"));
let res: Result<&str, &str> = nom.ok_or("nom manquant"); // Option -> Result
let tout = nom.map_or(0, |n| n.len());                    // transforme ou valeur de secours
```

### 9.4 — Tester

```rust
let x = Some(5);
x.is_some();
x.is_none();
x.is_some_and(|n| n > 3);     // true
x.is_none_or(|n| n > 3);      // true si None ou si la condition est vraie
```

### 9.5 — Emprunter le contenu

```rust
let nom: Option<String> = Some(String::from("alice"));

let r: Option<&String> = nom.as_ref();     // sans déplacer nom
let s: Option<&str> = nom.as_deref();      // Option<String> -> Option<&str>

let mut pris = Some(3);
let ancien = pris.take();                  // pris devient None, ancien = Some(3)
let mut slot = Some(1);
let precedent = slot.replace(2);           // slot = Some(2), precedent = Some(1)
let mut cache: Option<Vec<i32>> = None;
let v = cache.get_or_insert_with(Vec::new); // crée si absent, renvoie &mut
v.push(1);
```

`as_deref` est la façon courante de passer un `Option<String>` à une fonction qui attend `Option<&str>`.

### 9.6 — Combiner plusieurs `Option`

```rust
let a = Some(2);
let b = Some(3);

// 1. match sur le tuple
let somme = match (a, b) {
    (Some(x), Some(y)) => Some(x + y),
    _ => None,
};

// 2. zip
let somme = a.zip(b).map(|(x, y)| x + y);

// 3. ? dans une fonction
fn additionner(a: Option<i32>, b: Option<i32>) -> Option<i32> {
    Some(a? + b?)
}
```

## 10. `Result<T, E>`

### 10.1 — Définition

```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

`Result` porte soit la valeur, soit **la raison** de l'échec. Il est marqué `#[must_use]` : ignorer un `Result` provoque un avertissement.

### 10.2 — Extraire et transformer

```rust
let r: Result<i32, String> = "42".parse::<i32>().map_err(|e| e.to_string());

let v = match &r {
    Ok(n) => *n,
    Err(e) => { eprintln!("{e}"); 0 }
};

let v = r.clone().unwrap_or(0);
let v = r.clone().unwrap_or_else(|e| { eprintln!("{e}"); 0 });
let double = r.clone().map(|n| n * 2);                  // transforme Ok
let msg = r.clone().map_err(|e| format!("erreur : {e}")); // transforme Err
let suite = r.clone().and_then(|n| if n > 0 { Ok(n) } else { Err("négatif".into()) });
let opt: Option<i32> = r.clone().ok();                  // Result -> Option (l'erreur est perdue)
let est_ok = r.is_ok();
```

### 10.3 — `?` : propager

```rust
fn lire_age(texte: &str) -> Result<u32, std::num::ParseIntError> {
    let age = texte.trim().parse::<u32>()?;   // Err : la fonction renvoie l'erreur tout de suite
    Ok(age)
}
```

`?` est l'équivalent de :

```rust
let age = match texte.trim().parse::<u32>() {
    Ok(v) => v,
    Err(e) => return Err(e.into()),   // into : convertit l'erreur si From existe
};
```

Les erreurs, leurs types et leur conversion sont détaillés dans `gestion-des-erreurs`.

### 10.4 — `Option` ou `Result` ?

- `Option` : l'absence est **normale** et n'a pas besoin d'explication (`v.first()`, `map.get(k)`).
- `Result` : l'échec a une **cause** que l'appelant peut vouloir connaître ou afficher (fichier introuvable, saisie invalide).

## 11. Lire une enum : toutes les écritures

La même fonction, qui renvoie le texte d'un message, écrite de cinq façons.

### 11.1 — `match` avec déstructuration

```rust
enum Message {
    Quitter,
    Deplacer { x: i32, y: i32 },
    Ecrire(String),
}

fn decrire(m: &Message) -> String {
    match m {
        Message::Quitter => "quitter".to_string(),
        Message::Deplacer { x, y } => format!("aller en {x},{y}"),
        Message::Ecrire(texte) => texte.clone(),
    }
}
```

### 11.2 — Méthode sur l'enum avec `Self`

```rust
impl Message {
    fn decrire(&self) -> String {
        match self {
            Self::Quitter => "quitter".into(),
            Self::Deplacer { x, y } => format!("aller en {x},{y}"),
            Self::Ecrire(t) => t.clone(),
        }
    }
}
```

### 11.3 — Un seul cas : `if let` ou `let else`

```rust
fn texte(m: &Message) -> Option<&str> {
    if let Message::Ecrire(t) = m {
        Some(t.as_str())
    } else {
        None
    }
}

fn texte_bis(m: &Message) -> Option<&str> {
    let Message::Ecrire(t) = m else { return None };
    Some(t.as_str())
}
```

### 11.4 — Tester la variante : `matches!`

```rust
fn est_deplacement(m: &Message) -> bool {
    matches!(m, Message::Deplacer { .. })
}

fn deplacement_lointain(m: &Message) -> bool {
    matches!(m, Message::Deplacer { x, .. } if x.abs() > 100)
}
```

### 11.5 — Champs ignorés, liaison et gardes

```rust
fn resume(m: &Message) -> &'static str {
    match m {
        Message::Deplacer { x: 0, y: 0 } => "sur place",
        Message::Deplacer { x: 0, .. } => "vertical",
        Message::Deplacer { .. } => "ailleurs",
        Message::Ecrire(t) if t.is_empty() => "message vide",
        Message::Ecrire(_) => "message",
        Message::Quitter => "fin",
    }
}
```

Tous les motifs disponibles sont dans le cours `pattern-matching`.

## 12. Bien modéliser

### 12.1 — Des booléens qui se contredisent

```rust
// Mauvais : quatre combinaisons, dont une impossible (payée ET annulée ?)
struct Commande {
    payee: bool,
    annulee: bool,
}

// Bon : une seule valeur possible à la fois
enum EtatCommande {
    EnAttente,
    Payee,
    Annulee,
}
```

Rends les états impossibles **impossibles à écrire** : le compilateur fait alors le travail de vérification.

### 12.2 — Une donnée qui n'existe que dans un état

```rust
// Mauvais : date_envoi n'a de sens que si envoyée == true
struct Colis {
    envoye: bool,
    date_envoi: Option<String>,
    suivi: Option<String>,
}

// Bon : chaque état porte exactement ses données
enum Colis2 {
    EnPreparation,
    Envoye { date: String, suivi: String },
    Livre { date: String },
}
```

### 12.3 — Struct, enum ou trait ?

| Situation | Choix |
|---|---|
| Plusieurs valeurs présentes **ensemble** | struct |
| **Une** valeur parmi un ensemble connu et fermé | enum |
| Ensemble **ouvert** : d'autres crates doivent pouvoir ajouter des cas | trait (voir `box-dynamiques`) |
| Un état avec des données communes et des données propres | struct avec un champ enum |

```rust
struct Paiement {
    montant_centimes: i64,
    date: String,
    moyen: Moyen,          // données communes + partie variable
}

enum Moyen {
    Carte { quatre_derniers: String },
    Virement { iban: String },
    Especes,
}
```

## 13. Exemples pratiques

### Exemple 1 — comptes et rôles

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Role {
    Utilisateur,
    Moderateur,
    Admin,
}

#[derive(Debug)]
struct Compte {
    id: u32,
    nom: String,
    role: Role,
}

impl Compte {
    fn new(id: u32, nom: impl Into<String>) -> Self {
        Self { id, nom: nom.into(), role: Role::Utilisateur }
    }

    fn promouvoir(&mut self, role: Role) {
        self.role = role;
    }

    fn peut_moderer(&self) -> bool {
        self.role >= Role::Moderateur      // possible grâce à PartialOrd : l'ordre des variantes
    }
}

fn main() {
    let mut c = Compte::new(1, "Alice");
    println!("{}", c.peut_moderer());     // false
    c.promouvoir(Role::Admin);
    println!("{}", c.peut_moderer());     // true
    println!("{c:?}");
}
```

Attention : la comparaison `>=` dépend de l'ordre de déclaration. Ajouter une variante au mauvais endroit change silencieusement les droits. Pour des droits, un `match` explicite est plus sûr :

```rust
impl Compte {
    fn peut_moderer_explicite(&self) -> bool {
        match self.role {
            Role::Admin | Role::Moderateur => true,
            Role::Utilisateur => false,
        }
    }
}
```

Ajouter une variante oblige alors à décider de ses droits ici, sinon le code ne compile pas.

### Exemple 2 — calculatrice avec `Result`

```rust
#[derive(Debug, Clone, Copy)]
enum Operation {
    Addition,
    Soustraction,
    Multiplication,
    Division,
}

impl TryFrom<char> for Operation {
    type Error = String;
    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '+' => Ok(Self::Addition),
            '-' => Ok(Self::Soustraction),
            '*' => Ok(Self::Multiplication),
            '/' => Ok(Self::Division),
            _ => Err(format!("opérateur inconnu : {c}")),
        }
    }
}

fn calculer(a: f64, b: f64, op: Operation) -> Result<f64, String> {
    match op {
        Operation::Addition => Ok(a + b),
        Operation::Soustraction => Ok(a - b),
        Operation::Multiplication => Ok(a * b),
        Operation::Division if b == 0.0 => Err("division par zéro".to_string()),
        Operation::Division => Ok(a / b),
    }
}

fn evaluer(a: f64, c: char, b: f64) -> Result<f64, String> {
    let op = Operation::try_from(c)?;
    calculer(a, b, op)
}

fn main() {
    match evaluer(10.0, '/', 0.0) {
        Ok(v) => println!("{v}"),
        Err(e) => println!("erreur : {e}"),
    }
}
```

## 14. Exercices

### Exercice 1 — struct `Livre`

Crée une struct `Livre` (titre, auteur, pages), un constructeur `new`, une méthode `est_long` (plus de 300 pages) et un affichage `Display` « titre, de auteur ».

```rust
// Solution :
use std::fmt;

struct Livre {
    titre: String,
    auteur: String,
    pages: u32,
}

impl Livre {
    fn new(titre: impl Into<String>, auteur: impl Into<String>, pages: u32) -> Self {
        Self { titre: titre.into(), auteur: auteur.into(), pages }
    }
    fn est_long(&self) -> bool {
        self.pages > 300
    }
}

impl fmt::Display for Livre {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}, de {}", self.titre, self.auteur)
    }
}
```

### Exercice 2 — feu tricolore

Crée une enum `Feu` (Rouge, Orange, Vert) avec une méthode `suivant` et une méthode `duree_s` (30, 5, 25).

```rust
// Solution :
#[derive(Debug, Clone, Copy, PartialEq)]
enum Feu { Rouge, Orange, Vert }

impl Feu {
    fn suivant(self) -> Self {
        match self {
            Feu::Rouge => Feu::Vert,
            Feu::Vert => Feu::Orange,
            Feu::Orange => Feu::Rouge,
        }
    }
    fn duree_s(self) -> u32 {
        match self {
            Feu::Rouge => 30,
            Feu::Orange => 5,
            Feu::Vert => 25,
        }
    }
}
```

### Exercice 3 — `Option` en trois écritures

Écris `premiere_initiale(nom: Option<&str>) -> Option<char>` avec `match`, avec `?`, puis avec `and_then`.

```rust
// Solution :
fn v1(nom: Option<&str>) -> Option<char> {
    match nom {
        Some(n) => n.chars().next(),
        None => None,
    }
}

fn v2(nom: Option<&str>) -> Option<char> {
    nom?.chars().next()
}

fn v3(nom: Option<&str>) -> Option<char> {
    nom.and_then(|n| n.chars().next())
}
```

### Exercice 4 — newtype validé

Crée `Pourcentage(u8)` qu'on ne peut construire qu'avec une valeur de 0 à 100.

```rust
// Solution :
pub struct Pourcentage(u8);

impl TryFrom<u8> for Pourcentage {
    type Error = String;
    fn try_from(v: u8) -> Result<Self, Self::Error> {
        if v <= 100 {
            Ok(Pourcentage(v))
        } else {
            Err(format!("{v} n'est pas un pourcentage"))
        }
    }
}
```

Hors du module, le champ privé empêche d'écrire `Pourcentage(150)`.

## 15. Aide-mémoire

### Structs

```rust
struct S { a: i32, b: String }      // champs nommés
struct T(i32, i32);                 // tuple struct
struct U;                           // unit struct
struct Id(u32);                     // newtype

S { a, b }                          // raccourci
S { a: 1, ..autre }                 // mise à jour (déplace les champs non Copy)
S { a: 1, ..Default::default() }

impl S {
    fn new() -> Self { .. }         // fonction associée
    fn lire(&self) {}
    fn modifier(&mut self) {}
    fn consommer(self) {}
    const MAX: i32 = 10;
}
```

### Enums

```rust
enum E { A, B(i32), C { x: i32 } }
#[repr(u8)] enum F { X = 1, Y = 2 }   // F::X as u8
#[non_exhaustive] pub enum G { .. }   // ajout de variantes sans rupture
enum Arbre { Feuille, Noeud(Box<Arbre>, Box<Arbre>) }
```

### Option et Result

```rust
opt.unwrap_or(v)   opt.unwrap_or_else(|| v)   opt.unwrap_or_default()
opt.map(f)   opt.and_then(f)   opt.filter(p)   opt.ok_or(e)   opt.zip(autre)
opt.as_ref()   opt.as_deref()   opt.take()   opt.is_some_and(p)
res.map(f)   res.map_err(f)   res.and_then(f)   res.ok()   res?
```

### Points clés

- Une struct regroupe ce qui va ensemble, une enum choisit une valeur parmi plusieurs.
- `..autre` déplace les champs non `Copy`.
- Un champ privé + un constructeur garantissent un invariant.
- Le newtype distingue des valeurs de même type brut, sans coût.
- `Default` dérivé donne des zéros : écris-le à la main quand zéro n'a pas de sens.
- `Option` remplace `null`, `Result` remplace les exceptions.
- Rends les états impossibles impossibles à écrire : une enum plutôt que des booléens.
