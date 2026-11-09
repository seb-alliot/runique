# Cargo & Dépendances
> Gérer un projet Rust de A à Z : Cargo.toml, versions, features, workspaces, profils, publication — et ce que chaque déclaration change réellement à la compilation.

## Objectifs

- Comprendre ce que Cargo fait à chaque commande
- Lire et écrire un `Cargo.toml` complet
- Choisir une contrainte de version et comprendre `Cargo.lock`
- Déclarer des dépendances de toutes les façons (crates.io, git, chemin, plateforme, renommage)
- Concevoir des features : optionnelles, additives, exclusives
- Comprendre l'unification des features et le resolver
- Organiser un workspace et partager versions, dépendances et lints
- Régler les profils de compilation
- Publier sur crates.io

---

## Table des matières

1. Introduction à Cargo
2. Le manifeste `[package]`
3. Déclarer une dépendance
4. Versions et `Cargo.lock`
5. Les différents types de dépendances
6. Features : déclaration
7. Features : ce qu'elles changent à la compilation
8. Unification des features et resolver
9. Features exclusives
10. Workspaces
11. Cibles : lib, binaires, exemples
12. Profils de compilation
13. Lints dans Cargo.toml
14. Commandes essentielles
15. Publier sur crates.io
16. Aide-mémoire

---

## 1. Introduction à Cargo

Cargo est l'outil de compilation **et** le gestionnaire de paquets de Rust. Il :

- télécharge les dépendances depuis crates.io (ou git, ou un dossier local) ;
- choisit une version compatible pour chacune et la fige dans `Cargo.lock` ;
- appelle `rustc` pour chaque crate, dans le bon ordre, avec les bonnes features ;
- lance les tests, la documentation, le linter, le formateur ;
- publie une crate.

### Structure d'un projet

```
mon_projet/
├── Cargo.toml        ← le manifeste : nom, version, dépendances, features
├── Cargo.lock        ← versions exactes retenues (généré, ne pas éditer à la main)
├── src/
│   ├── main.rs       ← point d'entrée d'un binaire
│   ├── lib.rs        ← racine d'une bibliothèque (optionnel)
│   └── bin/
│       └── outil.rs  ← binaire supplémentaire
├── tests/            ← tests d'intégration
├── examples/         ← exemples exécutables (cargo run --example nom)
├── benches/          ← benchmarks
├── build.rs          ← script exécuté avant la compilation (optionnel)
└── target/           ← résultats de compilation (à ne pas versionner)
```

Cargo trouve tout seul ces fichiers par leur emplacement : aucune déclaration n'est nécessaire tant qu'on suit cette structure.

### Une crate, un package

- Un **package** est un dossier avec un `Cargo.toml`.
- Une **crate** est une unité de compilation : une bibliothèque (`lib.rs`) ou un binaire (`main.rs`, `src/bin/*.rs`).
- Un package contient **au plus une** bibliothèque et autant de binaires qu'on veut.

---

## 2. Le manifeste `[package]`

### 2.1 Champs principaux

```toml
[package]
name = "mon_projet"
version = "0.1.0"
edition = "2024"
rust-version = "1.85"
authors = ["Ton Nom <ton@email.com>"]
description = "Une courte description"
license = "MIT OR Apache-2.0"
repository = "https://github.com/moi/mon_projet"
readme = "README.md"
keywords = ["web", "framework"]
categories = ["web-programming::http-server"]
```

| Champ | À quoi il sert |
|---|---|
| `name` | nom de la crate ; les `-` deviennent `_` dans le code (`use mon_projet::...`) |
| `version` | version SemVer de ta crate |
| `edition` | version du **langage** utilisée pour compiler cette crate |
| `rust-version` | version minimale du compilateur (MSRV) ; Cargo refuse de compiler avec une version plus ancienne |
| `license`, `description` | obligatoires pour publier |
| `keywords` (5 max), `categories` | recherche sur crates.io |

### 2.2 L'édition : ce qu'elle change

L'édition n'est **pas** la version du compilateur : un compilateur récent sait compiler les éditions 2015, 2018, 2021 et 2024. Elle fixe certaines règles du langage pour la crate.

| Édition | Exemples de changements |
|---|---|
| 2018 | `use crate::...`, `async`/`await` deviennent des mots-clés |
| 2021 | `array.into_iter()` donne des valeurs, closures qui capturent des champs, resolver 2 par défaut |
| 2024 | let-chains (`if let ... && ...`), temporaires de `if let` libérés avant `else`, `gen` réservé, `unsafe extern`, resolver 3 par défaut |

Chaque crate a sa propre édition : ton projet en 2024 peut dépendre d'une crate en 2018 sans problème. `cargo fix --edition` migre une crate vers l'édition suivante.

L'édition 2024 demande au moins Rust 1.85, d'où le `rust-version` cohérent.

### 2.3 Hériter des champs du workspace

```toml
[package]
name = "runique"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
```

Voir la section 10.

---

## 3. Déclarer une dépendance

### 3.1 Toutes les formes d'écriture

```toml
[dependencies]
# Forme courte : une version depuis crates.io
serde_json = "1.0"

# Forme table en ligne : version + options
serde = { version = "1.0", features = ["derive"] }

# Sans les features par défaut, puis on choisit
imagesize = { version = "0.15", default-features = false, features = ["heif"] }

# Optionnelle : compilée seulement si une feature la demande
sea-orm = { version = "2.0", optional = true }

# Depuis git (branche, tag ou commit)
ma_crate = { git = "https://github.com/user/ma_crate" }
ma_crate2 = { git = "https://github.com/user/ma_crate", branch = "dev" }
ma_crate3 = { git = "https://github.com/user/ma_crate", tag = "v1.2.0" }
ma_crate4 = { git = "https://github.com/user/ma_crate", rev = "a1b2c3d" }

# Depuis un dossier local
utils = { path = "../utils" }

# Local pendant le développement, crates.io à la publication
derive_form = { path = "../derive_form", version = "4.0.0" }

# Renommer une dépendance : on l'utilise sous le nom `json` dans le code
json = { package = "serde_json", version = "1.0" }
```

### 3.2 Forme table développée

Les deux écritures sont strictement équivalentes ; la table développée se lit mieux quand il y a beaucoup d'options.

```toml
[dependencies]
tokio = { version = "1", features = ["rt-multi-thread", "macros", "net"] }

[dependencies.tokio]
version = "1"
features = ["rt-multi-thread", "macros", "net"]
```

### 3.3 Ajouter une dépendance en ligne de commande

```bash
cargo add serde --features derive        # [dependencies]
cargo add tokio -F full                  # -F est l'abréviation de --features
cargo add pretty_assertions --dev        # [dev-dependencies]
cargo add cc --build                     # [build-dependencies]
cargo add sea-orm --optional             # optional = true
cargo add imagesize --no-default-features -F heif
cargo add utils --path ../utils
cargo remove serde
```

`cargo add` écrit la dernière version compatible avec ton `rust-version`.

### 3.4 `default-features = false` : à quoi ça sert

La plupart des crates activent des features par défaut. Les désactiver sert à :

- **réduire la compilation et la taille** : ne compiler que ce dont tu te sers ;
- **choisir une implémentation** : par exemple un backend TLS (`rustls` plutôt que `native-tls`) ;
- **compiler sans la bibliothèque standard** (`no_std`) pour l'embarqué.

```toml
# La crate tire native-tls par défaut ; on la veut avec rustls à la place
reqwest = { version = "0.13", default-features = false, features = ["rustls", "json"] }
```

Attention : `default-features = false` ne s'applique qu'à **ta** déclaration. Si une autre crate de l'arbre dépend de `reqwest` avec ses features par défaut, elles seront activées quand même (section 8).

---

## 4. Versions et `Cargo.lock`

### 4.1 SemVer en bref

Une version s'écrit `MAJEUR.MINEUR.CORRECTIF` :

- **MAJEUR** change : rupture de compatibilité possible ;
- **MINEUR** change : ajouts compatibles ;
- **CORRECTIF** change : corrections compatibles.

Pour les versions `0.x`, Cargo considère que **le premier chiffre non nul** est le chiffre de rupture : passer de `0.3` à `0.4` peut casser ton code.

### 4.2 Contraintes de version

| Écriture | Versions acceptées | Usage |
|---|---|---|
| `"1.2.3"` ou `"^1.2.3"` | `>=1.2.3, <2.0.0` | par défaut (caret) |
| `"1.2"` | `>=1.2.0, <2.0.0` | idem, correctif quelconque |
| `"1"` | `>=1.0.0, <2.0.0` | n'importe quelle 1.x |
| `"0.3.5"` | `>=0.3.5, <0.4.0` | en 0.x, le mineur est le chiffre de rupture |
| `"0.0.4"` | `>=0.0.4, <0.0.5` | en 0.0.x, tout change peut casser |
| `"~1.2.3"` | `>=1.2.3, <1.3.0` | tilde : correctifs seulement |
| `"1.*"` | `>=1.0.0, <2.0.0` | joker |
| `"=2.0.4"` | exactement `2.0.4` | épingler une version |
| `">=1.2, <1.5"` | plage explicite | cas rares |
| `"*"` | toutes | **refusé** par crates.io à la publication |

La forme par défaut (`"1.2"`) est la bonne dans presque tous les cas : elle laisse Cargo prendre les correctifs, sans accepter de rupture.

### 4.3 Quand épingler avec `=`

```toml
sea-orm = "=2.0.4"
```

Épingler sert quand une crate casse la compatibilité **sans** changer son chiffre de rupture (cela arrive), ou quand deux crates doivent rester alignées exactement (`sea-orm` et `sea-orm-migration`). Le prix : ta crate ne pourra pas cohabiter avec une autre qui demande `sea-orm = "2.0.5"`, car Cargo ne peut pas choisir une version qui satisfait les deux.

### 4.4 Versions préliminaires

Une version comme `2.0.0-rc.3` n'est **jamais** choisie par une contrainte qui ne la nomme pas : `"2"` ne prendra pas une `-rc`. Il faut l'écrire explicitement (`"2.0.0-rc.3"`).

### 4.5 `Cargo.lock`

`Cargo.toml` dit **ce que tu acceptes** ; `Cargo.lock` dit **ce qui a été choisi**, au correctif près, pour toutes les dépendances, y compris indirectes.

```bash
cargo update                 # reprend la dernière version compatible de tout
cargo update -p serde        # seulement serde
cargo update -p serde --precise 1.0.200   # une version précise
```

| Type de projet | Versionner `Cargo.lock` ? | Pourquoi |
|---|---|---|
| Binaire, application, serveur | **oui** | toute la chaîne (CI, production) compile exactement les mêmes versions |
| Bibliothèque | conseillé aussi | il sert à ta CI ; il est **ignoré** par les projets qui dépendent de ta bibliothèque |

Point clé : le `Cargo.lock` d'une bibliothèque n'a **aucun** effet sur ses utilisateurs. Eux résolvent les versions selon les contraintes de ton `Cargo.toml`. Si ta bibliothèque ne compile qu'avec une version précise d'une dépendance, il faut l'écrire dans `Cargo.toml`.

`cargo install` ignore aussi le `Cargo.lock` d'une crate publiée, sauf avec `--locked`.

### 4.6 Plusieurs versions d'une même crate

Si deux dépendances demandent `rand = "0.8"` et `rand = "0.9"`, Cargo compile **les deux** : elles ne sont pas compatibles entre elles, donc ce sont deux crates différentes. Conséquences :

- compilation plus longue ;
- un `rand::Rng` de la 0.8 n'est pas le même trait que celui de la 0.9 : passer une valeur de l'une à l'autre donne une erreur de type déroutante (`expected Rng, found Rng`).

```bash
cargo tree -d                # liste les crates présentes en plusieurs versions
```

---

## 5. Les différents types de dépendances

### 5.1 `[dependencies]`

Compilées dans ta crate et visibles par ceux qui en dépendent (pour une bibliothèque).

### 5.2 `[dev-dependencies]`

```toml
[dev-dependencies]
pretty_assertions = "1.4"
reqwest = { version = "0.13", features = ["json"] }
```

Utilisées **uniquement** par les tests, les exemples et les benchmarks. Elles ne sont jamais compilées dans la bibliothèque livrée, et ne sont pas téléchargées par ceux qui dépendent de ta crate.

Utilité supplémentaire : activer une feature **seulement pour les tests**.

```toml
[dependencies]
runique = { version = "4.0", features = ["postgres"] }

[dev-dependencies]
runique = { version = "4.0", features = ["test-utils"] }
```

Pendant `cargo test`, les deux déclarations sont fusionnées : `postgres` et `test-utils` sont actives. En `cargo build`, seule `postgres` l'est.

### 5.3 `[build-dependencies]`

```toml
[build-dependencies]
cc = "1.0"
toml = "1"
```

Utilisées par `build.rs`, un programme compilé et exécuté **avant** ta crate (générer du code, compiler du C, lire un fichier de configuration). Elles ne sont pas disponibles dans `src/`. Si tu as besoin de la même crate dans les deux, déclare-la dans les deux sections (Runique le fait pour `toml`).

### 5.4 Dépendances par plateforme

```toml
[target.'cfg(unix)'.dependencies]
libc = "0.2"

[target.'cfg(windows)'.dependencies]
windows-sys = "0.59"

[target.'cfg(target_arch = "wasm32")'.dependencies]
getrandom = { version = "0.3", features = ["wasm_js"] }
```

Ne sont compilées que pour la plateforme cible. Le code qui s'en sert se protège par le même `#[cfg(unix)]`.

### 5.5 Remplacer une dépendance : `[patch]`

```toml
[patch.crates-io]
sea-orm = { git = "https://github.com/SeaQL/sea-orm", branch = "master" }
```

Remplace `sea-orm` **partout dans l'arbre**, y compris quand c'est une dépendance indirecte. À quoi ça sert : tester un correctif pas encore publié. Ne vaut que pour le projet racine (ignoré quand ta crate est utilisée comme dépendance).

### 5.6 Une déclaration de workspace ne fait rien seule

```toml
# Cargo.toml racine
[workspace.dependencies]
time = "=0.3.55"
```

Cette ligne ne force **rien** tant qu'aucun membre n'écrit `time.workspace = true`. Pour épingler une dépendance indirecte (que tu n'utilises pas toi-même), il faut la déclarer directement dans le membre concerné : c'est ce que fait Runique pour `time`, sinon `cargo install` reprendrait une version plus récente incompatible avec `cookie`.

---

## 6. Features : déclaration

Une **feature** est un nom qui active du code conditionnel dans ta crate et/ou des dépendances optionnelles ou des features de tes dépendances.

### 6.1 Les quatre choses qu'une feature peut activer

```toml
[features]
default = ["json"]                    # 1. d'autres features de ta crate
json = ["dep:serde_json"]             # 2. une dépendance optionnelle
derive = ["serde/derive"]             # 3. une feature d'une dépendance
full = ["json", "derive", "async"]    # groupe de features
async = ["dep:tokio", "tokio/rt"]     # combinaison
admin = []                            # 4. rien d'autre : seulement du code sous #[cfg]

[dependencies]
serde = "1.0"
serde_json = { version = "1.0", optional = true }
tokio = { version = "1", optional = true }
```

### 6.2 La feature `default`

```toml
[features]
default = ["orm"]
```

Activée automatiquement sauf si l'utilisateur écrit `default-features = false` (ou `--no-default-features`). Garde-la **petite** : tout ce qui y figure sera compilé chez tous tes utilisateurs, et ils ne peuvent la retirer qu'en bloc.

### 6.3 Dépendance optionnelle : `dep:` ou feature implicite

```toml
[dependencies]
serde_json = { version = "1.0", optional = true }
```

Écriture 1 — **sans** `dep:` : Cargo crée automatiquement une feature nommée `serde_json`. Les utilisateurs peuvent l'activer directement (`features = ["serde_json"]`), et ce nom fait partie de ton API publique.

```toml
[features]
json = ["serde_json"]       # json active la feature implicite serde_json
```

Écriture 2 — **avec** `dep:` (Rust 1.60+) : la feature implicite n'est plus créée. Seul `json` est visible.

```toml
[features]
json = ["dep:serde_json"]
```

La seconde forme est préférable : tu choisis les noms exposés, et tu peux remplacer `serde_json` par une autre crate sans casser personne. Runique l'utilise : `orm = ["dep:sea-orm", "dep:sea-orm-migration"]`.

### 6.4 Feature « faible » : `?`

```toml
[features]
serde = ["dep:serde", "chrono?/serde"]

[dependencies]
serde = { version = "1.0", optional = true }
chrono = { version = "0.4", optional = true }
```

- `"chrono/serde"` active `serde` de chrono **et** force la compilation de chrono.
- `"chrono?/serde"` active `serde` de chrono **seulement si** chrono est déjà activé par ailleurs.

À quoi ça sert : « si tu utilises les dates, rends-les sérialisables », sans imposer les dates à qui ne veut que serde.

### 6.5 Activer des features chez une dépendance

```toml
[dependencies]
runique = { version = "4.0", features = ["postgres", "admin"] }
```

```bash
cargo build --features "json,async"      # features du package courant
cargo build -F json -F async             # idem
cargo build --features runique/admin     # feature d'une dépendance, depuis la ligne de commande
cargo build --all-features               # toutes (attention : section 9)
cargo build --no-default-features        # aucune feature par défaut
cargo build --no-default-features -F json
```

---

## 7. Features : ce qu'elles changent à la compilation

Une feature n'est **pas** une option lue à l'exécution : elle change **le code compilé**. Chaque combinaison de features produit un programme différent.

### 7.1 `#[cfg(feature = "...")]` : le code disparaît

```rust
#[cfg(feature = "json")]
pub mod json {
    pub fn parse(input: &str) -> serde_json::Value {
        serde_json::from_str(input).unwrap_or_default()
    }
}

#[cfg(feature = "admin")]
pub fn with_admin(self) -> Self { self }
```

Sans la feature, le module n'existe pas : il n'est ni compilé, ni vérifié, ni présent dans le binaire. Un utilisateur qui appelle `mon_projet::json::parse` sans la feature obtient « cannot find module `json` ».

Conséquence directe : **une erreur dans un bloc `#[cfg(feature = "x")]` n'apparaît que si tu compiles avec `x`**. Il faut donc tester chaque combinaison qui compte (c'est pourquoi la CI de Runique compile plusieurs lots de features).

### 7.2 Les combinaisons de `cfg`

```rust
#[cfg(all(feature = "postgres", feature = "admin"))]
fn a() {}

#[cfg(any(feature = "sqlite", feature = "postgres"))]
fn b() {}

#[cfg(not(feature = "admin"))]
fn c() {}

#[cfg(all(unix, not(test)))]
fn d() {}
```

### 7.3 `cfg!` : le code reste, la valeur change

```rust
fn moteur() -> &'static str {
    if cfg!(feature = "postgres") {
        "postgres"
    } else {
        "sqlite"
    }
}
```

`cfg!(...)` est remplacé à la compilation par `true` ou `false`. Contrairement à `#[cfg]`, **les deux branches sont compilées** et doivent donc être valides même quand la feature est absente. On ne peut pas y appeler une fonction qui n'existe que sous la feature.

| Forme | Le code inactif est-il compilé ? | Usage |
|---|---|---|
| `#[cfg(feature = "x")]` | non, il disparaît | modules, fonctions, `use`, champs qui dépendent d'une crate optionnelle |
| `cfg!(feature = "x")` | oui, vérifié mais jamais exécuté | petit choix de valeur, message, chemin |

### 7.4 `#[cfg_attr]` : un attribut conditionnel

```rust
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Point {
    pub x: i32,
    pub y: i32,
}
```

`Point` n'implémente `Serialize` que si l'utilisateur active `serde`. À quoi ça sert : proposer l'intégration avec une crate sans l'imposer.

### 7.5 Sur les `use`, les champs et les variantes

```rust
#[cfg(feature = "orm")]
use sea_orm::DatabaseConnection;

pub struct Moteur {
    pub nom: String,
    #[cfg(feature = "orm")]
    pub db: DatabaseConnection,
}
```

Attention : un champ ou une variante d'enum sous `#[cfg]` change la **forme** du type. Un code utilisateur qui construit `Moteur { nom }` cessera de compiler le jour où quelqu'un d'autre dans l'arbre active `orm` (section 8). Pour un type public, préfère un champ toujours présent, ou un constructeur.

### 7.6 Afficher la feature dans la documentation

```toml
[package.metadata.docs.rs]
features = ["all-databases"]
# ou : all-features = true
```

docs.rs compile ta documentation avec ces features ; sans cette section, seules les features par défaut sont documentées et tout le code sous `#[cfg(feature)]` est absent de la doc.

---

## 8. Unification des features et resolver

### 8.1 Le principe : l'union

Dans **un même build**, une crate n'est compilée **qu'une fois** par version. Si plusieurs crates de l'arbre en dépendent avec des features différentes, Cargo active **l'union** de toutes les features demandées.

```
mon_app ──► serde (features: [])
   └─► ma_lib ──► serde (features: ["derive"])

Résultat : serde est compilé UNE fois, avec "derive".
```

Conséquences :

- `default-features = false` chez toi n'empêche pas une autre crate de réactiver les features par défaut ;
- ton code peut compiler **par accident** grâce à une feature activée par une autre crate, puis casser le jour où elle disparaît de l'arbre. Déclare toujours toi-même les features que tu utilises.

### 8.2 Règle d'or : les features sont additives

Puisque Cargo peut activer une feature sans que tu le demandes, **activer une feature ne doit jamais retirer ni changer un comportement existant**, seulement en ajouter.

```toml
# Mauvais : no_std retire la bibliothèque standard
[features]
no_std = []

# Bon : std l'ajoute, activée par défaut
[features]
default = ["std"]
std = []
```

```rust
// Mauvais : la feature change le sens d'une fonction existante
#[cfg(feature = "rapide")]
pub fn hash(x: &str) -> u64 { /* algorithme A */ 0 }
#[cfg(not(feature = "rapide"))]
pub fn hash(x: &str) -> u64 { /* algorithme B */ 1 }
// Une autre crate de l'arbre active "rapide" : tes hachages enregistrés ne correspondent plus.
```

### 8.3 Le resolver : quand l'union s'applique

```toml
[package]
edition = "2024"      # resolver 3 par défaut

# Dans un workspace, il se déclare à la racine :
[workspace]
resolver = "3"
```

| Resolver | Comportement |
|---|---|
| `"1"` | union de tout, partout : les features des dev-dependencies et des build-dependencies contaminent le build normal |
| `"2"` (défaut en édition 2021) | ne fusionne pas les features des **build-dependencies** et des macros procédurales avec celles du code normal ; ignore les dépendances d'une autre plateforme ; n'active les features des **dev-dependencies** que pour compiler tests et exemples |
| `"3"` (défaut en édition 2024) | comme le 2, et choisit en plus des versions compatibles avec `rust-version` |

Un workspace **virtuel** (racine sans `[package]`) n'a pas d'édition : il faut y écrire `resolver` explicitement, sinon Cargo reprend le resolver 1 et affiche un avertissement.

### 8.4 Pièges de l'unification dans un workspace

Si tu compiles tout le workspace (`cargo build` à la racine), les features demandées par **tous** les membres sont unifiées. Si tu compiles un seul membre (`cargo build -p ma_lib`, ou depuis son dossier), seules ses propres demandes comptent.

Résultat : `cargo test` à la racine peut réussir alors que `cargo test -p ma_lib` échoue (ou l'inverse). C'est la raison pour laquelle les tests de Runique se lancent depuis `runique/` : à la racine, les features de `demo-app` se mêleraient à celles du framework.

### 8.5 Voir qui active quoi : `cargo tree`

```bash
cargo tree                       # arbre des dépendances
cargo tree -i serde              # qui dépend de serde (arbre inversé)
cargo tree -e features           # affiche les features activées sur chaque arête
cargo tree -e features -i serde  # qui active quelle feature de serde
cargo tree -d                    # crates en plusieurs versions
cargo tree --no-default-features -F postgres   # l'arbre pour une combinaison précise
```

---

## 9. Features exclusives

### 9.1 Le problème

Parfois deux features ne peuvent **pas** coexister : deux moteurs de base, deux backends TLS, deux types de clé primaire. L'unification (section 8) rend cela dangereux : une autre crate peut activer la seconde à ton insu.

### 9.2 Bloquer à la compilation : `compile_error!`

```rust
#[cfg(all(feature = "postgres", feature = "sqlite"))]
compile_error!("les features `postgres` et `sqlite` sont exclusives : choisis un seul moteur");
```

Le build s'arrête avec un message clair, au lieu d'un comportement incohérent ou d'une erreur de type obscure.

Runique fait exactement cela pour `sqlite`, `postgres` et `mysql`, avec une échappatoire explicite :

```rust
#[cfg(all(
    not(doc),
    not(feature = "all-databases"),
    feature = "postgres",
    feature = "sqlite"
))]
compile_error!("features `postgres` and `sqlite` are mutually exclusive");
```

- `not(doc)` : `cargo doc` et docs.rs peuvent documenter tous les moteurs ;
- `all-databases` : une feature qu'il faut **demander par son nom**, pour les outils qui parlent aux trois moteurs.

### 9.3 Conséquence : `--all-features` ne marche plus

Avec des features exclusives, `cargo test --all-features` déclenche le `compile_error!`. Il faut tester **chaque combinaison valide** séparément :

```bash
cargo test --features postgres
cargo test --features sqlite
cargo test --features big-pk,all-databases
cargo test --features pk-uuid,all-databases
```

En CI, une matrice lance ces lots en parallèle.

### 9.4 L'alternative : choisir à l'exécution

Quand c'est possible, éviter l'exclusivité : compiler les deux implémentations et choisir à l'exécution (variable d'environnement, configuration). Les features restent additives, au prix d'un binaire plus gros. Les features exclusives se justifient quand les deux implémentations ne peuvent pas cohabiter dans le même binaire, ou changent des types publics (comme le type de clé primaire).

---

## 10. Workspaces

Un workspace regroupe plusieurs packages qui partagent un seul `Cargo.lock` et un seul dossier `target/`.

### 10.1 Racine

```toml
# Cargo.toml racine (workspace virtuel : pas de [package])
[workspace]
members = [
    "runique/runique_dsl",
    "runique/derive_form",
    "runique",
    "demo-app",
]
resolver = "3"

[workspace.package]
version = "4.0.0"
edition = "2024"
rust-version = "1.94"
license = "MIT"

[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
derive_form = { path = "runique/derive_form", version = "4.0.0" }
```

`members` accepte des jokers (`members = ["crates/*"]`) et `exclude` retire des dossiers.

### 10.2 Membres

```toml
# runique/Cargo.toml
[package]
name = "runique"
version.workspace = true
edition.workspace = true

[dependencies]
serde.workspace = true
# Hériter et ajouter : les features s'ajoutent à celles du workspace
tokio = { workspace = true, features = ["test-util"] }
# Hériter et rendre optionnelle
sea-orm = { workspace = true, optional = true }
derive_form.workspace = true
```

Ce qu'on peut ajouter à une dépendance héritée : `features` (ajoutées), `optional`. On ne peut pas changer la version ni retirer une feature.

### 10.3 À quoi ça sert

- une seule version de chaque dépendance pour tout le projet ;
- compiler une fois les dépendances communes ;
- une version commune pour des crates publiées ensemble ;
- `cargo test --workspace` teste tout d'un coup.

```bash
cargo build -p runique          # un seul membre
cargo test --workspace          # tous les membres
cargo build --workspace --exclude demo-app
```

### 10.4 Ordre de publication

Un membre qui dépend d'un autre par `path` doit aussi donner une `version` pour être publiable. On publie dans l'ordre des dépendances : chez Runique, `runique_dsl`, puis `derive_form`, puis `runique`.

---

## 11. Cibles : lib, binaires, exemples

### 11.1 Déclaration implicite et explicite

```toml
# Implicite : src/lib.rs, src/main.rs, src/bin/*.rs, examples/*.rs sont trouvés seuls.

# Explicite : nom ou chemin différent
[lib]
name = "runique"
path = "src/lib.rs"

[[bin]]
name = "runique"
path = "src/bin/runique.rs"

[[example]]
name = "demo"
required-features = ["admin"]
```

`[[bin]]` (double crochet) est un tableau : on peut en déclarer plusieurs.

### 11.2 `required-features`

```toml
[[bin]]
name = "outil-admin"
required-features = ["admin"]
```

Le binaire n'est compilé que si la feature est active. Sans elle, `cargo build` l'ignore sans erreur.

### 11.3 Un binaire est une crate à part

Un binaire dans `src/bin/` utilise la bibliothèque du même package **comme une dépendance externe** : il ne voit que ce qui est `pub`, pas ce qui est `pub(crate)`.

```bash
cargo run --bin outil
cargo run --example demo
cargo install --path . --features postgres
```

---

## 12. Profils de compilation

### 12.1 Les profils par défaut

| Profil | Utilisé par | Réglage par défaut |
|---|---|---|
| `dev` | `cargo build`, `cargo run` | `opt-level = 0`, debug complet, vérifications de dépassement d'entier actives |
| `release` | `--release` | `opt-level = 3`, pas de vérification de dépassement |
| `test` | `cargo test` | hérite de `dev` |
| `bench` | `cargo bench` | hérite de `release` |

### 12.2 Réglages courants

```toml
[profile.release]
opt-level = 3          # 0, 1, 2, 3, "s" (taille), "z" (taille minimale)
lto = "fat"            # optimisation à l'édition de liens : binaire plus rapide, compilation plus lente
codegen-units = 1      # moins de parallélisme, meilleure optimisation
strip = true           # retire les symboles : binaire plus petit
panic = "abort"        # pas de déroulement de pile : plus petit, mais plus de catch_unwind
debug = false

[profile.dev]
opt-level = 0
debug = true
incremental = true
```

### 12.3 Optimiser les dépendances en dev

```toml
[profile.dev.package."*"]
opt-level = 2
```

Toutes les dépendances sont optimisées, ton code reste en `opt-level = 0` (compilation rapide). À quoi ça sert : certaines crates sont inutilisables en debug. Le hachage argon2 d'un mot de passe peut prendre plusieurs secondes sans optimisation et quelques dizaines de millisecondes avec.

### 12.4 Profil personnalisé

```toml
[profile.profiling]
inherits = "release"
debug = true
```

```bash
cargo build --profile profiling
```

### 12.5 Ce qu'un profil change pour ton code

```rust
fn main() {
    // Valeur lue à l'exécution : avec une constante écrite en dur,
    // le compilateur détecterait le dépassement et refuserait de compiler.
    let x: u8 = std::env::args().count() as u8 + 254;
    let y = x + 1;     // dev : panique "attempt to add with overflow"
                       // release : vaut 0 (enroulement), sans erreur
}
```

Les profils n'appartiennent qu'au projet racine : ceux déclarés dans tes dépendances sont ignorés.

---

## 13. Lints dans Cargo.toml

Depuis Rust 1.74, les lints se déclarent dans le manifeste plutôt qu'en tête de `lib.rs`.

```toml
[lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"

[lints.clippy]
unwrap_used = "warn"
pedantic = { level = "warn", priority = -1 }
module_name_repetitions = "allow"
```

Dans un workspace :

```toml
# racine
[workspace.lints.clippy]
unwrap_used = "warn"

# membre
[lints]
workspace = true
```

`priority = -1` applique le groupe d'abord, pour qu'une règle précise (`allow`) puisse le contredire ensuite.

---

## 14. Commandes essentielles

```bash
cargo new mon_projet          # nouveau binaire
cargo new ma_lib --lib        # nouvelle bibliothèque
cargo init                    # dans un dossier existant

cargo check                   # analyse et vérifie les types sans produire de binaire : le plus rapide
cargo build                   # compile en debug
cargo build --release         # compile en release
cargo run -- arg1 arg2        # compile, exécute, passe les arguments après --

cargo test                    # tous les tests
cargo test nom_du_test        # tests dont le nom contient ce texte
cargo test -- --nocapture     # afficher les println! des tests

cargo clippy --all-targets -- -D warnings   # linter, avertissements en erreurs
cargo fmt                     # formater
cargo fmt --check             # vérifier sans modifier (CI)

cargo doc --open              # documentation locale
cargo tree                    # arbre des dépendances
cargo update                  # mettre à jour Cargo.lock
cargo clean                   # vider target/

cargo install ripgrep         # installer un binaire publié
cargo install --path .        # installer le binaire du projet courant
```

`cargo check` fait toute l'analyse (types, emprunts) mais s'arrête avant la génération de code machine : c'est la commande à lancer pendant le développement.

---

## 15. Publier sur crates.io

```bash
cargo login <ton_token>       # une fois
cargo package --list          # fichiers qui seront envoyés
cargo publish --dry-run       # vérifie que le paquet compile seul
cargo publish
```

### Checklist avant publication

- `name`, `version`, `description`, `license` renseignés
- `README.md` présent (affiché sur crates.io)
- Pas de contrainte `"*"`
- Chaque dépendance `path` ou `git` a aussi une `version` (sinon crates.io refuse)
- `exclude` ou `include` pour ne pas envoyer les tests lourds, bases SQLite, fichiers locaux
- Documentation `///` sur les éléments publics, `[package.metadata.docs.rs]` si des features sont nécessaires
- Tests passants **pour chaque combinaison de features** qui compte

### Après publication

- Une version publiée est **définitive** : on ne peut ni la modifier ni la supprimer.
- `cargo yank --version 1.2.3` empêche les **nouveaux** projets de la choisir ; ceux qui l'ont déjà dans leur `Cargo.lock` continuent de l'utiliser.
- Toute rupture d'API publique demande d'augmenter le chiffre de rupture (le majeur, ou le mineur en 0.x). Retirer une feature ou en rendre une obligatoire est une rupture.

---

## 16. Aide-mémoire

### Dépendances

```toml
a = "1.2"                                        # >=1.2.0, <2.0.0
b = "0.3"                                        # >=0.3.0, <0.4.0
c = "=2.0.4"                                     # exactement
d = { version = "1", features = ["x"] }
e = { version = "1", default-features = false }
f = { version = "1", optional = true }
g = { path = "../g", version = "1" }
h = { git = "https://...", tag = "v1" }
i = { package = "vrai_nom", version = "1" }      # renommage
j.workspace = true
```

### Features

```toml
[features]
default = ["a"]
a = []                       # code sous #[cfg(feature = "a")]
b = ["dep:crate_opt"]        # dépendance optionnelle sans feature implicite
c = ["serde/derive"]         # feature d'une dépendance
d = ["chrono?/serde"]        # seulement si chrono est déjà actif
```

```rust
#[cfg(feature = "a")]            // le code disparaît sans la feature
if cfg!(feature = "a") { }       // compilé dans tous les cas, valeur fixée
#[cfg_attr(feature = "s", derive(serde::Serialize))]
#[cfg(all(feature = "x", feature = "y"))]
compile_error!("x et y sont exclusives");
```

### Points clés

- L'édition fixe les règles du langage d'une crate, pas la version du compilateur.
- En `0.x`, le mineur est le chiffre de rupture.
- `Cargo.lock` d'une bibliothèque n'a aucun effet sur ses utilisateurs.
- Une feature change le code compilé : teste chaque combinaison importante.
- Cargo active l'union des features demandées dans l'arbre : les features doivent être additives.
- Des features exclusives se protègent par `compile_error!` et interdisent `--all-features`.
- `dep:` évite d'exposer les noms de tes dépendances optionnelles.
- Une entrée de `[workspace.dependencies]` ne fait rien tant qu'un membre ne la référence pas.
- `cargo tree -e features -i crate` répond à « qui active cette feature ? ».
