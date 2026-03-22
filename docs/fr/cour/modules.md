## Modules et Organisation

## Structurer vos Projets Rust 

Modules, Crates et Workspaces 

## Objectifs du cours

À la fin de ce cours, tu sauras : 

- Organiser ton code en modules 

- Gérer la visibilité avec pub 

- Créer une bibliothèque (lib) 

- Utiliser Cargo efficacement 

- Structurer de gros projets 

## Table des matières

1. Les Modules (mod) 

- 1.1 - Module inline 

- 1.2 - Module dans un fichier 

- 1.3 - Hiérarchie de modules 

2. Visibilité (pub) 

- 2.1 - Privé par défaut 

- 2.2 - pub et pub(crate) 

- 2.3 - Re-exports 

3. use et chemins 

- 3.1 - Importer des items 

- 3.2 - Chemins absolus vs relatifs 

- 3.3 - use as et glob 

4. Bibliothèques vs Binaires 

- 4.1 - lib.rs vs main.rs 

- 4.2 - Créer une lib 

5. Cargo.toml 

- 5.1 - Dépendances 

- 5.2 - Features 

6. Workspaces 

7. Best practices 

8. Exemple complet 

## 1. Les Modules (mod)

Les **modules** permettent d'organiser le code en namespaces. C'est essentiel pour structurer de gros projets ! 

## 1.1 - Module inline

**`// Module défini directement dans le fichier mod network { fn connect() { println!("Connexion..."); } pub fn send_data() { connect();  // Peut appeler connect() (même module) println!("Envoi de données"); } } fn main() { // network::connect();  //`** I **`ERREUR : connect est privé network::send_data();   //`** I **`OK : send_data est pub } // Modules imbriqués mod reseau { pub mod tcp { pub fn connect() {} } pub mod udp { pub fn send() {} } } fn main() { reseau::tcp::connect(); reseau::udp::send(); }`** 

## 1.2 - Module dans un fichier

```
// Structure de fichiers :
// src/
//   main.rs
//   network.rs
// Dans main.rs :
mod network;  // Cherche network.rs
fn main() {
    network::connect();
}
// Dans network.rs :
pub fn connect() {
    println!("Connexion établie");
}
// Avec sous-modules :
// src/
//   main.rs
//   network.rs
//   network/
//     tcp.rs
//     udp.rs
// Dans network.rs :
pub mod tcp;  // Cherche network/tcp.rs
pub mod udp;  // Cherche network/udp.rs
pub fn common_function() {}
```

I **Convention :** Utilise `mod.rs` pour les modules avec sous-modules, ou un fichier avec le nom du module. 

## 1.3 - Hiérarchie de modules

```
// Structure :
// src/
//   lib.rs
//   auth/
//     mod.rs
//     login.rs
//     register.rs
//   database/
//     mod.rs
//     connection.rs
//     query.rs
```

```
// Dans lib.rs :
pub mod auth;
pub mod database;
```

```
// Dans auth/mod.rs :
pub mod login;
pub mod register;
```

```
// Utilisation
use mon_projet::auth::login::authenticate;
use mon_projet::database::connection::connect;
```

## 2. Visibilité (pub)

## 2.1 - Privé par défaut

**`mod api { // Fonction privée (par défaut) fn interne() { println!("Fonction interne"); } // Fonction publique pub fn publique() { interne();  //`** I **`OK dans le même module println!("Fonction publique"); } // Struct privée struct Config { secret: String, } // Struct publique avec champ privé pub struct User { pub nom: String, mot_de_passe: String,  // Privé ! } impl User { pub fn new(nom: String, mdp: String) -> User { User { nom, mot_de_passe: mdp, } } } } fn main() { let user = api::User::new( String::from("Alice"), String::from("secret123") ); println!("{}", user.nom);  //`** I **`OK // println!("{}", user.mot_de_passe);  //`** I **`ERREUR : privé }`** 

**2.2 - pub et pub(crate)** 

```
// pub : visible partout
pub fn global() {}
// pub(crate) : visible dans la crate uniquement
pub(crate) fn interne_crate() {}
// pub(super) : visible dans le module parent
mod parent {
    pub(super) fn pour_parent() {}
    mod enfant {
        pub(in crate::parent) fn specifique() {}
    }
}
// Exemple pratique
mod database {
    pub struct Connection {
        url: String,
    }
    impl Connection {
        pub fn new(url: String) -> Self {
            Self { url }
        }
        // Méthode interne à la crate
        pub(crate) fn raw_query(&self, sql: &str) {
            // Fonction dangereuse, pas exposée publiquement
        }
    }
}
```

## 2.3 - Re-exports

```
// Dans lib.rs
mod internal {
    pub struct User {
        pub nom: String,
    }
}
// Re-export pour simplifier l'API
pub use internal::User;
// Les utilisateurs peuvent faire :
use ma_lib::User;  // Au lieu de ma_lib::internal::User
// Re-exports multiples
mod database {
    pub mod mysql {
        pub fn connect() {}
    }
    pub mod postgres {
        pub fn connect() {}
    }
}
// Simplifier l'API
pub use database::mysql;
pub use database::postgres;
```

## 3. use et chemins

## 3.1 - Importer des items

```
// Import simple
use std::collections::HashMap;
```

```
let mut map = HashMap::new();
```

```
// Import multiple
use std::collections::{HashMap, HashSet, BTreeMap};
```

```
// Import tout le module
use std::io;
```

```
io::stdin().read_line(&mut buffer)?;
```

```
// Import avec renommage
use std::collections::HashMap as Map;
```

```
let mut map = Map::new();
```

```
// Import imbriqué
use std::{
    io::{self, Write},
    collections::HashMap,
};
```

## 3.2 - Chemins absolus vs relatifs

```
// Chemin absolu (depuis la racine de la crate)
use crate::network::tcp::connect;
```

```
// Chemin relatif
mod network {
    pub mod tcp {
        pub fn connect() {}
    }
    pub mod udp {
        // Utiliser super pour remonter
        use super::tcp;
        pub fn send() {
            tcp::connect();
        }
    }
}
// self fait référence au module actuel
mod parent {
    pub fn fonction() {}
    mod enfant {
        use self::super::fonction;  // Remonte d'un niveau
    }
}
```

**3.3 - use as et glob** 

```
// Renommer pour éviter les conflits
use std::io::Result as IoResult;
use std::fmt::Result as FmtResult;
fn read() -> IoResult<String> { /* ... */ }
fn format() -> FmtResult { /* ... */ }
// Glob import (à éviter généralement)
use std::collections::*;
// Acceptable pour le prelude
use std::prelude::v1::*;
// Ou pour tests
#[cfg(test)]
mod tests {
    use super::*;  // Import tout du module parent
    #[test]
    fn test_fonction() {
        assert!(ma_fonction());
    }
}
```

II **Évite use * :** Ça pollue le namespace et rend le code moins clair. Utilise-le seulement dans les tests ou pour le prelude. 

## 4. Bibliothèques vs Binaires

## 4.1 - lib.rs vs main.rs

**`// Structure projet : // mon_projet/ //   Cargo.toml //   src/ //     lib.rs`** ← **`Bibliothèque (optionnel) //     main.rs`** ← **`Binaire //     bin/`** ← **`Binaires additionnels (optionnel) //       autre.rs`** 

```
// Dans lib.rs :
pub fn fonction_publique() -> i32 {
    42
}
fn fonction_privee() {
    // Utilisable seulement dans la lib
}
// Dans main.rs :
use mon_projet::fonction_publique;
fn main() {
    let x = fonction_publique();
    println!("{}", x);
}
// Cargo.toml :
// [package]
// name = "mon_projet"
// version = "0.1.0"
//
// [lib]
// name = "mon_projet"
// path = "src/lib.rs"
//
// [[bin]]
// name = "mon_projet"
// path = "src/main.rs"
```

## 4.2 - Créer une lib

```
# Créer une nouvelle bibliothèque
cargo new ma_lib --lib
# Structure générée :
# ma_lib/
#   Cargo.toml
#   src/
#     lib.rs
```

```
# Dans lib.rs :
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

```
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_add() {
        assert_eq!(add(2, 2), 4);
    }
}
# Utiliser la lib dans un autre projet :
# Cargo.toml :
# [dependencies]
# ma_lib = { path = "../ma_lib" }
# Dans le code :
use ma_lib::add;
```

```
fn main() {
    println!("{}", add(5, 3));
}
```

## 5. Cargo.toml

## 5.1 - Dépendances

```
# Cargo.toml
[package]
name = "mon_projet"
version = "0.1.0"
edition = "2021"
```

```
[dependencies]
# Depuis crates.io
serde = "1.0"
```

```
# Version spécifique
tokio = "=1.35.0"
```

```
# Features optionnelles
serde = { version = "1.0", features = ["derive"] }
# Depuis git
mon_lib = { git = "https://github.com/user/repo" }
# Depuis un chemin local
ma_lib = { path = "../ma_lib" }
```

```
[dev-dependencies]
# Seulement pour les tests
criterion = "0.5"
```

```
[build-dependencies]
# Pour build.rs
cc = "1.0"
```

```
# Différentes dépendances selon la plateforme
[target.'cfg(windows)'.dependencies]
winapi = "0.3"
```

```
[target.'cfg(unix)'.dependencies]
libc = "0.2"
```

## 5.2 - Features

```
# Dans Cargo.toml
[features]
default = ["json"]
json = ["serde_json"]
xml = ["quick-xml"]
full = ["json", "xml"]
```

```
[dependencies]
serde_json = { version = "1.0", optional = true }
quick-xml = { version = "0.31", optional = true }
# Dans le code (lib.rs) :
#[cfg(feature = "json")]
pub mod json_parser {
    pub fn parse() {}
}
#[cfg(feature = "xml")]
pub mod xml_parser {
    pub fn parse() {}
}
```

```
# Utilisation :
# cargo build --features json
# cargo build --features "json xml"
# cargo build --all-features
```

## 6. Workspaces

Les **workspaces** permettent de gérer plusieurs crates dans un même dépôt. 

**`# Structure : # mon_workspace/ #   Cargo.toml`** ← **`Workspace root #   ma_lib/ #     Cargo.toml #     src/lib.rs #   mon_app/ #     Cargo.toml #     src/main.rs #   mon_autre_lib/ #     Cargo.toml #     src/lib.rs # Dans mon_workspace/Cargo.toml : [workspace] members = [ "ma_lib", "mon_app", "mon_autre_lib" ] # Dépendances partagées [workspace.dependencies] serde = "1.0" tokio = "1.35" # Dans mon_app/Cargo.toml : [dependencies] ma_lib = { path = "../ma_lib" } serde = { workspace = true }`** 

**`# Commandes : # cargo build`** ← **`Build tout le workspace # cargo test`** ← **`Test tout le workspace # cargo build -p mon_app`** ← **`Build une crate spécifique`** 

## Avantages des workspaces :

• Dépendances partagées (une seule version) 

• Build et test unifiés 

• Facilite le développement de projets multi-crates 

## 7. Best practices

## • **1. Organisation claire** 

Un module = une responsabilité. Évite les modules fourre-tout. 

## • **2. API publique minimale** 

N'expose que ce qui est nécessaire avec `pub` . 

## • **3. Re-exports stratégiques** 

Simplifie l'API avec `pub use` dans lib.rs. 

## • **4. Documentation** 

Documente tout ce qui est `pub` avec //!. 

## • **5. Tests à côté du code** 

Utilise `#[cfg(test)]` pour tests unitaires. 

## • **6. Séparation lib/bin** 

Logique dans lib.rs, CLI dans main.rs. 

## • **7. Features optionnelles** 

Utilise features pour dépendances lourdes optionnelles. 

## 8. Exemple complet

```
// Structure :
// mon_api/
//   Cargo.toml
//   src/
//     lib.rs
//     models/
//       mod.rs
//       user.rs
//       post.rs
//     api/
//       mod.rs
//       routes.rs
//     database/
//       mod.rs
//       connection.rs
// Dans lib.rs :
pub mod models;
pub mod api;
mod database;  // Privé
```

```
// Re-exports pour API simple
pub use models::{User, Post};
pub use api::routes::configure_routes;
// Dans models/mod.rs :
pub mod user;
pub mod post;
// Dans models/user.rs :
#[derive(Debug)]
pub struct User {
    pub id: u32,
    pub nom: String,
}
impl User {
    pub fn new(id: u32, nom: String) -> Self {
        Self { id, nom }
    }
}
// Utilisation externe :
use mon_api::{User, configure_routes};
fn main() {
    let user = User::new(1, "Alice".to_string());
    println!("{:?}", user);
}
```

## Parfait !

Tu sais maintenant organiser tes projets Rust ! 

Points clés : 

- Modules pour organiser le code 

• pub pour contrôler la visibilité 

• lib.rs pour bibliothèques 

- Workspaces pour multi-crates 

I **Ton code sera propre et maintenable !** I
