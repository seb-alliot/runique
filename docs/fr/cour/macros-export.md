# Macros — Visibilité et export
> Rendre une macro accessible depuis d'autres modules et crates

## Objectifs

- Comprendre pourquoi les macros ont des règles de visibilité différentes des fonctions
- Utiliser `#[macro_export]` correctement
- Référencer sa propre crate avec `$crate::`
- Réexporter des macros depuis une lib

---

## Table des matières

1. [Visibilité par défaut](#1-visibilité-par-défaut)
2. [#[macro_export]](#2-macro_export)
3. [$crate:: — référence absolue](#3-crate--référence-absolue)
4. [Réexport depuis lib.rs](#4-réexport-depuis-librs)
5. [#[macro_use] — l'ancienne façon](#5-macro_use--lancienne-façon)

---

## 1. Visibilité par défaut

Contrairement aux fonctions et types, une macro déclarée dans un module n'est **pas automatiquement accessible** dans les sous-modules ou depuis l'extérieur.

```rust
// src/utils.rs
macro_rules! dire {

    ($msg:expr) => { println!("{}", $msg); };

}

// src/main.rs
mod utils;

fn main() {
    dire!("Bonjour"); // ❌ Erreur : macro non trouvée
}
```

Les macros suivent leur propre système de portée, basé sur l'ordre de déclaration dans le fichier, pas sur la hiérarchie des modules.

---

## 2. #[macro_export]

`#[macro_export]` rend une macro disponible à la racine de la crate, comme si elle était déclarée dans `lib.rs`.

```rust
// src/utils.rs
#[macro_export]
macro_rules! dire {

    ($msg:expr) => { println!("{}", $msg); };

}

// src/main.rs
mod utils; // Pas besoin d'importer la macro explicitement

fn main() {
    dire!("Bonjour"); // ✅ Fonctionne
}
```

> **Important :** `#[macro_export]` place la macro à la racine de la crate, peu importe où elle est définie. Si ta crate s'appelle `mon_crate`, la macro est accessible via `mon_crate::dire!`.

---

## 3. $crate:: — référence absolue

Le problème : dans une macro, tu veux appeler d'autres éléments de ta crate (types, fonctions). Si tu écris juste `MaStruct`, ça peut ne pas se résoudre dans le contexte de l'appelant.

```rust
// ❌ Peut échouer si MaStruct n'est pas dans le scope de l'appelant
#[macro_export]
macro_rules! creer {

    () => { MaStruct::new() };

}

// ✅ $crate:: pointe toujours vers la crate qui définit la macro
#[macro_export]
macro_rules! creer {

    () => { $crate::MaStruct::new() };

}
```

`$crate` se résout à la crate qui contient la définition de la macro, pas celle qui l'utilise. C'est la façon correcte de référencer des items internes.

### Exemple concret

```rust
// ma_lib/src/lib.rs

pub struct Config {
    pub debug: bool,
}

impl Config {
    pub fn new() -> Self {
        Self { debug: false }
    }
}

#[macro_export]
macro_rules! config_debug {

    () => {{
        let mut c = $crate::Config::new(); // $crate = ma_lib
        c.debug = true;
        c
    }};

}
```

```rust
// autre_crate/src/main.rs
use ma_lib::config_debug;

fn main() {
    let c = config_debug!(); // ✅ Config::new() est résolu dans ma_lib
    println!("debug: {}", c.debug);
}
```

---

## 4. Réexport depuis lib.rs

Pour une lib qui expose des macros, la convention est de les réexporter depuis `lib.rs` afin que l'utilisateur n'ait qu'un seul point d'import.

```rust
// ma_lib/src/lib.rs

mod macros; // Fichier qui contient les macro_rules!

// Réexport explicite
pub use macros::*; // ⚠️ N'exporte que les items pub — les macros #[macro_export] sont déjà à la racine

// Ou via le module lui-même
pub mod macros;
```

Avec `#[macro_export]`, les macros sont automatiquement à la racine — l'utilisateur importe juste :

```rust
use ma_lib::ma_macro; // Fonctionne sans pub use supplémentaire
// ou
ma_lib::ma_macro!();  // Chemin complet
```

---

## 5. #[macro_use] — l'ancienne façon

Avant Rust 2018, on utilisait `#[macro_use]` pour importer toutes les macros d'une crate :

```rust
// Ancien style (pre-2018)
#[macro_use]
extern crate serde;

// Nouveau style (2018+)
use serde::{Serialize, Deserialize};
```

Même chose pour les modules internes :

```rust
// Ancien style
#[macro_use]
mod macros;

// Nouveau style — préférer #[macro_export] + use explicite
```

> **À retenir :** `#[macro_use]` est encore valide mais déconseillé. Utilise `use crate::ma_macro` ou `use ma_lib::ma_macro` à la place. C'est plus explicite et compatible avec rust-analyzer.
