# Cargo & Dépendances
> Gérer un projet Rust de A à Z — Cargo.toml, workspaces, features, crates.io

## Objectifs

- Comprendre la structure d'un projet Cargo
- Gérer les dépendances et leurs versions
- Utiliser les features flags
- Travailler avec les workspaces
- Publier une crate sur crates.io

---

## Table des matières

1. [Introduction à Cargo](#1-introduction-à-cargo)
2. [Cargo.toml — la configuration](#2-cargotoml--la-configuration)
   - 2.1 [Métadonnées du projet](#21-métadonnées-du-projet)
   - 2.2 [Dépendances](#22-dépendances)
   - 2.3 [Profils de compilation](#23-profils-de-compilation)
3. [Commandes essentielles](#3-commandes-essentielles)
4. [Features flags](#4-features-flags)
5. [Workspaces](#5-workspaces)
6. [Publier sur crates.io](#6-publier-sur-cratesio)

---

## 1. Introduction à Cargo

Cargo est le gestionnaire de paquets et de compilation de Rust. Il gère :
- la compilation du projet (`cargo build`)
- les dépendances externes (crates)
- les tests (`cargo test`)
- la publication (`cargo publish`)

### Structure d'un projet

```
mon_projet/
├── Cargo.toml       ← configuration du projet
├── Cargo.lock       ← versions exactes verrouillées
└── src/
    ├── main.rs      ← binaire principal
    └── lib.rs       ← bibliothèque (optionnel)
```

> **Important :** `Cargo.lock` doit être commité pour les binaires, ignoré pour les bibliothèques.

---

## 2. Cargo.toml — la configuration

### 2.1 Métadonnées du projet

```toml
[package]
name    = "mon_projet"
version = "0.1.0"
edition = "2024"
authors = ["Ton Nom <ton@email.com>"]
description = "Une courte description"
license = "MIT"
```

### 2.2 Dépendances

```toml
[dependencies]
# Version exacte
serde = "1.0.210"

# Avec features
serde = { version = "1.0", features = ["derive"] }

# Depuis git
ma_crate = { git = "https://github.com/user/ma_crate" }

# Depuis un chemin local
utils = { path = "../utils" }

[dev-dependencies]
# Uniquement pour les tests
pretty_assertions = "1.4"

[build-dependencies]
# Pour le script build.rs
cc = "1.0"
```

**Sémantique des versions :**

| Notation | Signification |
|---|---|
| `"1.0"` | `>= 1.0.0, < 2.0.0` (compatible) |
| `"=1.0.5"` | exactement `1.0.5` |
| `">=1.0, <2.0"` | plage explicite |
| `"*"` | n'importe quelle version |

### 2.3 Profils de compilation

```toml
[profile.release]
opt-level = 3      # optimisation maximale
lto = true         # Link Time Optimization
strip = true       # supprime les symboles de debug

[profile.dev]
opt-level = 0      # pas d'optimisation, compilation rapide
debug = true       # symboles de debug
```

---

## 3. Commandes essentielles

```bash
cargo new mon_projet          # crée un nouveau binaire
cargo new ma_lib --lib        # crée une nouvelle bibliothèque

cargo build                   # compile en debug
cargo build --release         # compile en release (optimisé)
cargo run                     # compile et exécute
cargo run -- arg1 arg2        # avec des arguments

cargo test                    # lance tous les tests
cargo test nom_du_test        # lance un test spécifique
cargo test --release          # tests en mode release

cargo check                   # vérifie sans compiler (rapide)
cargo clippy                  # linter — suggestions de qualité
cargo fmt                     # formate le code

cargo add serde               # ajoute une dépendance
cargo remove serde            # supprime une dépendance
cargo update                  # met à jour Cargo.lock

cargo doc --open              # génère et ouvre la documentation
```

---

## 4. Features flags

Les features permettent d'activer des fonctionnalités optionnelles.

### Déclarer des features

```toml
[features]
default  = ["json"]          # features actives par défaut
json     = ["serde/derive"]  # active serde avec derive
async    = ["tokio"]         # feature async optionnelle
full     = ["json", "async"] # groupe de features

[dependencies]
serde = { version = "1.0", optional = true }
tokio = { version = "1.0", optional = true }
```

### Utiliser les features dans le code

```rust
#[cfg(feature = "json")]
pub mod json {
    pub fn parse(input: &str) -> serde_json::Value {
        serde_json::from_str(input).unwrap()
    }
}

// Compilation conditionnelle
#[cfg(feature = "async")]
pub async fn fetch_data() -> Result<String, reqwest::Error> {
    reqwest::get("https://example.com").await?.text().await
}
```

### Activer des features à la compilation

```bash
cargo build --features "json,async"
cargo build --all-features
cargo build --no-default-features
```

---

## 5. Workspaces

Un workspace regroupe plusieurs crates dans un même projet.

```toml
# Cargo.toml racine
[workspace]
members = [
    "app",
    "core",
    "utils",
]
resolver = "2"

# Versions partagées
[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }

[workspace.package]
version = "0.1.0"
edition = "2024"
```

```toml
# app/Cargo.toml
[package]
name    = "app"
version.workspace = true      # hérite la version du workspace
edition.workspace = true

[dependencies]
core  = { path = "../core" }
serde.workspace = true        # hérite la dépendance du workspace
```

```bash
cargo build -p core           # compile uniquement la crate 'core'
cargo test --workspace        # teste toutes les crates
```

---

## 6. Publier sur crates.io

```bash
# 1. Connexion (token sur crates.io)
cargo login <ton_token>

# 2. Vérifier avant de publier
cargo publish --dry-run

# 3. Publier
cargo publish
```

**Checklist avant publication :**

- `name`, `version`, `description`, `license` renseignés dans `Cargo.toml`
- `README.md` présent (sera affiché sur crates.io)
- Documentation avec `///` sur les éléments publics
- Tests passants (`cargo test`)
- Pas de chemins locaux dans `[dependencies]`
