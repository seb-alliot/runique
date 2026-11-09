# Modules et Organisation

**Structurer un projet Rust** — modules, fichiers, visibilité, chemins, `use`, re-exports, bibliothèque et binaires, avec les différentes façons d'écrire la même organisation.

## Objectifs du cours

À la fin de ce cours, tu sauras :

- Déclarer un module dans le même fichier, dans un fichier séparé ou dans un dossier
- Distinguer `mod` (déclarer) et `use` (importer)
- Choisir le bon niveau de visibilité : privé, `pub`, `pub(crate)`, `pub(super)`, `pub(in chemin)`
- Écrire un chemin absolu ou relatif, et importer de toutes les façons
- Construire une API publique propre avec les re-exports et un prélude
- Séparer la logique en bibliothèque et l'interface en binaires

## Table des matières

1. Pourquoi des modules
2. Module dans le même fichier
3. Module dans un fichier séparé
4. Arborescence de modules
5. `mod` et `use` : la différence
6. Visibilité
7. Visibilité des structs et des enums
8. Chemins
9. `use` : toutes les formes
10. Conventions d'import
11. Re-exports et API publique
12. Modules conditionnels
13. Bibliothèque et binaires
14. Un projet complet
15. Erreurs fréquentes
16. Exercices
17. Aide-mémoire

## 1. Pourquoi des modules

Un module regroupe du code qui va ensemble et décide de ce qu'il montre au reste du programme. Il sert à :

- **organiser** : retrouver le code d'un sujet dans un seul endroit ;
- **cacher** : ce qui est privé peut changer sans casser le reste ;
- **nommer** : `users::create` et `posts::create` coexistent sans conflit.

La racine d'une crate est `src/lib.rs` (bibliothèque) ou `src/main.rs` (binaire). Tous les autres modules en descendent.

## 2. Module dans le même fichier

```rust
mod geometrie {
    pub fn aire_carre(cote: f64) -> f64 {
        cote * cote
    }

    fn aide_interne() {}          // privé : invisible hors de geometrie

    pub mod cercle {
        pub fn aire(rayon: f64) -> f64 {
            std::f64::consts::PI * rayon * rayon
        }
    }
}

fn main() {
    println!("{}", geometrie::aire_carre(2.0));
    println!("{}", geometrie::cercle::aire(1.0));
    // geometrie::aide_interne();   // ERREUR : function `aide_interne` is private
}
```

À quoi ça sert : regrouper un petit ensemble de fonctions, ou le module de tests en bas d'un fichier (`mod tests`).

## 3. Module dans un fichier séparé

### 3.1 — Déclarer, puis écrire le fichier

```rust
// src/main.rs
mod geometrie;     // dit au compilateur : le module est dans geometrie.rs

fn main() {
    geometrie::aire_carre(2.0);
}
```

```rust
// src/geometrie.rs — pas de `mod geometrie { }` autour : le fichier EST le module
pub fn aire_carre(cote: f64) -> f64 {
    cote * cote
}
```

Un fichier `.rs` qui n'est déclaré par **aucun** `mod` n'est pas compilé du tout : ses erreurs n'apparaissent même pas.

### 3.2 — Module avec sous-modules : deux dispositions

```
Disposition 1 (édition 2018+, conseillée)     Disposition 2 (historique)
src/                                          src/
├── main.rs                                   ├── main.rs
├── geometrie.rs        ← mod cercle;         └── geometrie/
└── geometrie/                                    ├── mod.rs      ← mod cercle;
    └── cercle.rs                                 └── cercle.rs
```

Les deux sont équivalentes pour le compilateur. La première évite d'avoir dix onglets `mod.rs` ouverts dans l'éditeur ; la seconde garde tout le module dans un seul dossier. Choisis-en une et garde-la dans tout le projet (on ne peut pas avoir à la fois `geometrie.rs` et `geometrie/mod.rs`).

### 3.3 — Chemin personnalisé

```rust
#[path = "outils/vieux_code.rs"]
mod compat;
```

À réserver aux cas particuliers (code généré, fichier partagé entre plusieurs crates).

## 4. Arborescence de modules

```
src/
├── lib.rs
├── auth.rs
├── auth/
│   ├── login.rs
│   └── session.rs
└── forms.rs
```

```rust
// src/lib.rs
pub mod auth;
pub mod forms;
```

```rust
// src/auth.rs
pub mod login;
mod session;          // privé : utilisable dans auth et ses sous-modules seulement
```

L'arbre obtenu :

```
crate
├── auth
│   ├── login
│   └── session   (privé)
└── forms
```

Chaque module ne voit son parent et ses frères qu'à travers des chemins (section 8).

## 5. `mod` et `use` : la différence

| | `mod nom;` | `use chemin::Nom;` |
|---|---|---|
| Rôle | **déclare** un module : le fait exister, compile son fichier | **importe** un nom existant dans la portée courante |
| Combien de fois | une seule fois dans tout le projet, dans le module parent | autant de fois que nécessaire, dans chaque fichier qui en a besoin |
| Sans lui | le fichier n'est pas compilé | il faut écrire le chemin complet |

```rust
// src/lib.rs
mod utils;            // déclaration : une seule fois

// src/forms.rs
use crate::utils::nettoyer;   // import : dans chaque fichier qui s'en sert
// mod utils;                 // ERREUR fréquente : chercherait src/forms/utils.rs
```

`mod` n'est **pas** un `#include` : on ne l'écrit pas dans chaque fichier qui utilise le module.

## 6. Visibilité

### 6.1 — Privé par défaut

Tout élément (fonction, struct, module, constante, trait) est privé par défaut : visible dans le module où il est défini **et ses descendants**.

```rust
mod parent {
    fn secret() {}

    pub mod enfant {
        pub fn appeler() {
            super::secret();      // OK : un enfant voit les éléments privés de ses ancêtres
        }
    }
}

fn main() {
    parent::enfant::appeler();    // OK
    // parent::secret();          // ERREUR : privé
}
```

La règle se lit dans un sens : un module voit vers le haut (ses ancêtres), pas vers le bas (le privé de ses enfants).

### 6.2 — Les niveaux

```rust
// Ces déclarations se trouvent dans un sous-module de crate::auth
// (pub(in ...) doit nommer un ancêtre du module courant).
pub fn partout() {}                 // visible par tous, y compris les autres crates
pub(crate) fn dans_la_crate() {}    // visible dans toute la crate, pas au-dehors
pub(super) fn chez_le_parent() {}   // visible dans le module parent
pub(in crate::auth) fn dans_auth() {}   // visible dans crate::auth et ses descendants
pub(self) fn ici() {}               // équivaut à privé
fn prive() {}
```

| Niveau | Visible par | Usage |
|---|---|---|
| privé | le module et ses descendants | détails d'implémentation |
| `pub(super)` | le module parent | aide partagée entre frères |
| `pub(in chemin)` | un sous-arbre précis | API interne d'un sous-système |
| `pub(crate)` | toute la crate | outils internes à la bibliothèque |
| `pub` | tout le monde | API publique |

### 6.3 — À quoi sert une visibilité restreinte

Tout ce qui est `pub` dans une bibliothèque publiée fait partie de son **API** : le modifier est une rupture de compatibilité. Tout ce qui est `pub(crate)` ou plus restreint peut être renommé ou supprimé librement.

Runique s'en sert pour une garantie de sécurité : la preuve d'autorisation `Authorized` a un constructeur `pub(in crate::auth::gate)`. Seul le module qui décide des droits peut en fabriquer une ; tout autre code, même dans la crate, est obligé de passer par lui.

### 6.4 — `pub` dans un module privé

```rust
mod interne {
    pub fn outil() {}
}
```

`outil` est `pub`, mais `interne` est privé : hors de la crate, on ne peut pas l'atteindre (`interne::outil` n'est pas accessible). La visibilité effective est **la plus restrictive** sur le chemin. Le compilateur ne se plaint pas : `pub` sert ici aux autres modules de la crate.

## 7. Visibilité des structs et des enums

### 7.1 — Struct : chaque champ a sa visibilité

```rust
pub mod banque {
    pub struct Compte {
        pub titulaire: String,   // public
        solde: i64,              // privé
    }

    impl Compte {
        pub fn new(titulaire: String) -> Self {
            Self { titulaire, solde: 0 }
        }
        pub fn solde(&self) -> i64 {
            self.solde
        }
    }
}

fn main() {
    let c = banque::Compte::new("Ada".into());
    println!("{}", c.titulaire);     // OK
    println!("{}", c.solde());       // OK par la méthode
    // c.solde = 1_000;              // ERREUR : champ privé
    // banque::Compte { titulaire: "x".into(), solde: 5 };   // ERREUR : impossible à construire hors du module
}
```

Rendre `Compte` public ne rend pas ses champs publics. Un seul champ privé oblige les autres modules à passer par un constructeur.

### 7.2 — Enum : tout ou rien

```rust
pub mod reseau {
    pub enum Etat {
        Connecte,       // les variantes d'une enum pub sont toutes publiques
        Deconnecte,
    }
}

let e = reseau::Etat::Connecte;
```

Une enum publique expose toutes ses variantes et toutes leurs données. Pour cacher une représentation, on l'enveloppe dans une struct à champ privé.

### 7.3 — Méthodes

Les méthodes ont leur propre visibilité : un `impl` sur un type public peut contenir des méthodes privées.

```rust
impl banque::Compte {
    // dans le module banque :
    // pub fn deposer(&mut self, m: i64) { self.verifier(m); self.solde += m; }
    // fn verifier(&self, m: i64) {}
}
```

## 8. Chemins

### 8.1 — Absolu ou relatif

```rust
// Absolu : depuis la racine de la crate
crate::auth::login::verifier();

// Relatif : depuis le module courant
login::verifier();          // un sous-module du module courant
self::login::verifier();    // idem, explicite

// Relatif au parent
super::forms::valider();
super::super::config();     // deux niveaux plus haut

// Autre crate (dépendance)
serde_json::to_string(&x);
::serde_json::to_string(&x);  // :: force une crate externe en cas d'ambiguïté avec un module local
```

### 8.2 — Lequel choisir

- `crate::` : stable si le fichier courant déménage ; c'est la forme la plus lisible dans un projet de taille moyenne.
- `super::` : pratique dans un sous-module qui utilise son parent, en particulier `use super::*;` dans les tests.
- `self::` : rarement nécessaire.

## 9. `use` : toutes les formes

```rust
use std::collections::HashMap;                     // un nom
use std::collections::{HashMap, HashSet};          // plusieurs noms
use std::io::{self, Read, Write};                  // le module lui-même + des éléments
use std::{fmt, collections::BTreeMap};             // imbriqué
use std::fmt::Result as FmtResult;                 // renommer
use std::io::Result as IoResult;
use std::collections::*;                           // tout (glob) : à éviter hors tests et préludes
use crate::auth::login;                            // un module de sa propre crate
use super::Config;                                 // depuis le parent
use self::interne::outil;                          // depuis un sous-module
```

### 9.1 — Portée d'un `use`

Un `use` vaut pour le **module** (ou le bloc) où il est écrit, pas pour ses sous-modules.

```rust
use std::collections::HashMap;

mod enfant {
    // HashMap n'est pas visible ici
    use std::collections::HashMap;   // il faut le réimporter
    // ou : use super::HashMap;
}

fn f() {
    use std::fmt::Write;   // import limité à la fonction
}
```

### 9.2 — Résoudre un conflit de noms

```rust
use std::fmt;
use std::io;

// Écriture 1 : garder le module en préfixe
fn a() -> fmt::Result { Ok(()) }
fn b() -> io::Result<()> { Ok(()) }

// Écriture 2 : renommer à l'import
use std::fmt::Result as FmtResult;
fn c() -> FmtResult { Ok(()) }
```

### 9.3 — Importer un trait pour ses méthodes

```rust
use std::io::Write;     // sans cet import, .write_all() n'existe pas

fn ecrire(f: &mut std::fs::File) -> std::io::Result<()> {
    f.write_all(b"bonjour")
}

use std::fmt::Write as _;   // importer le trait sans réserver son nom
```

Les méthodes d'un trait ne sont appelables que si le trait est **en portée**. `as _` sert quand un autre élément s'appelle déjà `Write` (par exemple `std::io::Write` et `std::fmt::Write` dans le même fichier).

## 10. Conventions d'import

| Élément | Convention | Exemple |
|---|---|---|
| Fonction | importer le **module parent** | `use std::fs;` puis `fs::read_to_string(..)` |
| Struct, enum, trait | importer le **type** directement | `use std::collections::HashMap;` |
| Deux types de même nom | importer les modules, ou `as` | `fmt::Result`, `io::Result` |
| Prélude d'une crate | glob | `use runique::prelude::*;` |

À quoi sert la convention sur les fonctions : `fs::read_to_string` dit d'où vient la fonction, là où `read_to_string` seul pourrait venir de n'importe où.

`cargo fmt` trie les `use` et `rustfmt.toml` peut les regrouper (`imports_granularity`, `group_imports`) : inutile d'y passer du temps à la main.

## 11. Re-exports et API publique

### 11.1 — `pub use` : exposer sous un chemin plus court

```rust
// src/lib.rs
mod forms;
mod auth;

pub use forms::fields::TextField;      // les utilisateurs écrivent ma_lib::TextField
pub use auth::login::{login, logout};
```

L'organisation interne (`forms::fields::...`) peut changer sans casser personne : seul le chemin re-exporté fait partie de l'API.

### 11.2 — Un prélude

```rust
// src/prelude.rs
pub use crate::forms::{Form, TextField};
pub use crate::auth::{login, logout};
pub use crate::Request;
```

```rust
// src/lib.rs
pub mod prelude;
```

```rust
// Chez l'utilisateur
use ma_lib::prelude::*;
```

C'est ce que fait `runique::prelude` : un seul import donne accès aux types courants. Réserve le prélude aux éléments vraiment utilisés partout, sinon il entre en conflit avec les noms de l'utilisateur.

### 11.3 — Re-exporter une dépendance

```rust
// src/lib.rs
pub use sea_orm;          // les utilisateurs écrivent ma_lib::sea_orm::...
```

À quoi ça sert : l'utilisateur n'a pas à déclarer la dépendance lui-même, et il utilise **exactement** la version que ta bibliothèque utilise. Sans cela, s'il déclare une autre version, les types de l'une ne sont pas ceux de l'autre.

### 11.4 — Masquer de la documentation

```rust
#[doc(hidden)]
pub mod __interne {
    pub fn utilise_par_les_macros() {}
}
```

Un élément doit parfois être `pub` parce qu'une macro l'appelle depuis la crate de l'utilisateur. `#[doc(hidden)]` le retire de la documentation et signale qu'il n'est pas une API stable.

## 12. Modules conditionnels

```rust
#[cfg(test)]
mod tests;                   // compilé uniquement pour cargo test

#[cfg(feature = "admin")]
pub mod admin;               // compilé uniquement avec la feature admin

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;
```

Un module désactivé n'est ni compilé ni vérifié. Les features et leurs effets sont détaillés dans `cargo-dependances`.

### Le module de tests

```rust
pub fn additionner(a: i32, b: i32) -> i32 {
    a + b
}

fn interne(x: i32) -> i32 { x * 2 }

#[cfg(test)]
mod tests {
    use super::*;          // importe tout le module parent, y compris le privé

    #[test]
    fn additionne() {
        assert_eq!(additionner(2, 3), 5);
        assert_eq!(interne(2), 4);     // les tests unitaires voient le privé
    }
}
```

## 13. Bibliothèque et binaires

### 13.1 — Un package, plusieurs crates

```
mon_projet/
├── Cargo.toml
└── src/
    ├── lib.rs          ← crate bibliothèque `mon_projet`
    ├── main.rs         ← binaire `mon_projet`
    └── bin/
        ├── import.rs   ← binaire `import`
        └── outil/
            └── main.rs ← binaire `outil` (avec ses propres modules)
```

```bash
cargo run                  # lance main.rs
cargo run --bin import
```

### 13.2 — Le binaire utilise la bibliothèque comme une dépendance

```rust
// src/main.rs
use mon_projet::config::charger;     // on passe par le nom de la crate, pas par crate::

fn main() {
    let c = charger();
}
```

Un binaire est une **crate à part** : il ne voit de la bibliothèque que ce qui est `pub`. Un élément `pub(crate)` de `lib.rs` est invisible depuis `main.rs` ou `src/bin/`. Pour qu'un outil en ligne de commande l'utilise, il doit être `pub`.

### 13.3 — Pourquoi séparer

```rust
// Écriture 1 : tout dans main.rs — impossible à tester par les tests d'intégration
// Écriture 2 : logique dans lib.rs, main.rs ne fait que lire les arguments et appeler

// src/main.rs
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Err(e) = mon_projet::lancer(&args) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
```

La seconde forme permet aux tests (`tests/*.rs`), aux autres binaires et aux autres projets de réutiliser la logique.

### 13.4 — Plusieurs crates : le workspace

Quand un projet contient plusieurs bibliothèques (par exemple un framework, ses macros procédurales et une application de démonstration), on les sépare en packages d'un même workspace. Voir `cargo-dependances`, section Workspaces.

## 14. Un projet complet

```
mon_api/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── main.rs
    ├── models.rs
    ├── models/
    │   ├── user.rs
    │   └── post.rs
    ├── api.rs
    ├── api/
    │   └── routes.rs
    └── database.rs
```

```rust
// src/lib.rs
pub mod models;
pub mod api;
mod database;                         // privé : détail d'implémentation

pub use models::{Post, User};         // API courte
pub use api::routes::configurer;
```

```rust
// src/models.rs
mod post;
mod user;

pub use post::Post;                   // les sous-modules restent privés,
pub use user::User;                   // seuls les types sont exposés
```

```rust
// src/models/user.rs
#[derive(Debug)]
pub struct User {
    pub id: u32,
    pub nom: String,
}

impl User {
    pub fn new(id: u32, nom: impl Into<String>) -> Self {
        Self { id, nom: nom.into() }
    }
}
```

```rust
// src/database.rs
use crate::models::User;

pub(crate) fn sauver(u: &User) {
    println!("sauvegarde de {}", u.nom);
}
```

```rust
// src/api/routes.rs
use crate::database;
use crate::models::User;

pub fn configurer() {
    let u = User::new(1, "Alice");
    database::sauver(&u);
}
```

```rust
// src/main.rs
use mon_api::{configurer, User};

fn main() {
    configurer();
    println!("{:?}", User::new(2, "Bob"));
}
```

Lecture : `database` est privé à la crate, `sauver` est `pub(crate)` (utilisable par `api`, invisible pour `main.rs`), `User` et `configurer` sont exposés à la racine par re-export.

## 15. Erreurs fréquentes

| Message | Cause | Correction |
|---|---|---|
| `file not found for module` | `mod x;` sans `x.rs` ni `x/mod.rs` au bon endroit | créer le fichier à côté du module parent |
| `unresolved import` | chemin faux, module non déclaré par `mod` | vérifier le `mod` dans le parent, utiliser `crate::` |
| `function is private` | élément sans `pub` | ajouter le niveau de visibilité minimal nécessaire |
| `field is private` | champ sans `pub` | passer par une méthode, ou rendre le champ `pub` |
| `no method named ... found` alors qu'elle existe | trait non importé | `use chemin::Trait;` |
| Le code d'un fichier n'est jamais vérifié | aucun `mod` ne le déclare | ajouter `mod nom;` dans le parent |
| `pub(crate)` introuvable depuis `src/bin/` | un binaire est une autre crate | rendre l'élément `pub` |

## 16. Exercices

### Exercice 1 — découper un fichier

Déplace ce code dans `src/maths.rs` et `src/maths/stats.rs`, et appelle-le depuis `main.rs`.

```rust
mod maths {
    pub fn carre(x: i32) -> i32 { x * x }
    pub mod stats {
        pub fn moyenne(v: &[f64]) -> f64 { v.iter().sum::<f64>() / v.len() as f64 }
    }
}
```

```rust
// Solution :
// src/main.rs
mod maths;
fn main() {
    println!("{}", maths::carre(3));
    println!("{}", maths::stats::moyenne(&[1.0, 2.0]));
}

// src/maths.rs
pub mod stats;
pub fn carre(x: i32) -> i32 { x * x }

// src/maths/stats.rs
pub fn moyenne(v: &[f64]) -> f64 { v.iter().sum::<f64>() / v.len() as f64 }
```

### Exercice 2 — visibilité minimale

Dans une bibliothèque, `hash` ne doit être appelable que depuis le module `auth` et ses sous-modules, `verifier` depuis toute la crate, `login` par tout le monde. Donne les déclarations.

```rust
// Solution :
pub mod auth {
    pub(in crate::auth) fn hash(mdp: &str) -> String { mdp.to_string() }
    pub(crate) fn verifier(mdp: &str, h: &str) -> bool { hash(mdp) == h }
    pub fn login(mdp: &str) -> bool { verifier(mdp, "x") }
}
```

### Exercice 3 — API courte

Les utilisateurs écrivent `ma_lib::forms::fields::text::TextField`. Fais en sorte qu'ils puissent écrire `ma_lib::TextField` et `use ma_lib::prelude::*;`, sans déplacer le fichier.

```rust
// Solution :
// src/lib.rs
pub mod forms;
pub mod prelude;
pub use forms::fields::text::TextField;

// src/prelude.rs
pub use crate::TextField;
```

## 17. Aide-mémoire

```rust
mod a;                    // déclare a (a.rs ou a/mod.rs), une seule fois, dans le parent
mod b { }                 // module en ligne
pub mod c;                // module public

use crate::a::X;          // chemin absolu
use super::Y;             // parent
use self::b::Z;           // sous-module
use std::io::{self, Read};
use std::fmt::Result as FmtResult;
use std::io::Write as _;  // trait en portée, sans réserver le nom
pub use a::X;             // re-export

pub            pub(crate)            pub(super)            pub(in crate::a)
#[cfg(test)] mod tests { use super::*; }
#[cfg(feature = "x")] pub mod x;
```

### Points clés

- `mod` déclare un module une seule fois ; `use` importe un nom autant de fois que nécessaire.
- Un fichier non déclaré par `mod` n'est pas compilé.
- Tout est privé par défaut ; un enfant voit le privé de ses ancêtres, pas l'inverse.
- La visibilité réelle est la plus restrictive sur le chemin.
- Un champ privé suffit pour imposer un constructeur.
- `pub use` découple l'organisation interne de l'API publique.
- Un binaire est une crate séparée : il ne voit que le `pub` de la bibliothèque.
- Dépendances, features et workspaces : voir `cargo-dependances`.
