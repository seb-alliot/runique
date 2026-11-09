# Macros — Visibilité et export
> Où une macro est visible, comment la rendre utilisable dans d'autres modules et d'autres crates, et comment écrire une macro qui fonctionne chez l'utilisateur quels que soient ses imports.

## Objectifs

- Comprendre la portée « textuelle » d'une `macro_rules!`
- Utiliser une macro dans d'autres modules de la même crate, de deux façons
- Exporter une macro vers les autres crates avec `#[macro_export]`
- Écrire `$crate::` et des chemins absolus pour que la macro fonctionne partout
- Cacher les fonctions et macros d'aide avec `#[doc(hidden)]`
- Réexporter des macros procédurales et les dépendances qu'elles utilisent
- Documenter et tester une macro exportée

## Table des matières

1. Portée par défaut : l'ordre du fichier
2. Utiliser une macro dans d'autres modules
3. `#[macro_export]` : vers les autres crates
4. `$crate` : un chemin qui marche partout
5. Dépendances utilisées par une macro
6. Les éléments d'aide : `#[doc(hidden)]`
7. Réexporter des macros
8. `#[macro_use]` : l'ancienne façon
9. Documenter et tester une macro exportée
10. Exemples dans Runique
11. Exercices
12. Aide-mémoire

---

## 1. Portée par défaut : l'ordre du fichier

Une `macro_rules!` sans attribut a une portée **textuelle** : elle est visible de sa déclaration jusqu'à la fin du module qui la contient, **et dans les modules déclarés après elle**.

```rust
// avant!();                    // ERREUR : la macro n'est pas encore déclarée

macro_rules! avant {
    () => { 1 };
}

fn f() -> i32 { avant!() }      // OK : après la déclaration

mod enfant {
    pub fn g() -> i32 { avant!() }   // OK : module déclaré après la macro
}
```

À la différence d'une fonction, **l'ordre compte**. Dans `lib.rs`, le module qui définit les macros doit donc être déclaré **avant** ceux qui les utilisent :

```rust
// src/lib.rs
#[macro_use]
mod macros;        // définit les macros (voir section 2.1)
mod formulaires;   // peut les utiliser
mod vues;          // aussi
```

---

## 2. Utiliser une macro dans d'autres modules

### 2.1 Façon 1 : `#[macro_use]` sur le module

```rust
// src/lib.rs
#[macro_use]
mod macros;                     // les macros de ce module restent visibles après lui
mod utilisateurs;

// src/macros.rs
macro_rules! verifier {
    ($c:expr) => { assert!($c) };
}

// src/utilisateurs.rs
pub fn creer() {
    verifier!(true);            // visible grâce au #[macro_use] et à l'ordre des mod
}
```

### 2.2 Façon 2 : un import par chemin (édition 2018+)

```rust
// src/macros.rs
macro_rules! verifier {
    ($c:expr) => { assert!($c) };
}
pub(crate) use verifier;        // la macro devient un élément ordinaire du module

// src/utilisateurs.rs
use crate::macros::verifier;    // import explicite, comme une fonction

pub fn creer() {
    verifier!(true);
}
```

`pub(crate) use nom_de_macro;` juste après la définition transforme la macro en élément importable par son chemin. L'ordre des `mod` n'a alors plus d'importance, et l'éditeur sait d'où vient la macro.

| | `#[macro_use]` sur `mod` | `pub(crate) use` |
|---|---|---|
| Dépend de l'ordre des `mod` | oui | non |
| On voit d'où vient la macro | non | oui (`use crate::macros::verifier`) |
| Pour | petits projets | code récent, projets moyens et grands |

---

## 3. `#[macro_export]` : vers les autres crates

```rust
// Dans la crate `ma_lib`, n'importe quel fichier
#[macro_export]
macro_rules! dire {
    ($msg:expr) => { println!("ma_lib : {}", $msg) };
}
```

`#[macro_export]` place la macro **à la racine de la crate**, quel que soit le module où elle est écrite. Elle s'utilise alors comme ceci :

```rust
// Dans une crate qui dépend de ma_lib
use ma_lib::dire;              // depuis la racine, jamais ma_lib::module::dire

fn main() {
    dire!("bonjour");
    ma_lib::dire!("aussi possible sans use");
}
```

Dans la crate qui la définit, on l'appelle par `crate::dire!` (ou directement par son nom, selon la portée textuelle).

Une macro `#[macro_export]` est **toujours publique** : elle fait partie de l'API de la crate. La modifier (retirer un bras, changer la syntaxe) est une rupture de compatibilité.

---

## 4. `$crate` : un chemin qui marche partout

### 4.1 Le problème

```rust
// Dans ma_lib
pub fn journal(msg: &str) { println!("{msg}"); }

#[macro_export]
macro_rules! log_mauvais {
    ($m:expr) => { journal($m) };     // résolu à l'endroit de l'APPEL, chez l'utilisateur
}
```

Chez l'utilisateur, `journal` n'existe pas (ou désigne autre chose) : la macro casse. Écrire `ma_lib::journal` ne suffit pas non plus : l'utilisateur peut avoir renommé la dépendance (`ma_lib2 = { package = "ma_lib" }`), et dans `ma_lib` elle-même, `ma_lib::` ne désigne rien.

### 4.2 La solution

```rust
#[macro_export]
macro_rules! log {
    ($m:expr) => { $crate::journal($m) };
}
```

`$crate` est remplacé par **le chemin de la crate qui définit la macro**, quel que soit l'endroit d'où elle est appelée et le nom sous lequel elle est importée. C'est la seule façon fiable de désigner les éléments de sa propre crate dans une macro exportée.

### 4.3 Toutes les références d'une macro exportée

```rust
#[macro_export]
macro_rules! nouvelle_table {
    ($($k:expr => $v:expr),*) => {{
        let mut m = ::std::collections::HashMap::new();   // std : chemin absolu ::std
        $( m.insert($k, $v); )*
        $crate::Table::from(m)                             // propre crate : $crate
    }};
}
```

| Ce que la macro utilise | Chemin à écrire |
|---|---|
| un élément de sa propre crate | `$crate::chemin::Element` |
| la bibliothèque standard | `::std::...` ou `::core::...` |
| une macro de sa propre crate | `$crate::autre_macro!` |
| une dépendance de sa crate | `$crate::__dep::...` (section 5) |
| un trait dont il faut les méthodes | appel complet : `$crate::MonTrait::methode(&x)` |

Le dernier point est un piège fréquent : `x.methode()` dans le développement ne marche que si l'utilisateur a importé le trait. `$crate::MonTrait::methode(&x)` marche toujours.

### 4.4 Récursion

Une macro exportée qui s'appelle elle-même doit le faire par `$crate::` :

```rust
#[macro_export]
macro_rules! somme {
    () => { 0 };
    ($x:expr $(, $reste:expr)*) => { $x + $crate::somme!($($reste),*) };
}
```

---

## 5. Dépendances utilisées par une macro

Si le développement d'une macro utilise une **dépendance** de ta crate (serde, sea-orm...), l'utilisateur n'a peut-être pas cette dépendance, ou pas la même version.

```rust
// Dans ma_lib (qui dépend de serde_json)
#[doc(hidden)]
pub use serde_json as __serde_json;      // réexport caché

#[macro_export]
macro_rules! json_de {
    ($v:expr) => { $crate::__serde_json::to_string(&$v) };
}
```

L'utilisateur n'a rien à déclarer dans son `Cargo.toml`, et la macro utilise **exactement** la version de `serde_json` choisie par ta bibliothèque.

Le préfixe `__` et `#[doc(hidden)]` signalent que ce chemin est interne : il n'apparaît pas dans la documentation et n'est pas considéré comme une API stable.

---

## 6. Les éléments d'aide : `#[doc(hidden)]`

Une macro exportée ne peut appeler que des éléments **publics**. Les fonctions d'aide doivent donc être `pub`, alors qu'elles ne sont pas faites pour être appelées à la main.

```rust
pub struct Routes { /* ... */ }

impl Routes {
    #[doc(hidden)]
    pub fn new() -> Self { Routes { /* ... */ } }         // appelée par la macro

    #[doc(hidden)]
    pub fn ajouter(&mut self, chemin: &str) { /* ... */ }  // appelée par la macro
}

#[macro_export]
macro_rules! routes {
    ($($chemin:literal),* $(,)?) => {{
        let mut r = $crate::Routes::new();
        $( r.ajouter($chemin); )*
        r
    }};
}
```

Même chose pour une macro d'aide : `#[macro_export] #[doc(hidden)] macro_rules! __etape { .. }`, appelée par `$crate::__etape!`.

`#[doc(hidden)]` n'est **pas** une protection : l'élément reste appelable. C'est un signal (« pas une API publique, peut changer »). Pour une vraie garantie (un constructeur qu'on ne doit pas pouvoir contourner), il faut une visibilité restreinte, et donc une conception où la macro n'en a pas besoin.

---

## 7. Réexporter des macros

### 7.1 Une macro déclarative

```rust
// src/macros/mod.rs
#[macro_export]
macro_rules! ma_macro { () => {} }

// src/lib.rs
pub mod macros;
// pub use macros::ma_macro;      // inutile : #[macro_export] la met déjà à la racine
```

Une macro `#[macro_export]` est déjà à la racine. On peut l'ajouter à un prélude :

```rust
// src/prelude.rs
pub use crate::ma_macro;
```

### 7.2 Une macro procédurale

Une macro procédurale (`derive`, attribut, function-like) vit **obligatoirement** dans une crate à part, de type `proc-macro` (voir `macros-derive`). Pour que l'utilisateur n'ait qu'une dépendance à déclarer, la bibliothèque principale la réexporte :

```toml
# ma_lib/Cargo.toml
[dependencies]
ma_lib_macros = { path = "../ma_lib_macros", version = "1.0" }
```

```rust
// ma_lib/src/lib.rs
pub use ma_lib_macros::{MonDerive, mon_attribut};

pub mod prelude {
    pub use ma_lib_macros::{MonDerive, mon_attribut};
}
```

```rust
// Chez l'utilisateur : une seule dépendance
use ma_lib::prelude::*;

#[derive(MonDerive)]
struct S;
```

### 7.3 Le code généré par une macro procédurale

Une macro procédurale ne connaît pas `$crate`. Elle génère en général des chemins `::ma_lib::...`. Deux conséquences :

- l'utilisateur doit avoir la dépendance sous le nom `ma_lib` (pas renommée) ;
- dans les tests de `ma_lib` elle-même, `::ma_lib::` ne désigne rien : on ajoute, sous `cfg(test)`, `extern crate self as ma_lib;` en tête de `lib.rs`, ce qui donne à la crate un nom pour elle-même.

---

## 8. `#[macro_use]` : l'ancienne façon

Avant l'édition 2018, on importait les macros d'une dépendance avec :

```rust
#[macro_use]
extern crate serde_derive;
```

Ce style est encore valide mais déconseillé :

```rust
// Ancien
#[macro_use]
extern crate ma_lib;
fn main() { dire!("x"); }

// Moderne : un use explicite
use ma_lib::dire;
fn main() { dire!("x"); }
```

`#[macro_use]` reste utile **sur un module** de sa propre crate (section 2.1).

---

## 9. Documenter et tester une macro exportée

### 9.1 Documentation

```rust
/// Crée une `HashMap` à partir de paires `clé => valeur`.
///
/// # Exemple
///
/// ```
/// let m = ma_lib::table! { "a" => 1, "b" => 2 };
/// assert_eq!(m["a"], 1);
/// ```
#[macro_export]
macro_rules! table {
    ($($k:expr => $v:expr),* $(,)?) => {
        ::std::collections::HashMap::from([$(($k, $v)),*])
    };
}
```

Le commentaire `///` se place **au-dessus** de `#[macro_export]`. L'exemple est exécuté par `cargo test` comme un test de documentation, depuis une crate externe : il vérifie exactement ce que verra l'utilisateur (chemins par `$crate`, imports nécessaires).

### 9.2 Tester qu'une utilisation est refusée

```rust
/// ```compile_fail
/// let x = ma_lib::table!(1, 2);     // syntaxe sans => : doit être refusée
/// ```
```

### 9.3 Tester depuis l'extérieur

Un test d'intégration (`tests/macros.rs`) utilise la macro exactement comme une autre crate : c'est là qu'on détecte un chemin oublié qui marchait par hasard dans la crate.

---

## 10. Exemples dans Runique

### 10.1 `urlpatterns!` : `$crate`, méthodes cachées

```rust
// runique/src/macros/routeur/router.rs
#[macro_export]
macro_rules! urlpatterns {
    // ...
    (@parse $routes:ident; admin $path:expr => $handler:expr, /* ... */ $($rest:tt)*) => {
        $routes.add_admin_route::<$model>($path, $name, $handler, $view);
        $crate::urlpatterns!(@parse $routes; $($rest)*);      // récursion par $crate
    };
}

// runique/src/macros/routeur/routes.rs
impl Routes {
    /// Used by `urlpatterns!`; construct route sets through the macro.
    #[doc(hidden)]
    pub fn new() -> Self { /* ... */ }

    /// Used by `urlpatterns!` to add an ordinary route.
    #[doc(hidden)]
    pub fn add_route(&mut self, path: impl AsRef<str>, handler: MethodRouter) { /* ... */ }
}
```

La macro est exportée à la racine (`runique::urlpatterns!`), s'appelle elle-même par `$crate::`, et utilise des méthodes publiques mais cachées de la documentation, que l'utilisateur n'est pas censé appeler directement.

### 10.2 Une macro d'aide cachée

```rust
// runique/src/macros/bdd/filter.rs
#[macro_export]
#[doc(hidden)]
macro_rules! search_apply_op { /* eq, icontains, gte... */ }
```

`search!` traduit des opérateurs (`eq`, `icontains`, `gte`...) en conditions SeaORM par l'intermédiaire de cette macro d'aide, exportée (pour être appelable depuis le code développé chez l'utilisateur) mais cachée.

### 10.3 Réexport des macros procédurales

```rust
// runique/src/lib.rs
pub use derive_form::{extend, model};

pub mod prelude {
    pub use derive_form::{extend, form, model};
    // ...
}
```

`model!{}`, `extend!{}` et `#[form]` vivent dans la crate `derive_form` (une crate `proc-macro`). L'utilisateur ne déclare que `runique` et les reçoit par `use runique::prelude::*`.

### 10.4 Le nom de la crate pour elle-même

```rust
// runique/src/lib.rs
// In the crate's own tests, `::runique::…` (what `model!{}` and `extend!{}`
// generate) names the crate itself.
#[cfg(test)]
extern crate self as runique;
```

Le code généré par `model!{}` contient des chemins `::runique::...`. Chez l'utilisateur, `runique` est une dépendance : tout va bien. Dans les tests du framework lui-même, cette ligne donne à la crate le nom `runique`, pour que le même code généré compile (section 7.3).

---

## 11. Exercices

### Exercice 1 — utilisable partout

Cette macro marche dans sa crate mais pas chez l'utilisateur. Corrige-la.

```rust
pub struct Compteur(pub u32);

#[macro_export]
macro_rules! compteur {
    ($n:expr) => { Compteur($n) };
}
```

```rust
// Solution :
#[macro_export]
macro_rules! compteur {
    ($n:expr) => { $crate::Compteur($n) };
}
```

### Exercice 2 — macro interne à la crate

Rends la macro `verifier!` utilisable dans tous les modules de la crate, sans dépendre de l'ordre des `mod` et sans l'exporter aux autres crates.

```rust
// Solution :
// src/macros.rs
macro_rules! verifier {
    ($c:expr, $msg:expr) => { if !$c { return Err($msg.into()); } };
}
pub(crate) use verifier;

// ailleurs
use crate::macros::verifier;
```

### Exercice 3 — appel de méthode de trait

Pourquoi cette macro exportée échoue-t-elle chez un utilisateur qui n'a pas importé `Afficher` ? Corrige-la.

```rust
pub trait Afficher { fn afficher(&self) -> String; }

#[macro_export]
macro_rules! montrer {
    ($x:expr) => { println!("{}", $x.afficher()) };
}
```

```rust
// Solution : appel complet, qui ne dépend pas des imports de l'utilisateur.
#[macro_export]
macro_rules! montrer {
    ($x:expr) => { println!("{}", $crate::Afficher::afficher(&$x)) };
}
```

## 12. Aide-mémoire

```rust
macro_rules! m { .. }                // visible après sa déclaration, dans le module et les suivants
#[macro_use] mod macros;             // garder les macros visibles après le module
pub(crate) use m;                    // importer par chemin dans la crate : use crate::macros::m;
#[macro_export] macro_rules! m { .. } // racine de la crate, pour tout le monde : use ma_lib::m;
$crate::Element                      // chemin vers sa propre crate dans le développement
::std::collections::HashMap          // std en chemin absolu
$crate::Trait::methode(&x)           // méthode de trait sans import chez l'utilisateur
#[doc(hidden)] pub use dep as __dep; // dépendance utilisée par la macro
#[doc(hidden)] pub fn aide() {}      // aide publique, cachée de la doc
pub use ma_lib_macros::MonDerive;    // réexporter une macro procédurale
#[cfg(test)] extern crate self as ma_lib;   // ::ma_lib:: dans ses propres tests
```

### Points clés

- Une `macro_rules!` dépend de l'ordre de déclaration ; `pub(crate) use` lève cette contrainte.
- `#[macro_export]` place la macro à la racine et la rend publique pour toujours.
- Dans une macro exportée, tout chemin doit être absolu : `$crate::`, `::std::`.
- Les méthodes de trait s'appellent par leur chemin complet, pas par `.methode()`.
- Les dépendances utilisées par la macro se réexportent, cachées.
- `#[doc(hidden)]` signale une aide interne, il ne protège rien.
- Les macros procédurales se réexportent depuis la crate principale.
