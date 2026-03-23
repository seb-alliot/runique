\encoding utf8

-- ============================================================
-- seed_cours.sql — chapitres et blocs générés depuis docs/fr/cour/
-- Idempotent : DELETE + INSERT
-- ============================================================

DELETE FROM cour_block;
DELETE FROM chapitre;
DELETE FROM cour;

INSERT INTO cour (id, slug, lang, title, theme, difficulte, sort_order, ordre) VALUES
( 1, 'cargo-dependances',   'fr', 'Cargo & dépendances',    'Fondamentaux',     'debutant',      1,  1),
( 2, 'variables-fonctions', 'fr', 'Variables & fonctions',  'Fondamentaux',     'debutant',      2,  2),
( 3, 'structures-controle', 'fr', 'Structures & contrôle',  'Fondamentaux',     'debutant',      3,  3),
( 4, 'structures-enums',    'fr', 'Structures & enums',     'Fondamentaux',     'debutant',      4,  4),
( 5, 'pattern-matching',    'fr', 'Pattern matching',       'Fondamentaux',     'debutant',      5,  5),
( 6, 'collections',         'fr', 'Collections',            'Fondamentaux',     'debutant',      6,  6),
( 7, 'modules',             'fr', 'Modules',                'Fondamentaux',     'debutant',      7,  7),
( 8, 'tests-rust',          'fr', 'Tests en Rust',          'Fondamentaux',     'debutant',      8,  8),
( 9, 'closures-iterateurs', 'fr', 'Closures & itérateurs',  'Mémoire & sûreté', 'intermediaire', 1,  9),
(10, 'gestion-erreurs',     'fr', 'Gestion des erreurs',    'Mémoire & sûreté', 'intermediaire', 2, 10),
(11, 'generics',            'fr', 'Generics',               'Mémoire & sûreté', 'intermediaire', 3, 11),
(12, 'type-aliases',        'fr', 'Type aliases',           'Mémoire & sûreté', 'intermediaire', 4, 12),
(13, 'lifetimes',           'fr', 'Lifetimes',              'Mémoire & sûreté', 'intermediaire', 5, 13),
(14, 'box-dynamiques',      'fr', 'Box & types dynamiques', 'Mémoire & sûreté', 'intermediaire', 6, 14),
(15, 'smart-pointers',      'fr', 'Smart pointers',         'Mémoire & sûreté', 'intermediaire', 7, 15),
(16, 'send-sync',           'fr', 'Send & Sync',            'Mémoire & sûreté', 'intermediaire', 8, 16),
(17, 'traits-avances',      'fr', 'Traits avancés',         'Avancé',           'avance',        1, 17),
(27, 'traits-basics',       'fr', 'Traits — Les bases',      'Avancé',           'avance',        8, 27),
(28, 'concurrence',         'fr', 'Concurrence & état partagé', 'Avancé',        'avance',        9, 28),
(18, 'macros-declaratives',  'fr', 'Macros déclaratives',                 'Avancé', 'avance', 2, 18),
(23, 'macros-export',        'fr', 'Macros — Visibilité et export',      'Avancé', 'avance', 3, 23),
(24, 'macros-derive',        'fr', 'Macros procédurales — Derive',       'Avancé', 'avance', 4, 24),
(25, 'macros-attribut',      'fr', 'Macros procédurales — Attribute',    'Avancé', 'avance', 5, 25),
(26, 'macros-function-like', 'fr', 'Macros procédurales — Function-like','Avancé', 'avance', 6, 26),
(19, 'async-tokio',          'fr', 'Async & Tokio',                      'Avancé', 'avance', 7, 19),
(20, 'orm-seaorm',          'fr', 'ORM — SeaORM',           'Runique',          'specifique',    1, 20),
(21, 'filtre-admin',        'fr', 'Filtre admin',           'Runique',          'specifique',    2, 21),
(22, 'middleware-ordre',   'fr', 'Middlewares — Pièges et Solutions', 'Runique', 'specifique',    3, 22);


-- cargo-dependances.md (cour_id=1)
INSERT INTO chapitre (id, cour_id, slug, title, lead, sort_order) VALUES
(1, 1, 'cargo-dependances-1-introduction-a-cargo', '1. Introduction à Cargo', NULL, 1),
(2, 1, 'cargo-dependances-2-cargotoml-la-configuration', '2. Cargo.toml — la configuration', NULL, 2),
(3, 1, 'cargo-dependances-3-commandes-essentielles', '3. Commandes essentielles', NULL, 3),
(4, 1, 'cargo-dependances-4-features-flags', '4. Features flags', NULL, 4),
(5, 1, 'cargo-dependances-5-workspaces', '5. Workspaces', NULL, 5),
(6, 1, 'cargo-dependances-6-publier-sur-cratesio', '6. Publier sur crates.io', NULL, 6);
INSERT INTO cour_block (id, chapitre_id, heading, content, block_type, sort_order) VALUES
(10000, 1, NULL, $BLK10000$Cargo est le gestionnaire de paquets et de compilation de Rust. Il gère :
- la compilation du projet (`cargo build`)
- les dépendances externes (crates)
- les tests (`cargo test`)
- la publication (`cargo publish`)$BLK10000$, 'text', 1),
(10001, 1, 'Structure d''un projet', $BLK10001$```
mon_projet/
├── Cargo.toml       ← configuration du projet
├── Cargo.lock       ← versions exactes verrouillées
└── src/
    ├── main.rs      ← binaire principal
    └── lib.rs       ← bibliothèque (optionnel)
```

> **Important :** `Cargo.lock` doit être commité pour les binaires, ignoré pour les bibliothèques.

---$BLK10001$, 'code', 2),
(10002, 2, '2.1 Métadonnées du projet', $BLK10002$```toml
[package]
name    = "mon_projet"
version = "0.1.0"
edition = "2024"
authors = ["Ton Nom <ton@email.com>"]
description = "Une courte description"
license = "MIT"
```$BLK10002$, 'code', 1),
(10003, 2, '2.2 Dépendances', $BLK10003$```toml
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
| `"*"` | n'importe quelle version |$BLK10003$, 'code', 2),
(10004, 2, '2.3 Profils de compilation', $BLK10004$```toml
[profile.release]
opt-level = 3      # optimisation maximale
lto = true         # Link Time Optimization
strip = true       # supprime les symboles de debug

[profile.dev]
opt-level = 0      # pas d'optimisation, compilation rapide
debug = true       # symboles de debug
```

---$BLK10004$, 'code', 3),
(10005, 3, NULL, $BLK10005$```bash
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
```$BLK10005$, 'code', 1),
(10006, 3, NULL, $BLK10006$---$BLK10006$, 'text', 2),
(10007, 4, NULL, $BLK10007$Les features permettent d'activer des fonctionnalités optionnelles.$BLK10007$, 'text', 1),
(10008, 4, 'Déclarer des features', $BLK10008$```toml
[features]
default  = ["json"]          # features actives par défaut
json     = ["serde/derive"]  # active serde avec derive
async    = ["tokio"]         # feature async optionnelle
full     = ["json", "async"] # groupe de features

[dependencies]
serde = { version = "1.0", optional = true }
tokio = { version = "1.0", optional = true }
```$BLK10008$, 'code', 2),
(10009, 4, 'Utiliser les features dans le code', $BLK10009$```rust
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
```$BLK10009$, 'code', 3),
(10010, 4, 'Activer des features à la compilation', $BLK10010$```bash
cargo build --features "json,async"
cargo build --all-features
cargo build --no-default-features
```

---$BLK10010$, 'code', 4),
(10011, 5, NULL, $BLK10011$Un workspace regroupe plusieurs crates dans un même projet.$BLK10011$, 'text', 1),
(10012, 5, NULL, $BLK10012$```toml
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
```$BLK10012$, 'code', 2),
(10013, 5, NULL, $BLK10013$```toml
# app/Cargo.toml
[package]
name    = "app"
version.workspace = true      # hérite la version du workspace
edition.workspace = true

[dependencies]
core  = { path = "../core" }
serde.workspace = true        # hérite la dépendance du workspace
```$BLK10013$, 'code', 3),
(10014, 5, NULL, $BLK10014$```bash
cargo build -p core           # compile uniquement la crate 'core'
cargo test --workspace        # teste toutes les crates
```$BLK10014$, 'code', 4),
(10015, 5, NULL, $BLK10015$---$BLK10015$, 'text', 5),
(10016, 6, NULL, $BLK10016$```bash
# 1. Connexion (token sur crates.io)
cargo login <ton_token>

# 2. Vérifier avant de publier
cargo publish --dry-run

# 3. Publier
cargo publish
```$BLK10016$, 'code', 1),
(10017, 6, NULL, $BLK10017$**Checklist avant publication :**$BLK10017$, 'text', 2),
(10018, 6, NULL, $BLK10018$- `name`, `version`, `description`, `license` renseignés dans `Cargo.toml`
- `README.md` présent (sera affiché sur crates.io)
- Documentation avec `///` sur les éléments publics
- Tests passants (`cargo test`)
- Pas de chemins locaux dans `[dependencies]`$BLK10018$, 'list', 3);

-- variables-et-fonctions.md (cour_id=2)
INSERT INTO chapitre (id, cour_id, slug, title, lead, sort_order) VALUES
(7, 2, 'variables-et-fonctions-variables-mutabilite-et-fonctions', 'Variables, Mutabilité et Fonctions', NULL, 1),
(8, 2, 'variables-et-fonctions-1-les-variables-en-rust', '1. Les variables en Rust', NULL, 2),
(9, 2, 'variables-et-fonctions-11-declaration-de-base', '1.1 - Déclaration de base', NULL, 3),
(10, 2, 'variables-et-fonctions-12-immutabilite-par-defaut', '1.2 - Immutabilité par défaut', NULL, 4),
(11, 2, 'variables-et-fonctions-13-variables-mutables', '1.3 - Variables mutables', NULL, 5),
(12, 2, 'variables-et-fonctions-14-les-constantes', '1.4 - Les constantes', NULL, 6),
(13, 2, 'variables-et-fonctions-differences-entre-let-et-const', 'Différences entre let et const :', NULL, 7),
(14, 2, 'variables-et-fonctions-15-le-shadowing', '1.5 - Le shadowing', NULL, 8),
(15, 2, 'variables-et-fonctions-2-les-types-de-donnees', '2. Les types de données', NULL, 9),
(16, 2, 'variables-et-fonctions-21-types-scalaires', '2.1 - Types scalaires', NULL, 10),
(17, 2, 'variables-et-fonctions-types-dentiers-en-rust', 'Types d''entiers en Rust :', NULL, 11),
(18, 2, 'variables-et-fonctions-22-types-composes', '2.2 - Types composés', NULL, 12),
(19, 2, 'variables-et-fonctions-23-inference-de-type', '2.3 - Inférence de type', NULL, 13),
(20, 2, 'variables-et-fonctions-24-annotation-de-type', '2.4 - Annotation de type', NULL, 14),
(21, 2, 'variables-et-fonctions-3-les-fonctions', '3. Les fonctions', NULL, 15),
(22, 2, 'variables-et-fonctions-31-declaration-de-base', '3.1 - Déclaration de base', NULL, 16),
(23, 2, 'variables-et-fonctions-32-parametres', '3.2 - Paramètres', NULL, 17),
(24, 2, 'variables-et-fonctions-33-valeur-de-retour', '3.3 - Valeur de retour', NULL, 18),
(25, 2, 'variables-et-fonctions-34-expressions-vs-instructions', '3.4 - Expressions vs instructions', NULL, 19),
(26, 2, 'variables-et-fonctions-35-fonctions-avec-references', '3.5 - Fonctions avec références', NULL, 20),
(27, 2, 'variables-et-fonctions-4-ownership-et-borrowing-introduction', '4. Ownership et Borrowing (introduction)', NULL, 21),
(28, 2, 'variables-et-fonctions-41-le-concept-downership', '4.1 - Le concept d''ownership', NULL, 22),
(29, 2, 'variables-et-fonctions-42-les-references-et', '4.2 - Les références (&)', NULL, 23),
(30, 2, 'variables-et-fonctions-43-les-references-mutables-etmut', '4.3 - Les références mutables (&mut;)', NULL, 24),
(31, 2, 'variables-et-fonctions-5-exercices-pratiques', '5. Exercices pratiques', NULL, 25),
(32, 2, 'variables-et-fonctions-exercice-3-references', 'Exercice 3 : Références', NULL, 26),
(33, 2, 'variables-et-fonctions-variables', 'Variables', NULL, 27),
(34, 2, 'variables-et-fonctions-fonctions', 'Fonctions', NULL, 28),
(35, 2, 'variables-et-fonctions-types-courants', 'Types courants', NULL, 29),
(36, 2, 'variables-et-fonctions-bravo', 'Bravo !', NULL, 30),
(37, 2, 'variables-et-fonctions-les-prochaines-etapes', 'Les prochaines étapes :', NULL, 31),
(38, 2, 'variables-et-fonctions-ressources-recommandees', 'Ressources recommandées :', NULL, 32);
INSERT INTO cour_block (id, chapitre_id, heading, content, block_type, sort_order) VALUES
(10019, 7, NULL, $BLK10019$Guide Complet pour Débutants$BLK10019$, 'text', 1),
(10020, 8, NULL, $BLK10020$En Rust, les variables sont déclarées avec le mot-clé `let` . Une des particularités de Rust est que **les variables sont immutables par défaut** .$BLK10020$, 'text', 1),
(10021, 9, NULL, $BLK10021$```
// Déclaration simple
let x = 5;
println!("La valeur de x est : {}", x);
```$BLK10021$, 'code', 1),
(10022, 9, NULL, $BLK10022$```
// Déclaration avec type explicite
let y: i32 = 10;
println!("La valeur de y est : {}", y);
```$BLK10022$, 'code', 2),
(10023, 9, NULL, $BLK10023$```
// Déclaration de plusieurs variables
let a = 1;
let b = 2;
let c = 3;
```$BLK10023$, 'code', 3),
(10024, 9, NULL, $BLK10024$I **Note :** En Rust, on utilise `println!` (avec un !) pour afficher du texte. Le `{}` est remplacé par la valeur de la variable.$BLK10024$, 'text', 4),
(10025, 10, NULL, $BLK10025$Par défaut, les variables Rust sont **immutables** : on ne peut pas changer leur valeur après leur déclaration.$BLK10025$, 'text', 1),
(10026, 10, NULL, $BLK10026$```
let x = 5;
println!("x = {}", x);
```$BLK10026$, 'code', 2),
(10027, 10, NULL, $BLK10027$**`x = 6;  //`** I **`ERREUR DE COMPILATION ! // error[E0384]: cannot assign twice to immutable variable `x``**$BLK10027$, 'text', 3),
(10028, 10, NULL, $BLK10028$II **Pourquoi ?** L'immutabilité par défaut aide à éviter les bugs. Si tu veux modifier une variable, tu dois le déclarer explicitement avec `mut` .$BLK10028$, 'text', 4),
(10029, 11, NULL, $BLK10029$Pour rendre une variable modifiable, on ajoute `mut` après `let` .$BLK10029$, 'text', 1),
(10030, 11, NULL, $BLK10030$**`let mut x = 5; println!("x = {}", x);  // Affiche : x = 5 x = 6;  //`** I **`OK ! println!("x = {}", x);  // Affiche : x = 6`**$BLK10030$, 'text', 2),
(10031, 11, NULL, $BLK10031$```
// Modification multiple
let mut compteur = 0;
compteur = compteur + 1;
compteur = compteur + 1;
println!("compteur = {}", compteur);  // Affiche : compteur = 2
```$BLK10031$, 'code', 3),
(10032, 11, NULL, $BLK10032$I **Conseil :** Utilise `mut` seulement quand nécessaire. Cela rend ton code plus sûr et plus facile à comprendre.$BLK10032$, 'text', 4),
(10033, 12, NULL, $BLK10033$Les constantes sont déclarées avec `const` et sont **toujours immutables** . Elles doivent avoir un type explicite et sont évaluées à la compilation.$BLK10033$, 'text', 1),
(10034, 12, NULL, $BLK10034$```
// Constante (MAJUSCULES par convention)
const MAX_POINTS: u32 = 100_000;
const PI: f64 = 3.14159;
```$BLK10034$, 'code', 2),
(10035, 12, NULL, $BLK10035$```
fn main() {
```$BLK10035$, 'code', 3),
(10036, 12, NULL, $BLK10036$```
    println!("Le maximum est : {}", MAX_POINTS);
```$BLK10036$, 'code', 4),
(10037, 12, NULL, $BLK10037$**`//`** I **`Impossible avec une constante :`**$BLK10037$, 'text', 5),
(10038, 12, NULL, $BLK10038$- **`// const ne peut pas être mut`**$BLK10038$, 'list', 6),
(10039, 12, NULL, $BLK10039$```
    // const doit avoir un type explicite
}
```$BLK10039$, 'code', 7),
(10040, 13, NULL, $BLK10040$||**let**|**const**|
|---|---|---|
|**Immutable par défaut**|I|I|
|**Peut être mut**|I|I|
|**Type explicite requis**|I|I|
|**Portée**|Block|Globale|
|**Valeur calculée**|Runtime|Compilation|
|**Convention nom**|snake_case|UPPER_CASE|$BLK10040$, 'table', 1),
(10041, 14, NULL, $BLK10041$Le **shadowing** permet de redéclarer une variable avec le même nom. C'est différent de la mutabilité !$BLK10041$, 'text', 1),
(10042, 14, NULL, $BLK10042$**`let x = 5; println!("x = {}", x);  // 5 let x = x + 1;  //`** I **`Nouvelle variable qui masque la précédente println!("x = {}", x);  // 6 let x = x * 2;  //`** I **`Encore une nouvelle variable println!("x = {}", x);  // 12 // Shadowing permet de changer le type ! let espaces = "   ";  // Type: &str let espaces = espaces.len();  // Type: usize println!("Il y a {} espaces", espaces);  // 3`**$BLK10042$, 'text', 2),
(10043, 14, 'Shadowing vs mut :', $BLK10043$• `mut` : Modifie la valeur, **même type** 

- `let` (shadowing) : Crée une nouvelle variable, **peut changer de type**$BLK10043$, 'text', 3),
(10044, 15, NULL, $BLK10044$Rust est un langage **statiquement typé** : chaque variable a un type connu à la compilation. Rust peut souvent **inférer** le type, mais tu peux aussi l'annoter explicitement.$BLK10044$, 'text', 1),
(10045, 16, NULL, $BLK10045$Les types scalaires représentent une valeur unique. Rust en a quatre types principaux :$BLK10045$, 'text', 1),
(10046, 16, NULL, $BLK10046$```
// 1. ENTIERS (integers)
let a: i32 = 42;        // Entier signé 32 bits
let b: u64 = 100;       // Entier non signé 64 bits
let c = 5;              // i32 par défaut
```$BLK10046$, 'code', 2),
(10047, 16, NULL, $BLK10047$```
// Différentes tailles : i8, i16, i32, i64, i128, isize
//                       u8, u16, u32, u64, u128, usize
```$BLK10047$, 'code', 3),
(10048, 16, NULL, $BLK10048$```
// 2. FLOTTANTS (floating-point)
let x: f64 = 2.5;       // 64 bits (défaut)
let y: f32 = 3.14;      // 32 bits
```$BLK10048$, 'code', 4),
(10049, 16, NULL, $BLK10049$```
// 3. BOOLÉENS (boolean)
let vrai: bool = true;
let faux: bool = false;
```$BLK10049$, 'code', 5),
(10050, 16, NULL, $BLK10050$**`// 4. CARACTÈRES (char) let lettre: char = 'A'; let emoji: char = '`** I **`';  // Unicode !`**$BLK10050$, 'text', 6),
(10051, 17, NULL, $BLK10051$|**Type**|**Taille**|**Minimum**|**Maximum**|
|---|---|---|---|
|i8|8 bits|-128|127|
|u8|8 bits|0|255|
|i32|32 bits|-2 milliards|2 milliards|
|u32|32 bits|0|4 milliards|
|i64|64 bits|Très grand négatif|Très grand positif|
|isize|Taille du système|Variable|Variable|$BLK10051$, 'table', 1),
(10052, 18, NULL, $BLK10052$Les types composés regroupent plusieurs valeurs dans un seul type.$BLK10052$, 'text', 1),
(10053, 18, NULL, $BLK10053$```
// 1. TUPLES - Types différents, taille fixe
let personne: (&str, i32, bool) = ("Alice", 25, true);
```$BLK10053$, 'code', 2),
(10054, 18, NULL, $BLK10054$```
// Accès par déstructuration
let (nom, age, actif) = personne;
println!("{} a {} ans", nom, age);
```$BLK10054$, 'code', 3),
(10055, 18, NULL, $BLK10055$```
// Accès par index
println!("Nom : {}", personne.0);
println!("Age : {}", personne.1);
```$BLK10055$, 'code', 4),
(10056, 18, NULL, $BLK10056$```
// 2. TABLEAUX - Même type, taille fixe
let nombres: [i32; 5] = [1, 2, 3, 4, 5];
```$BLK10056$, 'code', 5),
(10057, 18, NULL, $BLK10057$```
// Accès par index
let premier = nombres[0];  // 1
let dernier = nombres[4];  // 5
```$BLK10057$, 'code', 6),
(10058, 18, NULL, $BLK10058$```
// Tableau avec valeur répétée
let zeros = [0; 10];  // [0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
```$BLK10058$, 'code', 7),
(10059, 18, NULL, $BLK10059$```
// 3. SLICES - Vue sur une partie d'un tableau
let slice = &nombres[1..3];  // [2, 3]
```$BLK10059$, 'code', 8),
(10060, 19, NULL, $BLK10060$```
// Rust infère le type automatiquement
let x = 5;           // i32 (défaut pour entiers)
let y = 2.5;         // f64 (défaut pour flottants)
let z = true;        // bool
let nom = "Alice";   // &str
```$BLK10060$, 'code', 1),
(10061, 19, NULL, $BLK10061$```
// Rust peut inférer selon l'utilisation
let mut nombres = Vec::new();  // Type inconnu
nombres.push(1);               // Maintenant Rust sait : Vec<i32>
```$BLK10061$, 'code', 2),
(10062, 20, NULL, $BLK10062$```
// Parfois nécessaire pour lever l'ambiguïté
let nombre: u32 = "42".parse().expect("Pas un nombre!");
```$BLK10062$, 'code', 1),
(10063, 20, NULL, $BLK10063$```
// Types complexes
let vecteur: Vec<i32> = vec![1, 2, 3];
let hashmap: std::collections::HashMap<String, i32>
    = std::collections::HashMap::new();
```$BLK10063$, 'code', 2),
(10064, 21, NULL, $BLK10064$Les fonctions sont omniprésentes en Rust. Elles sont déclarées avec `fn` et utilisent la convention **snake_case** pour les noms.$BLK10064$, 'text', 1),
(10065, 22, NULL, $BLK10065$**`// Fonction sans paramètre ni retour fn dire_bonjour() { println!("Bonjour !"); } // Fonction principale fn main() { dire_bonjour();  // Appel de fonction dire_bonjour();  // On peut l'appeler plusieurs fois } // Convention de nommage fn ma_fonction() { }        //`** I **`snake_case fn MaFonction() { }         //`** I **`éviter fn calculer_total() { }     //`** I **`fn calculerTotal() { }      //`** I **`éviter (camelCase)`**$BLK10065$, 'text', 1),
(10066, 23, NULL, $BLK10066$Les paramètres doivent **toujours avoir un type explicite** .$BLK10066$, 'text', 1),
(10067, 23, NULL, $BLK10067$```
// Un paramètre
fn afficher_nombre(x: i32) {
    println!("Le nombre est : {}", x);
}
// Plusieurs paramètres
fn additionner(a: i32, b: i32) {
    let somme = a + b;
    println!("{} + {} = {}", a, b, somme);
}
// Appel
fn main() {
    afficher_nombre(42);
    additionner(5, 7);
}
// Paramètres de types différents
fn presenter(nom: &str, age: i32, actif: bool) {
    println!("{} a {} ans, actif: {}", nom, age, actif);
}
```$BLK10067$, 'code', 2),
(10068, 24, NULL, $BLK10068$Une fonction retourne une valeur avec `->` suivi du type. La dernière expression est retournée automatiquement (pas de `return` nécessaire).$BLK10068$, 'text', 1),
(10069, 24, NULL, $BLK10069$**`// Fonction avec retour fn additionner(a: i32, b: i32) -> i32 { a + b  //`** II **`PAS de point-virgule ! }`**$BLK10069$, 'text', 2),
(10070, 24, NULL, $BLK10070$```
fn main() {
    let resultat = additionner(5, 3);
    println!("5 + 3 = {}", resultat);  // 8
}
```$BLK10070$, 'code', 3),
(10071, 24, NULL, $BLK10071$```
// Avec return explicite (utile pour retour anticipé)
fn valeur_absolue(x: i32) -> i32 {
    if x < 0 {
        return -x;  // Retour anticipé
    }
    x  // Retour normal
}
```$BLK10071$, 'code', 4),
(10072, 24, NULL, $BLK10072$```
// Plusieurs retours possibles
fn diviser(a: i32, b: i32) -> Option<i32> {
    if b == 0 {
        return None;  // Division par zéro
    }
    Some(a / b)
}
```$BLK10072$, 'code', 5),
(10073, 24, NULL, $BLK10073$II **Attention !** En Rust, une expression **sans point-virgule** est retournée. Avec point-virgule, c'est une instruction qui ne retourne rien.$BLK10073$, 'text', 6),
(10074, 25, NULL, $BLK10074$**`//`** I **`Expression (retourne une valeur) fn plus_un(x: i32) -> i32 { x + 1  // Pas de ;`**$BLK10074$, 'text', 1),
(10075, 25, NULL, $BLK10075$```
}
```$BLK10075$, 'code', 2),
(10076, 25, NULL, $BLK10076$**`//`** I **`Instruction (ne retourne rien) fn plus_un_incorrect(x: i32) -> i32 { x + 1;  // Avec ; = ne retourne rien ! // error: mismatched types (expected i32, found ()) }`**$BLK10076$, 'text', 3),
(10077, 25, NULL, $BLK10077$```
// Les blocs sont des expressions
fn exemple() -> i32 {
    let x = {
        let y = 3;
        y + 1  // Cette expression est la valeur du bloc
    };  // x vaut 4
```$BLK10077$, 'code', 4),
(10078, 25, NULL, $BLK10078$```
    x * 2  // Retourne 8
```$BLK10078$, 'code', 5),
(10079, 25, NULL, $BLK10079$```
}
```$BLK10079$, 'code', 6),
(10080, 26, NULL, $BLK10080$Pour éviter de copier les données, on peut passer des **références** aux fonctions.$BLK10080$, 'text', 1),
(10081, 26, NULL, $BLK10081$```
// Sans référence (copie la valeur)
fn afficher_nombre(x: i32) {
    println!("Nombre : {}", x);
}
```$BLK10081$, 'code', 2),
(10082, 26, NULL, $BLK10082$```
// Avec référence (emprunte la valeur)
fn afficher_chaine(s: &String) {
    println!("Chaîne : {}", s);
}
```$BLK10082$, 'code', 3),
(10083, 26, NULL, $BLK10083$```
// Référence mutable (peut modifier)
fn incrementer(x: &mut i32) {
    *x += 1;  // * = déréférencement
}
```$BLK10083$, 'code', 4),
(10084, 26, NULL, $BLK10084$```
fn main() {
    let nombre = 42;
    afficher_nombre(nombre);  // Copie
```$BLK10084$, 'code', 5),
(10085, 26, NULL, $BLK10085$```
    let texte = String::from("Hello");
    afficher_chaine(&texte);  // Emprunte
```$BLK10085$, 'code', 6),
(10086, 26, NULL, $BLK10086$```
    let mut compteur = 0;
    incrementer(&mut compteur);
    println!("Compteur : {}", compteur);  // 1
}
```$BLK10086$, 'code', 7),
(10087, 27, NULL, $BLK10087$L' **ownership** est le concept le plus important et unique de Rust. C'est ce qui permet à Rust d'être sûr sans garbage collector.$BLK10087$, 'text', 1),
(10088, 28, NULL, $BLK10088$Règles de base :$BLK10088$, 'text', 1),
(10089, 28, NULL, $BLK10089$1. Chaque valeur a un **propriétaire** unique$BLK10089$, 'list', 2),
(10090, 28, NULL, $BLK10090$2. Il ne peut y avoir qu' **un seul propriétaire** à la fois$BLK10090$, 'list', 3),
(10091, 28, NULL, $BLK10091$3. Quand le propriétaire sort du scope, la valeur est **libérée**$BLK10091$, 'list', 4),
(10092, 28, NULL, $BLK10092$**`fn main() { let s1 = String::from("hello"); let s2 = s1;  // s1 est "déplacé" vers s2 // println!("{}", s1);  //`** I **`ERREUR ! s1 n'est plus valide println!("{}", s2);     //`** I **`OK } // Exemple avec fonction fn prend_ownership(s: String) { println!("{}", s); }  // s est libéré ici`**$BLK10092$, 'text', 5),
(10093, 28, NULL, $BLK10093$```
fn main() {
    let texte = String::from("hello");
    prend_ownership(texte);
```$BLK10093$, 'code', 6),
(10094, 28, NULL, $BLK10094$**`// println!("{}", texte);  //`** I **`ERREUR ! // texte a été déplacé dans la fonction }`**$BLK10094$, 'text', 7),
(10095, 28, NULL, $BLK10095$I **Types Copy :** Les types simples (i32, f64, bool, char) sont `Copy` : ils sont copiés au lieu d'être déplacés.$BLK10095$, 'text', 8),
(10096, 29, NULL, $BLK10096$Les **références** permettent d'emprunter une valeur sans en prendre ownership.$BLK10096$, 'text', 1),
(10097, 29, NULL, $BLK10097$```
fn calculer_longueur(s: &String) -> usize {
    s.len()
```$BLK10097$, 'code', 2),
(10098, 29, NULL, $BLK10098$```
}  // s sort du scope, mais ne possède pas la String
```$BLK10098$, 'code', 3),
(10099, 29, NULL, $BLK10099$**`fn main() { let texte = String::from("hello"); let longueur = calculer_longueur(&texte); println!("'{}' a {} caractères", texte, longueur); //`** I **`texte est toujours valide ! }`**$BLK10099$, 'text', 4),
(10100, 29, NULL, $BLK10100$**`// Multiples références immutables OK fn main() { let s = String::from("hello"); let r1 = &s; let r2 = &s; println!("{} et {}", r1, r2);  //`** I **`OK }`**$BLK10100$, 'text', 5),
(10101, 30, NULL, $BLK10101$```
fn ajouter_monde(s: &mut String) {
    s.push_str(", world!");
}
```$BLK10101$, 'code', 1),
(10102, 30, NULL, $BLK10102$```
fn main() {
    let mut texte = String::from("hello");
    ajouter_monde(&mut texte);
    println!("{}", texte);  // "hello, world!"
}
```$BLK10102$, 'code', 2),
(10103, 30, NULL, $BLK10103$**`//`** II **`UNE SEULE référence mutable à la fois ! fn main() { let mut s = String::from("hello"); let r1 = &mut s; // let r2 = &mut s;  //`** I **`ERREUR ! // On ne peut pas avoir deux références mutables println!("{}", r1); } //`** II **`Pas de mélange immutable + mutable fn main() { let mut s = String::from("hello"); let r1 = &s;      // OK let r2 = &s;      // OK // let r3 = &mut s;  //`** I **`ERREUR ! println!("{} et {}", r1, r2); }`**$BLK10103$, 'text', 3),
(10104, 30, NULL, $BLK10104$II **Règles du borrowing :** 1. Soit **une** référence mutable 2. Soit **plusieurs** références immutables 3. Mais **jamais les deux en même temps** !$BLK10104$, 'text', 4),
(10105, 31, NULL, $BLK10105$Voici quelques exercices pour pratiquer :$BLK10105$, 'text', 1),
(10106, 31, 'Exercice 1 : Variables', $BLK10106$Complète ce code pour qu'il compile : 

```
fn main() {
    let x = 5;
    // Ajoute le code nécessaire pour que x devienne 10
    println!("x = {}", x);  // Doit afficher "x = 10"
}
// Solution :
fn main() {
    let mut x = 5;  // Ajouter mut
    x = 10;
    println!("x = {}", x);
}
```$BLK10106$, 'text', 2),
(10107, 31, 'Exercice 2 : Fonction simple', $BLK10107$Crée une fonction qui multiplie deux nombres : 

```
// À compléter
fn multiplier(/* paramètres */) /* -> type */ {
    // ton code
}
fn main() {
    let resultat = multiplier(6, 7);
    println!("6 x 7 = {}", resultat);  // Doit afficher 42
}
// Solution :
fn multiplier(a: i32, b: i32) -> i32 {
    a * b
}
```$BLK10107$, 'text', 3),
(10108, 32, NULL, $BLK10108$Crée une fonction qui double un nombre sans le déplacer :$BLK10108$, 'text', 1),
(10109, 32, NULL, $BLK10109$```
fn doubler(/* référence mutable */) {
    // ton code
}
fn main() {
    let mut nombre = 5;
    doubler(/* passer référence */);
    println!("Nombre : {}", nombre);  // Doit afficher 10
}
// Solution :
fn doubler(x: &mut i32) {
    *x *= 2;
}
fn main() {
    let mut nombre = 5;
    doubler(&mut nombre);
    println!("Nombre : {}", nombre);
}
```$BLK10109$, 'code', 2),
(10110, 33, NULL, $BLK10110$|**Syntaxe**|**Description**|**Exemple**|
|---|---|---|
|let x = 5;|Variable immutable|let nom = "Alice";|
|let mut x = 5;|Variable mutable|let mut compteur = 0;|
|const MAX: i32 = 100;|Constante|const PI: f64 = 3.14;|
|let x = 5; let x = 10;|Shadowing|let x = x + 1;|$BLK10110$, 'table', 1),
(10111, 34, NULL, $BLK10111$|**Syntaxe**|**Description**|**Exemple**|
|---|---|---|
|fn nom() { }|Fonction simple|fn dire_bonjour() { }|
|fn nom(x: i32) { }|Avec paramètres|fn afficher(n: i32) { }|
|fn nom() -> i32 { }|Avec retour|fn double(x: i32) -> i32 { x * 2 }|
|fn nom(s: &String) { }|Référence|fn longueur(s: &String) -> usize { }|
|fn nom(x: &mut i32) { }|Réf. mutable|fn incrementer(x: &mut i32) { }|$BLK10111$, 'table', 1),
(10112, 35, NULL, $BLK10112$|**Type**|**Description**|**Exemple**|
|---|---|---|
|i32, u32, i64...|Entiers|let x: i32 = 42;|
|f32, f64|Flottants|let pi: f64 = 3.14;|
|bool|Booléen|let actif: bool = true;|
|char|Caractère|let lettre: char = 'A';|
|&str|Chaîne immutable|let texte = "hello";|
|String|Chaîne mutable|let s = String::from("hi");|$BLK10112$, 'table', 1),
(10113, 36, NULL, $BLK10113$Tu connais maintenant les bases de Rust !$BLK10113$, 'text', 1),
(10114, 37, NULL, $BLK10114$• Pratiquer avec de petits programmes$BLK10114$, 'text', 1),
(10115, 37, NULL, $BLK10115$• Apprendre les structures (struct) et enums$BLK10115$, 'text', 2),
(10116, 37, NULL, $BLK10116$• Maîtriser le pattern matching$BLK10116$, 'text', 3),
(10117, 37, NULL, $BLK10117$• Explorer les collections (Vec, HashMap)$BLK10117$, 'text', 4),
(10118, 37, NULL, $BLK10118$I **Continue à coder et n'aie pas peur des erreurs du compilateur : elles sont là pour t'aider !** I$BLK10118$, 'text', 5),
(10119, 38, NULL, $BLK10119$I The Rust Book (en français) : https://jimskapt.github.io/rust-book-fr/$BLK10119$, 'text', 1),
(10120, 38, NULL, $BLK10120$I Rustlings (exercices) : https://github.com/rust-lang/rustlings$BLK10120$, 'text', 2),
(10121, 38, NULL, $BLK10121$I Rust by Example : https://doc.rust-lang.org/rust-by-example/$BLK10121$, 'text', 3),
(10122, 38, NULL, $BLK10122$I Forum Rust : https://users.rust-lang.org/$BLK10122$, 'text', 4);

-- structures-et-controle.md (cour_id=3)
INSERT INTO chapitre (id, cour_id, slug, title, lead, sort_order) VALUES
(39, 3, 'structures-et-controle-conditions-boucles-et-pattern-matching', 'Conditions, Boucles et Pattern Matching', NULL, 1),
(40, 3, 'structures-et-controle-1-les-conditions-if-else', '1. Les conditions (if/else)', NULL, 2),
(41, 3, 'structures-et-controle-2-les-boucles', '2. Les boucles', NULL, 3),
(42, 3, 'structures-et-controle-3-pattern-matching-match', '3. Pattern matching (match)', NULL, 4),
(43, 3, 'structures-et-controle-1-les-conditions-if-else', '1. Les conditions (if/else)', NULL, 5),
(44, 3, 'structures-et-controle-11-if-basique', '1.1 - if basique', NULL, 6),
(45, 3, 'structures-et-controle-12-else-et-else-if', '1.2 - else et else if', NULL, 7),
(46, 3, 'structures-et-controle-13-if-comme-expression', '1.3 - if comme expression', NULL, 8),
(47, 3, 'structures-et-controle-2-les-boucles', '2. Les boucles', NULL, 9),
(48, 3, 'structures-et-controle-21-loop-boucle-infinie', '2.1 - loop (boucle infinie)', NULL, 10),
(49, 3, 'structures-et-controle-22-while-avec-condition', '2.2 - while (avec condition)', NULL, 11),
(50, 3, 'structures-et-controle-23-for-iteration', '2.3 - for (itération)', NULL, 12),
(51, 3, 'structures-et-controle-24-break-et-continue', '2.4 - break et continue', NULL, 13),
(52, 3, 'structures-et-controle-25-labels-de-boucles', '2.5 - Labels de boucles', NULL, 14),
(53, 3, 'structures-et-controle-3-pattern-matching-match', '3. Pattern matching (match)', NULL, 15),
(54, 3, 'structures-et-controle-31-match-basique', '3.1 - match basique', NULL, 16),
(55, 3, 'structures-et-controle-32-patterns-avances', '3.2 - Patterns avancés', NULL, 17),
(56, 3, 'structures-et-controle-33-guards-conditions', '3.3 - Guards (conditions)', NULL, 18),
(57, 3, 'structures-et-controle-4-if-let-et-while-let', '4. if let et while let', NULL, 19),
(58, 3, 'structures-et-controle-5-exemples-pratiques', '5. Exemples pratiques', NULL, 20),
(59, 3, 'structures-et-controle-exemple-2-fizzbuzz', 'Exemple 2 : FizzBuzz', NULL, 21),
(60, 3, 'structures-et-controle-exemple-3-recherche-dans-un-tableau', 'Exemple 3 : Recherche dans un tableau', NULL, 22),
(61, 3, 'structures-et-controle-6-exercices-pratiques', '6. Exercices pratiques', NULL, 23),
(62, 3, 'structures-et-controle-conditions', 'Conditions', NULL, 24),
(63, 3, 'structures-et-controle-boucles', 'Boucles', NULL, 25),
(64, 3, 'structures-et-controle-pattern-matching', 'Pattern Matching', NULL, 26),
(65, 3, 'structures-et-controle-raccourcis', 'Raccourcis', NULL, 27),
(66, 3, 'structures-et-controle-points-cles', 'Points clés', NULL, 28),
(67, 3, 'structures-et-controle-tu-maitrises-les-structures-de-controle', 'Tu maîtrises les structures de contrôle !', NULL, 29);
INSERT INTO cour_block (id, chapitre_id, heading, content, block_type, sort_order) VALUES
(10123, 39, NULL, $BLK10123$Maîtriser le Flux de Contrôle$BLK10123$, 'text', 1),
(10124, 40, NULL, $BLK10124$- 1.1 - if basique$BLK10124$, 'list', 1),
(10125, 40, NULL, $BLK10125$1.2 - else et else if$BLK10125$, 'text', 2),
(10126, 40, NULL, $BLK10126$1.3 - if comme expression$BLK10126$, 'text', 3),
(10127, 41, NULL, $BLK10127$- 2.1 - loop (boucle infinie)$BLK10127$, 'list', 1),
(10128, 41, NULL, $BLK10128$- 2.2 - while (avec condition)$BLK10128$, 'list', 2),
(10129, 41, NULL, $BLK10129$- 2.3 - for (itération)$BLK10129$, 'list', 3),
(10130, 41, NULL, $BLK10130$- 2.4 - break et continue$BLK10130$, 'list', 4),
(10131, 41, NULL, $BLK10131$2.5 - Labels de boucles$BLK10131$, 'text', 5),
(10132, 42, NULL, $BLK10132$- 3.1 - match basique$BLK10132$, 'list', 1),
(10133, 42, NULL, $BLK10133$- 3.2 - Patterns avancés$BLK10133$, 'list', 2),
(10134, 42, NULL, $BLK10134$3.3 - Guards (conditions)$BLK10134$, 'text', 3),
(10135, 42, NULL, $BLK10135$**4. if let et while let**$BLK10135$, 'text', 4),
(10136, 42, NULL, $BLK10136$**5. Exemples pratiques**$BLK10136$, 'text', 5),
(10137, 42, NULL, $BLK10137$**6. Exercices**$BLK10137$, 'text', 6),
(10138, 42, NULL, $BLK10138$**7. Aide-mémoire**$BLK10138$, 'text', 7),
(10139, 43, NULL, $BLK10139$Les conditions permettent d'exécuter du code selon qu'une expression est vraie ou fausse. En Rust, la condition doit toujours être un booléen (bool).$BLK10139$, 'text', 1),
(10140, 44, NULL, $BLK10140$```
// Condition simple
fn main() {
    let nombre = 7;
    if nombre < 10 {
        println!("Le nombre est petit");
    }
}
```$BLK10140$, 'code', 1),
(10141, 44, NULL, $BLK10141$```
// ■■ Important : La condition DOIT être un bool
let x = 5;
// if x { } // ■ ERREUR ! x n'est pas un bool
if x != 0 { } // ■ OK !
```$BLK10141$, 'code', 2),
(10142, 45, NULL, $BLK10142$```
fn main() {
    let nombre = 7;
    // if-else
    if nombre % 2 == 0 {
        println!("Pair");
    } else {
        println!("Impair");
    }
```$BLK10142$, 'code', 1),
(10143, 45, NULL, $BLK10143$```
    // if-else if-else
    if nombre < 0 {
        println!("Négatif");
    } else if nombre == 0 {
        println!("Zéro");
    } else {
        println!("Positif");
    }
```$BLK10143$, 'code', 2),
(10144, 45, NULL, $BLK10144$```
}
```$BLK10144$, 'code', 3),
(10145, 46, NULL, $BLK10145$En Rust, if est une expression : il retourne une valeur !$BLK10145$, 'text', 1),
(10146, 46, NULL, $BLK10146$```
// if retourne une valeur
fn main() {
    let condition = true;
```$BLK10146$, 'code', 2),
(10147, 46, NULL, $BLK10147$```
    let nombre = if condition { 5 } else { 6 };
    println!("nombre = {}", nombre); // 5
```$BLK10147$, 'code', 3),
(10148, 46, NULL, $BLK10148$```
    // Exemple pratique
    let age = 20;
    let statut = if age >= 18 { "Majeur" } else { "Mineur" };
    println!("Statut : {}", statut);
}
```$BLK10148$, 'code', 4),
(10149, 46, NULL, $BLK10149$```
// ■■ Les types doivent correspondre !
let x = if condition {
    5 // Type: i32
} else {
    // "six" // ■ ERREUR : types incompatibles
    6 // ■ OK : même type
};
```$BLK10149$, 'code', 5),
(10150, 46, NULL, $BLK10150$**■ if comme expression : Les deux branches (if et else) doivent retourner le même type. Si une branche n'a pas de valeur de retour, elle retourne () (unit type).**$BLK10150$, 'text', 6),
(10151, 47, NULL, $BLK10151$Rust propose trois types de boucles : loop (infinie), while (avec condition), et for (itération).$BLK10151$, 'text', 1),
(10152, 48, NULL, $BLK10152$loop crée une boucle infinie. On doit utiliser break pour en sortir.$BLK10152$, 'text', 1),
(10153, 48, NULL, $BLK10153$```
// Boucle infinie simple
fn main() {
    let mut compteur = 0;
    loop {
        compteur += 1;
        println!("Compteur : {}", compteur);
        if compteur == 5 {
            break; // Sort de la boucle
        }
    }
}
// loop peut retourner une valeur !
fn main() {
    let mut compteur = 0;
    let resultat = loop {
        compteur += 1;
        if compteur == 10 {
            break compteur * 2; // Retourne 20
        }
    };
    println!("Résultat : {}", resultat); // 20
}
```$BLK10153$, 'code', 2),
(10154, 49, NULL, $BLK10154$while exécute une boucle tant qu'une condition est vraie.$BLK10154$, 'text', 1),
(10155, 49, NULL, $BLK10155$```
fn main() {
    let mut nombre = 3;
```$BLK10155$, 'code', 2),
(10156, 49, NULL, $BLK10156$```
    // Compte à rebours
```$BLK10156$, 'code', 3),
(10157, 49, NULL, $BLK10157$```
    while nombre != 0 {
        println!("{}!", nombre);
        nombre -= 1;
    }
    println!("Décollage !");
}
```$BLK10157$, 'code', 4),
(10158, 49, NULL, $BLK10158$```
// Exemple pratique : saisie utilisateur
use std::io;
fn main() {
    let mut input = String::new();
    let mut essais = 3;
    while essais > 0 {
        println!("Entrez le mot de passe ({} essais restants):",
essais);
        input.clear();
        io::stdin().read_line(&mut input).expect("Erreur de lecture");
        if input.trim() == "secret" {
            println!("Accès autorisé !");
            break;
        }
        essais -= 1;
    }
}
```$BLK10158$, 'code', 5),
(10159, 50, NULL, $BLK10159$for permet d'itérer sur une collection ou une plage de valeurs. C'est la boucle la plus utilisée en Rust !$BLK10159$, 'text', 1),
(10160, 50, NULL, $BLK10160$```
// Itérer sur une plage
fn main() {
    for i in 1..6 {
        println!("i = {}", i); // 1, 2, 3, 4, 5
    }
    // Plage inclusive (avec =)
    for i in 1..=5 {
        println!("i = {}", i); // 1, 2, 3, 4, 5
    }
}
// Itérer sur un tableau
fn main() {
    let nombres = [10, 20, 30, 40, 50];
    for nombre in nombres {
        println!("Nombre : {}", nombre);
    }
    // Avec index
    for (index, valeur) in nombres.iter().enumerate() {
        println!("Index {} : {}", index, valeur);
    }
}
// Itérer sur un vecteur
fn main() {
    let fruits = vec!["pomme", "banane", "orange"];
    for fruit in &fruits {
        println!("Fruit : {}", fruit);
    }
    // fruits est toujours utilisable ici
    println!("Nombre de fruits : {}", fruits.len());
```$BLK10160$, 'code', 2),
(10161, 50, NULL, $BLK10161$```
}
```$BLK10161$, 'code', 3),
(10162, 50, NULL, $BLK10162$**■ for vs while : Préfère toujours for quand tu itères sur une collection. C'est plus sûr (pas de risque d'index hors limite) et plus idiomatique en Rust !**$BLK10162$, 'text', 4),
(10163, 51, NULL, $BLK10163$break sort de la boucle, continue passe à l'itération suivante.$BLK10163$, 'text', 1),
(10164, 51, NULL, $BLK10164$```
// break : sortir de la boucle
fn main() {
    for i in 1..10 {
        if i == 5 {
            break; // Sort complètement de la boucle
        }
        println!("{}", i); // Affiche 1, 2, 3, 4
    }
}
```$BLK10164$, 'code', 2),
(10165, 51, NULL, $BLK10165$```
// continue : passer à l'itération suivante
fn main() {
    for i in 1..10 {
        if i % 2 == 0 {
            continue; // Saute les nombres pairs
        }
        println!("{}", i); // Affiche 1, 3, 5, 7, 9
    }
}
```$BLK10165$, 'code', 3),
(10166, 51, NULL, $BLK10166$```
// Exemple pratique : trouver un élément
fn main() {
    let nombres = vec![1, 5, 8, 12, 15, 20];
    let recherche = 12;
    let mut trouve = false;
    for nombre in nombres {
        if nombre == recherche {
            println!("Trouvé : {}", nombre);
            trouve = true;
            break; // Pas besoin de continuer
        }
    }
    if !trouve {
        println!("Non trouvé");
    }
```$BLK10166$, 'code', 4),
(10167, 51, NULL, $BLK10167$```
}
```$BLK10167$, 'code', 5),
(10168, 52, NULL, $BLK10168$Les labels permettent de contrôler des boucles imbriquées. `// Boucles imbriquées avec labels fn main() { let mut compteur = 0; 'exterieur: loop { println!("Compteur extérieur = {}", compteur); let mut restant = 10; loop { println!("  Restant = {}", restant); if restant == 7 { break; // Sort de la boucle intérieure } if compteur == 2 { break 'exterieur; // Sort des DEUX boucles } restant -= 1; } compteur += 1; } }`$BLK10168$, 'text', 1),
(10169, 52, NULL, $BLK10169$**■ Labels : Les labels commencent par ' (apostrophe) et permettent de break ou continue une boucle spécifique dans des boucles imbriquées.**$BLK10169$, 'text', 2),
(10170, 53, NULL, $BLK10170$match est l'outil le plus puissant de Rust pour le contrôle de flux. Il vérifie tous les cas possibles et le compilateur garantit que tu n'en oublies aucun !$BLK10170$, 'text', 1),
(10171, 54, NULL, $BLK10171$```
// Match simple
fn main() {
    let nombre = 3;
    match nombre {
        1 => println!("Un"),
        2 => println!("Deux"),
        3 => println!("Trois"),
        _ => println!("Autre"), // _ = tous les autres cas
    }
}
// match retourne une valeur
fn main() {
    let nombre = 2;
    let resultat = match nombre {
        1 => "premier",
        2 => "deuxième",
        3 => "troisième",
        _ => "autre",
    };
    println!("C'est le {} nombre", resultat);
}
// Match avec Option
fn diviser(a: i32, b: i32) -> Option<i32> {
    if b == 0 {
        None
    } else {
        Some(a / b)
    }
}
```$BLK10171$, 'code', 1),
(10172, 54, NULL, $BLK10172$```
fn main() {
    let resultat = diviser(10, 2);
    match resultat {
        Some(valeur) => println!("Résultat : {}", valeur),
        None => println!("Division par zéro !"),
    }
}
```$BLK10172$, 'code', 2),
(10173, 54, NULL, $BLK10173$**■■ Match exhaustif : Le compilateur vérifie que tu couvres TOUS les cas possibles. Utilise _ pour capturer tous les cas restants.**$BLK10173$, 'text', 3),
(10174, 55, NULL, $BLK10174$```
// Plusieurs valeurs (OR)
fn main() {
    let nombre = 5;
    match nombre {
        1 | 3 | 5 | 7 | 9 => println!("Impair"),
        2 | 4 | 6 | 8 | 10 => println!("Pair"),
        _ => println!("Hors plage"),
    }
}
// Plages de valeurs
fn main() {
    let nombre = 42;
    match nombre {
        0 => println!("Zéro"),
        1..=10 => println!("Petit"),
        11..=50 => println!("Moyen"),
        51..=100 => println!("Grand"),
        _ => println!("Très grand"),
    }
}
// Déstructuration de tuples
fn main() {
    let point = (0, 5);
    match point {
        (0, 0) => println!("Origine"),
        (0, y) => println!("Sur l'axe Y : {}", y),
        (x, 0) => println!("Sur l'axe X : {}", x),
        (x, y) => println!("Point ({}, {})", x, y),
    }
}
```$BLK10174$, 'code', 1),
(10175, 56, NULL, $BLK10175$Les guards ajoutent une condition supplémentaire après un pattern.$BLK10175$, 'text', 1),
(10176, 56, NULL, $BLK10176$```
fn main() {
```$BLK10176$, 'code', 2),
(10177, 56, NULL, $BLK10177$```
    let nombre = 4;
```$BLK10177$, 'code', 3),
(10178, 56, NULL, $BLK10178$```
    match nombre {
        n if n < 0 => println!("Négatif : {}", n),
        n if n % 2 == 0 => println!("Pair : {}", n),
        n => println!("Impair positif : {}", n),
    }
}
```$BLK10178$, 'code', 4),
(10179, 56, NULL, $BLK10179$```
// Exemple avec Option
fn main() {
    let nombre: Option<i32> = Some(7);
```$BLK10179$, 'code', 5),
(10180, 56, NULL, $BLK10180$```
    match nombre {
        Some(n) if n < 5 => println!("Petit : {}", n),
        Some(n) if n >= 5 && n < 10 => println!("Moyen : {}", n),
        Some(n) => println!("Grand : {}", n),
        None => println!("Pas de valeur"),
    }
```$BLK10180$, 'code', 6),
(10181, 56, NULL, $BLK10181$```
}
```$BLK10181$, 'code', 7),
(10182, 57, NULL, $BLK10182$if let et while let sont des raccourcis pour match quand tu t'intéresses à un seul cas.$BLK10182$, 'text', 1),
(10183, 57, NULL, $BLK10183$```
// Sans if let (verbeux)
fn main() {
    let nombre: Option<i32> = Some(5);
```$BLK10183$, 'code', 2),
(10184, 57, NULL, $BLK10184$```
    match nombre {
        Some(n) => println!("Valeur : {}", n),
        _ => (), // Ignore None
    }
}
// Avec if let (concis)
fn main() {
    let nombre: Option<i32> = Some(5);
    if let Some(n) = nombre {
        println!("Valeur : {}", n);
    }
}
// if let avec else
fn main() {
    let nombre: Option<i32> = None;
    if let Some(n) = nombre {
        println!("Valeur : {}", n);
    } else {
        println!("Pas de valeur");
    }
}
// while let : boucle tant que pattern match
fn main() {
    let mut pile = vec![1, 2, 3, 4, 5];
    // Vide la pile
    while let Some(sommet) = pile.pop() {
        println!("Sommet : {}", sommet); // 5, 4, 3, 2, 1
```$BLK10184$, 'code', 3),
(10185, 57, NULL, $BLK10185$```
    }
```$BLK10185$, 'code', 4),
(10186, 57, NULL, $BLK10186$```
}
```$BLK10186$, 'code', 5),
(10187, 57, NULL, $BLK10187$**■ Quand utiliser if let ? Quand tu t'intéresses à un seul cas et que tu veux ignorer les autres. Plus lisible que match avec _ quand approprié.**$BLK10187$, 'text', 6),
(10188, 58, NULL, $BLK10188$**Exemple 1 : Calculatrice simple**$BLK10188$, 'text', 1),
(10189, 58, NULL, $BLK10189$```
fn calculer(operateur: char, a: i32, b: i32) -> Option<i32> {
    match operateur {
        '+' => Some(a + b),
        '-' => Some(a - b),
        '*' => Some(a * b),
        '/' => {
            if b == 0 {
                None
            } else {
                Some(a / b)
            }
        }
        _ => None,
    }
}
fn main() {
    let resultat = calculer('+', 10, 5);
    match resultat {
        Some(n) => println!("Résultat : {}", n),
        None => println!("Opération invalide"),
    }
}
```$BLK10189$, 'code', 2),
(10190, 59, NULL, $BLK10190$```
fn main() {
    for i in 1..=100 {
        match (i % 3, i % 5) {
            (0, 0) => println!("FizzBuzz"),
            (0, _) => println!("Fizz"),
            (_, 0) => println!("Buzz"),
            _ => println!("{}", i),
        }
    }
}
```$BLK10190$, 'code', 1),
(10191, 60, NULL, $BLK10191$```
fn trouver_index(tableau: &[i32], valeur: i32) -> Option<usize> {
```$BLK10191$, 'code', 1),
(10192, 60, NULL, $BLK10192$```
    for (index, &element) in tableau.iter().enumerate() {
        if element == valeur {
            return Some(index);
        }
    }
    None
}
fn main() {
    let nombres = [10, 20, 30, 40, 50];
    if let Some(index) = trouver_index(&nombres, 30) {
        println!("Trouvé à l'index {}", index);
    } else {
        println!("Non trouvé");
    }
}
```$BLK10192$, 'code', 2),
(10193, 61, NULL, $BLK10193$**■ Exercice 1 : Classification d'âge**$BLK10193$, 'text', 1),
(10194, 61, NULL, $BLK10194$Écris une fonction qui classe un âge en catégorie :$BLK10194$, 'text', 2),
(10195, 61, NULL, $BLK10195$- 0-12 : Enfant$BLK10195$, 'list', 3),
(10196, 61, NULL, $BLK10196$- 13-17 : Adolescent$BLK10196$, 'list', 4),
(10197, 61, NULL, $BLK10197$- 18-64 : Adulte$BLK10197$, 'list', 5),
(10198, 61, NULL, $BLK10198$• 65+ : Senior `// Solution : fn classifier_age(age: u32) -> &'static str { match age { 0..=12 => "Enfant", 13..=17 => "Adolescent", 18..=64 => "Adulte", _ => "Senior", } }`$BLK10198$, 'text', 6),
(10199, 61, NULL, $BLK10199$**■ Exercice 2 : Somme avec boucle** Calcule la somme des nombres de 1 à n.$BLK10199$, 'text', 7),
(10200, 61, NULL, $BLK10200$```
// Solution :
fn somme_jusqua(n: i32) -> i32 {
    let mut somme = 0;
    for i in 1..=n {
        somme += i;
    }
    somme
}
fn main() {
    println!("Somme 1 à 10 : {}", somme_jusqua(10)); // 55
}
```$BLK10200$, 'code', 8),
(10201, 61, NULL, $BLK10201$**■ Exercice 3 : Nombres premiers** Vérifie si un nombre est premier.$BLK10201$, 'text', 9),
(10202, 61, NULL, $BLK10202$```
// Solution :
fn est_premier(n: u32) -> bool {
    if n < 2 {
        return false;
    }
    for i in 2..n {
        if n % i == 0 {
            return false;
```$BLK10202$, 'code', 10),
(10203, 61, NULL, $BLK10203$```
        }
    }
    true
}
```$BLK10203$, 'code', 11),
(10204, 61, NULL, $BLK10204$```
fn main() {
    for n in 2..=20 {
        if est_premier(n) {
            println!("{} est premier", n);
        }
    }
}
```$BLK10204$, 'code', 12),
(10205, 62, NULL, $BLK10205$```
if condition { } else { }
if condition { } else if condition2 { } else { }
let x = if condition { valeur1 } else { valeur2 };
```$BLK10205$, 'code', 1),
(10206, 63, NULL, $BLK10206$```
loop { break; }                    // Infinie
while condition { }                // Avec condition
for i in 0..10 { }                 // 0 à 9
for i in 0..=10 { }                // 0 à 10 (inclusif)
for item in collection { }         // Itération
break;                             // Sort de la boucle
continue;                          // Itération suivante
```$BLK10206$, 'code', 1),
(10207, 64, NULL, $BLK10207$```
match valeur {
```$BLK10207$, 'code', 1),
(10208, 64, NULL, $BLK10208$- `1 => expression,`$BLK10208$, 'list', 2),
(10209, 64, NULL, $BLK10209$- `2 | 3 => expression,`$BLK10209$, 'list', 3),
(10210, 64, NULL, $BLK10210$- `4..=10 => expression,`$BLK10210$, 'list', 4),
(10211, 64, NULL, $BLK10211$```
    n if n > 10 => expression,
```$BLK10211$, 'code', 5),
(10212, 64, NULL, $BLK10212$- `_ => expression,`$BLK10212$, 'list', 6),
(10213, 64, NULL, $BLK10213$```
}
```$BLK10213$, 'code', 7),
(10214, 65, NULL, $BLK10214$```
if let Some(x) = option { }        // Match un seul cas
while let Some(x) = iter.next() { } // Boucle avec pattern
```$BLK10214$, 'code', 1),
(10215, 66, NULL, $BLK10215$- Les conditions doivent être des booléens$BLK10215$, 'list', 1),
(10216, 66, NULL, $BLK10216$- if peut retourner une valeur$BLK10216$, 'list', 2),
(10217, 66, NULL, $BLK10217$- Préfère for aux boucles while avec index$BLK10217$, 'list', 3),
(10218, 66, NULL, $BLK10218$- match doit être exhaustif$BLK10218$, 'list', 4),
(10219, 66, NULL, $BLK10219$- Utilise if let pour simplifier match avec un seul cas$BLK10219$, 'list', 5),
(10220, 66, NULL, $BLK10220$- Les labels permettent de contrôler des boucles imbriquées$BLK10220$, 'list', 6),
(10221, 67, NULL, $BLK10221$Tu peux maintenant :$BLK10221$, 'text', 1),
(10222, 67, NULL, $BLK10222$- Contrôler le flux de ton programme$BLK10222$, 'list', 2),
(10223, 67, NULL, $BLK10223$- • Utiliser les boucles efficacement • Maîtriser le pattern matching$BLK10223$, 'list', 3),
(10224, 67, NULL, $BLK10224$- • Écrire du code Rust idiomatique$BLK10224$, 'list', 4),
(10225, 67, NULL, $BLK10225$**■ Continue ton apprentissage avec les structures et les enums ! ■**$BLK10225$, 'text', 5);

-- structures-enums.md (cour_id=4)
INSERT INTO chapitre (id, cour_id, slug, title, lead, sort_order) VALUES
(68, 4, 'structures-enums-organiser-et-structurer-vos-donnees', 'Organiser et structurer vos données', NULL, 1),
(69, 4, 'structures-enums-1-les-structures-struct', '1. Les Structures (struct)', NULL, 2),
(70, 4, 'structures-enums-11-definition-de-base', '1.1 - Définition de base', NULL, 3),
(71, 4, 'structures-enums-12-instanciation', '1.2 - Instanciation', NULL, 4),
(72, 4, 'structures-enums-raccourcis-dinitialisation', 'Raccourcis d''initialisation', NULL, 5),
(73, 4, 'structures-enums-13-methodes-et-fonctions-associees', '1.3 - Méthodes et fonctions associées', NULL, 6),
(74, 4, 'structures-enums-methode-vs-fonction-associee', '■ **Méthode vs Fonction associée :**', NULL, 7),
(75, 4, 'structures-enums-14-tuple-structs', '1.4 - Tuple structs', NULL, 8),
(76, 4, 'structures-enums-15-unit-structs', '1.5 - Unit structs', NULL, 9),
(77, 4, 'structures-enums-2-les-enumerations-enum', '2. Les Énumérations (enum)', NULL, 10),
(78, 4, 'structures-enums-21-definition-de-base', '2.1 - Définition de base', NULL, 11),
(79, 4, 'structures-enums-22-enums-avec-donnees', '2.2 - Enums avec données', NULL, 12),
(80, 4, 'structures-enums-23-optiont-valeurs-optionnelles', '2.3 - Option<T> (valeurs optionnelles)', NULL, 13),
(81, 4, 'structures-enums-unwrap-vs-unwrap-or', '■■ **unwrap() vs unwrap_or() :**', NULL, 14),
(82, 4, 'structures-enums-24-resultt-e-gestion-derreurs', '2.4 - Result<T, E> (gestion d''erreurs)', NULL, 15),
(83, 4, 'structures-enums-3-pattern-matching', '3. Pattern Matching', NULL, 16),
(84, 4, 'structures-enums-31-lexpression-match', '3.1 - L''expression match', NULL, 17),
(85, 4, 'structures-enums-32-patterns-avances', '3.2 - Patterns avancés', NULL, 18),
(86, 4, 'structures-enums-33-if-let-et-while-let', '3.3 - if let et while let', NULL, 19),
(87, 4, 'structures-enums-34-destructuration', '3.4 - Déstructuration', NULL, 20),
(88, 4, 'structures-enums-exemple-1-systeme-de-gestion-dutilisateurs', 'Exemple 1 : Système de gestion d''utilisateurs', NULL, 21),
(89, 4, 'structures-enums-exemple-2-calculatrice-avec-result', 'Exemple 2 : Calculatrice avec Result', NULL, 22),
(90, 4, 'structures-enums-exercice-1-creer-une-struct-livre', '■ **Exercice 1 : Créer une struct Livre**', NULL, 23),
(91, 4, 'structures-enums-exercice-3-option-et-result', '■ **Exercice 3 : Option et Result**', NULL, 24),
(92, 4, 'structures-enums-structures', '■ **Structures**', NULL, 25),
(93, 4, 'structures-enums-enumerations', '■ **Énumérations**', NULL, 26);
INSERT INTO cour_block (id, chapitre_id, heading, content, block_type, sort_order) VALUES
(10226, 68, NULL, $BLK10226$Guide Complet avec Pattern Matching$BLK10226$, 'text', 1),
(10227, 69, NULL, $BLK10227$Les **structures** permettent de regrouper plusieurs données liées ensemble. C'est similaire aux classes dans d'autres langages, mais sans héritage.$BLK10227$, 'text', 1),
(10228, 70, NULL, $BLK10228$```
// Définir une structure
struct Utilisateur {
    nom: String,
    email: String,
    age: u32,
    actif: bool,
}
// Structure avec types différents
struct Point {
    x: f64,
    y: f64,
}
struct Rectangle {
    largeur: u32,
    hauteur: u32,
}
```$BLK10228$, 'code', 1),
(10229, 70, NULL, $BLK10229$■ **Convention :** Les noms de struct utilisent **PascalCase** (première lettre de chaque mot en majuscule).$BLK10229$, 'text', 2),
(10230, 71, NULL, $BLK10230$```
fn main() {
    // Créer une instance
    let utilisateur1 = Utilisateur {
        nom: String::from("Alice"),
        email: String::from("alice@example.com"),
        age: 25,
        actif: true,
    };
    // Accéder aux champs
    println!("Nom : {}", utilisateur1.nom);
    println!("Email : {}", utilisateur1.email);
    // Instance mutable
    let mut utilisateur2 = Utilisateur {
        nom: String::from("Bob"),
        email: String::from("bob@example.com"),
        age: 30,
        actif: false,
    };
    // Modifier un champ
    utilisateur2.actif = true;
    utilisateur2.age = 31;
}
```$BLK10230$, 'code', 1),
(10231, 71, NULL, $BLK10231$■■ **Important :** Toute l'instance doit être mutable, on ne peut pas rendre seulement certains champs mutables.$BLK10231$, 'text', 2),
(10232, 72, NULL, $BLK10232$```
// Raccourci si variable = nom du champ
fn creer_utilisateur(nom: String, email: String) -> Utilisateur {
    Utilisateur {
        nom,      // Au lieu de nom: nom
        email,    // Au lieu de email: email
        age: 18,
        actif: true,
    }
}
```$BLK10232$, 'code', 1),
(10233, 72, NULL, $BLK10233$```
// Copier depuis une autre instance
fn main() {
    let utilisateur1 = Utilisateur {
        nom: String::from("Alice"),
        email: String::from("alice@example.com"),
        age: 25,
        actif: true,
    };
    // Créer utilisateur2 avec la plupart des champs de utilisateur1
    let utilisateur2 = Utilisateur {
        email: String::from("bob@example.com"),
        ..utilisateur1  // Copie le reste
    };
}
```$BLK10233$, 'code', 2),
(10234, 73, NULL, $BLK10234$On utilise `impl` pour définir des méthodes sur une struct.$BLK10234$, 'text', 1),
(10235, 73, NULL, $BLK10235$```
struct Rectangle {
    largeur: u32,
    hauteur: u32,
}
impl Rectangle {
    // Méthode (prend &self)
    fn aire(&self) -> u32 {
        self.largeur * self.hauteur
    }
    // Méthode avec référence mutable
    fn doubler(&mut self) {
        self.largeur *= 2;
        self.hauteur *= 2;
    }
    // Fonction associée (pas de self)
    fn carre(taille: u32) -> Rectangle {
        Rectangle {
            largeur: taille,
            hauteur: taille,
        }
    }
}
fn main() {
    let rect = Rectangle {
        largeur: 30,
        hauteur: 50,
    };
    println!("Aire : {}", rect.aire());  // 1500
    // Fonction associée (avec ::)
    let carre = Rectangle::carre(20);
    println!("Aire du carré : {}", carre.aire());  // 400
}
```$BLK10235$, 'code', 2),
(10236, 74, NULL, $BLK10236$• **Méthode** : Prend `self` , appelée avec `.`$BLK10236$, 'text', 1),
(10237, 74, NULL, $BLK10237$• **Fonction associée** : Pas de `self` , appelée avec `::` (comme `String::from` )$BLK10237$, 'text', 2),
(10238, 75, NULL, $BLK10238$```
// Struct sans noms de champs
struct Couleur(i32, i32, i32);
struct Point(i32, i32, i32);
```$BLK10238$, 'code', 1),
(10239, 75, NULL, $BLK10239$```
fn main() {
    let noir = Couleur(0, 0, 0);
    let origine = Point(0, 0, 0);
    // Accès par index
    println!("Rouge : {}", noir.0);
    println!("X : {}", origine.0);
    // Déstructuration
    let Couleur(r, g, b) = noir;
    println!("RGB : {}, {}, {}", r, g, b);
}
```$BLK10239$, 'code', 2),
(10240, 76, NULL, $BLK10240$```
// Struct sans champs (pour implémenter des traits)
struct AlwaysEqual;
```$BLK10240$, 'code', 1),
(10241, 76, NULL, $BLK10241$```
fn main() {
    let instance = AlwaysEqual;
}
```$BLK10241$, 'code', 2),
(10242, 77, NULL, $BLK10242$Les **énumérations** permettent de définir un type avec plusieurs variantes possibles. En Rust, les enums sont très puissants et peuvent contenir des données.$BLK10242$, 'text', 1),
(10243, 78, NULL, $BLK10243$```
// Enum simple
enum Mouvement {
    Haut,
    Bas,
    Gauche,
    Droite,
}
fn bouger(direction: Mouvement) {
    // Utilisation avec match
    match direction {
        Mouvement::Haut => println!("On monte"),
        Mouvement::Bas => println!("On descend"),
        Mouvement::Gauche => println!("On va à gauche"),
        Mouvement::Droite => println!("On va à droite"),
    }
}
fn main() {
    let dir = Mouvement::Haut;
    bouger(dir);
}
```$BLK10243$, 'code', 1),
(10244, 79, NULL, $BLK10244$C'est  la  vraie  puissance  des  enums  en  Rust  :  chaque  variante  peut  contenir  des  données différentes !$BLK10244$, 'text', 1),
(10245, 79, NULL, $BLK10245$```
// Chaque variante peut avoir des données différentes
enum Message {
    Quitter,                        // Pas de données
    Deplacer { x: i32, y: i32 },   // Struct anonyme
    Ecrire(String),                 // String
    ChangerCouleur(i32, i32, i32), // Trois i32
}
impl Message {
    fn appeler(&self) {
        match self {
            Message::Quitter => {
                println!("Quitter l'application");
            }
            Message::Deplacer { x, y } => {
                println!("Déplacer à ({}, {})", x, y);
            }
            Message::Ecrire(texte) => {
                println!("Écrire : {}", texte);
            }
            Message::ChangerCouleur(r, g, b) => {
                println!("Couleur RGB({}, {}, {})", r, g, b);
            }
        }
    }
}
fn main() {
    let msg1 = Message::Ecrire(String::from("Bonjour"));
    let msg2 = Message::Deplacer { x: 10, y: 20 };
    msg1.appeler();  // "Écrire : Bonjour"
    msg2.appeler();  // "Déplacer à (10, 20)"
}
```$BLK10245$, 'code', 2),
(10246, 80, NULL, $BLK10246$`Option<T>` est l'enum le plus important de Rust. Il remplace `null` des autres langages.$BLK10246$, 'text', 1),
(10247, 80, NULL, $BLK10247$```
// Définition de Option (déjà dans la bibliothèque standard)
enum Option<T> {
    Some(T),  // Contient une valeur
    None,     // Pas de valeur
}
// Utilisation
fn diviser(a: f64, b: f64) -> Option<f64> {
    if b == 0.0 {
        None  // Division par zéro impossible
    } else {
        Some(a / b)
    }
}
fn main() {
    let resultat = diviser(10.0, 2.0);
    match resultat {
        Some(valeur) => println!("Résultat : {}", valeur),
        None => println!("Division par zéro !"),
    }
    // Méthodes pratiques sur Option
    let x: Option<i32> = Some(5);
    println!("{}", x.is_some());     // true
    println!("{}", x.is_none());     // false
    println!("{}", x.unwrap());      // 5 (panic si None !)
    println!("{}", x.unwrap_or(0));  // 5 (ou 0 si None)
    // Map et autres transformations
    let doubled = x.map(|n| n * 2);  // Some(10)
    // Chaînage sûr
    let nombre = Some(5);
    let carre = nombre.map(|n| n * n);  // Some(25)
}
```$BLK10247$, 'code', 2),
(10248, 81, NULL, $BLK10248$• `unwrap()` : Panic si `None` (utiliser seulement si tu es sûr)$BLK10248$, 'text', 1),
(10249, 81, NULL, $BLK10249$- `unwrap_or(valeur)` : Retourne la valeur par défaut si `None` (plus sûr)$BLK10249$, 'list', 2),
(10250, 82, NULL, $BLK10250$`Result<T, E>` est utilisé pour les opérations qui peuvent échouer. C'est la base de la gestion d'erreurs en Rust.$BLK10250$, 'text', 1),
(10251, 82, NULL, $BLK10251$```
// Définition de Result
enum Result<T, E> {
    Ok(T),   // Succès avec valeur
    Err(E),  // Erreur
}
// Exemple pratique
use std::fs::File;
use std::io::Error;
fn ouvrir_fichier(nom: &str) -> Result<File, Error> {
    File::open(nom)
}
fn main() {
    match ouvrir_fichier("hello.txt") {
        Ok(fichier) => println!("Fichier ouvert !"),
        Err(erreur) => println!("Erreur : {}", erreur),
    }
    // Raccourci avec ? (propage l'erreur)
    fn lire_fichier() -> Result<String, Error> {
        let mut fichier = File::open("hello.txt")?;  // ? propage l'erreur
        let mut contenu = String::new();
        fichier.read_to_string(&mut contenu)?;
        Ok(contenu)
    }
    // Méthodes pratiques
    let resultat: Result<i32, &str> = Ok(42);
    println!("{}", resultat.is_ok());           // true
    println!("{}", resultat.is_err());          // false
    println!("{}", resultat.unwrap());          // 42
    println!("{}", resultat.unwrap_or(0));      // 42
    println!("{}", resultat.expect("Erreur")); // 42 (message si panic)
}
```$BLK10251$, 'code', 2),
(10252, 82, NULL, $BLK10252$■ **L'opérateur ? :** Propage automatiquement les erreurs. Si `Err` , retourne l'erreur immédiatement. Si `Ok` , extrait la valeur.$BLK10252$, 'text', 3),
(10253, 83, NULL, $BLK10253$Le **pattern matching** avec `match` est une des fonctionnalités les plus puissantes de Rust.$BLK10253$, 'text', 1),
(10254, 84, NULL, $BLK10254$```
// Match simple
fn valeur_en_centimes(piece: Piece) -> u8 {
    match piece {
        Piece::Penny => 1,
        Piece::Nickel => 5,
        Piece::Dime => 10,
        Piece::Quarter => 25,
    }
}
// Match avec enum contenant des données
enum Message {
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(i32, i32, i32),
}
fn traiter(msg: Message) {
    match msg {
        Message::Move { x, y } => {
            println!("Déplacer à ({}, {})", x, y);
        }
        Message::Write(texte) => {
            println!("Texte : {}", texte);
        }
        Message::ChangeColor(r, g, b) => {
            println!("RGB({}, {}, {})", r, g, b);
        }
    }
}
// Match avec Option
fn plus_un(x: Option<i32>) -> Option<i32> {
    match x {
        None => None,
        Some(i) => Some(i + 1),
    }
}
// Match exhaustif obligatoire !
fn nombre_pair(x: i32) -> bool {
    match x % 2 {
        0 => true,
        _ => false,  // _ = catch-all (tous les autres cas)
    }
}
```$BLK10254$, 'code', 1),
(10255, 84, NULL, $BLK10255$■■ **Match  exhaustif  :** Tu  dois  couvrir **tous  les  cas  possibles** .  Le  compilateur  vérifie  ! Utilise `_` pour attraper tous les cas restants.$BLK10255$, 'text', 2),
(10256, 85, NULL, $BLK10256$```
// Match avec plusieurs valeurs
fn decrire_nombre(x: i32) {
    match x {
        1 => println!("Un"),
        2 | 3 | 5 | 7 => println!("Nombre premier petit"),
        10..=20 => println!("Entre 10 et 20"),
        _ => println!("Autre chose"),
    }
}
// Match avec conditions (guards)
fn est_pair_et_positif(x: i32) -> bool {
    match x {
        n if n > 0 && n % 2 == 0 => true,
        _ => false,
    }
}
// Déstructuration dans match
struct Point {
    x: i32,
    y: i32,
}
fn position(point: Point) {
    match point {
        Point { x: 0, y: 0 } => println!("Origine"),
        Point { x: 0, y } => println!("Sur l'axe Y à y={}", y),
        Point { x, y: 0 } => println!("Sur l'axe X à x={}", x),
        Point { x, y } => println!("Point ({}, {})", x, y),
    }
}
// @ binding (capturer et tester)
fn analyser_age(age: i32) {
    match age {
        n @ 0..=12 => println!("Enfant de {} ans", n),
        n @ 13..=19 => println!("Ado de {} ans", n),
        n => println!("Adulte de {} ans", n),
    }
}
```$BLK10256$, 'code', 1),
(10257, 86, NULL, $BLK10257$Raccourcis quand tu t'intéresses à **un seul cas** d'un enum.$BLK10257$, 'text', 1),
(10258, 86, NULL, $BLK10258$```
// Avec match (verbeux)
let config_max = Some(3u8);
match config_max {
    Some(max) => println!("Max : {}", max),
    _ => (),  // On ignore None
}
// Avec if let (concis)
if let Some(max) = config_max {
    println!("Max : {}", max);
}
// if let avec else
let nombre = Some(7);
if let Some(n) = nombre {
    println!("Nombre : {}", n);
} else {
    println!("Pas de nombre");
}
// while let (boucle tant que pattern match)
let mut pile = vec![1, 2, 3];
while let Some(sommet) = pile.pop() {
    println!("{}", sommet);  // 3, 2, 1
}
```$BLK10258$, 'code', 2),
(10259, 86, NULL, $BLK10259$■ **Quand utiliser if let ?**$BLK10259$, 'text', 3),
(10260, 86, NULL, $BLK10260$Quand tu ne t'intéresses qu'à **un seul cas** et que tu veux ignorer les autres. Plus lisible que `match` avec `_` .$BLK10260$, 'text', 4),
(10261, 87, NULL, $BLK10261$```
// Déstructurer un tuple
let (x, y, z) = (1, 2, 3);
println!("{}, {}, {}", x, y, z);
```$BLK10261$, 'code', 1),
(10262, 87, NULL, $BLK10262$```
// Déstructurer une struct
struct Point { x: i32, y: i32 }
let p = Point { x: 0, y: 7 };
let Point { x, y } = p;
println!("x: {}, y: {}", x, y);
```$BLK10262$, 'code', 2),
(10263, 87, NULL, $BLK10263$```
// Renommer pendant la déstructuration
let Point { x: a, y: b } = p;
println!("a: {}, b: {}", a, b);
```$BLK10263$, 'code', 3),
(10264, 87, NULL, $BLK10264$```
// Ignorer des valeurs
let Point { x, .. } = p;  // Ignore y
println!("x: {}", x);
```$BLK10264$, 'code', 4),
(10265, 87, NULL, $BLK10265$```
// Dans les paramètres de fonction
fn afficher_point(&Point { x, y }: &Point) {
    println!("Point ({}, {})", x, y);
}
```$BLK10265$, 'code', 5),
(10266, 88, NULL, $BLK10266$```
enum Role {
    Admin,
    Moderateur,
    Utilisateur,
}
struct Compte {
    id: u32,
    nom: String,
    email: String,
    role: Role,
}
impl Compte {
    fn nouveau(id: u32, nom: String, email: String) -> Compte {
        Compte {
            id,
            nom,
            email,
            role: Role::Utilisateur,
        }
    }
    fn promouvoir(&mut self, nouveau_role: Role) {
        self.role = nouveau_role;
    }
    fn afficher_permissions(&self) {
        match self.role {
            Role::Admin => println!("{} : Accès total", self.nom),
            Role::Moderateur => println!("{} : Peut modérer", self.nom),
            Role::Utilisateur => println!("{} : Accès limité", self.nom),
        }
    }
}
fn main() {
    let mut compte = Compte::nouveau(
        1,
        String::from("Alice"),
        String::from("alice@example.com")
    );
    compte.afficher_permissions();
    compte.promouvoir(Role::Admin);
    compte.afficher_permissions();
}
```$BLK10266$, 'code', 1),
(10267, 89, NULL, $BLK10267$```
enum Operation {
    Addition,
    Soustraction,
    Multiplication,
    Division,
}
fn calculer(a: f64, b: f64, op: Operation) -> Result<f64, String> {
    match op {
        Operation::Addition => Ok(a + b),
        Operation::Soustraction => Ok(a - b),
        Operation::Multiplication => Ok(a * b),
        Operation::Division => {
            if b == 0.0 {
                Err(String::from("Division par zéro"))
            } else {
                Ok(a / b)
            }
        }
    }
}
fn main() {
    let resultat = calculer(10.0, 2.0, Operation::Division);
    match resultat {
        Ok(valeur) => println!("Résultat : {}", valeur),
        Err(erreur) => println!("Erreur : {}", erreur),
    }
    // Avec ?
    fn faire_calculs() -> Result<(), String> {
        let r1 = calculer(10.0, 2.0, Operation::Division)?;
        let r2 = calculer(r1, 3.0, Operation::Addition)?;
        println!("Résultat final : {}", r2);
        Ok(())
    }
}
```$BLK10267$, 'code', 1),
(10268, 90, NULL, $BLK10268$```
// Crée une struct Livre avec : titre, auteur, pages
// Ajoute une méthode est_long() qui retourne true si > 300 pages
// Solution :
struct Livre {
    titre: String,
    auteur: String,
    pages: u32,
}
impl Livre {
    fn est_long(&self) -> bool {
        self.pages > 300
    }
}
```$BLK10268$, 'code', 1),
(10269, 90, NULL, $BLK10269$■ **Exercice 2 : Enum avec match**$BLK10269$, 'text', 2),
(10270, 90, NULL, $BLK10270$```
// Crée un enum Saison avec 4 variantes
// Écris une fonction qui retourne le nombre de jours
// Solution :
enum Saison {
    Printemps,
    Ete,
    Automne,
    Hiver,
}
```$BLK10270$, 'code', 3),
(10271, 90, NULL, $BLK10271$```
fn jours_approximatifs(saison: Saison) -> u32 {
    match saison {
        Saison::Printemps | Saison::Automne => 92,
        Saison::Ete => 93,
        Saison::Hiver => 89,
    }
}
```$BLK10271$, 'code', 4),
(10272, 91, NULL, $BLK10272$```
// Écris une fonction qui trouve un élément dans un Vec
// Retourne Option<usize> (l'index)
```$BLK10272$, 'code', 1),
(10273, 91, NULL, $BLK10273$```
// Solution :
fn trouver<T: PartialEq>(vec: &Vec<T>, element: &T) -> Option<usize> {
    for (index, item) in vec.iter().enumerate() {
        if item == element {
            return Some(index);
        }
    }
    None
}
```$BLK10273$, 'code', 2),
(10274, 91, NULL, $BLK10274$```
fn main() {
    let nombres = vec![1, 2, 3, 4, 5];
```$BLK10274$, 'code', 3),
(10275, 91, NULL, $BLK10275$```
    match trouver(&nombres, &3) {
        Some(index) => println!("Trouvé à l'index {}", index),
        None => println!("Pas trouvé"),
    }
```$BLK10275$, 'code', 4),
(10276, 91, NULL, $BLK10276$```
}
```$BLK10276$, 'code', 5),
(10277, 92, NULL, $BLK10277$|**Syntaxe**|**Exemple**|
|---|---|
|`Définition`|`struct Point { x: i32, y: i32 }`|
|`Instanciation`|`let p = Point { x: 0, y: 0 };`|
|`Accès champ`|`p.x`|
|`Méthode`|`fn aire(&self) -> u32 { ... }`|
|`Fonction associée`|`fn new() -> Self { ... }`|
|`Tuple struct`|`struct Color(i32, i32, i32);`|$BLK10277$, 'table', 1),
(10278, 93, NULL, $BLK10278$|**Syntaxe**|**Exemple**|
|---|---|
|`Définition simple`|`enum Dir { Haut, Bas }`|
|`Avec données`|`enum Msg { Move { x: i32, y: i32 } }`|
|`Option`|`Some(5) ou None`|
|`Result`|`Ok(value) ou Err(error)`|
|`Match`|`match x { Some(n) => n, None => 0 }`|
|`if let`|`if let Some(n) = x { ... }`|$BLK10278$, 'table', 1);

-- pattern-matching.md (cour_id=5)
INSERT INTO chapitre (id, cour_id, slug, title, lead, sort_order) VALUES
(94, 5, 'pattern-matching-1-lexpression-match', '1. L''expression match', NULL, 1),
(95, 5, 'pattern-matching-2-motifs-courants', '2. Motifs courants', NULL, 2),
(96, 5, 'pattern-matching-3-gardes-de-motif', '3. Gardes de motif', NULL, 3),
(97, 5, 'pattern-matching-4-if-let-et-while-let', '4. if let et while let', NULL, 4),
(98, 5, 'pattern-matching-5-patterns-avances', '5. Patterns avancés', NULL, 5);
INSERT INTO cour_block (id, chapitre_id, heading, content, block_type, sort_order) VALUES
(10279, 94, NULL, $BLK10279$`match` compare une valeur contre une liste de motifs et exécute la branche correspondante.$BLK10279$, 'text', 1),
(10280, 94, NULL, $BLK10280$```rust
let nombre = 7;

match nombre {
    1       => println!("un"),
    2 | 3   => println!("deux ou trois"),
    4..=6   => println!("entre 4 et 6"),
    n       => println!("autre : {n}"),  // variable de capture
}
```$BLK10280$, 'code', 2),
(10281, 94, NULL, $BLK10281$> **Important :** `match` est **exhaustif** — toutes les variantes doivent être couvertes. Le compilateur l'impose.$BLK10281$, 'warning', 3),
(10282, 94, NULL, $BLK10282$```rust
// Utiliser _ pour ignorer les cas restants
match nombre {
    1 => println!("un"),
    _ => println!("autre"),
}
```$BLK10282$, 'code', 4),
(10283, 94, NULL, $BLK10283$---$BLK10283$, 'text', 5),
(10284, 95, '2.1 Littéraux et plages', $BLK10284$```rust
let c = 'a';

match c {
    'a'..='z' => println!("minuscule"),
    'A'..='Z' => println!("majuscule"),
    '0'..='9' => println!("chiffre"),
    _         => println!("autre caractère"),
}
```$BLK10284$, 'code', 1),
(10285, 95, '2.2 Destructurer les enums', $BLK10285$```rust
enum Message {
    Quitter,
    Deplacer { x: i32, y: i32 },
    Ecrire(String),
    ChangerCouleur(u8, u8, u8),
}

let msg = Message::Deplacer { x: 10, y: 20 };

match msg {
    Message::Quitter => println!("quitter"),
    Message::Deplacer { x, y } => println!("déplacer vers {x},{y}"),
    Message::Ecrire(texte) => println!("écrire : {texte}"),
    Message::ChangerCouleur(r, g, b) => println!("couleur : {r},{g},{b}"),
}
```$BLK10285$, 'code', 2),
(10286, 95, '2.3 Destructurer les structs', $BLK10286$```rust
struct Point { x: i32, y: i32 }

let p = Point { x: 3, y: 7 };

// Destructuration complète
let Point { x, y } = p;
println!("{x}, {y}");

// Dans un match
match p {
    Point { x: 0, y } => println!("sur l'axe Y à {y}"),
    Point { x, y: 0 } => println!("sur l'axe X à {x}"),
    Point { x, y }    => println!("({x}, {y})"),
}
```$BLK10286$, 'code', 3),
(10287, 95, '2.4 Destructurer les tuples', $BLK10287$```rust
let tuple = (1, true, "bonjour");

match tuple {
    (1, true, msg) => println!("un, vrai, {msg}"),
    (n, false, _)  => println!("n={n}, faux"),
    _              => println!("autre"),
}

// Destructuration directe
let (a, b, c) = tuple;
```$BLK10287$, 'code', 4),
(10288, 95, '2.5 Slices', $BLK10288$```rust
let nombres = vec![1, 2, 3, 4, 5];

match nombres.as_slice() {
    []         => println!("vide"),
    [seul]     => println!("un seul : {seul}"),
    [premier, .., dernier] => println!("de {premier} à {dernier}"),
}
```

---$BLK10288$, 'code', 5),
(10289, 96, NULL, $BLK10289$Une garde (`if condition`) ajoute un test supplémentaire après le motif.$BLK10289$, 'text', 1),
(10290, 96, NULL, $BLK10290$```rust
let pair = (2, -3);

match pair {
    (x, y) if x == y       => println!("égaux"),
    (x, y) if x + y == 0   => println!("opposés"),
    (x, _) if x % 2 == 0   => println!("x est pair"),
    _                       => println!("autre"),
}
```$BLK10290$, 'code', 2),
(10291, 96, NULL, $BLK10291$> **Attention :** La garde ne participe pas à l'exhaustivité — le compilateur ne peut pas vérifier les conditions arbitraires.$BLK10291$, 'warning', 3),
(10292, 96, NULL, $BLK10292$---$BLK10292$, 'text', 4),
(10293, 97, 'if let — pattern unique sans exhaustivité', $BLK10293$```rust
let valeur: Option<i32> = Some(42);

// Verbeux avec match
match valeur {
    Some(n) => println!("got {n}"),
    None    => {},
}

// Concis avec if let
if let Some(n) = valeur {
    println!("got {n}");
}

// Avec else
if let Some(n) = valeur {
    println!("got {n}");
} else {
    println!("rien");
}
```$BLK10293$, 'code', 1),
(10294, 97, 'while let — boucle tant que le motif correspond', $BLK10294$```rust
let mut pile = vec![1, 2, 3];

while let Some(sommet) = pile.pop() {
    println!("{sommet}");
}
// Affiche : 3, 2, 1
```

---$BLK10294$, 'code', 2),
(10295, 98, '`@` — capturer et tester', $BLK10295$```rust
let n = 15;

match n {
    x @ 1..=12 => println!("mois {x}"),
    x @ 13..=19 => println!("ado {x}"),
    _ => println!("autre"),
}
```$BLK10295$, 'code', 1),
(10296, 98, '`..` — ignorer les champs restants', $BLK10296$```rust
struct Point3D { x: i32, y: i32, z: i32 }

let p = Point3D { x: 1, y: 2, z: 3 };

match p {
    Point3D { x, .. } => println!("x = {x}"),  // y et z ignorés
}

// Dans un tuple
let tuple = (1, 2, 3, 4, 5);
let (premier, .., dernier) = tuple;
```$BLK10296$, 'code', 2),
(10297, 98, '`|` — plusieurs motifs', $BLK10297$```rust
match valeur {
    1 | 2 | 3 => println!("petit"),
    4 | 5 | 6 => println!("moyen"),
    _          => println!("grand"),
}
```$BLK10297$, 'code', 3),
(10298, 98, '`ref` et `ref mut` — emprunter dans un motif', $BLK10298$```rust
let texte = String::from("bonjour");

match texte {
    ref s => println!("longueur : {}", s.len()),  // s est &String
}

// texte est toujours valide ici
println!("{texte}");
```$BLK10298$, 'code', 4),
(10299, 98, 'let-else (Rust 1.65+)', $BLK10299$```rust
fn parse_id(s: &str) -> u32 {
    let Ok(id) = s.parse::<u32>() else {
        panic!("id invalide : {s}");
    };
    id
}
```$BLK10299$, 'code', 5);

-- collections.md (cour_id=6)
INSERT INTO chapitre (id, cour_id, slug, title, lead, sort_order) VALUES
(99, 6, 'collections-vec-hashmap-hashset-et-iterateurs', 'Vec, HashMap, HashSet et Itérateurs', NULL, 1),
(100, 6, 'collections-1-les-vecteurs-vect', '1. Les Vecteurs (Vec<T>)', NULL, 2),
(101, 6, 'collections-11-creation-et-initialisation', '1.1 - Création et initialisation', NULL, 3),
(102, 6, 'collections-12-ajouter-et-retirer-des-elements', '1.2 - Ajouter et retirer des éléments', NULL, 4),
(103, 6, 'collections-13-acceder-aux-elements', '1.3 - Accéder aux éléments', NULL, 5),
(104, 6, 'collections-14-iterer-sur-un-vecteur', '1.4 - Itérer sur un vecteur', NULL, 6),
(105, 6, 'collections-2-les-hashmapk-v', '2. Les HashMap<K, V>', NULL, 7),
(106, 6, 'collections-21-creation-et-insertion', '2.1 - Création et insertion', NULL, 8),
(107, 6, 'collections-22-acces-et-modification', '2.2 - Accès et modification', NULL, 9),
(108, 6, 'collections-23-verifier-lexistence', '2.3 - Vérifier l''existence', NULL, 10),
(109, 6, 'collections-24-iterer-sur-une-hashmap', '2.4 - Itérer sur une HashMap', NULL, 11),
(110, 6, 'collections-3-les-hashsett', '3. Les HashSet<T>', NULL, 12),
(111, 6, 'collections-31-creation-et-ajout', '3.1 - Création et ajout', NULL, 13),
(112, 6, 'collections-32-operations-densemble', '3.2 - Opérations d''ensemble', NULL, 14),
(113, 6, 'collections-33-verification-dappartenance', '3.3 - Vérification d''appartenance', NULL, 15),
(114, 6, 'collections-4-les-iterateurs', '4. Les Itérateurs', NULL, 16),
(115, 6, 'collections-41-methodes-de-base', '4.1 - Méthodes de base', NULL, 17),
(116, 6, 'collections-42-transformations-map-filter', '4.2 - Transformations (map, filter)', NULL, 18),
(117, 6, 'collections-43-collecte-et-consommation', '4.3 - Collecte et consommation', NULL, 19),
(118, 6, 'collections-5-choisir-la-bonne-collection', '5. Choisir la bonne collection', NULL, 20),
(119, 6, 'collections-guide-de-decision', 'Guide de décision :', NULL, 21),
(120, 6, 'collections-exercice-1-statistiques-sur-un-vec', 'Exercice 1 : Statistiques sur un Vec', NULL, 22),
(121, 6, 'collections-exercice-2-compter-les-occurrences', 'Exercice 2 : Compter les occurrences', NULL, 23),
(122, 6, 'collections-exercice-3-supprimer-les-doublons', 'Exercice 3 : Supprimer les doublons', NULL, 24),
(123, 6, 'collections-vect', 'Vec<T>', NULL, 25),
(124, 6, 'collections-hashmapk-v', 'HashMap<K, V>', NULL, 26),
(125, 6, 'collections-iterateurs', 'Itérateurs', NULL, 27),
(126, 6, 'collections-prochaines-etapes', 'Prochaines étapes :', NULL, 28);
INSERT INTO cour_block (id, chapitre_id, heading, content, block_type, sort_order) VALUES
(10300, 99, NULL, $BLK10300$Guide Complet des Structures de Données$BLK10300$, 'text', 1),
(10301, 100, NULL, $BLK10301$Les **vecteurs** sont des tableaux dynamiques : leur taille peut changer. C'est la collection la plus utilisée en Rust !$BLK10301$, 'text', 1),
(10302, 101, NULL, $BLK10302$```
// Vecteur vide
let mut v: Vec<i32> = Vec::new();
// Avec la macro vec!
let v = vec![1, 2, 3, 4, 5];
// Vecteur avec capacité
let mut v: Vec<i32> = Vec::with_capacity(10);
// Vecteur de répétitions
let v = vec![0; 5];  // [0, 0, 0, 0, 0]
// Vecteur de String
let mut v: Vec<String> = Vec::new();
v.push(String::from("Hello"));
v.push(String::from("World"));
```$BLK10302$, 'code', 1),
(10303, 101, 'Capacity vs Length :', $BLK10303$• `len()` : Nombre d'éléments actuels • `capacity()` : Espace alloué en mémoire Utilise `with_capacity()` si tu connais la taille finale pour éviter les réallocations.$BLK10303$, 'text', 2),
(10304, 102, NULL, $BLK10304$```
let mut v = vec![1, 2, 3];
```$BLK10304$, 'code', 1),
(10305, 102, NULL, $BLK10305$```
// Ajouter à la fin
v.push(4);
println!("{:?}", v);  // [1, 2, 3, 4]
// Retirer le dernier élément
let dernier = v.pop();  // Some(4)
println!("{:?}", v);    // [1, 2, 3]
// Insérer à un index
v.insert(1, 10);
println!("{:?}", v);  // [1, 10, 2, 3]
// Retirer à un index
let element = v.remove(1);  // 10
println!("{:?}", v);        // [1, 2, 3]
// Vider le vecteur
v.clear();
println!("{:?}", v);  // []
// Ajouter plusieurs éléments
let mut v = vec![1, 2, 3];
v.extend(vec![4, 5, 6]);
println!("{:?}", v);  // [1, 2, 3, 4, 5, 6]
```$BLK10305$, 'code', 2),
(10306, 102, NULL, $BLK10306$II **Attention !** `remove()` et `insert()` sont coûteux car ils déplacent tous les éléments après l'index. Préfère `push()` et `pop()` quand c'est possible.$BLK10306$, 'text', 3),
(10307, 103, NULL, $BLK10307$```
let v = vec![1, 2, 3, 4, 5];
```$BLK10307$, 'code', 1),
(10308, 103, NULL, $BLK10308$```
// Accès par index (panic si hors limites)
let troisieme = v[2];
println!("Troisième : {}", troisieme);  // 3
```$BLK10308$, 'code', 2),
(10309, 103, NULL, $BLK10309$```
// Accès sécurisé avec get()
match v.get(2) {
    Some(valeur) => println!("Troisième : {}", valeur),
    None => println!("Pas d'élément à cet index"),
}
```$BLK10309$, 'code', 3),
(10310, 103, NULL, $BLK10310$```
// Premier et dernier élément
let premier = v.first();   // Some(&1)
let dernier = v.last();    // Some(&5)
// Slice (portion du vecteur)
let slice = &v[1..3];  // [2, 3]
```$BLK10310$, 'code', 4),
(10311, 103, NULL, $BLK10311$```
// Vérifier si vide
if v.is_empty() {
    println!("Vecteur vide");
} else {
    println!("Longueur : {}", v.len());
}
```$BLK10311$, 'code', 5),
(10312, 104, NULL, $BLK10312$```
let v = vec![10, 20, 30];
// Itération immutable
for element in &v {
    println!("{}", element);
}
// Itération mutable
let mut v = vec![10, 20, 30];
for element in &mut v {
    *element += 10;
}
println!("{:?}", v);  // [20, 30, 40]
// Itération avec index
for (index, valeur) in v.iter().enumerate() {
    println!("v[{}] = {}", index, valeur);
}
// Consommer le vecteur
for element in v {
    println!("{}", element);
}
// v n'est plus accessible ici
```$BLK10312$, 'code', 1),
(10313, 105, NULL, $BLK10313$Les **HashMap** stockent des paires clé-valeur. C'est l'équivalent des dictionnaires en Python ou des objets en JavaScript.$BLK10313$, 'text', 1),
(10314, 106, NULL, $BLK10314$```
use std::collections::HashMap;
// Création vide
let mut scores: HashMap<String, i32> = HashMap::new();
// Insérer des paires clé-valeur
scores.insert(String::from("Bleu"), 10);
scores.insert(String::from("Rouge"), 50);
// Créer depuis deux vecteurs
let equipes = vec![String::from("Bleu"), String::from("Rouge")];
let scores_initiaux = vec![10, 50];
let scores: HashMap<_, _> = equipes
    .iter()
    .zip(scores_initiaux.iter())
    .collect();
// HashMap avec macro (nécessite une crate externe)
// use maplit::hashmap;
// let scores = hashmap!{
//     "Bleu" => 10,
//     "Rouge" => 50,
// };
```$BLK10314$, 'code', 1),
(10315, 106, NULL, $BLK10315$I **Ownership :** Quand tu insères une valeur dans une HashMap, elle prend ownership des valeurs qui n'implémentent pas `Copy` . Pour `i32` , c'est copié. Pour `String` , c'est déplacé.$BLK10315$, 'text', 2),
(10316, 107, NULL, $BLK10316$```
use std::collections::HashMap;
```$BLK10316$, 'code', 1),
(10317, 107, NULL, $BLK10317$```
let mut scores = HashMap::new();
scores.insert(String::from("Bleu"), 10);
scores.insert(String::from("Rouge"), 50);
```$BLK10317$, 'code', 2),
(10318, 107, NULL, $BLK10318$```
// Accéder à une valeur
let equipe = String::from("Bleu");
let score = scores.get(&equipe);  // Some(&10)
```$BLK10318$, 'code', 3),
(10319, 107, NULL, $BLK10319$```
match score {
    Some(s) => println!("Score : {}", s),
    None => println!("Équipe inconnue"),
}
```$BLK10319$, 'code', 4),
(10320, 107, NULL, $BLK10320$```
// Avec unwrap_or
let score = scores.get("Vert").unwrap_or(&0);
println!("Score : {}", score);  // 0
// Modifier une valeur existante
scores.insert(String::from("Bleu"), 25);  // Écrase l'ancienne
// Insérer seulement si la clé n'existe pas
scores.entry(String::from("Jaune")).or_insert(50);
scores.entry(String::from("Bleu")).or_insert(100);  // Pas d'effet
// Mettre à jour basé sur l'ancienne valeur
let texte = "hello world wonderful world";
let mut map = HashMap::new();
```$BLK10320$, 'code', 5),
(10321, 107, NULL, $BLK10321$```
for mot in texte.split_whitespace() {
    let count = map.entry(mot).or_insert(0);
    *count += 1;
```$BLK10321$, 'code', 6),
(10322, 107, NULL, $BLK10322$```
}
println!("{:?}", map);  // {"hello": 1, "world": 2, "wonderful": 1}
```$BLK10322$, 'code', 7),
(10323, 108, NULL, $BLK10323$```
let mut scores = HashMap::new();
scores.insert("Bleu", 10);
```$BLK10323$, 'code', 1),
(10324, 108, NULL, $BLK10324$```
// Vérifier si une clé existe
if scores.contains_key("Bleu") {
    println!("L'équipe Bleu existe");
}
```$BLK10324$, 'code', 2),
(10325, 108, NULL, $BLK10325$```
// Nombre de paires
println!("Nombre d'équipes : {}", scores.len());
```$BLK10325$, 'code', 3),
(10326, 108, NULL, $BLK10326$```
// Retirer une paire
let score = scores.remove("Bleu");  // Some(10)
```$BLK10326$, 'code', 4),
(10327, 108, NULL, $BLK10327$```
// Vider la HashMap
scores.clear();
```$BLK10327$, 'code', 5),
(10328, 109, NULL, $BLK10328$```
let mut scores = HashMap::new();
scores.insert(String::from("Bleu"), 10);
scores.insert(String::from("Rouge"), 50);
```$BLK10328$, 'code', 1),
(10329, 109, NULL, $BLK10329$```
// Itérer sur les paires clé-valeur
for (cle, valeur) in &scores {
    println!("{}: {}", cle, valeur);
}
// Itérer seulement sur les clés
for cle in scores.keys() {
    println!("{}", cle);
}
// Itérer seulement sur les valeurs
for valeur in scores.values() {
    println!("{}", valeur);
}
// Modifier les valeurs en itérant
for valeur in scores.values_mut() {
    *valeur += 10;
}
```$BLK10329$, 'code', 2),
(10330, 109, NULL, $BLK10330$II **Ordre non garanti !** Les HashMap ne maintiennent pas d'ordre. Si tu as besoin d'ordre, utilise `BTreeMap` ou garde une liste séparée des clés.$BLK10330$, 'text', 3),
(10331, 110, NULL, $BLK10331$Les **HashSet** sont des ensembles : collections d'éléments **uniques** sans ordre particulier.$BLK10331$, 'text', 1),
(10332, 111, NULL, $BLK10332$```
use std::collections::HashSet;
```$BLK10332$, 'code', 1),
(10333, 111, NULL, $BLK10333$```
// Création vide
let mut nombres: HashSet<i32> = HashSet::new();
```$BLK10333$, 'code', 2),
(10334, 111, NULL, $BLK10334$```
// Ajouter des éléments
nombres.insert(1);
nombres.insert(2);
nombres.insert(3);
nombres.insert(2);  // Ignoré (déjà présent)
println!("{:?}", nombres);  // {1, 2, 3}
```$BLK10334$, 'code', 3),
(10335, 111, NULL, $BLK10335$```
// Créer depuis un vecteur
let v = vec![1, 2, 3, 2, 1];
let set: HashSet<i32> = v.into_iter().collect();
println!("{:?}", set);  // {1, 2, 3}
```$BLK10335$, 'code', 4),
(10336, 111, NULL, $BLK10336$```
// Retirer des éléments
nombres.remove(&2);
println!("{:?}", nombres);  // {1, 3}
```$BLK10336$, 'code', 5),
(10337, 112, NULL, $BLK10337$```
use std::collections::HashSet;
```$BLK10337$, 'code', 1),
(10338, 112, NULL, $BLK10338$```
let set1: HashSet<i32> = [1, 2, 3, 4].iter().cloned().collect();
let set2: HashSet<i32> = [3, 4, 5, 6].iter().cloned().collect();
```$BLK10338$, 'code', 2),
(10339, 112, NULL, $BLK10339$```
// Union (tous les éléments)
let union: HashSet<_> = set1.union(&set2).collect();
println!("Union : {:?}", union);  // {1, 2, 3, 4, 5, 6}
```$BLK10339$, 'code', 3),
(10340, 112, NULL, $BLK10340$```
// Intersection (éléments communs)
let inter: HashSet<_> = set1.intersection(&set2).collect();
println!("Intersection : {:?}", inter);  // {3, 4}
```$BLK10340$, 'code', 4),
(10341, 112, NULL, $BLK10341$```
// Différence (dans set1 mais pas set2)
let diff: HashSet<_> = set1.difference(&set2).collect();
println!("Différence : {:?}", diff);  // {1, 2}
```$BLK10341$, 'code', 5),
(10342, 112, NULL, $BLK10342$```
// Différence symétrique (dans l'un ou l'autre mais pas les deux)
let sym_diff: HashSet<_> = set1.symmetric_difference(&set2).collect();
println!("Sym diff : {:?}", sym_diff);  // {1, 2, 5, 6}
```$BLK10342$, 'code', 6),
(10343, 113, NULL, $BLK10343$```
let set: HashSet<i32> = [1, 2, 3].iter().cloned().collect();
```$BLK10343$, 'code', 1),
(10344, 113, NULL, $BLK10344$```
// Vérifier si un élément existe
if set.contains(&2) {
    println!("2 est dans l'ensemble");
}
```$BLK10344$, 'code', 2),
(10345, 113, NULL, $BLK10345$```
// Sous-ensemble et super-ensemble
let set1: HashSet<i32> = [1, 2].iter().cloned().collect();
let set2: HashSet<i32> = [1, 2, 3].iter().cloned().collect();
```$BLK10345$, 'code', 3),
(10346, 113, NULL, $BLK10346$```
println!("{}", set1.is_subset(&set2));     // true
println!("{}", set2.is_superset(&set1));   // true
```$BLK10346$, 'code', 4),
(10347, 113, NULL, $BLK10347$```
// Disjoints (aucun élément en commun)
let set3: HashSet<i32> = [4, 5].iter().cloned().collect();
println!("{}", set1.is_disjoint(&set3));   // true
```$BLK10347$, 'code', 5),
(10348, 114, NULL, $BLK10348$Les **itérateurs** permettent de traiter des séquences d'éléments de manière paresseuse et efficace. C'est une fonctionnalité très puissante de Rust !$BLK10348$, 'text', 1),
(10349, 115, NULL, $BLK10349$```
let v = vec![1, 2, 3, 4, 5];
```$BLK10349$, 'code', 1),
(10350, 115, NULL, $BLK10350$```
// iter() - référence immutable
for val in v.iter() {
    println!("{}", val);  // &i32
}
```$BLK10350$, 'code', 2),
(10351, 115, NULL, $BLK10351$```
// iter_mut() - référence mutable
let mut v = vec![1, 2, 3];
for val in v.iter_mut() {
    *val += 10;
}
```$BLK10351$, 'code', 3),
(10352, 115, NULL, $BLK10352$```
// into_iter() - prend ownership
for val in v.into_iter() {
    println!("{}", val);  // i32
```$BLK10352$, 'code', 4),
(10353, 115, NULL, $BLK10353$```
}
// v n'est plus accessible
```$BLK10353$, 'code', 5),
(10354, 115, NULL, $BLK10354$```
// next() - obtenir le prochain élément
let v = vec![1, 2, 3];
let mut iter = v.iter();
```$BLK10354$, 'code', 6),
(10355, 115, NULL, $BLK10355$```
println!("{:?}", iter.next());  // Some(&1)
println!("{:?}", iter.next());  // Some(&2)
println!("{:?}", iter.next());  // Some(&3)
println!("{:?}", iter.next());  // None
```$BLK10355$, 'code', 7),
(10356, 116, NULL, $BLK10356$```
let v = vec![1, 2, 3, 4, 5];
```$BLK10356$, 'code', 1),
(10357, 116, NULL, $BLK10357$```
// map - transformer chaque élément
let doubles: Vec<i32> = v.iter()
    .map(|x| x * 2)
    .collect();
println!("{:?}", doubles);  // [2, 4, 6, 8, 10]
```$BLK10357$, 'code', 2),
(10358, 116, NULL, $BLK10358$```
// filter - garder seulement certains éléments
let pairs: Vec<i32> = v.iter()
    .filter(|x| *x % 2 == 0)
    .cloned()
    .collect();
println!("{:?}", pairs);  // [2, 4]
```$BLK10358$, 'code', 3),
(10359, 116, NULL, $BLK10359$```
// Chaînage de méthodes
let resultat: Vec<i32> = v.iter()
    .filter(|x| *x % 2 == 0)  // Garder pairs
    .map(|x| x * 2)            // Doubler
    .collect();
println!("{:?}", resultat);    // [4, 8]
```$BLK10359$, 'code', 4),
(10360, 116, NULL, $BLK10360$```
// find - trouver le premier élément
let trouve = v.iter().find(|&&x| x > 3);
println!("{:?}", trouve);  // Some(&4)
```$BLK10360$, 'code', 5),
(10361, 116, NULL, $BLK10361$```
// any / all - tester des conditions
println!("{}", v.iter().any(|&x| x > 4));  // true
println!("{}", v.iter().all(|&x| x > 0));  // true
```$BLK10361$, 'code', 6),
(10362, 116, NULL, $BLK10362$```
// take / skip - prendre/sauter des éléments
let premiers: Vec<i32> = v.iter()
    .take(3)
    .cloned()
    .collect();
println!("{:?}", premiers);  // [1, 2, 3]
```$BLK10362$, 'code', 7),
(10363, 117, NULL, $BLK10363$```
let v = vec![1, 2, 3, 4, 5];
```$BLK10363$, 'code', 1),
(10364, 117, NULL, $BLK10364$```
// collect() - transformer en collection
let doubles: Vec<i32> = v.iter().map(|x| x * 2).collect();
// sum() - additionner tous les éléments
let somme: i32 = v.iter().sum();
println!("Somme : {}", somme);  // 15
```$BLK10364$, 'code', 2),
(10365, 117, NULL, $BLK10365$```
// product() - multiplier tous les éléments
let produit: i32 = v.iter().product();
println!("Produit : {}", produit);  // 120
```$BLK10365$, 'code', 3),
(10366, 117, NULL, $BLK10366$```
// max() / min() - trouver max/min
let max = v.iter().max();
println!("Max : {:?}", max);  // Some(&5)
```$BLK10366$, 'code', 4),
(10367, 117, NULL, $BLK10367$```
// count() - compter les éléments
let compte = v.iter().count();
println!("Compte : {}", compte);  // 5
```$BLK10367$, 'code', 5),
(10368, 117, NULL, $BLK10368$```
// fold() - réduction personnalisée
let somme = v.iter().fold(0, |acc, x| acc + x);
println!("Somme avec fold : {}", somme);  // 15
```$BLK10368$, 'code', 6),
(10369, 117, NULL, $BLK10369$```
// enumerate() - avec index
for (i, val) in v.iter().enumerate() {
    println!("v[{}] = {}", i, val);
}
```$BLK10369$, 'code', 7),
(10370, 117, NULL, $BLK10370$```
// zip() - combiner deux itérateurs
let noms = vec!["Alice", "Bob", "Charlie"];
let ages = vec![25, 30, 35];
```$BLK10370$, 'code', 8),
(10371, 117, NULL, $BLK10371$```
for (nom, age) in noms.iter().zip(ages.iter()) {
    println!("{} a {} ans", nom, age);
}
```$BLK10371$, 'code', 9),
(10372, 117, NULL, $BLK10372$I **Itérateurs paresseux :** Les itérateurs ne font rien tant que tu n'appelles pas une méthode de consommation comme `collect()` , `sum()` , ou une boucle `for` .$BLK10372$, 'text', 10),
(10373, 118, NULL, $BLK10373$|**Collection**|**Quand utiliser**|**Complexité**|
|---|---|---|
|Vec<T>|Liste ordonnée, accès par index|O(1) accès, O(n) insert|
|HashMap<K,V>|Paires clé-valeur, recherche rapide|O(1) moyen|
|HashSet<T>|Ensemble unique, appartenance|O(1) moyen|
|BTreeMap<K,V>|Clés ordonnées|O(log n)|
|BTreeSet<T>|Ensemble ordonné|O(log n)|
|VecDeque<T>|File FIFO/LIFO|O(1) aux extrémités|$BLK10373$, 'table', 1),
(10374, 119, NULL, $BLK10374$- **Besoin d'ordre ?** → Vec ou VecDeque$BLK10374$, 'list', 1),
(10375, 119, NULL, $BLK10375$- **Recherche par clé ?** → HashMap ou BTreeMap$BLK10375$, 'list', 2),
(10376, 119, NULL, $BLK10376$- **Éléments uniques ?** → HashSet ou BTreeSet$BLK10376$, 'list', 3),
(10377, 119, NULL, $BLK10377$- **Accès fréquent par index ?** → Vec$BLK10377$, 'list', 4),
(10378, 119, NULL, $BLK10378$- **Insertion/suppression au début ?** → VecDeque$BLK10378$, 'list', 5),
(10379, 119, NULL, $BLK10379$- **Itération dans l'ordre des clés ?** → BTreeMap/BTreeSet$BLK10379$, 'list', 6),
(10380, 120, NULL, $BLK10380$```
// Écris une fonction qui prend un Vec<i32> et retourne :
// - La moyenne
```$BLK10380$, 'code', 1),
(10381, 120, NULL, $BLK10381$```
// - Le minimum
// - Le maximum
```$BLK10381$, 'code', 2),
(10382, 120, NULL, $BLK10382$```
// Solution :
fn statistiques(v: &Vec<i32>) -> (f64, i32, i32) {
    let somme: i32 = v.iter().sum();
    let moyenne = somme as f64 / v.len() as f64;
    let min = *v.iter().min().unwrap();
    let max = *v.iter().max().unwrap();
```$BLK10382$, 'code', 3),
(10383, 120, NULL, $BLK10383$```
    (moyenne, min, max)
```$BLK10383$, 'code', 4),
(10384, 120, NULL, $BLK10384$```
}
```$BLK10384$, 'code', 5),
(10385, 120, NULL, $BLK10385$```
fn main() {
```$BLK10385$, 'code', 6),
(10386, 120, NULL, $BLK10386$```
    let nombres = vec![1, 5, 3, 9, 2];
    let (moy, min, max) = statistiques(&nombres);
    println!("Moy: {}, Min: {}, Max: {}", moy, min, max);
}
```$BLK10386$, 'code', 7),
(10387, 121, NULL, $BLK10387$```
// Écris une fonction qui compte les occurrences de chaque mot
// dans une phrase
```$BLK10387$, 'code', 1),
(10388, 121, NULL, $BLK10388$```
// Solution :
use std::collections::HashMap;
```$BLK10388$, 'code', 2),
(10389, 121, NULL, $BLK10389$```
fn compter_mots(texte: &str) -> HashMap<String, u32> {
    let mut compteur = HashMap::new();
```$BLK10389$, 'code', 3),
(10390, 121, NULL, $BLK10390$```
    for mot in texte.split_whitespace() {
        let count = compteur.entry(mot.to_string()).or_insert(0);
        *count += 1;
    }
```$BLK10390$, 'code', 4),
(10391, 121, NULL, $BLK10391$```
    compteur
}
```$BLK10391$, 'code', 5),
(10392, 121, NULL, $BLK10392$```
fn main() {
    let texte = "hello world hello rust world";
    let compte = compter_mots(texte);
    for (mot, freq) in &compte {
        println!("{}: {}", mot, freq);
    }
```$BLK10392$, 'code', 6),
(10393, 121, NULL, $BLK10393$```
}
```$BLK10393$, 'code', 7),
(10394, 122, NULL, $BLK10394$```
// Écris une fonction qui enlève les doublons d'un Vec
```$BLK10394$, 'code', 1),
(10395, 122, NULL, $BLK10395$```
// Solution :
use std::collections::HashSet;
```$BLK10395$, 'code', 2),
(10396, 122, NULL, $BLK10396$```
fn sans_doublons(v: Vec<i32>) -> Vec<i32> {
    let set: HashSet<i32> = v.into_iter().collect();
    set.into_iter().collect()
}
```$BLK10396$, 'code', 3),
(10397, 122, NULL, $BLK10397$```
fn main() {
    let nombres = vec![1, 2, 2, 3, 1, 4, 5, 3];
    let uniques = sans_doublons(nombres);
    println!("{:?}", uniques);  // Ordre non garanti !
}
```$BLK10397$, 'code', 4),
(10398, 123, NULL, $BLK10398$|**Méthode**|**Description**|**Exemple**|
|---|---|---|
|**`push(val)`**|Ajouter à la fin|v.push(5)|
|**`pop()`**|Retirer de la fin|v.pop()|
|**`len()`**|Nombre d'éléments|v.len()|
|**`is_empty()`**|Vérifier si vide|v.is_empty()|
|**`get(i)`**|Accès sécurisé|v.get(2)|
|**`clear()`**|Vider|v.clear()|$BLK10398$, 'table', 1),
(10399, 124, NULL, $BLK10399$|**Méthode**|**Description**|**Exemple**|
|---|---|---|
|**`insert(k, v)`**|Ajouter/modifier|map.insert("a", 1)|
|**`get(k)`**|Obtenir valeur|map.get("a")|
|**`remove(k)`**|Supprimer|map.remove("a")|
|**`contains_key(k)`**|Vérifier clé|map.contains_key("a")|
|**`entry(k).or_insert(v)`**|Insert si absent|map.entry("a").or_insert(0)|$BLK10399$, 'table', 1),
(10400, 125, NULL, $BLK10400$|**Méthode**|**Description**|**Type retour**|
|---|---|---|
|**`map(f)`**|Transformer|Iterator|
|**`filter(f)`**|Filtrer|Iterator|
|**`collect()`**|Collecter|Collection|
|**`sum()`**|Additionner|Nombre|
|**`any(f)`**|Tester si au moins un|bool|
|**`all(f)`**|Tester si tous|bool|
|**`find(f)`**|Trouver premier|Option<T>|$BLK10400$, 'table', 1),
(10401, 125, 'Bravo !', $BLK10401$Tu maîtrises maintenant les collections en Rust !$BLK10401$, 'text', 2),
(10402, 126, NULL, $BLK10402$• Pratiquer avec des projets réels • Explorer les lifetimes • Apprendre les traits avancés • Découvrir la programmation asynchrone$BLK10402$, 'text', 1),
(10403, 126, NULL, $BLK10403$I **Tu es maintenant un vrai Rustacean !** I$BLK10403$, 'text', 2);

-- modules.md (cour_id=7)
INSERT INTO chapitre (id, cour_id, slug, title, lead, sort_order) VALUES
(127, 7, 'modules-structurer-vos-projets-rust', 'Structurer vos Projets Rust', NULL, 1),
(128, 7, 'modules-1-les-modules-mod', '1. Les Modules (mod)', NULL, 2),
(129, 7, 'modules-11-module-inline', '1.1 - Module inline', NULL, 3),
(130, 7, 'modules-12-module-dans-un-fichier', '1.2 - Module dans un fichier', NULL, 4),
(131, 7, 'modules-13-hierarchie-de-modules', '1.3 - Hiérarchie de modules', NULL, 5),
(132, 7, 'modules-21-prive-par-defaut', '2.1 - Privé par défaut', NULL, 6),
(133, 7, 'modules-23-re-exports', '2.3 - Re-exports', NULL, 7),
(134, 7, 'modules-31-importer-des-items', '3.1 - Importer des items', NULL, 8),
(135, 7, 'modules-32-chemins-absolus-vs-relatifs', '3.2 - Chemins absolus vs relatifs', NULL, 9),
(136, 7, 'modules-41-librs-vs-mainrs', '4.1 - lib.rs vs main.rs', NULL, 10),
(137, 7, 'modules-42-creer-une-lib', '4.2 - Créer une lib', NULL, 11),
(138, 7, 'modules-51-dependances', '5.1 - Dépendances', NULL, 12),
(139, 7, 'modules-52-features', '5.2 - Features', NULL, 13),
(140, 7, 'modules-6-workspaces', '6. Workspaces', NULL, 14),
(141, 7, 'modules-avantages-des-workspaces', 'Avantages des workspaces :', NULL, 15),
(142, 7, 'modules-1-organisation-claire', '• **1. Organisation claire**', NULL, 16),
(143, 7, 'modules-2-api-publique-minimale', '• **2. API publique minimale**', NULL, 17),
(144, 7, 'modules-3-re-exports-strategiques', '• **3. Re-exports stratégiques**', NULL, 18),
(145, 7, 'modules-4-documentation', '• **4. Documentation**', NULL, 19),
(146, 7, 'modules-5-tests-a-cote-du-code', '• **5. Tests à côté du code**', NULL, 20),
(147, 7, 'modules-6-separation-lib-bin', '• **6. Séparation lib/bin**', NULL, 21),
(148, 7, 'modules-7-features-optionnelles', '• **7. Features optionnelles**', NULL, 22),
(149, 7, 'modules-8-exemple-complet', '8. Exemple complet', NULL, 23),
(150, 7, 'modules-parfait', 'Parfait !', NULL, 24);
INSERT INTO cour_block (id, chapitre_id, heading, content, block_type, sort_order) VALUES
(10404, 127, NULL, $BLK10404$Modules, Crates et Workspaces$BLK10404$, 'text', 1),
(10405, 128, NULL, $BLK10405$Les **modules** permettent d'organiser le code en namespaces. C'est essentiel pour structurer de gros projets !$BLK10405$, 'text', 1),
(10406, 129, NULL, $BLK10406$**`// Module défini directement dans le fichier mod network { fn connect() { println!("Connexion..."); } pub fn send_data() { connect();  // Peut appeler connect() (même module) println!("Envoi de données"); } } fn main() { // network::connect();  //`** I **`ERREUR : connect est privé network::send_data();   //`** I **`OK : send_data est pub } // Modules imbriqués mod reseau { pub mod tcp { pub fn connect() {} } pub mod udp { pub fn send() {} } } fn main() { reseau::tcp::connect(); reseau::udp::send(); }`**$BLK10406$, 'text', 1),
(10407, 130, NULL, $BLK10407$```
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
```$BLK10407$, 'code', 1),
(10408, 130, NULL, $BLK10408$I **Convention :** Utilise `mod.rs` pour les modules avec sous-modules, ou un fichier avec le nom du module.$BLK10408$, 'text', 2),
(10409, 131, NULL, $BLK10409$```
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
```$BLK10409$, 'code', 1),
(10410, 131, NULL, $BLK10410$```
// Dans lib.rs :
pub mod auth;
pub mod database;
```$BLK10410$, 'code', 2),
(10411, 131, NULL, $BLK10411$```
// Dans auth/mod.rs :
pub mod login;
pub mod register;
```$BLK10411$, 'code', 3),
(10412, 131, NULL, $BLK10412$```
// Utilisation
use mon_projet::auth::login::authenticate;
use mon_projet::database::connection::connect;
```$BLK10412$, 'code', 4),
(10413, 132, NULL, $BLK10413$**`mod api { // Fonction privée (par défaut) fn interne() { println!("Fonction interne"); } // Fonction publique pub fn publique() { interne();  //`** I **`OK dans le même module println!("Fonction publique"); } // Struct privée struct Config { secret: String, } // Struct publique avec champ privé pub struct User { pub nom: String, mot_de_passe: String,  // Privé ! } impl User { pub fn new(nom: String, mdp: String) -> User { User { nom, mot_de_passe: mdp, } } } } fn main() { let user = api::User::new( String::from("Alice"), String::from("secret123") ); println!("{}", user.nom);  //`** I **`OK // println!("{}", user.mot_de_passe);  //`** I **`ERREUR : privé }`**$BLK10413$, 'text', 1),
(10414, 132, NULL, $BLK10414$**2.2 - pub et pub(crate)**$BLK10414$, 'text', 2),
(10415, 132, NULL, $BLK10415$```
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
```$BLK10415$, 'code', 3),
(10416, 133, NULL, $BLK10416$```
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
```$BLK10416$, 'code', 1),
(10417, 134, NULL, $BLK10417$```
// Import simple
use std::collections::HashMap;
```$BLK10417$, 'code', 1),
(10418, 134, NULL, $BLK10418$```
let mut map = HashMap::new();
```$BLK10418$, 'code', 2),
(10419, 134, NULL, $BLK10419$```
// Import multiple
use std::collections::{HashMap, HashSet, BTreeMap};
```$BLK10419$, 'code', 3),
(10420, 134, NULL, $BLK10420$```
// Import tout le module
use std::io;
```$BLK10420$, 'code', 4),
(10421, 134, NULL, $BLK10421$```
io::stdin().read_line(&mut buffer)?;
```$BLK10421$, 'code', 5),
(10422, 134, NULL, $BLK10422$```
// Import avec renommage
use std::collections::HashMap as Map;
```$BLK10422$, 'code', 6),
(10423, 134, NULL, $BLK10423$```
let mut map = Map::new();
```$BLK10423$, 'code', 7),
(10424, 134, NULL, $BLK10424$```
// Import imbriqué
use std::{
    io::{self, Write},
    collections::HashMap,
};
```$BLK10424$, 'code', 8),
(10425, 135, NULL, $BLK10425$```
// Chemin absolu (depuis la racine de la crate)
use crate::network::tcp::connect;
```$BLK10425$, 'code', 1),
(10426, 135, NULL, $BLK10426$```
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
```$BLK10426$, 'code', 2),
(10427, 135, NULL, $BLK10427$**3.3 - use as et glob**$BLK10427$, 'text', 3),
(10428, 135, NULL, $BLK10428$```
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
```$BLK10428$, 'code', 4),
(10429, 135, NULL, $BLK10429$II **Évite use * :** Ça pollue le namespace et rend le code moins clair. Utilise-le seulement dans les tests ou pour le prelude.$BLK10429$, 'text', 5),
(10430, 136, NULL, $BLK10430$**`// Structure projet : // mon_projet/ //   Cargo.toml //   src/ //     lib.rs`** ← **`Bibliothèque (optionnel) //     main.rs`** ← **`Binaire //     bin/`** ← **`Binaires additionnels (optionnel) //       autre.rs`**$BLK10430$, 'text', 1),
(10431, 136, NULL, $BLK10431$```
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
```$BLK10431$, 'code', 2),
(10432, 137, NULL, $BLK10432$```
# Créer une nouvelle bibliothèque
cargo new ma_lib --lib
# Structure générée :
# ma_lib/
#   Cargo.toml
#   src/
#     lib.rs
```$BLK10432$, 'code', 1),
(10433, 137, NULL, $BLK10433$```
# Dans lib.rs :
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```$BLK10433$, 'code', 2),
(10434, 137, NULL, $BLK10434$```
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
```$BLK10434$, 'code', 3),
(10435, 137, NULL, $BLK10435$```
fn main() {
    println!("{}", add(5, 3));
}
```$BLK10435$, 'code', 4),
(10436, 138, NULL, $BLK10436$```
# Cargo.toml
[package]
name = "mon_projet"
version = "0.1.0"
edition = "2021"
```$BLK10436$, 'code', 1),
(10437, 138, NULL, $BLK10437$```
[dependencies]
# Depuis crates.io
serde = "1.0"
```$BLK10437$, 'code', 2),
(10438, 138, NULL, $BLK10438$```
# Version spécifique
tokio = "=1.35.0"
```$BLK10438$, 'code', 3),
(10439, 138, NULL, $BLK10439$```
# Features optionnelles
serde = { version = "1.0", features = ["derive"] }
# Depuis git
mon_lib = { git = "https://github.com/user/repo" }
# Depuis un chemin local
ma_lib = { path = "../ma_lib" }
```$BLK10439$, 'code', 4),
(10440, 138, NULL, $BLK10440$```
[dev-dependencies]
# Seulement pour les tests
criterion = "0.5"
```$BLK10440$, 'code', 5),
(10441, 138, NULL, $BLK10441$```
[build-dependencies]
# Pour build.rs
cc = "1.0"
```$BLK10441$, 'code', 6),
(10442, 138, NULL, $BLK10442$```
# Différentes dépendances selon la plateforme
[target.'cfg(windows)'.dependencies]
winapi = "0.3"
```$BLK10442$, 'code', 7),
(10443, 138, NULL, $BLK10443$```
[target.'cfg(unix)'.dependencies]
libc = "0.2"
```$BLK10443$, 'code', 8),
(10444, 139, NULL, $BLK10444$```
# Dans Cargo.toml
[features]
default = ["json"]
json = ["serde_json"]
xml = ["quick-xml"]
full = ["json", "xml"]
```$BLK10444$, 'code', 1),
(10445, 139, NULL, $BLK10445$```
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
```$BLK10445$, 'code', 2),
(10446, 139, NULL, $BLK10446$```
# Utilisation :
# cargo build --features json
# cargo build --features "json xml"
# cargo build --all-features
```$BLK10446$, 'code', 3),
(10447, 140, NULL, $BLK10447$Les **workspaces** permettent de gérer plusieurs crates dans un même dépôt.$BLK10447$, 'text', 1),
(10448, 140, NULL, $BLK10448$**`# Structure : # mon_workspace/ #   Cargo.toml`** ← **`Workspace root #   ma_lib/ #     Cargo.toml #     src/lib.rs #   mon_app/ #     Cargo.toml #     src/main.rs #   mon_autre_lib/ #     Cargo.toml #     src/lib.rs # Dans mon_workspace/Cargo.toml : [workspace] members = [ "ma_lib", "mon_app", "mon_autre_lib" ] # Dépendances partagées [workspace.dependencies] serde = "1.0" tokio = "1.35" # Dans mon_app/Cargo.toml : [dependencies] ma_lib = { path = "../ma_lib" } serde = { workspace = true }`**$BLK10448$, 'text', 2),
(10449, 140, NULL, $BLK10449$**`# Commandes : # cargo build`** ← **`Build tout le workspace # cargo test`** ← **`Test tout le workspace # cargo build -p mon_app`** ← **`Build une crate spécifique`**$BLK10449$, 'text', 3),
(10450, 141, NULL, $BLK10450$• Dépendances partagées (une seule version)$BLK10450$, 'text', 1),
(10451, 141, NULL, $BLK10451$• Build et test unifiés$BLK10451$, 'text', 2),
(10452, 141, NULL, $BLK10452$• Facilite le développement de projets multi-crates$BLK10452$, 'text', 3),
(10453, 142, NULL, $BLK10453$Un module = une responsabilité. Évite les modules fourre-tout.$BLK10453$, 'text', 1),
(10454, 143, NULL, $BLK10454$N'expose que ce qui est nécessaire avec `pub` .$BLK10454$, 'text', 1),
(10455, 144, NULL, $BLK10455$Simplifie l'API avec `pub use` dans lib.rs.$BLK10455$, 'text', 1),
(10456, 145, NULL, $BLK10456$Documente tout ce qui est `pub` avec //!.$BLK10456$, 'text', 1),
(10457, 146, NULL, $BLK10457$Utilise `#[cfg(test)]` pour tests unitaires.$BLK10457$, 'text', 1),
(10458, 147, NULL, $BLK10458$Logique dans lib.rs, CLI dans main.rs.$BLK10458$, 'text', 1),
(10459, 148, NULL, $BLK10459$Utilise features pour dépendances lourdes optionnelles.$BLK10459$, 'text', 1),
(10460, 149, NULL, $BLK10460$```
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
```$BLK10460$, 'code', 1),
(10461, 149, NULL, $BLK10461$```
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
```$BLK10461$, 'code', 2),
(10462, 150, NULL, $BLK10462$Tu sais maintenant organiser tes projets Rust !$BLK10462$, 'text', 1),
(10463, 150, NULL, $BLK10463$Points clés :$BLK10463$, 'text', 2),
(10464, 150, NULL, $BLK10464$- Modules pour organiser le code$BLK10464$, 'list', 3),
(10465, 150, NULL, $BLK10465$• pub pour contrôler la visibilité$BLK10465$, 'text', 4),
(10466, 150, NULL, $BLK10466$• lib.rs pour bibliothèques$BLK10466$, 'text', 5),
(10467, 150, NULL, $BLK10467$- Workspaces pour multi-crates$BLK10467$, 'list', 6),
(10468, 150, NULL, $BLK10468$I **Ton code sera propre et maintenable !** I$BLK10468$, 'text', 7);

-- tests-rust.md (cour_id=8)
INSERT INTO chapitre (id, cour_id, slug, title, lead, sort_order) VALUES
(151, 8, 'tests-rust-1-tests-unitaires', '1. Tests unitaires', NULL, 1),
(152, 8, 'tests-rust-2-organisation-avec-cfgtest', '2. Organisation avec cfg(test)', NULL, 2),
(153, 8, 'tests-rust-3-tests-dintegration', '3. Tests d''intégration', NULL, 3),
(154, 8, 'tests-rust-4-tests-de-documentation', '4. Tests de documentation', NULL, 4),
(155, 8, 'tests-rust-5-commandes-cargo-test', '5. Commandes cargo test', NULL, 5),
(156, 8, 'tests-rust-6-bonnes-pratiques', '6. Bonnes pratiques', NULL, 6);
INSERT INTO cour_block (id, chapitre_id, heading, content, block_type, sort_order) VALUES
(10469, 151, '1.1 Syntaxe de base', $BLK10469$```rust
pub fn ajouter(a: i32, b: i32) -> i32 {
    a + b
}

pub fn est_pair(n: i32) -> bool {
    n % 2 == 0
}

#[cfg(test)]
mod tests {
    use super::*;  // importe tout du module parent

    #[test]
    fn test_ajouter() {
        assert_eq!(ajouter(2, 3), 5);
    }

    #[test]
    fn test_est_pair() {
        assert!(est_pair(4));
        assert!(!est_pair(3));
    }
}
```$BLK10469$, 'code', 1),
(10470, 151, '1.2 Macros d''assertion', $BLK10470$```rust
// assert! — vérifie qu'une condition est vraie
assert!(2 + 2 == 4);
assert!(vec![1, 2, 3].len() == 3, "longueur incorrecte");

// assert_eq! — vérifie l'égalité (affiche les deux valeurs si échec)
assert_eq!(2 + 2, 4);
assert_eq!(
    "bonjour".to_uppercase(),
    "BONJOUR",
    "conversion en majuscules échouée"
);

// assert_ne! — vérifie l'inégalité
assert_ne!(2 + 2, 5);

// Pour les flottants — comparer avec une tolérance
let resultat = 0.1 + 0.2;
assert!((resultat - 0.3).abs() < 1e-10);
```$BLK10470$, 'code', 2),
(10471, 151, '1.3 Tests qui doivent paniquer', $BLK10471$```rust
pub fn diviser(a: i32, b: i32) -> i32 {
    if b == 0 { panic!("division par zéro"); }
    a / b
}

#[test]
#[should_panic]
fn test_division_par_zero() {
    diviser(10, 0);
}

// Vérifier le message de panique
#[test]
#[should_panic(expected = "division par zéro")]
fn test_message_panique() {
    diviser(10, 0);
}
```$BLK10471$, 'code', 3),
(10472, 151, '1.4 Tests ignorés', $BLK10472$```rust
#[test]
#[ignore = "trop lent pour le CI"]
fn test_lent() {
    // prend plusieurs minutes...
}
```

```bash
cargo test -- --ignored          # exécute seulement les tests ignorés
cargo test -- --include-ignored  # exécute tous les tests y compris ignorés
```

---$BLK10472$, 'code', 4),
(10473, 152, NULL, $BLK10473$```rust
// src/lib.rs

pub struct Calculatrice {
    valeur: f64,
}

impl Calculatrice {
    pub fn new(v: f64) -> Self { Calculatrice { valeur: v } }
    pub fn ajouter(&mut self, n: f64) -> &mut Self { self.valeur += n; self }
    pub fn resultat(&self) -> f64 { self.valeur }

    // Méthode privée — testable uniquement dans cfg(test)
    fn valeur_interne(&self) -> f64 { self.valeur }
}

#[cfg(test)]  // ce module n'est compilé que lors des tests
mod tests {
    use super::*;

    #[test]
    fn test_addition_chainee() {
        let mut calc = Calculatrice::new(0.0);
        calc.ajouter(5.0).ajouter(3.0);
        assert_eq!(calc.resultat(), 8.0);
    }

    #[test]
    fn test_valeur_interne() {
        let calc = Calculatrice::new(42.0);
        assert_eq!(calc.valeur_interne(), 42.0); // accès au privé OK dans tests
    }
}
```$BLK10473$, 'code', 1),
(10474, 152, NULL, $BLK10474$---$BLK10474$, 'text', 2),
(10475, 153, NULL, $BLK10475$Les tests d'intégration testent l'API publique comme un utilisateur externe.$BLK10475$, 'text', 1),
(10476, 153, NULL, $BLK10476$```
mon_projet/
├── src/
│   └── lib.rs
└── tests/              ← dossier des tests d'intégration
    ├── api_test.rs
    └── helpers/
        └── mod.rs
```$BLK10476$, 'code', 2),
(10477, 153, NULL, $BLK10477$```rust
// tests/api_test.rs
use mon_projet::Calculatrice;  // uniquement l'API publique

#[test]
fn test_integration_calcul() {
    let mut calc = Calculatrice::new(10.0);
    calc.ajouter(5.0);
    assert_eq!(calc.resultat(), 15.0);
}
```$BLK10477$, 'code', 3),
(10478, 153, NULL, $BLK10478$```rust
// tests/helpers/mod.rs — helpers partagés entre tests
pub fn creer_calculatrice_initialisee() -> mon_projet::Calculatrice {
    mon_projet::Calculatrice::new(100.0)
}
```$BLK10478$, 'code', 4),
(10479, 153, NULL, $BLK10479$```rust
// tests/autre_test.rs
mod helpers;

#[test]
fn test_avec_helper() {
    let calc = helpers::creer_calculatrice_initialisee();
    assert_eq!(calc.resultat(), 100.0);
}
```$BLK10479$, 'code', 5),
(10480, 153, NULL, $BLK10480$---$BLK10480$, 'text', 6),
(10481, 154, NULL, $BLK10481$Les exemples dans la documentation sont **exécutés comme des tests**.$BLK10481$, 'text', 1),
(10482, 154, NULL, $BLK10482$```rust
/// Additionne deux nombres.
///
/// # Exemples
///
/// ```
/// let resultat = mon_projet::ajouter(2, 3);
/// assert_eq!(resultat, 5);
/// ```
///
/// ```
/// // Un exemple qui doit paniquer
/// # use mon_projet::diviser;
/// let r = diviser(10, 2);
/// assert_eq!(r, 5);
/// ```
pub fn ajouter(a: i32, b: i32) -> i32 {
    a + b
}
```$BLK10482$, 'code', 2),
(10483, 154, NULL, $BLK10483$```bash
cargo test --doc  # exécute uniquement les tests de documentation
```$BLK10483$, 'code', 3),
(10484, 154, NULL, $BLK10484$**Syntaxe spéciale dans les doc-tests :**$BLK10484$, 'text', 4),
(10485, 154, NULL, $BLK10485$```rust
/// ```
/// # // Les lignes préfixées par # sont exécutées mais pas affichées
/// # let x = 5;
/// println!("{x}"); // affiché dans la doc
/// # assert_eq!(x, 5);
/// ```
```$BLK10485$, 'code', 5),
(10486, 154, NULL, $BLK10486$---$BLK10486$, 'text', 6),
(10487, 155, NULL, $BLK10487$```bash
cargo test                        # tous les tests
cargo test nom_du_test            # filtre par nom (sous-chaîne)
cargo test tests::                # tests dans le module 'tests'

cargo test -- --nocapture         # affiche println! dans les tests
cargo test -- --test-threads=1    # séquentiel (utile si tests partagent état)
cargo test -- --ignored           # tests marqués #[ignore]

cargo test --lib                  # tests unitaires uniquement (src/)
cargo test --test api_test        # un fichier d'intégration spécifique
cargo test --doc                  # tests de documentation uniquement

cargo test -p ma_crate            # dans un workspace, une crate spécifique
```$BLK10487$, 'code', 1),
(10488, 155, NULL, $BLK10488$---$BLK10488$, 'text', 2),
(10489, 156, NULL, $BLK10489$**Nommer les tests clairement :**$BLK10489$, 'text', 1),
(10490, 156, NULL, $BLK10490$```rust
// ❌ Trop vague
#[test] fn test1() { }

// ✅ Nom descriptif
#[test] fn ajouter_deux_positifs_retourne_leur_somme() { }
#[test] fn diviser_par_zero_panique() { }
```$BLK10490$, 'code', 2),
(10491, 156, NULL, $BLK10491$**Arranger / Agir / Vérifier (AAA) :**$BLK10491$, 'text', 3),
(10492, 156, NULL, $BLK10492$```rust
#[test]
fn test_insertion_element() {
    // Arranger
    let mut liste = Vec::new();

    // Agir
    liste.push(42);

    // Vérifier
    assert_eq!(liste.len(), 1);
    assert_eq!(liste[0], 42);
}
```$BLK10492$, 'code', 4),
(10493, 156, NULL, $BLK10493$**Tests paramétrés avec une boucle :**$BLK10493$, 'text', 5),
(10494, 156, NULL, $BLK10494$```rust
#[test]
fn test_est_pair_plusieurs_valeurs() {
    let cas = [(0, true), (1, false), (2, true), (99, false), (100, true)];
    for (entree, attendu) in cas {
        assert_eq!(est_pair(entree), attendu, "est_pair({entree}) devrait être {attendu}");
    }
}
```$BLK10494$, 'code', 6);

-- closures-iterateurs.md (cour_id=9)
INSERT INTO chapitre (id, cour_id, slug, title, lead, sort_order) VALUES
(157, 9, 'closures-iterateurs-1-les-closures', '1. Les closures', NULL, 1),
(158, 9, 'closures-iterateurs-2-literator-trait', '2. L''Iterator trait', NULL, 2),
(159, 9, 'closures-iterateurs-3-methodes-diterateurs', '3. Méthodes d''itérateurs', NULL, 3),
(160, 9, 'closures-iterateurs-4-creer-un-iterateur', '4. Créer un itérateur', NULL, 4);
INSERT INTO cour_block (id, chapitre_id, heading, content, block_type, sort_order) VALUES
(10495, 157, '1.1 Syntaxe', $BLK10495$```rust
// Fonction classique
fn ajouter(x: i32, y: i32) -> i32 { x + y }

// Closure équivalente
let ajouter = |x: i32, y: i32| -> i32 { x + y };

// Types inférés, corps implicite (expression)
let ajouter = |x, y| x + y;

// Sans paramètres
let dire_bonjour = || println!("Bonjour !");

// Corps multi-lignes
let traiter = |x: i32| {
    let double = x * 2;
    double + 1
};
```$BLK10495$, 'code', 1),
(10496, 157, '1.2 Capture de l''environnement', $BLK10496$```rust
let seuil = 5;

// Capture par référence (par défaut)
let est_grand = |x| x > seuil;
println!("{}", est_grand(10)); // true
println!("{seuil}");           // seuil toujours accessible

// Capture par valeur avec `move`
let texte = String::from("bonjour");
let afficher = move || println!("{texte}");
afficher();
// println!("{texte}"); // ERREUR : texte a été moved
```

> **Règle :** Utilisez `move` quand la closure doit vivre plus longtemps que son environnement (ex : threads, `async`).$BLK10496$, 'code', 2),
(10497, 157, '1.3 Traits Fn, FnMut, FnOnce', $BLK10497$| Trait | Capture | Peut être appelée |
|---|---|---|
| `FnOnce` | par valeur (move) | une seule fois |
| `FnMut` | par référence mutable | plusieurs fois (modifie env) |
| `Fn` | par référence partagée | plusieurs fois (lecture seule) |

```rust
// FnOnce — consomme une valeur capturée
let texte = String::from("adieu");
let consommer = move || drop(texte);
consommer();
// consommer(); // ERREUR : ne peut être appelée qu'une fois

// FnMut — modifie l'environnement
let mut compteur = 0;
let mut incrementer = || { compteur += 1; compteur };
println!("{}", incrementer()); // 1
println!("{}", incrementer()); // 2

// Fn — lit seulement
let nom = String::from("Rust");
let saluer = || println!("Bonjour {nom}");
saluer();
saluer(); // OK, Fn peut être appelée plusieurs fois
```$BLK10497$, 'table', 3),
(10498, 157, '1.4 Retourner une closure', $BLK10498$```rust
// Avec Box<dyn Fn>
fn multiplicateur(n: i32) -> Box<dyn Fn(i32) -> i32> {
    Box::new(move |x| x * n)
}

let triple = multiplicateur(3);
println!("{}", triple(5)); // 15

// Avec impl Fn (plus idiomatique)
fn multiplicateur(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x * n
}
```

---$BLK10498$, 'code', 4),
(10499, 158, NULL, $BLK10499$```rust
pub trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
    // + des dizaines de méthodes par défaut
}
```$BLK10499$, 'code', 1),
(10500, 158, NULL, $BLK10500$**Créer un itérateur :**$BLK10500$, 'text', 2),
(10501, 158, NULL, $BLK10501$```rust
let v = vec![1, 2, 3];

let iter = v.iter();           // itère sur &T
let iter = v.iter_mut();       // itère sur &mut T
let iter = v.into_iter();      // itère sur T (consomme v)

// Plages
let plage = 1..=5;             // 1, 2, 3, 4, 5
let plage = (0..10).step_by(2); // 0, 2, 4, 6, 8
```$BLK10501$, 'code', 3),
(10502, 158, NULL, $BLK10502$---$BLK10502$, 'text', 4),
(10503, 159, '3.1 Transformation', $BLK10503$```rust
let nombres = vec![1, 2, 3, 4, 5];

// map — transforme chaque élément
let doubles: Vec<i32> = nombres.iter().map(|x| x * 2).collect();
// [2, 4, 6, 8, 10]

// flat_map — map + aplatit
let mots = vec!["bonjour monde", "rust langage"];
let lettres: Vec<&str> = mots.iter()
    .flat_map(|s| s.split_whitespace())
    .collect();
// ["bonjour", "monde", "rust", "langage"]

// enumerate — ajoute l'index
for (i, val) in nombres.iter().enumerate() {
    println!("{i}: {val}");
}

// zip — combine deux itérateurs
let lettres = vec!['a', 'b', 'c'];
let paires: Vec<_> = nombres.iter().zip(lettres.iter()).collect();
// [(1, 'a'), (2, 'b'), (3, 'c')]
```$BLK10503$, 'code', 1),
(10504, 159, '3.2 Filtrage', $BLK10504$```rust
// filter — garde les éléments qui satisfont la condition
let pairs: Vec<i32> = nombres.iter()
    .filter(|&&x| x % 2 == 0)
    .copied()
    .collect();
// [2, 4]

// filter_map — filtre et transforme en même temps
let strings = vec!["1", "deux", "3", "quatre"];
let entiers: Vec<i32> = strings.iter()
    .filter_map(|s| s.parse().ok())
    .collect();
// [1, 3]

// take / skip
let premiers_trois: Vec<_> = nombres.iter().take(3).collect();
let sans_deux: Vec<_>      = nombres.iter().skip(2).collect();

// take_while / skip_while
let avant_grand: Vec<_> = nombres.iter()
    .take_while(|&&x| x < 4)
    .collect();
// [1, 2, 3]
```$BLK10504$, 'code', 2),
(10505, 159, '3.3 Réduction', $BLK10505$```rust
// fold — accumulateur général
let somme = nombres.iter().fold(0, |acc, &x| acc + x); // 15

// sum et product
let somme: i32   = nombres.iter().sum();     // 15
let produit: i32 = nombres.iter().product(); // 120

// count
let n = nombres.iter().filter(|&&x| x > 2).count(); // 3

// min / max / min_by / max_by
let min = nombres.iter().min(); // Some(1)
let max = nombres.iter().max(); // Some(5)

// any / all
let a_pair  = nombres.iter().any(|&x| x % 2 == 0); // true
let tous_pos = nombres.iter().all(|&x| x > 0);     // true

// find / position
let premier_pair = nombres.iter().find(|&&x| x % 2 == 0); // Some(2)
let pos          = nombres.iter().position(|&x| x == 3);   // Some(2)
```$BLK10505$, 'code', 3),
(10506, 159, '3.4 Collecte', $BLK10506$```rust
// Vec
let v: Vec<i32> = (1..=5).collect();

// HashSet
use std::collections::HashSet;
let set: HashSet<i32> = vec![1, 2, 2, 3].into_iter().collect();

// HashMap
use std::collections::HashMap;
let map: HashMap<&str, i32> = vec![("un", 1), ("deux", 2)]
    .into_iter()
    .collect();

// String
let s: String = vec!['R', 'u', 's', 't'].into_iter().collect();
```

---$BLK10506$, 'code', 4),
(10507, 160, NULL, $BLK10507$```rust
struct Compte {
    valeur: u32,
    max: u32,
}

impl Compte {
    fn new(max: u32) -> Self {
        Compte { valeur: 0, max }
    }
}

impl Iterator for Compte {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        if self.valeur < self.max {
            self.valeur += 1;
            Some(self.valeur)
        } else {
            None
        }
    }
}

// Utilisation
let somme: u32 = Compte::new(5)
    .zip(Compte::new(5).skip(1))
    .map(|(a, b)| a * b)
    .filter(|x| x % 3 == 0)
    .sum();
// Compte a toutes les méthodes d'Iterator gratuitement !
```$BLK10507$, 'code', 1);

-- gestion-des-erreurs.md (cour_id=10)
INSERT INTO chapitre (id, cour_id, slug, title, lead, sort_order) VALUES
(161, 10, 'gestion-des-erreurs-gestion-derreurs-avancee', 'Gestion d''Erreurs Avancée', NULL, 1),
(162, 10, 'gestion-des-erreurs-1-loperateur-en-profondeur', '1. L''opérateur ? en profondeur', NULL, 2),
(163, 10, 'gestion-des-erreurs-11-fonctionnement-de-base', '1.1 - Fonctionnement de base', NULL, 3),
(164, 10, 'gestion-des-erreurs-12-conversion-automatique', '1.2 - Conversion automatique', NULL, 4),
(165, 10, 'gestion-des-erreurs-deux-types-derreurs-differents', '`// Deux types d''erreurs différents !`', NULL, 5),
(166, 10, 'gestion-des-erreurs-13-chainage-doperations', '1.3 - Chaînage d''opérations', NULL, 6),
(167, 10, 'gestion-des-erreurs-2-panic-vs-result', '2. panic! vs Result', NULL, 7),
(168, 10, 'gestion-des-erreurs-21-quand-utiliser-panic', '2.1 - Quand utiliser panic!', NULL, 8),
(169, 10, 'gestion-des-erreurs-22-quand-utiliser-result', '2.2 - Quand utiliser Result', NULL, 9),
(170, 10, 'gestion-des-erreurs-23-unwrap-et-expect', '2.3 - unwrap() et expect()', NULL, 10),
(171, 10, 'gestion-des-erreurs-3-creer-ses-propres-erreurs', '3. Créer ses propres erreurs', NULL, 11),
(172, 10, 'gestion-des-erreurs-31-enum-derreurs-basique', '3.1 - Enum d''erreurs basique', NULL, 12),
(173, 10, 'gestion-des-erreurs-32-implementer-error-trait', '3.2 - Implémenter Error trait', NULL, 13),
(174, 10, 'gestion-des-erreurs-33-utiliser-thiserror', '3.3 - Utiliser thiserror', NULL, 14),
(175, 10, 'gestion-des-erreurs-4-anyhow-pour-les-applications', '4. anyhow pour les applications', NULL, 15),
(176, 10, 'gestion-des-erreurs-type-derreur-generique', '`// Type d''erreur générique`', NULL, 16),
(177, 10, 'gestion-des-erreurs-avantages', '`// Avantages:`', NULL, 17),
(178, 10, 'gestion-des-erreurs-inconvenients', '`// Inconvénients:`', NULL, 18),
(179, 10, 'gestion-des-erreurs-42-anyhowresult', '4.2 - anyhow::Result', NULL, 19),
(180, 10, 'gestion-des-erreurs-43-context-et-with-context', '4.3 - Context et with_context', NULL, 20),
(181, 10, 'gestion-des-erreurs-51-conversion-derreurs', '5.1 - Conversion d''erreurs', NULL, 21),
(182, 10, 'gestion-des-erreurs-52-erreurs-avec-backtrace', '5.2 - Erreurs avec backtrace', NULL, 22),
(183, 10, 'gestion-des-erreurs-53-early-returns', '5.3 - Early returns', NULL, 23),
(184, 10, 'gestion-des-erreurs-1-ne-pas-abuser-de-unwrap', '• **1. Ne pas abuser de unwrap()**', NULL, 24),
(185, 10, 'gestion-des-erreurs-2-propager-les-erreurs', '• **2. Propager les erreurs**', NULL, 25),
(186, 10, 'gestion-des-erreurs-3-messages-derreur-clairs', '• **3. Messages d''erreur clairs**', NULL, 26),
(187, 10, 'gestion-des-erreurs-4-types-derreurs-specifiques', '• **4. Types d''erreurs spécifiques**', NULL, 27),
(188, 10, 'gestion-des-erreurs-5-anyhow-pour-les-apps', '• **5. anyhow pour les apps**', NULL, 28),
(189, 10, 'gestion-des-erreurs-6-documenter-les-erreurs', '• **6. Documenter les erreurs**', NULL, 29),
(190, 10, 'gestion-des-erreurs-7-tests-derreurs', '• **7. Tests d''erreurs**', NULL, 30),
(191, 10, 'gestion-des-erreurs-7-exercices-pratiques', '7. Exercices pratiques', NULL, 31),
(192, 10, 'gestion-des-erreurs-exercice-2-utiliser-anyhow', 'Exercice 2 : Utiliser anyhow', NULL, 32);
INSERT INTO cour_block (id, chapitre_id, heading, content, block_type, sort_order) VALUES
(10508, 161, NULL, $BLK10508$Result, ?, Custom Errors et Best Practices$BLK10508$, 'text', 1),
(10509, 161, NULL, $BLK10509$Maîtriser l'Handling d'Erreurs en Rust$BLK10509$, 'text', 2),
(10510, 162, NULL, $BLK10510$L'opérateur `?` est un sucre syntaxique pour propager les erreurs. C'est l'outil le plus utilisé en Rust !$BLK10510$, 'text', 1),
(10511, 163, NULL, $BLK10511$```
use std::fs::File;
use std::io::{self, Read};
```$BLK10511$, 'code', 1),
(10512, 163, NULL, $BLK10512$```
// Sans ?
fn lire_fichier_verbeux(nom: &str) -> Result<String, io::Error> {
    let fichier = File::open(nom);
    let mut fichier = match fichier {
        Ok(f) => f,
        Err(e) => return Err(e),
    };
    let mut contenu = String::new();
    match fichier.read_to_string(&mut contenu) {
        Ok(_) => Ok(contenu),
        Err(e) => Err(e),
    }
}
// Avec ? (équivalent !)
fn lire_fichier(nom: &str) -> Result<String, io::Error> {
    let mut fichier = File::open(nom)?;
    let mut contenu = String::new();
    fichier.read_to_string(&mut contenu)?;
    Ok(contenu)
}
// Encore plus court
fn lire_fichier_court(nom: &str) -> Result<String, io::Error> {
    let mut contenu = String::new();
    File::open(nom)?.read_to_string(&mut contenu)?;
    Ok(contenu)
}
```$BLK10512$, 'code', 2),
(10513, 163, 'Comment ? fonctionne :', $BLK10513$1. Si `Ok(valeur)` → extrait la valeur 2. Si `Err(e)` → retourne immédiatement `Err(e)`$BLK10513$, 'list', 3),
(10514, 164, NULL, $BLK10514$```
use std::fs::File;
use std::io;
use std::num::ParseIntError;
```$BLK10514$, 'code', 1),
(10515, 165, NULL, $BLK10515$```
fn lire_et_parser(nom: &str) -> Result<i32, Box<dyn std::error::Error>> {
    let mut contenu = String::new();
    File::open(nom)?.read_to_string(&mut contenu)?;  // io::Error
    let nombre: i32 = contenu.trim().parse()?;       // ParseIntError
    Ok(nombre)
```$BLK10515$, 'code', 1),
(10516, 165, NULL, $BLK10516$```
}
```$BLK10516$, 'code', 2),
(10517, 165, NULL, $BLK10517$```
// ? convertit automatiquement vers le type d'erreur de retour
// grâce au trait From
```$BLK10517$, 'code', 3),
(10518, 166, NULL, $BLK10518$```
use std::fs::File;
use std::io::Read;
```$BLK10518$, 'code', 1),
(10519, 166, NULL, $BLK10519$```
fn obtenir_contenu() -> Result<String, std::io::Error> {
    let mut contenu = String::new();
```$BLK10519$, 'code', 2),
(10520, 166, NULL, $BLK10520$```
    // Chaîner plusieurs opérations
    File::open("config.txt")?
        .read_to_string(&mut contenu)?;
```$BLK10520$, 'code', 3),
(10521, 166, NULL, $BLK10521$```
    Ok(contenu)
}
```$BLK10521$, 'code', 4),
(10522, 166, NULL, $BLK10522$```
// Avec des méthodes qui retournent Result
fn traiter_donnees() -> Result<i32, Box<dyn std::error::Error>> {
    let texte = std::fs::read_to_string("nombre.txt")?;
    let nombre: i32 = texte.trim().parse()?;
    let resultat = nombre.checked_mul(2)
        .ok_or("Overflow")?;
```$BLK10522$, 'code', 5),
(10523, 166, NULL, $BLK10523$```
    Ok(resultat)
```$BLK10523$, 'code', 6),
(10524, 166, NULL, $BLK10524$```
}
```$BLK10524$, 'code', 7),
(10525, 167, NULL, $BLK10525$Choisir entre `panic!` et `Result` est crucial pour un code robuste.$BLK10525$, 'text', 1),
(10526, 168, NULL, $BLK10526$```
// 1. Bug dans le code (impossible normalement)
fn diviser(a: i32, b: i32) -> i32 {
    if b == 0 {
        panic!("Division par zéro - bug dans le code!");
    }
    a / b
}
```$BLK10526$, 'code', 1),
(10527, 168, NULL, $BLK10527$```
// 2. Tests
#[test]
fn test_division() {
    assert_eq!(diviser(10, 2), 5);
}
```$BLK10527$, 'code', 2),
(10528, 168, NULL, $BLK10528$```
// 3. Situation vraiment irrécupérable
fn initialiser_systeme() {
    let config = charger_config()
        .expect("Impossible de charger la config - arrêt");
    // Le programme ne peut pas continuer sans config
}
```$BLK10528$, 'code', 3),
(10529, 168, NULL, $BLK10529$```
// 4. Prototypes et exemples
fn main() {
    let fichier = File::open("data.txt")
        .unwrap();  // OK pour un proto rapide
}
```$BLK10529$, 'code', 4),
(10530, 168, NULL, $BLK10530$II **panic! termine le thread !** Dans une application web, ça peut tuer tout le serveur. Utilise `Result` pour les erreurs récupérables.$BLK10530$, 'text', 5),
(10531, 169, NULL, $BLK10531$```
// 1. Erreurs attendues et récupérables
fn ouvrir_fichier(nom: &str) -> Result<File, io::Error> {
    File::open(nom)  // Le fichier peut ne pas exister
}
```$BLK10531$, 'code', 1),
(10532, 169, NULL, $BLK10532$```
// 2. Opérations réseau
async fn fetch_data(url: &str) -> Result<String, reqwest::Error> {
    reqwest::get(url).await?.text().await
```$BLK10532$, 'code', 2),
(10533, 169, NULL, $BLK10533$```
}
```$BLK10533$, 'code', 3),
(10534, 169, NULL, $BLK10534$```
// 3. Parsing et validation
fn parser_age(texte: &str) -> Result<u8, String> {
    let age: u8 = texte.parse()
        .map_err(|_| "Pas un nombre valide".to_string())?;
    if age > 150 {
        return Err("Âge irréaliste".to_string());
    }
```$BLK10534$, 'code', 4),
(10535, 169, NULL, $BLK10535$```
    Ok(age)
}
```$BLK10535$, 'code', 5),
(10536, 169, NULL, $BLK10536$```
// 4. Logique métier
fn retirer_argent(compte: &mut Compte, montant: f64)
    -> Result<(), String>
```$BLK10536$, 'code', 6),
(10537, 169, NULL, $BLK10537$```
{
    if montant > compte.solde {
        return Err("Solde insuffisant".to_string());
    }
    compte.solde -= montant;
    Ok(())
}
```$BLK10537$, 'code', 7),
(10538, 170, NULL, $BLK10538$```
// unwrap() - panic si Err
let x: Result<i32, &str> = Ok(5);
let valeur = x.unwrap();  // 5
```$BLK10538$, 'code', 1),
(10539, 170, NULL, $BLK10539$**`let y: Result<i32, &str> = Err("erreur"); // let valeur = y.unwrap();  //`** I **`PANIC!`**$BLK10539$, 'text', 2),
(10540, 170, NULL, $BLK10540$```
// expect() - panic avec message personnalisé
let config = charger_config()
```$BLK10540$, 'code', 3),
(10541, 170, NULL, $BLK10541$```
    .expect("Config manquante - vérifier config.toml");
```$BLK10541$, 'code', 4),
(10542, 170, NULL, $BLK10542$```
// unwrap_or() - valeur par défaut si Err
let x: Result<i32, &str> = Err("erreur");
let valeur = x.unwrap_or(0);  // 0
```$BLK10542$, 'code', 5),
(10543, 170, NULL, $BLK10543$```
// unwrap_or_else() - calculer la valeur par défaut
let valeur = x.unwrap_or_else(|err| {
    eprintln!("Erreur: {}", err);
    0
```$BLK10543$, 'code', 6),
(10544, 170, NULL, $BLK10544$```
});
```$BLK10544$, 'code', 7),
(10545, 170, NULL, $BLK10545$```
// unwrap_or_default() - valeur par défaut du type
let valeur: i32 = x.unwrap_or_default();  // 0
```$BLK10545$, 'code', 8),
(10546, 171, NULL, $BLK10546$Pour un code professionnel, crée tes propres types d'erreurs spécifiques à ton domaine.$BLK10546$, 'text', 1),
(10547, 172, NULL, $BLK10547$```
#[derive(Debug)]
enum ConfigError {
    FichierManquant,
    FormatInvalide,
    CleManquante(String),
}
fn charger_config(chemin: &str) -> Result<Config, ConfigError> {
    let contenu = std::fs::read_to_string(chemin)
        .map_err(|_| ConfigError::FichierManquant)?;
    // Parser le contenu...
    if !contenu.contains("port") {
        return Err(ConfigError::CleManquante("port".to_string()));
    }
    Ok(Config { /* ... */ })
}
```$BLK10547$, 'code', 1),
(10548, 172, NULL, $BLK10548$```
// Utilisation
fn main() {
    match charger_config("config.toml") {
        Ok(config) => println!("Config chargée"),
        Err(ConfigError::FichierManquant) => {
            eprintln!("Créer config.toml");
        }
        Err(ConfigError::FormatInvalide) => {
            eprintln!("Format TOML invalide");
        }
        Err(ConfigError::CleManquante(cle)) => {
            eprintln!("Clé manquante: {}", cle);
        }
    }
```$BLK10548$, 'code', 2),
(10549, 172, NULL, $BLK10549$```
}
```$BLK10549$, 'code', 3),
(10550, 173, NULL, $BLK10550$```
use std::fmt;
use std::error::Error;
#[derive(Debug)]
enum MonErreur {
    Io(std::io::Error),
    Parse(std::num::ParseIntError),
    Custom(String),
}
// Implémenter Display (requis)
impl fmt::Display for MonErreur {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MonErreur::Io(e) => write!(f, "Erreur IO: {}", e),
            MonErreur::Parse(e) => write!(f, "Erreur parse: {}", e),
            MonErreur::Custom(msg) => write!(f, "Erreur: {}", msg),
        }
    }
}
// Implémenter Error
impl Error for MonErreur {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            MonErreur::Io(e) => Some(e),
            MonErreur::Parse(e) => Some(e),
            MonErreur::Custom(_) => None,
        }
    }
}
// Conversions automatiques avec From
impl From<std::io::Error> for MonErreur {
    fn from(err: std::io::Error) -> Self {
        MonErreur::Io(err)
    }
}
impl From<std::num::ParseIntError> for MonErreur {
    fn from(err: std::num::ParseIntError) -> Self {
        MonErreur::Parse(err)
    }
}
```$BLK10550$, 'code', 1),
(10551, 174, NULL, $BLK10551$La crate `thiserror` génère automatiquement l'implémentation de `Error` .$BLK10551$, 'text', 1),
(10552, 174, NULL, $BLK10552$```
use thiserror::Error;
```$BLK10552$, 'code', 2),
(10553, 174, NULL, $BLK10553$```
#[derive(Error, Debug)]
enum AppError {
    #[error("Fichier non trouvé: {0}")]
    FileNotFound(String),
    #[error("Erreur IO")]
    Io(#[from] std::io::Error),
    #[error("Erreur de parsing")]
    Parse(#[from] std::num::ParseIntError),
    #[error("Valeur invalide: {valeur}, attendu: {attendu}")]
    InvalidValue { valeur: i32, attendu: i32 },
```$BLK10553$, 'code', 3),
(10554, 174, NULL, $BLK10554$```
    #[error("Erreur réseau: {0}")]
    Network(#[from] reqwest::Error),
}
// Utilisation (beaucoup plus simple !)
fn traiter() -> Result<(), AppError> {
    let contenu = std::fs::read_to_string("data.txt")?;  // Converti auto
    let nombre: i32 = contenu.trim().parse()?;            // Converti auto
    if nombre < 0 {
        return Err(AppError::InvalidValue {
            valeur: nombre,
            attendu: 0,
        });
    }
    Ok(())
}
```$BLK10554$, 'code', 4),
(10555, 174, NULL, $BLK10555$I **thiserror** est parfait pour les **bibliothèques** où tu veux des types d'erreurs précis et bien définis.$BLK10555$, 'text', 5),
(10556, 175, NULL, $BLK10556$Pour les **applications** (pas les bibliothèques), `anyhow` simplifie énormément la gestion d'erreurs.$BLK10556$, 'text', 1),
(10557, 176, NULL, $BLK10557$```
fn faire_tout() -> Result<String, Box<dyn Error>> {
    let contenu = std::fs::read_to_string("file.txt")?;
    let nombre: i32 = contenu.trim().parse()?;
    let resultat = fetch_data(&nombre.to_string()).await?;
    Ok(resultat)
```$BLK10557$, 'code', 1),
(10558, 177, NULL, $BLK10558$**`//`** I **`Accepte n'importe quel type d'erreur`**$BLK10558$, 'text', 1),
(10559, 177, NULL, $BLK10559$**`//`** I **`Simple pour débuter`**$BLK10559$, 'text', 2),
(10560, 178, NULL, $BLK10560$**`//`** I **`Allocation heap`**$BLK10560$, 'text', 1),
(10561, 178, NULL, $BLK10561$**`//`** I **`Perd le type exact de l'erreur`**$BLK10561$, 'text', 2),
(10562, 178, NULL, $BLK10562$**`//`** I **`Pas de downcast facile`**$BLK10562$, 'text', 3),
(10563, 179, NULL, $BLK10563$```
use anyhow::{Result, Context};
```$BLK10563$, 'code', 1),
(10564, 179, NULL, $BLK10564$```
// Type alias: Result<T> = Result<T, anyhow::Error>
fn charger_config() -> Result<Config> {
    let contenu = std::fs::read_to_string("config.toml")
        .context("Impossible de lire config.toml")?;
```$BLK10564$, 'code', 2),
(10565, 179, NULL, $BLK10565$```
    let config: Config = toml::from_str(&contenu)
        .context("Format TOML invalide")?;
```$BLK10565$, 'code', 3),
(10566, 179, NULL, $BLK10566$```
    Ok(config)
}
```$BLK10566$, 'code', 4),
(10567, 179, NULL, $BLK10567$```
// Avantages d'anyhow:
```$BLK10567$, 'code', 5),
(10568, 179, NULL, $BLK10568$**`//`** I **`Messages d'erreur détaillés automatiques`**$BLK10568$, 'text', 6),
(10569, 179, NULL, $BLK10569$**`//`** I **`Backtrace si RUST_BACKTRACE=1`**$BLK10569$, 'text', 7),
(10570, 179, NULL, $BLK10570$**`//`** I **`Plus simple que Box<dyn Error>`**$BLK10570$, 'text', 8),
(10571, 179, NULL, $BLK10571$- **`//`** I **`Conversion automatique de tous les types d'erreur`**$BLK10571$, 'list', 9),
(10572, 180, NULL, $BLK10572$```
use anyhow::{Context, Result};
```$BLK10572$, 'code', 1),
(10573, 180, NULL, $BLK10573$```
fn traiter_fichier(chemin: &str) -> Result<()> {
    let contenu = std::fs::read_to_string(chemin)
        .context(format!("Lecture de {}", chemin))?;
    let lignes: Vec<&str> = contenu.lines().collect();
    for (i, ligne) in lignes.iter().enumerate() {
        traiter_ligne(ligne)
            .with_context(|| format!("Erreur ligne {}", i + 1))?;
    }
```$BLK10573$, 'code', 2),
(10574, 180, NULL, $BLK10574$```
    Ok(())
}
```$BLK10574$, 'code', 3),
(10575, 180, NULL, $BLK10575$```
// Chaîne de contexte complète en cas d'erreur:
// Error: Erreur ligne 5
// Caused by:
//     0: Valeur invalide
//     1: Lecture de data.txt
```$BLK10575$, 'code', 4),
(10576, 180, 'Quand utiliser quoi ?', $BLK10576$• **Bibliothèques** → `thiserror` (types d'erreurs précis) 

• **Applications** → `anyhow` (simplicité et contexte)$BLK10576$, 'text', 5),
(10577, 181, NULL, $BLK10577$```
// map_err pour convertir les erreurs
fn lire_nombre(chemin: &str) -> Result<i32, String> {
    let contenu = std::fs::read_to_string(chemin)
        .map_err(|e| format!("Lecture échouée: {}", e))?;
    contenu.trim()
        .parse()
        .map_err(|e| format!("Parse échoué: {}", e))
}
```$BLK10577$, 'code', 1),
(10578, 181, NULL, $BLK10578$```
// ok_or et ok_or_else pour Option -> Result
fn trouver_user(id: u32) -> Result<User, String> {
    DATABASE.get(&id)
        .ok_or_else(|| format!("User {} introuvable", id))
}
```$BLK10578$, 'code', 2),
(10579, 181, NULL, $BLK10579$```
// and_then pour chaîner des Results
fn traiter() -> Result<i32, String> {
    lire_fichier("data.txt")
        .and_then(|contenu| parser(&contenu))
        .and_then(|nombre| calculer(nombre))
}
```$BLK10579$, 'code', 3),
(10580, 182, NULL, $BLK10580$```
use std::backtrace::Backtrace;
```$BLK10580$, 'code', 1),
(10581, 182, NULL, $BLK10581$```
#[derive(Debug)]
struct MonErreur {
    message: String,
    backtrace: Backtrace,
}
impl MonErreur {
    fn new(message: String) -> Self {
        Self {
            message,
            backtrace: Backtrace::capture(),
        }
    }
}
// Avec anyhow, le backtrace est automatique !
// RUST_BACKTRACE=1 cargo run
```$BLK10581$, 'code', 2),
(10582, 183, NULL, $BLK10582$```
// Pattern: vérifier les conditions tôt
fn traiter_commande(cmd: &Commande) -> Result<(), String> {
    // Vérifications en premier
    if cmd.montant <= 0.0 {
        return Err("Montant invalide".to_string());
    }
```$BLK10582$, 'code', 1),
(10583, 183, NULL, $BLK10583$```
    if cmd.items.is_empty() {
        return Err("Commande vide".to_string());
    }
```$BLK10583$, 'code', 2),
(10584, 183, NULL, $BLK10584$```
    if !cmd.client_existe() {
        return Err("Client inconnu".to_string());
    }
```$BLK10584$, 'code', 3),
(10585, 183, NULL, $BLK10585$```
    // Logique principale ensuite
    valider_stock(cmd)?;
    calculer_prix(cmd)?;
    enregistrer(cmd)?;
```$BLK10585$, 'code', 4),
(10586, 183, NULL, $BLK10586$```
    Ok(())
```$BLK10586$, 'code', 5),
(10587, 183, NULL, $BLK10587$```
}
```$BLK10587$, 'code', 6),
(10588, 184, NULL, $BLK10588$Utilise `?` , `unwrap_or()` ou `expect()` avec un message clair.$BLK10588$, 'text', 1),
(10589, 185, NULL, $BLK10589$Laisse l'appelant décider comment gérer l'erreur. N'utilise `panic!` que pour les bugs.$BLK10589$, 'text', 1),
(10590, 186, NULL, $BLK10590$Inclus le contexte : quel fichier, quelle ligne, quelle valeur était attendue.$BLK10590$, 'text', 1),
(10591, 187, NULL, $BLK10591$Dans les bibliothèques, crée des enums d'erreurs précis avec `thiserror` .$BLK10591$, 'text', 1),
(10592, 188, NULL, $BLK10592$Pour les applications, utilise `anyhow` avec `.context()` .$BLK10592$, 'text', 1),
(10593, 189, NULL, $BLK10593$Dans la doc, indique quelles erreurs peuvent survenir et pourquoi.$BLK10593$, 'text', 1),
(10594, 190, NULL, $BLK10594$Teste les cas d'erreur autant que les cas de succès !$BLK10594$, 'text', 1),
(10595, 191, 'Exercice 1 : Créer un type d''erreur', $BLK10595$```
// Crée un enum ValidationError pour valider un email
// Doit gérer: vide, pas de @, domaine invalide
```

```
// Solution avec thiserror:
use thiserror::Error;
```

```
#[derive(Error, Debug)]
enum ValidationError {
    #[error("Email vide")]
    Empty,
```

```
    #[error("@ manquant")]
    NoAtSign,
```

```
    #[error("Domaine invalide: {0}")]
    InvalidDomain(String),
}
```

```
fn valider_email(email: &str) -> Result<(), ValidationError> {
    if email.is_empty() {
```

```
        return Err(ValidationError::Empty);
    }
```

```
    if !email.contains('@') {
        return Err(ValidationError::NoAtSign);
    }
```

```
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 || !parts[1].contains('.') {
        return Err(ValidationError::InvalidDomain(
            parts.get(1).unwrap_or(&"").to_string()
        ));
    }
```

```
    Ok(())
```

```
}
```$BLK10595$, 'code', 1),
(10596, 192, NULL, $BLK10596$```
// Réécris cette fonction avec anyhow et context
```$BLK10596$, 'code', 1),
(10597, 192, NULL, $BLK10597$```
use anyhow::{Context, Result};
```$BLK10597$, 'code', 2),
(10598, 192, NULL, $BLK10598$```
fn charger_et_traiter(chemin: &str) -> Result<i32> {
    let contenu = std::fs::read_to_string(chemin)
        .context(format!("Impossible de lire {}", chemin))?;
```$BLK10598$, 'code', 3),
(10599, 192, NULL, $BLK10599$```
    let nombre: i32 = contenu.trim()
        .parse()
        .context("Le fichier ne contient pas un nombre valide")?;
```$BLK10599$, 'code', 4),
(10600, 192, NULL, $BLK10600$```
    if nombre < 0 {
        anyhow::bail!("Le nombre doit être positif");
```$BLK10600$, 'code', 5),
(10601, 192, NULL, $BLK10601$- **`}`**$BLK10601$, 'list', 6),
(10602, 192, NULL, $BLK10602$```
    Ok(nombre * 2)
}
```$BLK10602$, 'code', 7),
(10603, 192, 'Excellent !', $BLK10603$Tu maîtrises maintenant la gestion d'erreurs en Rust ! 

Points clés à retenir : • Utilise `?` pour propager • `thiserror` pour les libs • `anyhow` pour les apps • Toujours ajouter du contexte ! 

I **Ton code sera robuste !** I$BLK10603$, 'text', 8);

-- generics.md (cour_id=11)
INSERT INTO chapitre (id, cour_id, slug, title, lead, sort_order) VALUES
(193, 11, 'generics-1-fonctions-generiques', '1. Fonctions génériques', NULL, 1),
(194, 11, 'generics-2-structs-et-enums-generiques', '2. Structs et enums génériques', NULL, 2),
(195, 11, 'generics-3-trait-bounds', '3. Trait bounds', NULL, 3),
(196, 11, 'generics-4-implementations-conditionnelles', '4. Implémentations conditionnelles', NULL, 4),
(197, 11, 'generics-5-generics-dans-les-traits', '5. Generics dans les traits', NULL, 5),
(198, 11, 'generics-6-monomorphisation', '6. Monomorphisation', NULL, 6);
INSERT INTO cour_block (id, chapitre_id, heading, content, block_type, sort_order) VALUES
(10604, 193, NULL, $BLK10604$```rust
// Sans génériques — dupliqué
fn plus_grand_i32(a: i32, b: i32) -> i32 {
    if a > b { a } else { b }
}
fn plus_grand_f64(a: f64, b: f64) -> f64 {
    if a > b { a } else { b }
}

// Avec génériques — un seul code
fn plus_grand<T: PartialOrd>(a: T, b: T) -> T {
    if a > b { a } else { b }
}

println!("{}", plus_grand(5, 10));       // 10
println!("{}", plus_grand(3.14, 2.71)); // 3.14
println!("{}", plus_grand('a', 'z'));   // z
```$BLK10604$, 'code', 1),
(10605, 193, NULL, $BLK10605$---$BLK10605$, 'text', 2),
(10606, 194, NULL, $BLK10606$```rust
// Struct générique
struct Paire<T> {
    premier: T,
    second: T,
}

impl<T> Paire<T> {
    fn new(premier: T, second: T) -> Self {
        Paire { premier, second }
    }
}

// Plusieurs paramètres de type
struct Couple<T, U> {
    gauche: T,
    droite: U,
}

let c = Couple { gauche: 42, droite: "bonjour" };

// Enums génériques — tu les connais déjà !
enum Option<T> {
    Some(T),
    None,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}
```$BLK10606$, 'code', 1),
(10607, 194, NULL, $BLK10607$---$BLK10607$, 'text', 2),
(10608, 195, '3.1 Syntaxe inline', $BLK10608$```rust
// T doit implémenter Display
fn afficher<T: std::fmt::Display>(valeur: T) {
    println!("{valeur}");
}

// T doit implémenter Display + Debug
fn afficher_debug<T: std::fmt::Display + std::fmt::Debug>(val: T) {
    println!("Display: {val}  Debug: {val:?}");
}
```$BLK10608$, 'code', 1),
(10609, 195, '3.2 Clauses where', $BLK10609$La clause `where` rend le code plus lisible quand les bounds sont nombreux.

```rust
// Inline — difficile à lire
fn comparer<T: PartialOrd + std::fmt::Display, U: std::fmt::Debug>(a: T, b: T, extra: U) -> bool {
    println!("extra: {extra:?}");
    a > b
}

// Avec where — beaucoup plus clair
fn comparer<T, U>(a: T, b: T, extra: U) -> bool
where
    T: PartialOrd + std::fmt::Display,
    U: std::fmt::Debug,
{
    println!("extra: {extra:?}");
    a > b
}
```$BLK10609$, 'text', 2),
(10610, 195, '3.3 Plusieurs bounds', $BLK10610$```rust
use std::fmt::{Display, Debug};

fn traiter<T>(valeur: T)
where
    T: Display + Debug + Clone + PartialEq,
{
    let copie = valeur.clone();
    println!("Display: {valeur}");
    println!("Debug:   {valeur:?}");
    println!("Égaux:   {}", valeur == copie);
}
```

---$BLK10610$, 'code', 3),
(10611, 196, NULL, $BLK10611$```rust
use std::fmt::Display;

struct Wrapper<T>(T);

// Méthode disponible pour tous les T
impl<T> Wrapper<T> {
    fn valeur(&self) -> &T {
        &self.0
    }
}

// Méthode disponible uniquement si T: Display
impl<T: Display> Wrapper<T> {
    fn afficher(&self) {
        println!("{}", self.0);
    }
}

// Implémenter un trait conditionnellement (blanket impl)
impl<T: Display> std::fmt::Debug for Wrapper<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Wrapper({})", self.0)
    }
}

let w = Wrapper(42);
w.afficher();       // disponible car i32: Display
```$BLK10611$, 'code', 1),
(10612, 196, NULL, $BLK10612$---$BLK10612$, 'text', 2),
(10613, 197, 'Types associés vs paramètres génériques', $BLK10613$```rust
// Type associé — une seule implémentation par type
trait Convertir {
    type Sortie;
    fn convertir(self) -> Self::Sortie;
}

impl Convertir for i32 {
    type Sortie = f64;
    fn convertir(self) -> f64 { self as f64 }
}

// Paramètre générique — plusieurs implémentations possibles
trait ConvertirEn<T> {
    fn convertir_en(self) -> T;
}

impl ConvertirEn<f64> for i32 {
    fn convertir_en(self) -> f64 { self as f64 }
}
impl ConvertirEn<String> for i32 {
    fn convertir_en(self) -> String { self.to_string() }
}
```$BLK10613$, 'code', 1),
(10614, 197, '`impl Trait` en paramètre et en retour', $BLK10614$```rust
// En paramètre : syntaxe courte pour un trait bound
fn afficher(val: impl Display) {
    println!("{val}");
}
// Équivalent à : fn afficher<T: Display>(val: T)

// En retour : type concret opaque
fn creer_iterateur() -> impl Iterator<Item = i32> {
    vec![1, 2, 3].into_iter()
}
// Le type exact est caché, seul Iterator est exposé
```

---$BLK10614$, 'code', 2),
(10615, 198, NULL, $BLK10615$Rust génère **une version spécialisée** du code pour chaque type concret utilisé.$BLK10615$, 'text', 1),
(10616, 198, NULL, $BLK10616$```rust
fn identite<T>(x: T) -> T { x }

identite(5i32);     // génère : fn identite_i32(x: i32) -> i32
identite(3.14f64);  // génère : fn identite_f64(x: f64) -> f64
identite("texte");  // génère : fn identite_str(x: &str) -> &str
```$BLK10616$, 'code', 2),
(10617, 198, NULL, $BLK10617$**Conséquences :**
- ✅ Zéro coût à l'exécution (pas d'appel indirect, pas de boxing)
- ✅ Le compilateur peut optimiser chaque version
- ⚠️ Binaire plus grand si beaucoup de types différents utilisés$BLK10617$, 'text', 3),
(10618, 198, NULL, $BLK10618$**Comparaison avec `dyn Trait` (dispatch dynamique) :**$BLK10618$, 'text', 4),
(10619, 198, NULL, $BLK10619$```rust
// Generics (statique) — résolu à la compilation
fn appeler_generique<T: Parler>(animal: &T) {
    animal.parler();
}

// dyn Trait (dynamique) — résolu à l'exécution via vtable
fn appeler_dynamique(animal: &dyn Parler) {
    animal.parler();
}

// dyn Trait utile pour des collections hétérogènes
let animaux: Vec<Box<dyn Parler>> = vec![
    Box::new(Chien),
    Box::new(Chat),
];
```$BLK10619$, 'code', 5),
(10620, 198, NULL, $BLK10620$> **Règle :** Préférez les génériques pour la performance. Utilisez `dyn Trait` quand vous avez besoin de types hétérogènes à l'exécution.$BLK10620$, 'warning', 6);

-- type-aliases.md (cour_id=12)
INSERT INTO chapitre (id, cour_id, slug, title, lead, sort_order) VALUES
(199, 12, 'type-aliases-guide-complet-et-pratique', 'Guide Complet et Pratique', NULL, 1),
(200, 12, 'type-aliases-1-introduction-aux-type-aliases', '1. Introduction aux Type Aliases', NULL, 2),
(201, 12, 'type-aliases-pourquoi-utiliser-des-type-aliases', 'Pourquoi utiliser des type aliases ?', NULL, 3),
(202, 12, 'type-aliases-21-syntaxe-generale', '2.1 Syntaxe générale', NULL, 4),
(203, 12, 'type-aliases-22-premier-exemple-concret', '2.2 Premier exemple concret', NULL, 5),
(204, 12, 'type-aliases-23-type-aliases-dans-les-structures', '2.3 Type aliases dans les structures', NULL, 6),
(205, 12, 'type-aliases-31-simplifier-les-types-de-retour', '3.1 Simplifier les types de retour', NULL, 7),
(206, 12, 'type-aliases-32-types-complexes-recurrents', '3.2 Types complexes récurrents', NULL, 8),
(207, 12, 'type-aliases-33-abstraire-les-details-dimplementation', '3.3 Abstraire les détails d''implémentation', NULL, 9),
(208, 12, 'type-aliases-41-la-difference-fondamentale', '4.1 La différence fondamentale', NULL, 10),
(209, 12, 'type-aliases-42-quand-utiliser-quoi', '4.2 Quand utiliser quoi ?', NULL, 11),
(210, 12, 'type-aliases-43-recommandations', '4.3 Recommandations', NULL, 12),
(211, 12, 'type-aliases-51-alias-generiques-basiques', '5.1 Alias génériques basiques', NULL, 13),
(212, 12, 'type-aliases-52-specialisation-partielle', '5.2 Spécialisation partielle', NULL, 14),
(213, 12, 'type-aliases-53-alias-pour-types-complexes', '5.3 Alias pour types complexes', NULL, 15),
(214, 12, 'type-aliases-61-organiser-vos-aliases', '6.1 Organiser vos aliases', NULL, 16),
(215, 12, 'type-aliases-62-structure-recommandee', '6.2 Structure recommandée', NULL, 17),
(216, 12, 'type-aliases-63-conventions-de-nommage', '6.3 Conventions de nommage', NULL, 18),
(217, 12, 'type-aliases-71-pas-de-verification-de-type-supplementaire', '7.1 Pas de vérification de type supplémentaire', NULL, 19),
(218, 12, 'type-aliases-72-messages-derreur-du-compilateur', '7.2 Messages d''erreur du compilateur', NULL, 20),
(219, 12, 'type-aliases-73-pas-dimplementation-de-traits', '7.3 Pas d''implémentation de traits', NULL, 21),
(220, 12, 'type-aliases-81-types-de-vue', '8.1 Types de vue', NULL, 22),
(221, 12, 'type-aliases-82-types-de-base-de-donnees', '8.2 Types de base de données', NULL, 23),
(222, 12, 'type-aliases-83-types-de-contexte', '8.3 Types de contexte', NULL, 24),
(223, 12, 'type-aliases-91-type-aliases-conditionnels', '9.1 Type Aliases conditionnels', NULL, 25),
(224, 12, 'type-aliases-92-chainage-daliases', '9.2 Chaînage d''aliases', NULL, 26),
(225, 12, 'type-aliases-93-aliases-pour-traits-objets', '9.3 Aliases pour traits objets', NULL, 27),
(226, 12, 'type-aliases-exercice-1-refactoring-basique', 'Exercice 1 : Refactoring basique', NULL, 28),
(227, 12, 'type-aliases-exercice-2-organisation-modulaire', 'Exercice 2 : Organisation modulaire', NULL, 29),
(228, 12, 'type-aliases-exercice-3-genericite', 'Exercice 3 : Généricité', NULL, 30),
(229, 12, 'type-aliases-solution-exercice-1', 'Solution Exercice 1', NULL, 31),
(230, 12, 'type-aliases-solution-exercice-2', 'Solution Exercice 2', NULL, 32),
(231, 12, 'type-aliases-solution-exercice-3', 'Solution Exercice 3', NULL, 33),
(232, 12, 'type-aliases-conclusion', 'Conclusion', NULL, 34),
(233, 12, 'type-aliases-points-cles-a-retenir', 'Points clés à retenir :', NULL, 35),
(234, 12, 'type-aliases-ressources-complementaires', 'Ressources complémentaires', NULL, 36);
INSERT INTO cour_block (id, chapitre_id, heading, content, block_type, sort_order) VALUES
(10621, 199, NULL, $BLK10621$_Pour développeurs Rust intermédiaires_$BLK10621$, 'text', 1),
(10622, 199, NULL, $BLK10622$Généré le 28 January 2026$BLK10622$, 'text', 2),
(10623, 200, NULL, $BLK10623$Les **type aliases** (alias de types) sont un outil puissant en Rust qui permet de créer des noms alternatifs pour des types existants. Contrairement à ce qu'on pourrait penser, ils ne créent pas de nouveaux types, mais simplement des _synonymes_ pour des types existants.$BLK10623$, 'text', 1),
(10624, 201, NULL, $BLK10624$- **Lisibilité** : Simplifier des types complexes ou longs$BLK10624$, 'list', 1),
(10625, 201, NULL, $BLK10625$- **Maintenabilité** : Centraliser les définitions de types$BLK10625$, 'list', 2),
(10626, 201, NULL, $BLK10626$- **Documentation** : Donner un sens métier aux types techniques$BLK10626$, 'list', 3),
(10627, 201, NULL, $BLK10627$- **Réduction de verbosité** : Éviter la répétition de types génériques$BLK10627$, 'list', 4),
(10628, 201, NULL, $BLK10628$- **Abstraction** : Masquer les détails d'implémentation$BLK10628$, 'list', 5),
(10629, 201, NULL, $BLK10629$I **Note importante :** Les type aliases n'ajoutent AUCUN overhead au runtime. Le compilateur les remplace par le type réel lors de la compilation. C'est du _zero-cost abstraction_ .$BLK10629$, 'text', 6),
(10630, 202, NULL, $BLK10630$```
type NomAlias = TypeExistant; // Exemples type UserId = i32; type Username =
String; type Result = std::result::Result;
```$BLK10630$, 'code', 1),
(10631, 203, NULL, $BLK10631$`// Sans type alias` I `fn create_user(id: i32, name: String) -> i32 { // ... id } fn get_user(id: i32) -> Option { // ... None } // Avec type alias` I `type UserId = i32; type Username = String; fn create_user(id: UserId, name: Username) -> UserId { // ... id } fn get_user(id: UserId) -> Option { // ... None }`$BLK10631$, 'text', 1),
(10632, 203, NULL, $BLK10632$Dans cet exemple, l'intention du code devient **beaucoup plus claire** . On comprend immédiatement qu'on manipule un identifiant utilisateur et un nom d'utilisateur, pas juste des entiers et des chaînes génériques.$BLK10632$, 'text', 2),
(10633, 204, NULL, $BLK10633$```
type Timestamp = i64; type JsonData = serde_json::Value; struct Event { id:
UserId, created_at: Timestamp, data: JsonData, } // Utilisation let event =
Event { id: 42, created_at: 1706400000, data: serde_json::json!({"action":
"login"}), };
```$BLK10633$, 'code', 1),
(10634, 205, NULL, $BLK10634$`// Avant` I `fn process_data(input: &str;) -> Result>, Box> { // ... } // Après` I `type ProcessResult = Result>, Box>; fn process_data(input: &str;) -> ProcessResult { // ... } // Encore mieux` II `type DataMap = HashMap; type ProcessResult = Result, Box>; fn process_data(input: &str;) -> ProcessResult { // ... }`$BLK10634$, 'text', 1),
(10635, 206, NULL, $BLK10635$```
// Types de bases de données type DbPool = Arc>; type DbResult = Result; //
Utilisation cohérente async fn get_user(pool: &DbPool;, id: UserId) -> DbResult
{ // ... } async fn create_user(pool: &DbPool;, user: User) -> DbResult<()> { //
... } async fn delete_user(pool: &DbPool;, id: UserId) -> DbResult<()> { // ...
}
```$BLK10635$, 'code', 1),
(10636, 206, NULL, $BLK10636$II **Attention :** Trop d'alias peut rendre le code _moins_ lisible. Utilisez-les avec parcimonie et seulement quand ils apportent une vraie valeur ajoutée.$BLK10636$, 'text', 2),
(10637, 207, NULL, $BLK10637$```
// Dans votre API publique pub type Cache = HashMap; // Plus tard, vous pouvez
changer l'implémentation // pub type Cache = LruCache; // ou // pub type Cache =
DashMap; // Les utilisateurs de votre API n'ont pas besoin de changer leur code
!
```$BLK10637$, 'code', 1),
(10638, 208, NULL, $BLK10638$```
// Type Alias - PAS un nouveau type type UserId = i32; // Newtype Pattern -
NOUVEAU type struct UserId(i32); // Conséquences : let id1: UserId = 42; // Type
alias - OK let id2: i32 = id1; // OK - même type ! let id3 = UserId(42); //
Newtype - OK let id4: i32 = id3; // ERREUR - types différents ! let id5: i32 =
id3.0; // OK - accès explicite
```$BLK10638$, 'code', 1),
(10639, 209, NULL, $BLK10639$|**Critère**|**Type Alias**|**Newtype**|
|---|---|---|
|Type-safety|IIFaible|IForte|
|Runtime overhead|IAucun|IAucun (optimisé)|
|Méthodes custom|INon|IOui|
|Traits custom|INon|IOui|
|Verbosité|IFaible|IIMoyenne|
|Interopérabilité|ITransparente|IIConversion manuelle|$BLK10639$, 'table', 1),
(10640, 210, NULL, $BLK10640$- **Type Alias** : Pour simplifier la syntaxe sans ajouter de garanties de type supplémentaires$BLK10640$, 'list', 1),
(10641, 210, NULL, $BLK10641$- **Newtype** : Pour créer des types distincts avec validation ou méthodes spécifiques$BLK10641$, 'list', 2),
(10642, 210, NULL, $BLK10642$- **Exemple Type Alias** : Result<T>, collections spécifiques, types de callback$BLK10642$, 'list', 3),
(10643, 210, NULL, $BLK10643$- **Exemple Newtype** : Unités (Meters, Seconds), identifiants validés, types métier$BLK10643$, 'list', 4),
(10644, 211, NULL, $BLK10644$```
// Alias pour Result personnalisé type AppResult = Result; // Utilisation fn
create_user(name: &str;) -> AppResult { // ... } fn delete_user(id: UserId) ->
AppResult<()> { // ... }
```$BLK10644$, 'code', 1),
(10645, 212, NULL, $BLK10645$```
// Type générique complet type GenericResult = Result; // Spécialisation de
l'erreur type AppResult = Result; // Spécialisation complète type UserResult =
Result; // Hiérarchie de spécialisation type DbResult = Result; type
UserDbResult = DbResult;
```$BLK10645$, 'code', 1),
(10646, 213, NULL, $BLK10646$```
// Collection de callbacks type EventHandler = Box () + Send + Sync>; type
EventHandlers = Vec>; // State management type StateUpdater = Arc>; type
SharedState = Arc>; // Async futures type AsyncResult = Pin> + Send>>;
```$BLK10646$, 'code', 1),
(10647, 213, NULL, $BLK10647$I **Astuce Pro :** Les type aliases génériques sont parfaits pour créer des APIs consistantes dans tout votre codebase. Définissez-les une fois dans un module central.$BLK10647$, 'text', 2),
(10648, 214, NULL, $BLK10648$`//` I `Mauvais - dispersé partout mod user { type UserId = i32; // ... } mod product { type ProductId = i32; // ... } //` I `Bon - centralisé // types.rs ou common_types.rs pub type UserId = i32; pub type ProductId = i32; pub type Timestamp = i64; pub type JsonData = serde_json::Value; // Usage dans les autres modules use crate::types::*;`$BLK10648$, 'text', 1),
(10649, 215, NULL, $BLK10649$```
// src/types/mod.rs pub mod db; pub mod api; pub mod errors; pub use db::*; pub
use api::*; pub use errors::*; // src/types/db.rs pub type DbPool = Arc; pub
type DbResult = Result; // src/types/api.rs pub type ApiResult = Result; pub
type JsonResponse = Json; // src/types/errors.rs pub type AppError = Box; pub
type AppResult = Result;
```$BLK10649$, 'code', 1),
(10650, 216, NULL, $BLK10650$- **Suffixes descriptifs** : UserId, UserResult, UserError$BLK10650$, 'list', 1),
(10651, 216, NULL, $BLK10651$- **Préfixes de module** : DbPool, ApiResponse, WebConfig$BLK10651$, 'list', 2),
(10652, 216, NULL, $BLK10652$- **Contexte métier** : OrderId plutôt que Id, Price plutôt que Decimal$BLK10652$, 'list', 3),
(10653, 216, NULL, $BLK10653$- **Évitez les abréviations** : DatabaseConnection, pas DbConn (sauf conventions établies)$BLK10653$, 'list', 4),
(10654, 216, NULL, $BLK10654$- **PascalCase obligatoire** : Suivez les conventions Rust$BLK10654$, 'list', 5),
(10655, 217, NULL, $BLK10655$`type UserId = i32; type ProductId = i32; fn get_user(id: UserId) -> User { /* ... */ } //` II `Ceci compile sans erreur ! let product_id: ProductId = 123; let user = get_user(product_id); // BUG silencieux // Solution : Utilisez le Newtype Pattern pour une vraie type-safety struct UserId(i32); struct ProductId(i32); fn get_user(id: UserId) -> User { /* ... */ } let product_id = ProductId(123); // get_user(product_id); //` I `ERREUR de compilation !`$BLK10655$, 'text', 1),
(10656, 218, NULL, $BLK10656$```
type ComplexType = HashMap, Error>>>; fn process(data: ComplexType) { /* ... */
} // Erreur du compilateur affichera le type COMPLET, pas l'alias ! // expected
`HashMap, Error>>>`, // found `HashMap, Error>>>`
```$BLK10656$, 'code', 1),
(10657, 218, NULL, $BLK10657$II **Limitation :** Les messages d'erreur montrent toujours le type réel, pas l'alias. Cela peut rendre les erreurs plus difficiles à comprendre.$BLK10657$, 'text', 2),
(10658, 219, NULL, $BLK10658$`type UserId = i32; //` I `Impossible d'implémenter des traits sur un alias impl Display for UserId { // ERREUR fn fmt(&self;, f: &mut; Formatter) -> fmt::Result { write!(f, "User #{}", self) } } //` I `Solution : Utilisez un Newtype struct UserId(i32); impl Display for UserId { // OK fn fmt(&self;, f: &mut; Formatter) -> fmt::Result { write!(f, "User #{}", self.0) } }`$BLK10658$, 'text', 1),
(10659, 220, NULL, $BLK10659$```
// runique/src/forms/types.rs use crate::forms::utils::ViewContext; use
crate::forms::fields::*; // Vues de formulaires pub type RegisterView =
ViewContext; pub type LoginView = ViewContext; pub type ContactView =
ViewContext; pub type ProfileView = ViewContext; // Usage dans les handlers pub
async fn register_view(view: RegisterView) -> AppResult { if view.is_get() {
return view.handle_get("register.html"); } // ... }
```$BLK10659$, 'code', 1),
(10660, 221, NULL, $BLK10660$```
// runique/src/db/types.rs use sea_orm::{DatabaseConnection, DbErr}; use
std::sync::Arc; // Pool de connexions pub type DbPool = Arc; // Résultats de
requêtes pub type DbResult = Result; // Collections courantes pub type UserList
= Vec; pub type UserMap = HashMap; // Usage pub async fn get_users(pool:
&DbPool;) -> DbResult { User::find().all(pool.as_ref()).await }
```$BLK10660$, 'code', 1),
(10661, 222, NULL, $BLK10661$```
// runique/src/context/types.rs use crate::context::{AppError,
TemplateContext}; use axum::response::Response; // Résultats applicatifs pub
type AppResult = Result; pub type AppResponse = Result; // Context handlers pub
type HandlerResult = AppResult; // Extractors pub type CtxResult = Result;
```$BLK10661$, 'code', 1),
(10662, 223, NULL, $BLK10662$```
// Différents types selon la configuration #[cfg(feature = "async")] pub type
Handler = Box Pin>>>; #[cfg(not(feature = "async"))] pub type Handler = Box ()>;
// Usage identique dans le code fn register_handler(handler: Handler) { // ... }
```$BLK10662$, 'code', 1),
(10663, 224, NULL, $BLK10663$```
// Construction progressive type RawData = Vec; type ParsedData = Result; type
ValidatedData = Result; // Chaîne de traitement fn parse(raw: RawData) ->
ParsedData { /* ... */ } fn validate(parsed: ParsedData) -> ValidatedData { /*
... */ }
```$BLK10663$, 'code', 1),
(10664, 225, NULL, $BLK10664$```
// Simplifier les trait objects type EventListener = Box; type AsyncHandler =
Box Pin>> + Send>; // Collections de handlers type EventHandlers = Vec; type
Middleware = Vec Response + Send + Sync>>;
```$BLK10664$, 'code', 1),
(10665, 225, NULL, $BLK10665$I **Pattern Pro :** Combinez type aliases et génériques pour créer des APIs flexibles et faciles à utiliser. C'est exactement ce que fait la stdlib avec Result, Option, etc.$BLK10665$, 'text', 2),
(10666, 226, NULL, $BLK10666$Refactorez ce code en utilisant des type aliases appropriés :$BLK10666$, 'text', 1),
(10667, 226, NULL, $BLK10667$```
// Code à refactorer fn get_user(id: i32, db: &Arc;>) -> Result, Box> { // ... }
fn create_user( name: String, email: String, db: &Arc;> ) -> Result> { // ... }
```$BLK10667$, 'code', 2),
(10668, 227, NULL, $BLK10668$Organisez ces types dans une hiérarchie de modules appropriée :$BLK10668$, 'text', 1),
(10669, 227, NULL, $BLK10669$```
// Types en vrac type UserId = i32; type ProductId = i32; type OrderId = i32;
type UserResult = Result; type ProductResult = Result; type ApiError = Box; type
JsonPayload = serde_json::Value;
```$BLK10669$, 'code', 2),
(10670, 228, NULL, $BLK10670$Créez une hiérarchie de type aliases génériques pour ce système de cache :$BLK10670$, 'text', 1),
(10671, 228, NULL, $BLK10671$```
// Système de cache à implémenter struct Cache { data: HashMap, } // Créez des
aliases pour : // 1. Un cache de chaînes vers chaînes // 2. Un cache générique
avec erreurs // 3. Un cache asynchrone avec timeout
```$BLK10671$, 'code', 2),
(10672, 229, NULL, $BLK10672$```
// Types centralisés type UserId = i32; type DbPool = Arc>; type AppError = Box;
type AppResult = Result; // Code refactoré fn get_user(id: UserId, db: &DbPool;)
-> AppResult> { // ... } fn create_user(name: String, email: String, db:
&DbPool;) -> AppResult { // ... }
```$BLK10672$, 'code', 1),
(10673, 230, NULL, $BLK10673$```
// src/types/mod.rs pub mod ids; pub mod db; pub mod api; // src/types/ids.rs
pub type UserId = i32; pub type ProductId = i32; pub type OrderId = i32; //
src/types/db.rs use super::ids::*; pub type UserResult = Result; pub type
ProductResult = Result; // src/types/api.rs pub type ApiError = Box; pub type
JsonPayload = serde_json::Value; pub type ApiResult = Result;
```$BLK10673$, 'code', 1),
(10674, 231, NULL, $BLK10674$```
// 1. Cache simple type StringCache = Cache; // 2. Cache avec gestion d'erreur
type CacheResult = Result, CacheError>; // 3. Cache asynchrone type AsyncCache =
Arc>>; type CacheFuture = Pin>>>; // Bonus: Cache avec TTL type TtlCache =
Cache;
```$BLK10674$, 'code', 1),
(10675, 232, NULL, $BLK10675$Les **type aliases** sont un outil simple mais puissant en Rust. Utilisés correctement, ils améliorent significativement la lisibilité et la maintenabilité de votre code sans aucun coût au runtime.$BLK10675$, 'text', 1),
(10676, 233, NULL, $BLK10676$- Les type aliases sont des _synonymes_ , pas de nouveaux types$BLK10676$, 'list', 1),
(10677, 233, NULL, $BLK10677$- Zero-cost abstraction : aucun overhead au runtime$BLK10677$, 'list', 2),
(10678, 233, NULL, $BLK10678$- Excellents pour simplifier les types complexes récurrents$BLK10678$, 'list', 3),
(10679, 233, NULL, $BLK10679$- Organisez-les dans des modules dédiés (types.rs)$BLK10679$, 'list', 4),
(10680, 233, NULL, $BLK10680$- Utilisez le Newtype Pattern quand vous avez besoin de vraie type-safety$BLK10680$, 'list', 5),
(10681, 233, NULL, $BLK10681$- Les messages d'erreur du compilateur montrent le type réel, pas l'alias$BLK10681$, 'list', 6),
(10682, 234, NULL, $BLK10682$- **The Rust Book** : Chapitre sur les type aliases$BLK10682$, 'list', 1),
(10683, 234, NULL, $BLK10683$- **Rust by Example** : Section sur les types personnalisés$BLK10683$, 'list', 2),
(10684, 234, NULL, $BLK10684$- **Rust API Guidelines** : Conventions de nommage$BLK10684$, 'list', 3),
(10685, 234, NULL, $BLK10685$- **Documentation Rust std** : Exemples dans std::result, std::io$BLK10685$, 'list', 4);

-- lifetimes.md (cour_id=13)
INSERT INTO chapitre (id, cour_id, slug, title, lead, sort_order) VALUES
(235, 13, 'lifetimes-les-lifetimes-en-rust', 'Les Lifetimes en Rust', NULL, 1),
(236, 13, 'lifetimes-1-quest-ce-quune-lifetime', '1. Qu''est-ce qu''une lifetime ?', NULL, 2),
(237, 13, 'lifetimes-21-syntaxe-de-base', '2.1 - Syntaxe de base', NULL, 3),
(238, 13, 'lifetimes-22-dans-les-fonctions', '2.2 - Dans les fonctions', NULL, 4),
(239, 13, 'lifetimes-23-plusieurs-lifetimes', '2.3 - Plusieurs lifetimes', NULL, 5),
(240, 13, 'lifetimes-3-lifetime-elision', '3. Lifetime Elision', NULL, 6),
(241, 13, 'lifetimes-31-regles-delision', '3.1 - Règles d''élision', NULL, 7),
(242, 13, 'lifetimes-32-quand-annoter', '3.2 - Quand annoter', NULL, 8),
(243, 13, 'lifetimes-41-struct-avec-references', '4.1 - Struct avec références', NULL, 9),
(244, 13, 'lifetimes-42-methodes-et-lifetimes', '4.2 - Méthodes et lifetimes', NULL, 10),
(245, 13, 'lifetimes-5-static-lifetime', '5. ''static lifetime', NULL, 11),
(246, 13, 'lifetimes-6-patterns-avances', '6. Patterns avancés', NULL, 12),
(247, 13, 'lifetimes-7-resoudre-les-erreurs', '7. Résoudre les erreurs', NULL, 13),
(248, 13, 'lifetimes-erreur-lifetime-mismatch', 'Erreur : "lifetime mismatch"', NULL, 14),
(249, 13, 'lifetimes-exercice-1-annoter-les-lifetimes', 'Exercice 1 : Annoter les lifetimes', NULL, 15),
(250, 13, 'lifetimes-exercice-2-struct-avec-lifetime', 'Exercice 2 : Struct avec lifetime', NULL, 16),
(251, 13, 'lifetimes-aide-memoire', 'Aide-mémoire', NULL, 17),
(252, 13, 'lifetimes-felicitations', 'Félicitations !', NULL, 18);
INSERT INTO cour_block (id, chapitre_id, heading, content, block_type, sort_order) VALUES
(10686, 235, NULL, $BLK10686$Comprendre et Maîtriser les Durées de Vie$BLK10686$, 'text', 1),
(10687, 235, NULL, $BLK10687$Le Concept le Plus Important de Rust$BLK10687$, 'text', 2),
(10688, 235, 'Objectifs du cours', $BLK10688$À la fin de ce cours, tu sauras : 

- Comprendre ce qu'est une lifetime 

- Annoter les lifetimes correctement 

- Utiliser lifetimes dans les structs 

- Maîtriser lifetime elision 

- Résoudre les erreurs du borrow checker$BLK10688$, 'text', 3),
(10689, 236, NULL, $BLK10689$Une **lifetime** (durée de vie) est la portée pendant laquelle une référence est valide. C'est le mécanisme qui permet à Rust de garantir la sécurité mémoire sans garbage collector.$BLK10689$, 'text', 1),
(10690, 236, NULL, $BLK10690$**`// Problème sans lifetimes (hypothétique) { let r; { let x = 5; r = &x;  // x va être détruit ! } println!("{}", r);  //`** I **`Dangling pointer ! } // Rust empêche ça avec les lifetimes fn main() { let r; { let x = 5; r = &x;  //`** I **`Erreur de compilation ! // `x` does not live long enough } // println!("{}", r); } // Version correcte fn main() { let x = 5; let r = &x; println!("{}", r);  //`** I **`OK }`**$BLK10690$, 'text', 2),
(10691, 236, NULL, $BLK10691$I **Lifetime = portée** : Une référence ne peut jamais vivre plus longtemps que la donnée qu'elle référence. C'est le borrow checker qui vérifie ça !$BLK10691$, 'text', 3),
(10692, 237, NULL, $BLK10692$```
// Annotation de lifetime avec '
// 'a se lit "lifetime a"
fn exemple<'a>(x: &'a str) -> &'a str {
    x
}
```$BLK10692$, 'code', 1),
(10693, 237, NULL, $BLK10693$```
// Plusieurs paramètres
fn plus_long<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}
// Convention de nommage
// 'a, 'b, 'c sont les plus courants
// Mais tu peux utiliser n'importe quel nom: 'lifetime, 'input, etc.
```$BLK10693$, 'code', 2),
(10694, 238, NULL, $BLK10694$```
// Fonction qui retourne une référence
fn premier_mot<'a>(texte: &'a str) -> &'a str {
    let bytes = texte.as_bytes();
    for (i, &byte) in bytes.iter().enumerate() {
        if byte == b' ' {
            return &texte[0..i];
        }
    }
    texte
}
// Utilisation
fn main() {
    let phrase = String::from("hello world");
    let mot = premier_mot(&phrase);
    println!("{}", mot);  // "hello"
}
// Pourquoi 'a ?
// Le compilateur sait que le retour vit aussi longtemps que l'input
```$BLK10694$, 'code', 1),
(10695, 239, NULL, $BLK10695$**`// Deux lifetimes différentes fn comparer<'a, 'b>(x: &'a str, y: &'b str) -> bool { x.len() > y.len() } // Le retour peut dépendre d'une seule fn choisir<'a, 'b>(x: &'a str, y: &'b str, premier: bool) -> &'a str { if premier { x  //`** I **`OK, retourne 'a } else { // y  //`** I **`ERREUR ! y a lifetime 'b, pas 'a x  // Doit retourner x } } // Lifetime commune fn plus_long<'a>(x: &'a str, y: &'a str) -> &'a str { if x.len() > y.len() { x } else { y } }`**$BLK10695$, 'text', 1),
(10696, 239, NULL, $BLK10696$```
// Signifie: le retour vit aussi longtemps que
// la plus petite des deux lifetimes
```$BLK10696$, 'code', 2),
(10697, 239, NULL, $BLK10697$II **Règle d'or :** Si une fonction retourne une référence, elle doit venir d'un des paramètres (ou être 'static).$BLK10697$, 'text', 3),
(10698, 240, NULL, $BLK10698$Le compilateur peut souvent **inférer** les lifetimes automatiquement grâce à des règles d'élision. C'est pourquoi tu n'as pas toujours besoin de les écrire !$BLK10698$, 'text', 1),
(10699, 241, NULL, $BLK10699$```
// Ces deux fonctions sont équivalentes:
```$BLK10699$, 'code', 1),
(10700, 241, NULL, $BLK10700$```
// Sans annotation (élision)
fn premier(x: &str) -> &str {
    &x[0..1]
```$BLK10700$, 'code', 2),
(10701, 241, NULL, $BLK10701$```
}
```$BLK10701$, 'code', 3),
(10702, 241, NULL, $BLK10702$```
// Avec annotation explicite
fn premier<'a>(x: &'a str) -> &'a str {
    &x[0..1]
```$BLK10702$, 'code', 4),
(10703, 241, NULL, $BLK10703$```
}
```$BLK10703$, 'code', 5),
(10704, 241, NULL, $BLK10704$```
// Règles d'élision:
// 1. Chaque paramètre référence a sa propre lifetime
fn f(x: &str, y: &str)  // devient
fn f<'a, 'b>(x: &'a str, y: &'b str)
```$BLK10704$, 'code', 6),
(10705, 241, NULL, $BLK10705$```
// 2. Si un seul param référence, sa lifetime = lifetime du retour
fn f(x: &str) -> &str  // devient
fn f<'a>(x: &'a str) -> &'a str
```$BLK10705$, 'code', 7),
(10706, 241, NULL, $BLK10706$```
// 3. Si &self ou &mut self, sa lifetime = lifetime du retour
impl MonType {
    fn get_data(&self) -> &str  // devient
    fn get_data<'a>(&'a self) -> &'a str
}
```$BLK10706$, 'code', 8),
(10707, 242, NULL, $BLK10707$**`//`** I **`PAS BESOIN d'annoter fn premiere_ligne(texte: &str) -> &str { texte.lines().next().unwrap_or("") } //`** I **`BESOIN d'annoter (plusieurs inputs) fn plus_long<'a>(x: &'a str, y: &'a str) -> &'a str { if x.len() > y.len() { x } else { y } } //`** I **`BESOIN d'annoter (struct avec référence) struct Extrait<'a> { contenu: &'a str, }`**$BLK10707$, 'text', 1),
(10708, 242, NULL, $BLK10708$**`//`** I **`PAS BESOIN (méthode avec &self) impl<'a> Extrait<'a> { fn contenu(&self) -> &str { self.contenu  // lifetime inférée de &self } }`**$BLK10708$, 'text', 2),
(10709, 242, NULL, $BLK10709$I **Conseil :** Commence sans annotations. Si le compilateur se plaint, ajoute-les. Le message d'erreur te dira souvent quoi faire !$BLK10709$, 'text', 3),
(10710, 243, NULL, $BLK10710$```
// Struct qui contient une référence
struct Article<'a> {
    titre: &'a str,
    contenu: &'a str,
}
impl<'a> Article<'a> {
    fn new(titre: &'a str, contenu: &'a str) -> Self {
        Article { titre, contenu }
    }
    fn apercu(&self) -> &str {
        &self.contenu[0..100.min(self.contenu.len())]
    }
}
// Utilisation
fn main() {
    let titre = String::from("Rust Lifetimes");
    let contenu = String::from("Les lifetimes sont...");
    let article = Article::new(&titre, &contenu);
    println!("{}", article.titre);
    // titre et contenu doivent vivre plus longtemps qu'article !
}
```$BLK10710$, 'code', 1),
(10711, 244, NULL, $BLK10711$```
struct Parser<'a> {
    texte: &'a str,
    position: usize,
}
impl<'a> Parser<'a> {
    fn new(texte: &'a str) -> Self {
        Parser { texte, position: 0 }
    }
    // Lifetime du retour = lifetime de &self
    fn reste(&self) -> &'a str {
        &self.texte[self.position..]
    }
    // Plusieurs lifetimes
    fn avec_prefixe<'b>(&'a self, prefixe: &'b str) -> String {
        format!("{}{}", prefixe, self.reste())
    }
}
```$BLK10711$, 'code', 1),
(10712, 245, NULL, $BLK10712$`'static` est une lifetime spéciale qui dure pendant **toute l'exécution du programme** .$BLK10712$, 'text', 1),
(10713, 245, NULL, $BLK10713$```
// Références 'static
let s: &'static str = "Hello world";
```$BLK10713$, 'code', 2),
(10714, 245, NULL, $BLK10714$```
// Littéraux de chaîne sont toujours 'static
const MESSAGE: &'static str = "Constant";
```$BLK10714$, 'code', 3),
(10715, 245, NULL, $BLK10715$```
// Owned data n'a pas besoin de 'static
let s = String::from("owned");  // Pas de lifetime !
```$BLK10715$, 'code', 4),
(10716, 245, NULL, $BLK10716$```
// Trait bound 'static
fn process<T: 'static>(value: T) {
    // T doit être owned ou contenir seulement 'static refs
}
```$BLK10716$, 'code', 5),
(10717, 245, NULL, $BLK10717$**`// Exemples valides process(42);                          //`** I **`i32 is 'static process(String::from("hello"));       //`** I **`String is 'static process("literal");                   //`** I **`&'static str`**$BLK10717$, 'text', 6),
(10718, 245, NULL, $BLK10718$**`// Exemple invalide let s = String::from("temp"); // process(&s);  //`** I **`&s n'est pas 'static`**$BLK10718$, 'text', 7),
(10719, 245, NULL, $BLK10719$II **'static ne veut PAS dire immortel !** Un `String` peut être `T: 'static` mais être drop quand même. Ça veut juste dire "pas de références non-'static".$BLK10719$, 'text', 8),
(10720, 246, NULL, $BLK10720$```
// 1. Lifetime bounds
struct Conteneur<'a, T: 'a> {
    item: &'a T,
}
// 2. Multiple struct lifetimes
struct Context<'s, 'c> {
    source: &'s str,
    config: &'c Config,
}
// 3. Lifetime in trait
trait Parser<'a> {
    fn parse(&self, input: &'a str) -> Result<&'a str, Error>;
}
```$BLK10720$, 'code', 1),
(10721, 246, NULL, $BLK10721$```
// 4. Higher-rank trait bounds (HRTB)
fn apply<F>(f: F)
where
    F: for<'a> Fn(&'a str) -> &'a str
```$BLK10721$, 'code', 2),
(10722, 246, NULL, $BLK10722$```
{
    let s = String::from("hello");
    println!("{}", f(&s));
}
```$BLK10722$, 'code', 3),
(10723, 247, NULL, $BLK10723$**Erreur : "does not live long enough"**$BLK10723$, 'text', 1),
(10724, 247, NULL, $BLK10724$**`//`** I **`Problème fn dangling_ref() -> &String { let s = String::from("hello"); &s  // s est détruit ici ! } //`** I **`Solutions // 1. Retourner owned fn owned() -> String { String::from("hello") }`**$BLK10724$, 'text', 2),
(10725, 247, NULL, $BLK10725$```
// 2. Utiliser 'static
fn static_ref() -> &'static str {
    "hello"
}
// 3. Prendre un paramètre
fn borrow_from_param(s: &String) -> &String {
    s
}
```$BLK10725$, 'code', 3),
(10726, 248, NULL, $BLK10726$**`//`** I **`Problème fn choisir<'a, 'b>(x: &'a str, y: &'b str) -> &'a str { if true { x } else { y }  // y n'a pas lifetime 'a ! } //`** I **`Solution: même lifetime fn choisir<'a>(x: &'a str, y: &'a str) -> &'a str { if true { x } else { y } }`**$BLK10726$, 'text', 1),
(10727, 249, NULL, $BLK10727$```
// Ajoute les annotations nécessaires
fn plus_court(x: &str, y: &str) -> &str {
    if x.len() < y.len() { x } else { y }
}
// Solution :
fn plus_court<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() < y.len() { x } else { y }
}
```$BLK10727$, 'code', 1),
(10728, 250, NULL, $BLK10728$```
// Crée une struct Citation qui contient:
// - une référence vers le texte
// - le nom de l'auteur (référence)
// Solution :
struct Citation<'a> {
    texte: &'a str,
    auteur: &'a str,
}
impl<'a> Citation<'a> {
    fn new(texte: &'a str, auteur: &'a str) -> Self {
        Citation { texte, auteur }
    }
    fn afficher(&self) {
        println!("'{}' - {}", self.texte, self.auteur);
    }
}
```$BLK10728$, 'code', 1),
(10729, 251, NULL, $BLK10729$|**Syntaxe**|**Signification**|
|---|---|
|**`&'a T`**|Référence avec lifetime 'a|
|**`fn f<'a>`**|Fonction avec paramètre de lifetime|
|**`struct S<'a>`**|Struct avec lifetime|
|**`impl<'a> S<'a>`**|Implémentation pour S avec lifetime|
|**`T: 'a`**|T contient seulement refs qui vivent au moins 'a|
|**`'static`**|Lifetime de toute l'exécution|$BLK10729$, 'table', 1),
(10730, 251, NULL, $BLK10730$- **Lifetime = portée** d'une référence$BLK10730$, 'list', 2),
(10731, 251, NULL, $BLK10731$- **Borrow checker** vérifie les lifetimes à la compilation$BLK10731$, 'list', 3),
(10732, 251, NULL, $BLK10732$- **Élision** permet d'éviter les annotations dans 90% des cas$BLK10732$, 'list', 4),
(10733, 251, NULL, $BLK10733$- **'a, 'b, 'c** sont juste des noms de variables de lifetime$BLK10733$, 'list', 5),
(10734, 251, NULL, $BLK10734$- **'static** = vit pendant toute l'exécution$BLK10734$, 'list', 6),
(10735, 251, NULL, $BLK10735$- **Références** ne peuvent jamais outlive leur donnée$BLK10735$, 'list', 7),
(10736, 252, NULL, $BLK10736$Tu as dompté les lifetimes !$BLK10736$, 'text', 1),
(10737, 252, NULL, $BLK10737$C'est le concept le plus difficile de Rust, mais aussi le plus puissant. Maintenant tu comprends vraiment la magie du borrow checker !$BLK10737$, 'text', 2),
(10738, 252, NULL, $BLK10738$I **Tu es un vrai Rustacean !** I$BLK10738$, 'text', 3);

-- box-dynamiques.md (cour_id=14)
INSERT INTO chapitre (id, cour_id, slug, title, lead, sort_order) VALUES
(253, 14, 'box-dynamiques-encapsulation-de-boxdyn-trait-dans-des-structs-en-rust', 'Encapsulation de Box<dyn Trait> dans des Structs en Rust', NULL, 1),
(254, 14, 'box-dynamiques-11-quest-ce-quun-box', '1.1 Qu''est-ce qu''un Box ?', NULL, 2),
(255, 14, 'box-dynamiques-12-quest-ce-quun-trait-object', '1.2 Qu''est-ce qu''un Trait Object ?', NULL, 3),
(256, 14, 'box-dynamiques-13-pourquoi-boxdyn-trait', '1.3 Pourquoi Box<dyn Trait> ?', NULL, 4),
(257, 14, 'box-dynamiques-21-anatomie-dun-trait-object', '2.1 Anatomie d''un Trait Object', NULL, 5),
(258, 14, 'box-dynamiques-22-exemple-complet', '2.2 Exemple Complet', NULL, 6),
(259, 14, 'box-dynamiques-23-limitations-des-trait-objects', '2.3 Limitations des Trait Objects', NULL, 7),
(260, 14, 'box-dynamiques-31-pattern-basique', '3.1 Pattern Basique', NULL, 8),
(261, 14, 'box-dynamiques-32-collections-de-trait-objects', '3.2 Collections de Trait Objects', NULL, 9),
(262, 14, 'box-dynamiques-33-pattern-builder-avec-trait-objects', '3.3 Pattern Builder avec Trait Objects', NULL, 10),
(263, 14, 'box-dynamiques-41-state-pattern', '4.1 State Pattern', NULL, 11),
(264, 14, 'box-dynamiques-42-command-pattern', '4.2 Command Pattern', NULL, 12),
(265, 14, 'box-dynamiques-43-observer-pattern', '4.3 Observer Pattern', NULL, 13),
(266, 14, 'box-dynamiques-les-trait-objects-ont-un-cout-en-performance', 'Les trait objects ont un coût en performance :', NULL, 14),
(267, 14, 'box-dynamiques-52-alternatives-aux-trait-objects', '5.2 Alternatives aux Trait Objects', NULL, 15),
(268, 14, 'box-dynamiques-53-optimisations', '5.3 Optimisations', NULL, 16),
(269, 14, 'box-dynamiques-54-lifetime-et-ownership', '5.4 Lifetime et Ownership', NULL, 17),
(270, 14, 'box-dynamiques-exercice-1-systeme-de-fichiers-virtuel', 'Exercice 1: Système de fichiers virtuel', NULL, 18),
(271, 14, 'box-dynamiques-exercice-2-calculatrice-avec-extensions', 'Exercice 2: Calculatrice avec extensions', NULL, 19),
(272, 14, 'box-dynamiques-exercice-3-event-system', 'Exercice 3: Event System', NULL, 20),
(273, 14, 'box-dynamiques-solution-exercice-1', 'Solution Exercice 1', NULL, 21),
(274, 14, 'box-dynamiques-conclusion', 'Conclusion', NULL, 22),
(275, 14, 'box-dynamiques-points-cles-a-retenir', 'Points Clés à Retenir:', NULL, 23),
(276, 14, 'box-dynamiques-ressources-complementaires', 'Ressources Complémentaires:', NULL, 24);
INSERT INTO cour_block (id, chapitre_id, heading, content, block_type, sort_order) VALUES
(10739, 253, NULL, $BLK10739$Guide pratique pour la programmation orientée objet en Rust$BLK10739$, 'text', 1),
(10740, 254, NULL, $BLK10740$Un **Box** est un pointeur intelligent (smart pointer) qui alloue de la mémoire sur le tas (heap). Il permet de stocker des données dont la taille n'est pas connue à la compilation ou qui sont trop volumineuses pour la pile (stack).$BLK10740$, 'text', 1),
(10741, 254, NULL, $BLK10741$```
// Exemple simple de Box
let x = Box::new(5);
println!("Valeur dans le box: {}", x);
// Box pour les types de taille inconnue
struct Node {
    value: i32,
    next: Option<Box<Node>>,
}
```$BLK10741$, 'code', 2),
(10742, 255, NULL, $BLK10742$Un **trait object** (objet trait) est une référence dynamique à un type qui implémente un trait particulier. On utilise le mot-clé **dyn** pour indiquer qu'il s'agit d'un dispatch dynamique.$BLK10742$, 'text', 1),
(10743, 255, NULL, $BLK10743$```
// Définition d'un trait
trait Drawable {
    fn draw(&self);
}
// Utilisation d'un trait object
fn render(item: &dyn Drawable) {
    item.draw();
}
```$BLK10743$, 'code', 2),
(10744, 256, NULL, $BLK10744$La combinaison **Box<dyn Trait>** permet de :$BLK10744$, 'text', 1),
(10745, 256, NULL, $BLK10745$- `Stocker différents types implémentant le même trait`$BLK10745$, 'list', 2),
(10746, 256, NULL, $BLK10746$- `Avoir des collections hétérogènes de types`$BLK10746$, 'list', 3),
(10747, 256, NULL, $BLK10747$- `Implémenter le polymorphisme à l'exécution`$BLK10747$, 'list', 4),
(10748, 256, NULL, $BLK10748$- `Cacher l'implémentation concrète (abstraction)`$BLK10748$, 'list', 5),
(10749, 256, NULL, $BLK10749$- `Gérer des types récursifs`$BLK10749$, 'list', 6),
(10750, 257, NULL, $BLK10750$Un trait object est composé de deux pointeurs :$BLK10750$, 'text', 1),
(10751, 257, NULL, $BLK10751$```
1. Un pointeur vers les données (l'instance concrète)
```$BLK10751$, 'code', 2),
(10752, 257, NULL, $BLK10752$```
2. Un pointeur vers la vtable (table de méthodes virtuelles)
```$BLK10752$, 'code', 3),
(10753, 258, NULL, $BLK10753$```
trait Animal {
    fn make_sound(&self) -> String;
    fn name(&self) -> &str;
}
struct Dog {
    name: String,
}
impl Animal for Dog {
    fn make_sound(&self) -> String {
        "Woof!".to_string()
    }
    fn name(&self) -> &str {
        &self.name
    }
}
struct Cat {
    name: String,
}
impl Animal for Cat {
    fn make_sound(&self) -> String {
        "Meow!".to_string()
    }
    fn name(&self) -> &str {
        &self.name
    }
}
// Utilisation
fn main() {
    let dog: Box<dyn Animal> = Box::new(Dog {
        name: "Rex".to_string(),
    });
    let cat: Box<dyn Animal> = Box::new(Cat {
        name: "Whiskers".to_string(),
    });
    println!("{} says {}", dog.name(), dog.make_sound());
    println!("{} says {}", cat.name(), cat.make_sound());
}
```$BLK10753$, 'code', 1),
(10754, 259, NULL, $BLK10754$Tous les traits ne peuvent pas être utilisés comme trait objects. Un trait doit être **object-safe** :$BLK10754$, 'text', 1),
(10755, 259, NULL, $BLK10755$`• Pas de méthodes génériques • Pas de méthodes retournant Self • Pas de constantes associées • Pas de types associés avec des bornes //` I `Ce trait n'est PAS object-safe trait NotObjectSafe { fn generic_method<T>(&self, x: T);  // Méthode générique fn clone_self(&self) -> Self;        // Retourne Self }`$BLK10755$, 'text', 2),
(10756, 259, NULL, $BLK10756$`//` I `Ce trait EST object-safe trait ObjectSafe { fn method(&self) -> String; fn another(&mut self, x: i32); }`$BLK10756$, 'text', 3),
(10757, 260, NULL, $BLK10757$Le pattern le plus simple consiste à stocker un Box<dyn Trait> comme champ d'une struct :$BLK10757$, 'text', 1),
(10758, 260, NULL, $BLK10758$```
trait Strategy {
    fn execute(&self, data: &[i32]) -> i32;
}
struct Context {
    strategy: Box<dyn Strategy>,
}
impl Context {
    fn new(strategy: Box<dyn Strategy>) -> Self {
        Context { strategy }
    }
    fn execute_strategy(&self, data: &[i32]) -> i32 {
        self.strategy.execute(data)
    }
    // Permet de changer la stratégie dynamiquement
    fn set_strategy(&mut self, strategy: Box<dyn Strategy>) {
        self.strategy = strategy;
    }
}
// Implémentations concrètes
struct SumStrategy;
impl Strategy for SumStrategy {
    fn execute(&self, data: &[i32]) -> i32 {
        data.iter().sum()
    }
}
struct MaxStrategy;
impl Strategy for MaxStrategy {
    fn execute(&self, data: &[i32]) -> i32 {
        *data.iter().max().unwrap_or(&0)
    }
}
// Utilisation
fn main() {
    let data = vec![1, 5, 3, 9, 2];
    let mut context = Context::new(Box::new(SumStrategy));
    println!("Sum: {}", context.execute_strategy(&data));
    context.set_strategy(Box::new(MaxStrategy));
    println!("Max: {}", context.execute_strategy(&data));
}
```$BLK10758$, 'code', 2),
(10759, 261, NULL, $BLK10759$On peut créer des collections hétérogènes avec Vec<Box<dyn Trait>> :$BLK10759$, 'text', 1),
(10760, 261, NULL, $BLK10760$```
trait Component {
    fn render(&self) -> String;
    fn update(&mut self);
}
struct Application {
    components: Vec<Box<dyn Component>>,
}
impl Application {
    fn new() -> Self {
        Application {
            components: Vec::new(),
        }
```$BLK10760$, 'code', 2),
(10761, 261, NULL, $BLK10761$```
    }
    fn add_component(&mut self, component: Box<dyn Component>) {
        self.components.push(component);
    }
    fn render_all(&self) -> String {
        self.components
            .iter()
            .map(|c| c.render())
            .collect::<Vec<_>>()
            .join("\n")
    }
    fn update_all(&mut self) {
        for component in &mut self.components {
            component.update();
        }
    }
}
// Exemples de composants
struct Button {
    label: String,
    clicks: u32,
}
impl Component for Button {
    fn render(&self) -> String {
        format!("[Button: {} (clicks: {})]", self.label, self.clicks)
    }
    fn update(&mut self) {
        self.clicks += 1;
    }
}
struct Label {
    text: String,
}
impl Component for Label {
    fn render(&self) -> String {
        format!("[Label: {}]", self.text)
    }
    fn update(&mut self) {
        // Labels ne changent pas
    }
}
```$BLK10761$, 'code', 3),
(10762, 262, NULL, $BLK10762$```
trait Plugin {
    fn initialize(&mut self);
    fn execute(&self, input: &str) -> String;
}
struct Engine {
    plugins: Vec<Box<dyn Plugin>>,
}
impl Engine {
    fn new() -> Self {
        Engine {
            plugins: Vec::new(),
        }
    }
    fn add_plugin(mut self, mut plugin: Box<dyn Plugin>) -> Self {
        plugin.initialize();
        self.plugins.push(plugin);
        self
    }
    fn process(&self, input: &str) -> String {
        let mut result = input.to_string();
        for plugin in &self.plugins {
            result = plugin.execute(&result);
        }
        result
    }
}
// Plugins concrets
struct UpperCasePlugin;
impl Plugin for UpperCasePlugin {
    fn initialize(&mut self) {
        println!("UpperCase plugin initialized");
    }
    fn execute(&self, input: &str) -> String {
        input.to_uppercase()
    }
}
struct ReversePlugin;
impl Plugin for ReversePlugin {
    fn initialize(&mut self) {
        println!("Reverse plugin initialized");
    }
    fn execute(&self, input: &str) -> String {
        input.chars().rev().collect()
    }
}
// Utilisation
fn main() {
    let engine = Engine::new()
        .add_plugin(Box::new(UpperCasePlugin))
        .add_plugin(Box::new(ReversePlugin));
    let result = engine.process("hello");
    println!("Result: {}", result); // OLLEH
}
```$BLK10762$, 'code', 1),
(10763, 263, NULL, $BLK10763$Le pattern State permet à un objet de changer son comportement quand son état interne change :$BLK10763$, 'text', 1),
(10764, 263, NULL, $BLK10764$```
trait State {
    fn handle(self: Box<Self>) -> Box<dyn State>;
    fn description(&self) -> &str;
}
struct DraftState;
struct PendingReviewState;
struct PublishedState;
impl State for DraftState {
    fn handle(self: Box<Self>) -> Box<dyn State> {
        Box::new(PendingReviewState)
    }
    fn description(&self) -> &str {
        "Draft"
    }
}
impl State for PendingReviewState {
    fn handle(self: Box<Self>) -> Box<dyn State> {
        Box::new(PublishedState)
    }
    fn description(&self) -> &str {
        "Pending Review"
    }
}
impl State for PublishedState {
    fn handle(self: Box<Self>) -> Box<dyn State> {
        self // Reste publié
    }
    fn description(&self) -> &str {
        "Published"
    }
}
struct Post {
    state: Box<dyn State>,
    content: String,
}
impl Post {
    fn new(content: String) -> Self {
        Post {
            state: Box::new(DraftState),
            content,
        }
    }
    fn request_review(&mut self) {
        self.state = self.state.handle();
    }
    fn status(&self) -> &str {
        self.state.description()
    }
}
// Utilisation
fn main() {
    let mut post = Post::new("My article".to_string());
    println!("Status: {}", post.status()); // Draft
    post.request_review();
    println!("Status: {}", post.status()); // Pending Review
    post.request_review();
```$BLK10764$, 'code', 2),
(10765, 263, NULL, $BLK10765$```
    println!("Status: {}", post.status()); // Published
}
```$BLK10765$, 'code', 3),
(10766, 264, NULL, $BLK10766$```
trait Command {
    fn execute(&mut self);
    fn undo(&mut self);
}
struct TextEditor {
    content: String,
}
impl TextEditor {
    fn new() -> Self {
        TextEditor {
            content: String::new(),
        }
    }
    fn append(&mut self, text: &str) {
        self.content.push_str(text);
    }
    fn delete(&mut self, count: usize) {
        let new_len = self.content.len().saturating_sub(count);
        self.content.truncate(new_len);
    }
    fn content(&self) -> &str {
        &self.content
    }
}
struct AppendCommand {
    editor: *mut TextEditor,
    text: String,
}
impl AppendCommand {
    fn new(editor: &mut TextEditor, text: String) -> Self {
        AppendCommand {
            editor: editor as *mut TextEditor,
            text,
        }
    }
}
impl Command for AppendCommand {
    fn execute(&mut self) {
        unsafe {
            (*self.editor).append(&self.text);
        }
    }
    fn undo(&mut self) {
        unsafe {
            (*self.editor).delete(self.text.len());
        }
    }
}
struct CommandManager {
    history: Vec<Box<dyn Command>>,
    current: usize,
}
impl CommandManager {
    fn new() -> Self {
        CommandManager {
            history: Vec::new(),
            current: 0,
        }
    }
    fn execute(&mut self, mut command: Box<dyn Command>) {
        command.execute();
        self.history.truncate(self.current);
        self.history.push(command);
        self.current += 1;
    }
```$BLK10766$, 'code', 1),
(10767, 264, NULL, $BLK10767$```
    fn undo(&mut self) {
        if self.current > 0 {
            self.current -= 1;
            self.history[self.current].undo();
        }
    }
    fn redo(&mut self) {
        if self.current < self.history.len() {
            self.history[self.current].execute();
            self.current += 1;
        }
    }
}
```$BLK10767$, 'code', 2),
(10768, 265, NULL, $BLK10768$```
trait Observer {
    fn update(&mut self, event: &str);
}
struct Subject {
    observers: Vec<Box<dyn Observer>>,
    state: String,
}
impl Subject {
    fn new() -> Self {
        Subject {
            observers: Vec::new(),
            state: String::new(),
        }
    }
    fn attach(&mut self, observer: Box<dyn Observer>) {
        self.observers.push(observer);
    }
    fn set_state(&mut self, state: String) {
        self.state = state.clone();
        self.notify(&state);
    }
    fn notify(&mut self, event: &str) {
        for observer in &mut self.observers {
            observer.update(event);
        }
    }
}
struct Logger {
    name: String,
}
impl Observer for Logger {
    fn update(&mut self, event: &str) {
        println!("[{}] Logged: {}", self.name, event);
    }
}
struct EmailNotifier {
    email: String,
}
impl Observer for EmailNotifier {
    fn update(&mut self, event: &str) {
        println!("Email to {}: {}", self.email, event);
    }
}
// Utilisation
fn main() {
    let mut subject = Subject::new();
    subject.attach(Box::new(Logger {
        name: "FileLogger".to_string(),
    }));
    subject.attach(Box::new(EmailNotifier {
        email: "admin@example.com".to_string(),
    }));
    subject.set_state("New event occurred".to_string());
}
```$BLK10768$, 'code', 1),
(10769, 266, NULL, $BLK10769$- `Indirection via pointeur (accès mémoire supplémentaire)`$BLK10769$, 'list', 1),
(10770, 266, NULL, $BLK10770$- `Dispatch dynamique via vtable (impossible d'inliner)`$BLK10770$, 'list', 2),
(10771, 266, NULL, $BLK10771$- `Allocation sur le tas avec Box`$BLK10771$, 'list', 3),
(10772, 266, NULL, $BLK10772$- `Pas d'optimisations du compilateur (monomorphisation)`$BLK10772$, 'list', 4),
(10773, 267, NULL, $BLK10773$```
// 1. Générics (dispatch statique) - PLUS RAPIDE
fn process_generic<T: Drawable>(item: &T) {
    item.draw();
}
// 2. Enum pour types connus - ENCORE PLUS RAPIDE
enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}
impl Shape {
    fn draw(&self) {
        match self {
            Shape::Circle(c) => c.draw(),
            Shape::Rectangle(r) => r.draw(),
        }
    }
}
// 3. Trait Objects - FLEXIBLE mais PLUS LENT
fn process_dynamic(item: &dyn Drawable) {
    item.draw();
}
```$BLK10773$, 'code', 1),
(10774, 268, NULL, $BLK10774$```
// Éviter les allocations inutiles
struct Container {
    // Au lieu de: items: Vec<Box<dyn Item>>
    // Considérer: items prépooled ou arena allocation
    items: Vec<Box<dyn Item>>,
}
// Utiliser Rc/Arc pour partager sans copier
use std::rc::Rc;
struct Shared {
    data: Rc<dyn Data>,
}
// Pour le multithreading
use std::sync::Arc;
struct ThreadSafe {
    data: Arc<dyn Send + Sync + Data>,
}
```$BLK10774$, 'code', 1),
(10775, 269, NULL, $BLK10775$```
// Box possède ses données
struct Owner {
    item: Box<dyn Trait>,  // Owner possède l'objet
}
// Référence ne possède pas
struct Borrower<'a> {
    item: &'a dyn Trait,  // Borrower emprunte seulement
}
```$BLK10775$, 'code', 1),
(10776, 269, NULL, $BLK10776$```
// Exemple complet
trait Processor {
    fn process(&self, data: &str) -> String;
}
struct Pipeline<'a> {
    processors: Vec<&'a dyn Processor>,  // Emprunte
}
impl<'a> Pipeline<'a> {
    fn new() -> Self {
        Pipeline {
            processors: Vec::new(),
        }
    }
    fn add(&mut self, processor: &'a dyn Processor) {
        self.processors.push(processor);
    }
    fn execute(&self, data: &str) -> String {
        let mut result = data.to_string();
        for processor in &self.processors {
            result = processor.process(&result);
        }
        result
    }
}
```$BLK10776$, 'code', 2),
(10777, 270, NULL, $BLK10777$Créez un système de fichiers virtuel avec des fichiers et des dossiers :$BLK10777$, 'text', 1),
(10778, 270, NULL, $BLK10778$```
trait FileSystemItem {
    fn name(&self) -> &str;
    fn size(&self) -> u64;
    fn print(&self, indent: usize);
}
struct File {
    name: String,
    size: u64,
}
struct Directory {
    name: String,
    items: Vec<Box<dyn FileSystemItem>>,
}
// À implémenter:
// - impl FileSystemItem for File
// - impl FileSystemItem for Directory
// - Méthode pour ajouter des items au Directory
// - Calcul récursif de la taille du Directory
```$BLK10778$, 'code', 2),
(10779, 271, NULL, $BLK10779$```
trait Operation {
    fn execute(&self, a: f64, b: f64) -> f64;
    fn symbol(&self) -> &str;
}
struct Calculator {
    operations: Vec<Box<dyn Operation>>,
}
// À implémenter:
// - Opérations: Add, Subtract, Multiply, Divide, Power
// - Méthode Calculator::register_operation
// - Méthode Calculator::calculate(a, symbol, b)
// - Gestion des erreurs (division par zéro, etc.)
```$BLK10779$, 'code', 1),
(10780, 272, NULL, $BLK10780$```
trait EventHandler {
    fn handle(&mut self, event_type: &str, data: &str);
    fn can_handle(&self, event_type: &str) -> bool;
}
struct EventBus {
    handlers: Vec<Box<dyn EventHandler>>,
}
// À implémenter:
// - EventBus::register_handler
// - EventBus::emit(event_type, data)
// - Plusieurs handlers concrets
// - Système de priorité pour les handlers
```$BLK10780$, 'code', 1),
(10781, 273, NULL, $BLK10781$`impl FileSystemItem for File { fn name(&self) -> &str { &self.name } fn size(&self) -> u64 { self.size } fn print(&self, indent: usize) { println!("{}- {} ({} bytes)", " ".repeat(indent), self.name, self.size); } } impl Directory { fn new(name: String) -> Self { Directory { name, items: Vec::new(), } } fn add(&mut self, item: Box<dyn FileSystemItem>) { self.items.push(item); } } impl FileSystemItem for Directory { fn name(&self) -> &str { &self.name } fn size(&self) -> u64 { self.items.iter().map(|item| item.size()).sum() } fn print(&self, indent: usize) { println!("{}` I `{} ({} bytes total)", " ".repeat(indent), self.name, self.size()); for item in &self.items { item.print(indent + 2); } } } fn main() { let mut root = Directory::new("root".to_string()); root.add(Box::new(File { name: "file1.txt".to_string(), size: 100, })); let mut subdir = Directory::new("documents".to_string()); subdir.add(Box::new(File { name: "doc.pdf".to_string(), size: 500, })); root.add(Box::new(subdir)); root.print(0); }`$BLK10781$, 'text', 1),
(10782, 274, NULL, $BLK10782$L'encapsulation de Box<dyn Trait> dans des structs est une technique puissante en Rust qui permet d'implémenter des patterns de programmation orientée objet tout en conservant les garanties de sécurité mémoire de Rust.$BLK10782$, 'text', 1),
(10783, 275, NULL, $BLK10783$- `Box<dyn Trait> combine allocation heap et polymorphisme`$BLK10783$, 'list', 1),
(10784, 275, NULL, $BLK10784$- `Permet de créer des collections hétérogènes`$BLK10784$, 'list', 2),
(10785, 275, NULL, $BLK10785$- `Essentiel pour les patterns OOP (Strategy, Observer, Command)`$BLK10785$, 'list', 3),
(10786, 275, NULL, $BLK10786$- `A un coût en performance (dispatch dynamique)`$BLK10786$, 'list', 4),
(10787, 275, NULL, $BLK10787$- `Le trait doit être object-safe`$BLK10787$, 'list', 5),
(10788, 275, NULL, $BLK10788$- `Alternatives: generics (plus rapide) ou enums (si types connus)`$BLK10788$, 'list', 6),
(10789, 276, NULL, $BLK10789$- `The Rust Programming Language Book (Chapitre 17)`$BLK10789$, 'list', 1),
(10790, 276, NULL, $BLK10790$- `Rust Design Patterns (rust-unofficial/patterns)`$BLK10790$, 'list', 2),
(10791, 276, NULL, $BLK10791$- `Trait Objects - Rust Reference`$BLK10791$, 'list', 3),
(10792, 276, NULL, $BLK10792$- `Object Safety - Rust RFC`$BLK10792$, 'list', 4);

-- smart-pointers.md (cour_id=15)
INSERT INTO chapitre (id, cour_id, slug, title, lead, sort_order) VALUES
(277, 15, 'smart-pointers-1-boxt-allocation-tas', '1. Box<T> — allocation tas', NULL, 1),
(278, 15, 'smart-pointers-2-rct-reference-comptee', '2. Rc<T> — référence comptée', NULL, 2),
(279, 15, 'smart-pointers-3-arct-reference-comptee-atomique', '3. Arc<T> — référence comptée atomique', NULL, 3),
(280, 15, 'smart-pointers-4-refcellt-mutabilite-interieure', '4. RefCell<T> — mutabilité intérieure', NULL, 4),
(281, 15, 'smart-pointers-5-weakt-reference-faible', '5. Weak<T> — référence faible', NULL, 5),
(282, 15, 'smart-pointers-6-combinaisons-courantes', '6. Combinaisons courantes', NULL, 6),
(283, 15, 'smart-pointers-7-tableau-recapitulatif', '7. Tableau récapitulatif', NULL, 7);
INSERT INTO cour_block (id, chapitre_id, heading, content, block_type, sort_order) VALUES
(10793, 277, NULL, $BLK10793$`Box<T>` alloue une valeur sur le **tas** (heap) au lieu de la pile (stack).$BLK10793$, 'text', 1),
(10794, 277, NULL, $BLK10794$```rust
// Valeur sur la pile
let x = 5;

// Valeur sur le tas
let b = Box::new(5);
println!("{b}"); // se déréférence automatiquement

// Box se libère automatiquement quand il sort de portée
```$BLK10794$, 'code', 2),
(10795, 277, NULL, $BLK10795$**Quand utiliser `Box<T>` :**$BLK10795$, 'text', 3),
(10796, 277, NULL, $BLK10796$```rust
// 1. Type récursif (taille inconnue à la compilation)
enum Liste {
    Element(i32, Box<Liste>),
    Fin,
}

let liste = Liste::Element(1,
    Box::new(Liste::Element(2,
        Box::new(Liste::Fin)
    ))
);

// 2. Grand objet à déplacer sans copier la pile
let grand = Box::new([0u8; 1_000_000]);

// 3. Trait object (dyn Trait)
trait Animal { fn parler(&self); }
struct Chien;
impl Animal for Chien { fn parler(&self) { println!("Woof"); } }

let animal: Box<dyn Animal> = Box::new(Chien);
animal.parler();
```$BLK10796$, 'code', 4),
(10797, 277, NULL, $BLK10797$---$BLK10797$, 'text', 5),
(10798, 278, NULL, $BLK10798$`Rc<T>` (*Reference Counted*) permet **plusieurs propriétaires** d'une même valeur, en thread unique.$BLK10798$, 'text', 1),
(10799, 278, NULL, $BLK10799$```rust
use std::rc::Rc;

let valeur = Rc::new(String::from("bonjour"));

let ref1 = Rc::clone(&valeur);  // incrémente le compteur
let ref2 = Rc::clone(&valeur);  // idem

println!("compteur : {}", Rc::strong_count(&valeur)); // 3
println!("{valeur}");

// La valeur est libérée quand le compteur atteint 0
drop(ref1);
println!("compteur : {}", Rc::strong_count(&valeur)); // 2
```$BLK10799$, 'code', 2),
(10800, 278, NULL, $BLK10800$> **Important :** `Rc<T>` n'est **pas thread-safe**. Pour les threads, utilisez `Arc<T>`.$BLK10800$, 'warning', 3),
(10801, 278, NULL, $BLK10801$`Rc<T>` donne un accès **en lecture seule**. Pour modifier, combinez avec `RefCell<T>`.$BLK10801$, 'text', 4),
(10802, 278, NULL, $BLK10802$---$BLK10802$, 'text', 5),
(10803, 279, NULL, $BLK10803$`Arc<T>` (*Atomically Reference Counted*) est identique à `Rc<T>` mais **thread-safe**.$BLK10803$, 'text', 1),
(10804, 279, NULL, $BLK10804$```rust
use std::sync::Arc;
use std::thread;

let valeur = Arc::new(vec![1, 2, 3]);

let mut handles = vec![];

for _ in 0..3 {
    let clone = Arc::clone(&valeur);
    let handle = thread::spawn(move || {
        println!("{:?}", clone);
    });
    handles.push(handle);
}

for h in handles { h.join().unwrap(); }
```$BLK10804$, 'code', 2),
(10805, 279, NULL, $BLK10805$> `Arc<T>` a un coût légèrement supérieur à `Rc<T>` (opérations atomiques). N'utilisez `Arc` que si vous en avez besoin.$BLK10805$, 'warning', 3),
(10806, 279, NULL, $BLK10806$---$BLK10806$, 'text', 4),
(10807, 280, NULL, $BLK10807$`RefCell<T>` déplace les vérifications du borrow checker de la **compilation** vers l'**exécution**.$BLK10807$, 'text', 1),
(10808, 280, NULL, $BLK10808$```rust
use std::cell::RefCell;

let donnees = RefCell::new(vec![1, 2, 3]);

// Emprunt immutable
let lecture = donnees.borrow();
println!("{:?}", *lecture);
drop(lecture); // libère l'emprunt

// Emprunt mutable
donnees.borrow_mut().push(4);
println!("{:?}", donnees.borrow()); // [1, 2, 3, 4]
```$BLK10808$, 'code', 2),
(10809, 280, NULL, $BLK10809$> **Attention :** Si les règles du borrow checker sont violées à l'exécution, `RefCell` **panique** (`panic!`).$BLK10809$, 'warning', 3),
(10810, 280, NULL, $BLK10810$```rust
// Ceci panique à l'exécution !
let cellule = RefCell::new(5);
let _ref1 = cellule.borrow();
let _ref2 = cellule.borrow_mut(); // PANIC : déjà emprunté immutablement
```$BLK10810$, 'code', 4),
(10811, 280, NULL, $BLK10811$**`try_borrow` et `try_borrow_mut`** pour éviter la panique :$BLK10811$, 'text', 5),
(10812, 280, NULL, $BLK10812$```rust
match donnees.try_borrow_mut() {
    Ok(mut val) => val.push(99),
    Err(_)      => println!("déjà emprunté"),
}
```$BLK10812$, 'code', 6),
(10813, 280, NULL, $BLK10813$---$BLK10813$, 'text', 7),
(10814, 281, NULL, $BLK10814$`Weak<T>` est une référence qui **ne possède pas** la valeur — ne compte pas dans le `Rc`/`Arc`.$BLK10814$, 'text', 1),
(10815, 281, NULL, $BLK10815$Utilisé pour briser les **cycles de référence** (qui empêcheraient la libération).$BLK10815$, 'text', 2),
(10816, 281, NULL, $BLK10816$```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

#[derive(Debug)]
struct Noeud {
    valeur: i32,
    parent: Option<Weak<RefCell<Noeud>>>,   // référence faible vers le parent
    enfants: Vec<Rc<RefCell<Noeud>>>,        // références fortes vers les enfants
}

let parent = Rc::new(RefCell::new(Noeud {
    valeur: 1,
    parent: None,
    enfants: vec![],
}));

let enfant = Rc::new(RefCell::new(Noeud {
    valeur: 2,
    parent: Some(Rc::downgrade(&parent)),  // Weak depuis un Rc
    enfants: vec![],
}));

// Accéder via Weak — retourne Option<Rc<T>>
if let Some(p) = enfant.borrow().parent.as_ref().and_then(|w| w.upgrade()) {
    println!("parent : {}", p.borrow().valeur);
}
```$BLK10816$, 'code', 3),
(10817, 281, NULL, $BLK10817$---$BLK10817$, 'text', 4),
(10818, 282, NULL, $BLK10818$```rust
use std::rc::Rc;
use std::cell::RefCell;

// Rc<RefCell<T>> — plusieurs propriétaires + mutation en thread unique
let partagé = Rc::new(RefCell::new(vec![1, 2, 3]));

let clone1 = Rc::clone(&partagé);
let clone2 = Rc::clone(&partagé);

clone1.borrow_mut().push(4);
clone2.borrow_mut().push(5);
println!("{:?}", partagé.borrow()); // [1, 2, 3, 4, 5]

// Arc<Mutex<T>> — plusieurs propriétaires + mutation entre threads
use std::sync::{Arc, Mutex};

let partagé = Arc::new(Mutex::new(vec![]));
let clone   = Arc::clone(&partagé);

std::thread::spawn(move || {
    partagé.lock().unwrap().push(1);
}).join().unwrap();

println!("{:?}", clone.lock().unwrap()); // [1]
```$BLK10818$, 'code', 1),
(10819, 282, NULL, $BLK10819$---$BLK10819$, 'text', 2),
(10820, 283, NULL, $BLK10820$| Type | Propriétaires | Thread-safe | Mutation | Coût |
|---|---|---|---|---|
| `T` | 1 | — | oui (`mut`) | nul |
| `Box<T>` | 1 | — | oui (`mut`) | allocation tas |
| `Rc<T>` | N | ❌ | non (seul) | compteur |
| `Arc<T>` | N | ✅ | non (seul) | compteur atomique |
| `RefCell<T>` | 1 | ❌ | oui (runtime) | vérification runtime |
| `Rc<RefCell<T>>` | N | ❌ | oui (runtime) | compteur + runtime |
| `Arc<Mutex<T>>` | N | ✅ | oui (lock) | atomique + lock |$BLK10820$, 'table', 1);

-- send-sync.md (cour_id=16)
INSERT INTO chapitre (id, cour_id, slug, title, lead, sort_order) VALUES
(284, 16, 'send-sync-rust-send-et-sync', '■ **Rust : Send et Sync**', NULL, 1),
(285, 16, 'send-sync-1-introduction-aux-marker-traits', '1. Introduction aux Marker Traits', NULL, 2),
(286, 16, 'send-sync-quest-ce-quun-marker-trait', 'Qu''est-ce qu''un Marker Trait ?', NULL, 3),
(287, 16, 'send-sync-definition', 'Définition', NULL, 4),
(288, 16, 'send-sync-exemple-de-base', 'Exemple de base', NULL, 5),
(289, 16, 'send-sync-types-send-courants', 'Types Send courants', NULL, 6),
(290, 16, 'send-sync-exemple-avec-un-type-non-send', 'Exemple avec un type non-Send', NULL, 7),
(291, 16, 'send-sync-definition', 'Définition', NULL, 8),
(292, 16, 'send-sync-regle-fondamentale', '`// Règle fondamentale`', NULL, 9),
(293, 16, 'send-sync-exemple-de-base', 'Exemple de base', NULL, 10),
(294, 16, 'send-sync-types-sync-courants', 'Types Sync courants', NULL, 11),
(295, 16, 'send-sync-4-send-vs-sync-differences-cles', '4. Send vs Sync : Différences clés', NULL, 12),
(296, 16, 'send-sync-diagramme-mental', 'Diagramme mental', NULL, 13),
(297, 16, 'send-sync-5-types-courants-et-leurs-proprietes', '5. Types courants et leurs propriétés', NULL, 14),
(298, 16, 'send-sync-6-cas-pratiques-axum-et-async', '6. Cas pratiques : Axum et async', NULL, 15),
(299, 16, 'send-sync-pourquoi-sync-est-necessaire-dans-les-traits', 'Pourquoi Sync est nécessaire dans les traits', NULL, 16),
(300, 16, 'send-sync-exemple-concret-avec-cell', 'Exemple concret avec Cell', NULL, 17),
(301, 16, 'send-sync-solution-correcte', 'Solution correcte', NULL, 18),
(302, 16, 'send-sync-erreur-1-rc-dans-un-contexte-async', 'Erreur 1 : Rc dans un contexte async', NULL, 19),
(303, 16, 'send-sync-erreur-2-cell-refcell-dans-un-trait-sync', 'Erreur 2 : Cell/RefCell dans un trait Sync', NULL, 20),
(304, 16, 'send-sync-erreur-3-oublier-sync-dans-un-trait', 'Erreur 3 : Oublier Sync dans un trait', NULL, 21),
(305, 16, 'send-sync-exemple-1-trait-de-formulaire-correct', 'Exemple 1 : Trait de formulaire correct', NULL, 22),
(306, 16, 'send-sync-exemple-2-etat-partage-dans-axum', 'Exemple 2 : État partagé dans Axum', NULL, 23),
(307, 16, 'send-sync-async-fn-handler', '`async fn handler(`', NULL, 24),
(308, 16, 'send-sync-1-toujours-ajouter-send-sync-aux-traits-publics', '1. Toujours ajouter Send + Sync aux traits publics', NULL, 25),
(309, 16, 'send-sync-2-preferer-arc-a-rc-pour-le-code-async', '2. Préférer Arc à Rc pour le code async', NULL, 26),
(310, 16, 'send-sync-3-utiliser-atomicxxx-au-lieu-de-cell-refcell', '3. Utiliser AtomicXxx au lieu de Cell/RefCell', NULL, 27),
(311, 16, 'send-sync-4-documenter-les-contraintes-send-sync', '4. Documenter les contraintes Send/Sync', NULL, 28),
(312, 16, 'send-sync-5-tester-avec-des-references-arc', '5. Tester avec des références Arc', NULL, 29),
(313, 16, 'send-sync-6-comprendre-les-erreurs-du-compilateur', '6. Comprendre les erreurs du compilateur', NULL, 30),
(314, 16, 'send-sync-exercice-1-identifier-send-et-sync', 'Exercice 1 : Identifier Send et Sync', NULL, 31),
(315, 16, 'send-sync-ce-code-ne-compile-pas-pourquoi-comment-le-corriger', '`// Ce code ne compile pas. Pourquoi ? Comment le corriger ?`', NULL, 32),
(316, 16, 'send-sync-fn-main', '`fn main() {`', NULL, 33),
(317, 16, 'send-sync-exercice-3-implementer-un-trait-thread-safe', 'Exercice 3 : Implémenter un trait thread-safe', NULL, 34),
(318, 16, 'send-sync-solution-exercice-1', 'Solution Exercice 1', NULL, 35),
(319, 16, 'send-sync-solution-exercice-2', 'Solution Exercice 2', NULL, 36),
(320, 16, 'send-sync-fn-main', '`fn main() {`', NULL, 37),
(321, 16, 'send-sync-solution-exercice-3', 'Solution Exercice 3', NULL, 38),
(322, 16, 'send-sync-conclusion', '■ **Conclusion**', NULL, 39),
(323, 16, 'send-sync-ressources', '■ **Ressources**', NULL, 40);
INSERT INTO cour_block (id, chapitre_id, heading, content, block_type, sort_order) VALUES
(10821, 284, NULL, $BLK10821$**Guide Complet pour la Concurrence Thread-Safe**$BLK10821$, 'text', 1),
(10822, 284, NULL, $BLK10822$Comprendre les Marker Traits$BLK10822$, 'text', 2),
(10823, 284, NULL, $BLK10823$Décembre 2025$BLK10823$, 'text', 3),
(10824, 285, NULL, $BLK10824$En  Rust, **Send** et **Sync** sont  des _marker  traits_ qui  garantissent  la  sécurité  de  la  concurrence  à  la compilation. Ils sont fondamentaux pour écrire du code concurrent sans data races.$BLK10824$, 'text', 1),
(10825, 286, NULL, $BLK10825$Un marker trait est un trait sans méthodes qui sert uniquement à marquer un type avec une propriété particulière.  Send  et  Sync  sont  implémentés  automatiquement  par  le  compilateur  pour  la  plupart  des types.$BLK10825$, 'text', 1),
(10826, 286, NULL, $BLK10826$```
// Définitions dans std::marker
```$BLK10826$, 'code', 2),
(10827, 286, NULL, $BLK10827$```
pub unsafe auto trait Send { }
pub unsafe auto trait Sync { }
```$BLK10827$, 'code', 3),
(10828, 286, NULL, $BLK10828$```
// auto trait = implémenté automatiquement
```$BLK10828$, 'code', 4),
(10829, 286, NULL, $BLK10829$```
// unsafe = si implémenté manuellement, responsabilité du développeur
```$BLK10829$, 'code', 5),
(10830, 287, NULL, $BLK10830$Send  signifie  qu'une  valeur  peut  être  transférée  (moved)  entre  threads  en  toute  sécurité.  Si  un  type implémente Send, vous pouvez le déplacer d'un thread à un autre sans risque.$BLK10830$, 'text', 1),
(10831, 288, NULL, $BLK10831$```
use std::thread;
```$BLK10831$, 'code', 1),
(10832, 288, NULL, $BLK10832$```
fn main() {
```$BLK10832$, 'code', 2),
(10833, 288, NULL, $BLK10833$```
    let data = String::from("Hello");  // String est Send
```$BLK10833$, 'code', 3),
(10834, 288, NULL, $BLK10834$```
    thread::spawn(move || {
```$BLK10834$, 'code', 4),
(10835, 288, NULL, $BLK10835$**`//`** ■ **`OK : String est Send, on peut le déplacer dans un autre thread println!("{}", data); });`**$BLK10835$, 'text', 5),
(10836, 288, NULL, $BLK10836$```
}
```$BLK10836$, 'code', 6),
(10837, 289, NULL, $BLK10837$|**Type**|**Send ?**|**Raison**|
|---|---|---|
|String|■Oui|Donnéespossédées,pas de référencespartagées|
|Vec<T>|■Oui (si T: Send)|Idem,possède ses données|
|i32, u64, bool|■Oui|Typesprimitifs copiables|
|Arc<T>|■Oui (si T: Send + Sync)|Pointeur atomique thread-safe|
|Rc<T>|■Non|Compteur de références non atomique|
|Cell<T>|■Non|Mutabilité intérieure non thread-safe|$BLK10837$, 'table', 1),
(10838, 290, NULL, $BLK10838$```
use std::rc::Rc;
use std::thread;
```$BLK10838$, 'code', 1),
(10839, 290, NULL, $BLK10839$```
fn main() {
    let data = Rc::new(String::from("Hello"));
```$BLK10839$, 'code', 2),
(10840, 290, NULL, $BLK10840$**`//`** ■ **`ERREUR : Rc<String> n'est pas Send ! thread::spawn(move || { println!("{}", data); }); }`**$BLK10840$, 'text', 3),
(10841, 290, NULL, $BLK10841$```
// Erreur du compilateur :
```$BLK10841$, 'code', 4),
(10842, 290, NULL, $BLK10842$```
// error[E0277]: `Rc<String>` cannot be sent between threads safely
//    = help: the trait `Send` is not implemented for `Rc<String>`
```$BLK10842$, 'code', 5),
(10843, 291, NULL, $BLK10843$Sync signifie qu'une référence (&T;) peut être partagée entre threads en toute sécurité. Si T est Sync, alors &T; est Send.$BLK10843$, 'text', 1),
(10844, 292, NULL, $BLK10844$**`T is Sync`** ■ **`&T is Send`**$BLK10844$, 'text', 1),
(10845, 292, NULL, $BLK10845$```
// Si T implémente Sync, alors une référence &T peut être envoyée entre threads
```$BLK10845$, 'code', 2),
(10846, 293, NULL, $BLK10846$```
use std::thread;
use std::sync::Arc;
```$BLK10846$, 'code', 1),
(10847, 293, NULL, $BLK10847$```
fn main() {
```$BLK10847$, 'code', 2),
(10848, 293, NULL, $BLK10848$```
    let data = Arc::new(String::from("Hello"));
```$BLK10848$, 'code', 3),
(10849, 293, NULL, $BLK10849$```
    let data_ref = Arc::clone(&data);
```$BLK10849$, 'code', 4),
(10850, 293, NULL, $BLK10850$```
    thread::spawn(move || {
```$BLK10850$, 'code', 5),
(10851, 293, NULL, $BLK10851$**`//`** ■ **`OK : String est Sync, donc &String est Send`**$BLK10851$, 'text', 6),
(10852, 293, NULL, $BLK10852$```
        // Arc permet de partager la référence
        println!("{}", data_ref);
    });
```$BLK10852$, 'code', 7),
(10853, 293, NULL, $BLK10853$```
    println!("{}", data);
}
```$BLK10853$, 'code', 8),
(10854, 294, NULL, $BLK10854$|**Type**|**Sync ?**|**Raison**|
|---|---|---|
|String|■Oui|Immuable, pas de mutabilité intérieure|
|Vec<T>|■Oui (si T: Sync)|Idem|
|i32, u64, bool|■Oui|Types primitifs|
|Mutex<T>|■Oui (si T: Send)|Synchronisation explicite|
|Arc<T>|■Oui (si T: Sync)|Pointeur atomique|
|Rc<T>|■Non|Compteur non atomique|
|Cell<T>|■Non|Mutabilité intérieure non atomique|
|RefCell<T>|■Non|Vérifications à l'exécution non thread-safe|$BLK10854$, 'table', 1),
(10855, 295, NULL, $BLK10855$|**Aspect**|**Send**|**Sync**|
|---|---|---|
|Signification|Je peux être déplacé entre threads|Ma référence peut être partagée entre threads|
|Ownership|Transfert de propriété|Partage de référence|
|Exemple usage|move dans thread::spawn|&T accessible depuis plusieurs threads|
|Pattern typique|thread::spawn(move || data)|Arc<T> partagé entre threads|
|Vérification|À la compilation|À la compilation|$BLK10855$, 'table', 1),
(10856, 296, NULL, $BLK10856$|**Situation**|**Trait requis**|
|---|---|
|Je déplace une valeur dans un autre thread|Send|
|Je partage une référence entre threads|Sync|
|J'utilise Arc<T> partagé|T: Send + Sync|
|J'utilise Mutex<T> partagé|T: Send|$BLK10856$, 'table', 1),
(10857, 297, NULL, $BLK10857$|**Type**|**Send**|**Sync**|**Notes**|
|---|---|---|---|
|String|■|■|Sûr pour concurrence|
|Vec<T>|■*|■*|* si T: Send/Sync|
|HashMap<K,V>|■*|■*|* si K,V: Send/Sync|
|i32, u64, bool|■|■|Types primitifs|
|Arc<T>|■*|■*|* si T: Send+Sync / T:Sync|
|Rc<T>|■|■|Compteur non atomique|
|Cell<T>|■*|■|* si T: Send, mais pas Sync|
|RefCell<T>|■*|■|Borrow checking runtime|
|Mutex<T>|■*|■*|* si T: Send|
|RwLock<T>|■*|■*|* si T: Send+Sync|$BLK10857$, 'table', 1),
(10858, 298, NULL, $BLK10858$Dans Axum et Tokio, les traits Send et Sync sont cruciaux car les futures peuvent être déplacées entre threads.$BLK10858$, 'text', 1),
(10859, 299, NULL, $BLK10859$```
// Trait pour formulaires Runique
```$BLK10859$, 'code', 1),
(10860, 299, NULL, $BLK10860$**`pub trait FormulaireTrait: Send + Sync {  //`** ← **`Sync important !`**$BLK10860$, 'text', 2),
(10861, 299, NULL, $BLK10861$```
    fn new() -> Self;
```$BLK10861$, 'code', 3),
(10862, 299, NULL, $BLK10862$```
    fn validate(&mut self, raw_data: &HashMap<String, String>) -> bool;
```$BLK10862$, 'code', 4),
(10863, 299, NULL, $BLK10863$```
}
```$BLK10863$, 'code', 5),
(10864, 299, NULL, $BLK10864$```
// Sans Sync, cette erreur peut survenir :
```$BLK10864$, 'code', 6),
(10865, 299, NULL, $BLK10865$```
#[async_trait]
impl<S, T> FromRequest<S> for AxumForm<T>
where
```$BLK10865$, 'code', 7),
(10866, 299, NULL, $BLK10866$**`T: FormulaireTrait + 'static,  //`** ← **`Doit être Send + Sync`**$BLK10866$, 'text', 8),
(10867, 299, NULL, $BLK10867$```
{
    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
```$BLK10867$, 'code', 9),
(10868, 299, NULL, $BLK10868$```
        // Cette future peut être déplacée entre threads par Tokio
```$BLK10868$, 'code', 10),
(10869, 299, NULL, $BLK10869$```
        // Si T n'est pas Sync et qu'une référence &T existe,
```$BLK10869$, 'code', 11),
(10870, 299, NULL, $BLK10870$```
        // le compilateur rejettera le code
    }
```$BLK10870$, 'code', 12),
(10871, 299, NULL, $BLK10871$```
}
```$BLK10871$, 'code', 13),
(10872, 300, NULL, $BLK10872$```
use std::cell::Cell;
```$BLK10872$, 'code', 1),
(10873, 300, NULL, $BLK10873$**`//`** ■ **`Ce code ne compile PAS pub struct BadForm { inner: Forms, counter: Cell<u32>,  // Cell n'est pas Sync ! }`**$BLK10873$, 'text', 2),
(10874, 300, NULL, $BLK10874$```
impl FormulaireTrait for BadForm {
    fn new() -> Self {
        Self {
            inner: Forms::new(),
            counter: Cell::new(0),
        }
    }
```$BLK10874$, 'code', 3),
(10875, 300, NULL, $BLK10875$```
    fn validate(&mut self, raw_data: &HashMap<String, String>) -> bool {
        self.counter.set(self.counter.get() + 1);
        self.inner.is_valid()
    }
```$BLK10875$, 'code', 4),
(10876, 300, NULL, $BLK10876$```
}
```$BLK10876$, 'code', 5),
(10877, 300, NULL, $BLK10877$- **`// Erreur du compilateur :`**$BLK10877$, 'list', 6),
(10878, 300, NULL, $BLK10878$```
// error[E0277]: `Cell<u32>` cannot be shared between threads safely
//    = help: the trait `Sync` is not implemented for `Cell<u32>`
```$BLK10878$, 'code', 7),
(10879, 301, NULL, $BLK10879$```
use std::sync::atomic::{AtomicU32, Ordering};
```$BLK10879$, 'code', 1),
(10880, 301, NULL, $BLK10880$**`//`** ■ **`Ce code compile ! pub struct GoodForm { inner: Forms, counter: AtomicU32,  // AtomicU32 est Send + Sync } impl FormulaireTrait for GoodForm { fn new() -> Self { Self { inner: Forms::new(), counter: AtomicU32::new(0), } } fn validate(&mut self, raw_data: &HashMap<String, String>) -> bool { self.counter.fetch_add(1, Ordering::Relaxed); self.inner.is_valid() }`**$BLK10880$, 'text', 2),
(10881, 301, NULL, $BLK10881$```
}
```$BLK10881$, 'code', 3),
(10882, 302, NULL, $BLK10882$**`//`** ■ **`Erreur use std::rc::Rc;`**$BLK10882$, 'text', 1),
(10883, 302, NULL, $BLK10883$```
async fn handler(data: Rc<String>) {
    // Erreur : Rc is not Send
}
```$BLK10883$, 'code', 2),
(10884, 302, NULL, $BLK10884$**`//`** ■ **`Solution use std::sync::Arc;`**$BLK10884$, 'text', 3),
(10885, 302, NULL, $BLK10885$```
async fn handler(data: Arc<String>) {
    // OK : Arc is Send + Sync
}
```$BLK10885$, 'code', 4),
(10886, 303, NULL, $BLK10886$**`//`** ■ **`Erreur use std::cell::Cell;`**$BLK10886$, 'text', 1),
(10887, 303, NULL, $BLK10887$```
struct MyStruct {
    value: Cell<i32>,  // Cell n'est pas Sync
}
```$BLK10887$, 'code', 2),
(10888, 303, NULL, $BLK10888$**`//`** ■ **`Solution : Utiliser des types atomiques use std::sync::atomic::{AtomicI32, Ordering};`**$BLK10888$, 'text', 3),
(10889, 303, NULL, $BLK10889$```
struct MyStruct {
    value: AtomicI32,  // AtomicI32 est Send + Sync
}
```$BLK10889$, 'code', 4),
(10890, 304, NULL, $BLK10890$**`//`** ■ **`Risque futur pub trait MyTrait: Send {  // Manque Sync // ... } //`** ■ **`Meilleure pratique pub trait MyTrait: Send + Sync {  // Complet et sûr // ...`**$BLK10890$, 'text', 1),
(10891, 304, NULL, $BLK10891$```
}
```$BLK10891$, 'code', 2),
(10892, 305, NULL, $BLK10892$**`//`** ■ **`Trait correct pour Axum pub trait FormulaireTrait: Send + Sync { fn new() -> Self; fn validate(&mut self, raw_data: &HashMap<String, String>) -> bool; }`**$BLK10892$, 'text', 1),
(10893, 305, NULL, $BLK10893$```
// Implémentation
pub struct UserForm(Forms);
impl FormulaireTrait for UserForm {
    fn new() -> Self {
        Self(Forms::new())
    }
    fn validate(&mut self, raw_data: &HashMap<String, String>) -> bool {
        // Validation...
        self.0.is_valid()
    }
}
// Extracteur Axum
#[axum::async_trait]
impl<S, T> FromRequest<S> for AxumForm<T>
where
    S: Send + Sync,
    T: FormulaireTrait + 'static,  // T est automatiquement Send + Sync
{
    type Rejection = Response;
    async fn from_request(req: Request<Body>,state: &S)→
Result<Self,Self::Rejection> {
        // Le compilateur garantit que c'est thread-safe
        // ...
    }
}
```$BLK10893$, 'code', 2),
(10894, 306, NULL, $BLK10894$```
use std::sync::Arc;
use tokio::sync::Mutex;
#[derive(Clone)]
struct AppState {
    counter: Arc<Mutex<i32>>,  // Mutex<T> est Sync si T: Send
    config: Arc<Settings>,      // Arc<T> est Sync si T: Sync
}
```$BLK10894$, 'code', 1),
(10895, 307, NULL, $BLK10895$```
    State(state): State<AppState>,
```$BLK10895$, 'code', 1),
(10896, 307, NULL, $BLK10896$- **`) -> Response {`**$BLK10896$, 'list', 2),
(10897, 307, NULL, $BLK10897$- **`//`** ■ **`OK : AppState est Send + Sync`**$BLK10897$, 'list', 3),
(10898, 307, NULL, $BLK10898$```
    let mut counter = state.counter.lock().await;
```$BLK10898$, 'code', 4),
(10899, 307, NULL, $BLK10899$- **`*counter += 1;`**$BLK10899$, 'list', 5),
(10900, 307, NULL, $BLK10900$- **`// ...`**$BLK10900$, 'list', 6),
(10901, 307, NULL, $BLK10901$- **`}`**$BLK10901$, 'list', 7),
(10902, 308, NULL, $BLK10902$Garantit la compatibilité avec async/await et Tokio.$BLK10902$, 'text', 1),
(10903, 309, NULL, $BLK10903$Arc est thread-safe, Rc ne l'est pas.$BLK10903$, 'text', 1),
(10904, 310, NULL, $BLK10904$Pour la mutabilité intérieure thread-safe.$BLK10904$, 'text', 1),
(10905, 311, NULL, $BLK10905$Facilite la compréhension pour les futurs développeurs.$BLK10905$, 'text', 1),
(10906, 312, NULL, $BLK10906$Vérifie que vos types sont bien Sync.$BLK10906$, 'text', 1),
(10907, 313, NULL, $BLK10907$Les messages d'erreur Send/Sync sont très précis.$BLK10907$, 'text', 1),
(10908, 314, NULL, $BLK10908$Pour chaque type, déterminez s'il est Send et/ou Sync :$BLK10908$, 'text', 1),
(10909, 314, NULL, $BLK10909$|**Type**|**Send ?**|**Sync ?**|
|---|---|---|
|String|?|?|
|Vec<Rc<i32>>|?|?|
|Arc<Mutex<String>>|?|?|
|Cell<String>|?|?|
|&str|?|?|$BLK10909$, 'table', 2),
(10910, 315, NULL, $BLK10910$```
use std::rc::Rc;
use std::thread;
```$BLK10910$, 'code', 1),
(10911, 316, NULL, $BLK10911$```
    let data = Rc::new(vec![1, 2, 3]);
```$BLK10911$, 'code', 1),
(10912, 316, NULL, $BLK10912$```
    thread::spawn(move || {
        println!("{:?}", data);
    });
```$BLK10912$, 'code', 2),
(10913, 316, NULL, $BLK10913$```
}
```$BLK10913$, 'code', 3),
(10914, 317, NULL, $BLK10914$Créez un trait CacheTrait qui :$BLK10914$, 'text', 1),
(10915, 317, NULL, $BLK10915$- Soit utilisable dans du code async$BLK10915$, 'list', 2),
(10916, 317, NULL, $BLK10916$- Permette de stocker et récupérer des valeurs$BLK10916$, 'list', 3),
(10917, 317, NULL, $BLK10917$- Soit thread-safe$BLK10917$, 'list', 4),
(10918, 318, NULL, $BLK10918$|**Type**|**Send**|**Sync**|**Explication**|
|---|---|---|---|
|String|■|■|Type standard thread-safe|
|Vec<Rc<i32>>|■|■|Rc n'est ni Send ni Sync|
|Arc<Mutex<String>>|■|■|Arc + Mutex = thread-safe|
|Cell<String>|■|■|Send car String:Send, mais pas Sync|
|&str|■|■|Référence immuable|$BLK10918$, 'table', 1),
(10919, 319, NULL, $BLK10919$```
// Problème : Rc n'est pas Send
```$BLK10919$, 'code', 1),
(10920, 319, NULL, $BLK10920$```
// Solution : Utiliser Arc au lieu de Rc
```$BLK10920$, 'code', 2),
(10921, 319, NULL, $BLK10921$**`use std::sync::Arc;  //`** ← **`Changement ici`**$BLK10921$, 'text', 3),
(10922, 319, NULL, $BLK10922$```
use std::thread;
```$BLK10922$, 'code', 4),
(10923, 320, NULL, $BLK10923$**`let data = Arc::new(vec![1, 2, 3]);  //`** ← **`Arc au lieu de Rc`**$BLK10923$, 'text', 1),
(10924, 320, NULL, $BLK10924$```
    thread::spawn(move || {
        println!("{:?}", data);
    });
}
```$BLK10924$, 'code', 2),
(10925, 320, NULL, $BLK10925$**`//`** ■ **`Compile et fonctionne !`**$BLK10925$, 'text', 3),
(10926, 321, NULL, $BLK10926$```
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
```$BLK10926$, 'code', 1),
(10927, 321, NULL, $BLK10927$```
// Trait thread-safe pour cache
pub trait CacheTrait: Send + Sync {
    type Key: Send + Sync;
    type Value: Send + Sync;
```$BLK10927$, 'code', 2),
(10928, 321, NULL, $BLK10928$```
    fn get(&self, key: &Self::Key) -> Option<Self::Value>;
    fn set(&self, key: Self::Key, value: Self::Value);
```$BLK10928$, 'code', 3),
(10929, 321, NULL, $BLK10929$```
}
```$BLK10929$, 'code', 4),
(10930, 321, NULL, $BLK10930$```
// Implémentation avec Mutex
pub struct Cache<K, V> {
```$BLK10930$, 'code', 5),
(10931, 321, NULL, $BLK10931$```
    data: Arc<Mutex<HashMap<K, V>>>,
}
impl<K, V> Cache<K, V>
where
    K: Send + Sync + Eq + std::hash::Hash + Clone,
    V: Send + Sync + Clone,
{
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
```$BLK10931$, 'code', 6),
(10932, 321, NULL, $BLK10932$```
impl<K, V> CacheTrait for Cache<K, V>
where
    K: Send + Sync + Eq + std::hash::Hash + Clone,
    V: Send + Sync + Clone,
{
    type Key = K;
    type Value = V;
```$BLK10932$, 'code', 7),
(10933, 321, NULL, $BLK10933$```
    fn get(&self, key: &Self::Key) -> Option<Self::Value> {
        self.data.lock().unwrap().get(key).cloned()
    }
    fn set(&self, key: Self::Key, value: Self::Value) {
        self.data.lock().unwrap().insert(key, value);
    }
}
```$BLK10933$, 'code', 8),
(10934, 321, NULL, $BLK10934$**`//`** ■ **`Ce cache est Send + Sync et utilisable dans du code async !`**$BLK10934$, 'text', 9),
(10935, 322, NULL, $BLK10935$Send et Sync sont les piliers de la programmation concurrente sûre en Rust. Le compilateur les vérifie automatiquement, éliminant toute possibilité de data races.$BLK10935$, 'text', 1),
(10936, 322, NULL, $BLK10936$■ Send = Peut être déplacé entre threads$BLK10936$, 'text', 2),
(10937, 322, NULL, $BLK10937$■ Sync = Peut être partagé (référence) entre threads$BLK10937$, 'text', 3),
(10938, 322, NULL, $BLK10938$- Vérification à la compilation = Pas de data races$BLK10938$, 'list', 4),
(10939, 322, NULL, $BLK10939$- Auto-implémenté pour la plupart des types$BLK10939$, 'list', 5),
(10940, 322, NULL, $BLK10940$■ Essentiel pour Axum, Tokio et async/await$BLK10940$, 'text', 6),
(10941, 323, NULL, $BLK10941$- The Rust Book - Chapter 16 (Concurrency)$BLK10941$, 'list', 1),
(10942, 323, NULL, $BLK10942$- Rust Nomicon - Send and Sync$BLK10942$, 'list', 2),
(10943, 323, NULL, $BLK10943$- Tokio documentation$BLK10943$, 'list', 3),
(10944, 323, NULL, $BLK10944$- Axum documentation$BLK10944$, 'list', 4);

-- traits-avances.md (cour_id=17)
INSERT INTO chapitre (id, cour_id, slug, title, lead, sort_order) VALUES
(324, 17, 'traits-avances-maitriser-les-traits-et-le-polymorphisme', 'Maîtriser les Traits et le Polymorphisme', NULL, 1),
(325, 17, 'traits-avances-1-traits-de-la-stdlib', '1. Traits de la stdlib', NULL, 2),
(326, 17, 'traits-avances-1-traits-de-la-stdlib', '1. Traits de la stdlib', NULL, 3),
(327, 17, 'traits-avances-12-clone-et-copy', '1.2 - Clone et Copy', NULL, 4),
(328, 17, 'traits-avances-13-default', '1.3 - Default', NULL, 5),
(329, 17, 'traits-avances-14-partialeq-et-eq', '1.4 - PartialEq et Eq', NULL, 6),
(330, 17, 'traits-avances-21-difference-avec-generiques', '2.1 - Différence avec génériques', NULL, 7),
(331, 17, 'traits-avances-quand-utiliser-quoi', 'Quand utiliser quoi ?', NULL, 8),
(332, 17, 'traits-avances-22-exemples-pratiques', '2.2 - Exemples pratiques', NULL, 9),
(333, 17, 'traits-avances-3-trait-objects-dyn-trait', '3. Trait Objects (dyn Trait)', NULL, 10),
(334, 17, 'traits-avances-31-box', '3.1 - Box', NULL, 11),
(335, 17, 'traits-avances-ii-static-vs-dynamic-dispatch', 'II **Static vs Dynamic dispatch :**', NULL, 12),
(336, 17, 'traits-avances-32-object-safety', '3.2 - Object safety', NULL, 13),
(337, 17, 'traits-avances-41-where-clauses', '4.1 - Where clauses', NULL, 14),
(338, 17, 'traits-avances-42-bounds-multiples', '4.2 - Bounds multiples', NULL, 15),
(339, 17, 'traits-avances-5-default-implementations', '5. Default Implementations', NULL, 16),
(340, 17, 'traits-avances-6-supertraits', '6. Supertraits', NULL, 17),
(341, 17, 'traits-avances-7-patterns-avances', '7. Patterns avancés', NULL, 18),
(342, 17, 'traits-avances-8-exercices-pratiques', '8. Exercices pratiques', NULL, 19),
(343, 17, 'traits-avances-aide-memoire', 'Aide-mémoire', NULL, 20);
INSERT INTO cour_block (id, chapitre_id, heading, content, block_type, sort_order) VALUES
(10945, 324, NULL, $BLK10945$Associated Types, Trait Objects et Plus$BLK10945$, 'text', 1),
(10946, 324, 'Objectifs du cours', $BLK10946$À la fin de ce cours, tu sauras : 

- Maîtriser les traits standard (Debug, Display, Clone) 

- Utiliser les associated types 

- Comprendre les trait objects (dyn Trait) 

- Appliquer les trait bounds avancés 

- Créer des APIs flexibles avec traits$BLK10946$, 'text', 2),
(10947, 325, NULL, $BLK10947$- 1.1 - Debug et Display$BLK10947$, 'list', 1),
(10948, 325, NULL, $BLK10948$- 1.2 - Clone et Copy$BLK10948$, 'list', 2),
(10949, 325, NULL, $BLK10949$- 1.3 - Default$BLK10949$, 'list', 3),
(10950, 325, NULL, $BLK10950$- 1.4 - PartialEq et Eq$BLK10950$, 'list', 4),
(10951, 325, NULL, $BLK10951$2. Associated Types$BLK10951$, 'list', 5),
(10952, 325, NULL, $BLK10952$- 2.1 - Différence avec génériques$BLK10952$, 'list', 6),
(10953, 325, NULL, $BLK10953$- 2.2 - Exemples pratiques$BLK10953$, 'list', 7),
(10954, 325, NULL, $BLK10954$3. Trait Objects (dyn Trait)$BLK10954$, 'list', 8),
(10955, 325, NULL, $BLK10955$- 3.1 - Box$BLK10955$, 'list', 9),
(10956, 325, NULL, $BLK10956$- 3.2 - Object safety$BLK10956$, 'list', 10),
(10957, 325, NULL, $BLK10957$4. Trait Bounds Avancés$BLK10957$, 'list', 11),
(10958, 325, NULL, $BLK10958$- 4.1 - Where clauses$BLK10958$, 'list', 12),
(10959, 325, NULL, $BLK10959$- 4.2 - Bounds multiples$BLK10959$, 'list', 13),
(10960, 325, NULL, $BLK10960$5. Default Implementations$BLK10960$, 'list', 14),
(10961, 325, NULL, $BLK10961$6. Supertraits$BLK10961$, 'list', 15),
(10962, 325, NULL, $BLK10962$7. Patterns avancés$BLK10962$, 'list', 16),
(10963, 325, NULL, $BLK10963$8. Exercices$BLK10963$, 'list', 17),
(10964, 326, NULL, $BLK10964$**1.1 - Debug et Display**$BLK10964$, 'text', 1),
(10965, 326, NULL, $BLK10965$```
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
```$BLK10965$, 'code', 2),
(10966, 326, 'Debug vs Display :', $BLK10966$• `Debug` : Pour les développeurs (débogage) • `Display` : Pour les utilisateurs (affichage)$BLK10966$, 'text', 3),
(10967, 327, NULL, $BLK10967$```
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
```$BLK10967$, 'code', 1),
(10968, 327, NULL, $BLK10968$```
let p1 = Point { x: 1, y: 2 };
let p2 = p1;  // Copié automatiquement
println!("{}, {}", p1.x, p2.x);  // p1 toujours valide !
```$BLK10968$, 'code', 2),
(10969, 327, NULL, $BLK10969$**`//`** II **`Copy seulement pour types sans heap allocation // String ne peut pas être Copy (contient des données heap)`**$BLK10969$, 'text', 3),
(10970, 328, NULL, $BLK10970$```
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
```$BLK10970$, 'code', 1),
(10971, 329, NULL, $BLK10971$```
// PartialEq : égalité partielle
#[derive(PartialEq)]
struct Point {
    x: i32,
    y: i32,
}
let p1 = Point { x: 1, y: 2 };
let p2 = Point { x: 1, y: 2 };
assert!(p1 == p2);
```$BLK10971$, 'code', 1),
(10972, 329, NULL, $BLK10972$```
// Eq : égalité totale (réflexive)
// Pour les types sans NaN
#[derive(PartialEq, Eq)]
struct User {
    id: u32,
    nom: String,
}
```$BLK10972$, 'code', 2),
(10973, 329, NULL, $BLK10973$```
// f32 et f64 sont seulement PartialEq (à cause de NaN)
let x = f64::NAN;
assert!(x != x);  // NaN != NaN !
```$BLK10973$, 'code', 3),
(10974, 329, NULL, $BLK10974$```
// Implémentation manuelle
impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
```$BLK10974$, 'code', 4),
(10975, 329, NULL, $BLK10975$```
}
```$BLK10975$, 'code', 5),
(10976, 330, NULL, $BLK10976$```
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
```$BLK10976$, 'code', 1),
(10977, 330, NULL, $BLK10977$```
    fn convert(&self, input: i32) -> String {
        input.to_string()
    }
}
```$BLK10977$, 'code', 2),
(10978, 331, NULL, $BLK10978$• **Générique** : Plusieurs implémentations possibles pour un type$BLK10978$, 'text', 1),
(10979, 331, NULL, $BLK10979$• **Associated type** : Une seule implémentation logique$BLK10979$, 'text', 2),
(10980, 332, NULL, $BLK10980$```
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
```$BLK10980$, 'code', 1),
(10981, 333, NULL, $BLK10981$Les **trait objects** permettent le polymorphisme dynamique : stocker différents types qui implémentent le même trait.$BLK10981$, 'text', 1),
(10982, 334, NULL, $BLK10982$```
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
```$BLK10982$, 'code', 1),
(10983, 335, NULL, $BLK10983$• Génériques ( `<T: Trait>` ) : Static dispatch (plus rapide)$BLK10983$, 'text', 1),
(10984, 335, NULL, $BLK10984$- Trait objects ( `dyn Trait` ) : Dynamic dispatch (plus flexible)$BLK10984$, 'list', 2),
(10985, 336, NULL, $BLK10985$**`//`** I **`Object-safe (peut être dyn) trait Draw { fn draw(&self); }`**$BLK10985$, 'text', 1),
(10986, 336, NULL, $BLK10986$**`//`** I **`Pas object-safe (ne peut pas être dyn) trait Clone { fn clone(&self) -> Self;  // Retourne Self }`**$BLK10986$, 'text', 2),
(10987, 336, NULL, $BLK10987$```
trait Generic {
    fn method<T>(&self, x: T);  // Méthode générique
}
```$BLK10987$, 'code', 3),
(10988, 336, NULL, $BLK10988$```
// Règles d'object safety :
// 1. Pas de méthodes retournant Self
// 2. Pas de méthodes génériques
// 3. Pas de associated functions (sans &self)
```$BLK10988$, 'code', 4),
(10989, 336, NULL, $BLK10989$```
// Solution : diviser le trait
trait Draw {
    fn draw(&self);
}
```$BLK10989$, 'code', 5),
(10990, 336, NULL, $BLK10990$```
trait Clone {
    fn clone_box(&self) -> Box<dyn Draw>;
}
```$BLK10990$, 'code', 6),
(10991, 336, NULL, $BLK10991$```
// Maintenant on peut avoir :
let shapes: Vec<Box<dyn Draw>> = vec![
    Box::new(Circle),
    Box::new(Square),
];
```$BLK10991$, 'code', 7),
(10992, 337, NULL, $BLK10992$```
// Sans where (devient illisible)
fn fonction<T: Display + Clone, U: Clone + Debug>(t: T, u: U) -> i32 {
    // ...
}
```$BLK10992$, 'code', 1),
(10993, 337, NULL, $BLK10993$```
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
```$BLK10993$, 'code', 2),
(10994, 338, NULL, $BLK10994$```
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
```$BLK10994$, 'code', 1),
(10995, 339, NULL, $BLK10995$```
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
```$BLK10995$, 'code', 1),
(10996, 340, NULL, $BLK10996$```
use std::fmt::Display;
```$BLK10996$, 'code', 1),
(10997, 340, NULL, $BLK10997$```
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
```$BLK10997$, 'code', 2),
(10998, 340, NULL, $BLK10998$```
// Utilisation
let p = Point { x: 1, y: 3 };
p.outline_print();
```$BLK10998$, 'code', 3),
(10999, 341, NULL, $BLK10999$```
// 1. Newtype pattern
struct Wrapper(Vec<String>);
```$BLK10999$, 'code', 1),
(11000, 341, NULL, $BLK11000$```
impl Display for Wrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[{}]", self.0.join(", "))
    }
}
```$BLK11000$, 'code', 2),
(11001, 341, NULL, $BLK11001$```
// 2. Extension methods
trait VecExt<T> {
    fn first_or_default(&self) -> T
    where
        T: Default;
}
```$BLK11001$, 'code', 3),
(11002, 341, NULL, $BLK11002$```
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
```$BLK11002$, 'code', 4),
(11003, 341, NULL, $BLK11003$```
impl<T: Display> MyTrait for T {
    fn method(&self) {
        println!("{}", self);
    }
}
```$BLK11003$, 'code', 5),
(11004, 342, 'Exercice 1 : Créer un trait Shape', $BLK11004$```
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
```$BLK11004$, 'code', 1),
(11005, 343, NULL, $BLK11005$|**Trait**|**Usage**|
|---|---|
|**`Debug`**|println!("{:?}", x)|
|**`Display`**|println!("{}", x)|
|**`Clone`**|x.clone()|
|**`Copy`**|Copie implicite|
|**`Default`**|T::default()|
|**`PartialEq`**|x == y|$BLK11005$, 'table', 1),
(11006, 343, NULL, $BLK11006$- **Traits** = interfaces de Rust$BLK11006$, 'list', 2),
(11007, 343, NULL, $BLK11007$- **Associated types** = un type par implémentation$BLK11007$, 'list', 3),
(11008, 343, NULL, $BLK11008$- **dyn Trait** = polymorphisme dynamique$BLK11008$, 'list', 4),
(11009, 343, NULL, $BLK11009$- **Where clauses** = contraintes lisibles$BLK11009$, 'list', 5),
(11010, 343, NULL, $BLK11010$- **Default impl** = comportement par défaut$BLK11010$, 'list', 6),
(11011, 343, NULL, $BLK11011$- **Supertraits** = dépendances entre traits$BLK11011$, 'list', 7),
(11012, 343, 'Bravo !', $BLK11012$Tu maîtrises les traits avancés ! 

Tu peux maintenant créer des APIs flexibles et réutilisables. C'est la clé du polymorphisme en Rust ! 

I **Expert Rust en approche !** I$BLK11012$, 'text', 8);

-- macros-declaratives.md (cour_id=18)
INSERT INTO chapitre (id, cour_id, slug, title, lead, sort_order) VALUES
(344, 18, 'macros-declaratives-1-pourquoi-les-macros', '1. Pourquoi les macros ?', NULL, 1),
(345, 18, 'macros-declaratives-2-syntaxe-de-base', '2. Syntaxe de base', NULL, 2),
(346, 18, 'macros-declaratives-3-fragments', '3. Fragments', NULL, 3),
(347, 18, 'macros-declaratives-4-repetitions', '4. Répétitions', NULL, 4),
(348, 18, 'macros-declaratives-5-exemples-simples', '5. Exemples simples', NULL, 5),
(349, 18, 'macros-declaratives-6-exemples-intermediaires', '6. Exemples intermédiaires', NULL, 6),
(350, 18, 'macros-declaratives-7-bonnes-pratiques', '7. Bonnes pratiques', NULL, 7);
INSERT INTO cour_block (id, chapitre_id, heading, content, block_type, sort_order) VALUES
(11013, 344, NULL, $BLK11013$Les macros déclaratives permettent ce que les fonctions ne peuvent pas faire :$BLK11013$, 'text', 1),
(11014, 344, NULL, $BLK11014$- **Nombre variable d'arguments** : `println!("val: {}", x)` ou `println!("{} {}", a, b)`
- **Générer du code répétitif** : déclarer plusieurs méthodes d'un coup
- **DSL** : syntaxe personnalisée qui ressemble à du Rust mais n'en est pas$BLK11014$, 'list', 2),
(11015, 344, NULL, $BLK11015$> **Important :** Quand une fonction normale peut faire le travail, utilise une fonction. Les macros sont plus difficiles à déboguer.$BLK11015$, 'warning', 3),
(11016, 344, NULL, $BLK11016$---$BLK11016$, 'text', 4),
(11017, 345, NULL, $BLK11017$```rust
macro_rules! nom_macro {
    (motif) => {
        // Code généré
    };
    (autre_motif) => {
        // Autre branche
    };
}
```$BLK11017$, 'code', 1),
(11018, 345, NULL, $BLK11018$Chaque branche est un `(motif) => { expansion }`. Rust essaie les branches dans l'ordre et utilise la première qui correspond.$BLK11018$, 'text', 2),
(11019, 345, NULL, $BLK11019$---$BLK11019$, 'text', 3),
(11020, 346, NULL, $BLK11020$Les fragments définissent ce que chaque variable de pattern peut capturer :$BLK11020$, 'text', 1),
(11021, 346, NULL, $BLK11021$| Désignateur | Description | Exemple |
|---|---|---|
| `$x:expr` | Expression | `2 + 2`, `x.foo()` |
| `$x:ident` | Identificateur | `nom_variable`, `foo` |
| `$x:ty` | Type | `u32`, `Vec<String>` |
| `$x:pat` | Pattern | `Some(x)`, `(a, b)` |
| `$x:stmt` | Statement | `let x = 5;` |
| `$x:block` | Bloc de code | `{ ... }` |
| `$x:item` | Item | `struct`, `fn`, `impl` |
| `$x:tt` | Token tree | N'importe quel token |
| `$x:literal` | Littéral | `42`, `"texte"` |$BLK11021$, 'table', 2),
(11022, 346, NULL, $BLK11022$---$BLK11022$, 'text', 3),
(11023, 347, NULL, $BLK11023$```rust
$(...)*   // Zéro ou plusieurs
$(...)+   // Une ou plusieurs
$(...)?   // Zéro ou une
```$BLK11023$, 'code', 1),
(11024, 347, 'Comment ça marche', $BLK11024$```rust
macro_rules! afficher_tout {
    ($($val:expr),*) => {
        $(
            println!("{:?}", $val);
        )*
    };
}

fn main() {
    afficher_tout!(1, "hello", true, 3.14);
}
```

Le `$($val:expr),*` capture zéro ou plusieurs expressions séparées par des virgules. Le `$( ... )*` dans l'expansion répète le bloc pour chaque valeur capturée.

---$BLK11024$, 'code', 2),
(11025, 348, 'Macro avec un message préfixé', $BLK11025$```rust
macro_rules! info {

    ($msg:expr) => {
        println!("[INFO] {}", $msg);
    };

}

fn main() {
    info!("Démarrage");
    info!("Connexion établie");
}
```$BLK11025$, 'code', 1),
(11026, 348, 'Plusieurs patterns — macro de log', $BLK11026$```rust
macro_rules! log {

    (info $msg:expr)  => { println!("[INFO]  {}", $msg); };

    (warn $msg:expr)  => { println!("[WARN]  {}", $msg); };

    (error $msg:expr) => { eprintln!("[ERROR] {}", $msg); };

}

fn main() {
    log!(info  "Démarrage");
    log!(warn  "Mémoire faible");
    log!(error "Échec de connexion");
}
```$BLK11026$, 'code', 2),
(11027, 348, 'Calcul à la compilation — le piège des parenthèses', $BLK11027$```rust
// ❌ Sans parenthèses — bug subtil
macro_rules! carre_mauvais {
    ($n:expr) => { $n * $n };
}

// ✅ Avec parenthèses — correct
macro_rules! carre {
    ($n:expr) => { ($n) * ($n) };
}

fn main() {
    let a = carre_mauvais!(2 + 3); // Devient 2 + 3 * 2 + 3 = 11, pas 25!
    let b = carre!(2 + 3);         // Devient (2 + 3) * (2 + 3) = 25 ✅
}
```

> **Règle :** Toujours entourer les paramètres `$x:expr` de parenthèses dans l'expansion.

---$BLK11027$, 'code', 3),
(11028, 349, 'Créer un HashMap avec une syntaxe facilitée', $BLK11028$```rust
use std::collections::HashMap;

macro_rules! hashmap {

    () => { HashMap::new() };

    ($($key:expr => $val:expr),+ $(,)?) => {{
        let mut map = HashMap::new();

        $(
            map.insert($key, $val);
        )+

        map
    }};

}

fn main() {
    let scores = hashmap! {
        "Alice" => 100,
        "Bob"   => 85,
    };

    println!("{:?}", scores);
}
```

Le `$(,)?` à la fin accepte une virgule trailing facultative — comme Rust le fait partout.$BLK11028$, 'code', 1),
(11029, 349, 'Recréer vec!', $BLK11029$```rust
macro_rules! mon_vec {

    () => {
        Vec::new()
    };

    ($($e:expr),+ $(,)?) => {{
        let mut v = Vec::new();
        $(v.push($e);)+
        v
    }};

    ($e:expr; $n:expr) => {{
        let mut v = Vec::with_capacity($n);
        for _ in 0..$n { v.push($e); }
        v
    }};

}

fn main() {
    let a = mon_vec![];
    let b = mon_vec![1, 2, 3];
    let c = mon_vec![0; 10];
}
```$BLK11029$, 'code', 2),
(11030, 349, 'Générateur de getters', $BLK11030$```rust
macro_rules! create_getter {
    ($nom:ident, $type:ty, $field:ident) => {
        pub fn $nom(&self) -> &$type {
            &self.$field
        }
    };
}

struct Personne {
    nom:   String,
    age:   u32,
}

impl Personne {
    create_getter!(get_nom, String, nom);
    create_getter!(get_age, u32,    age);
}
```

---$BLK11030$, 'code', 3),
(11031, 350, 'Évaluation multiple — le piège', $BLK11031$```rust
// ❌ $a et $b sont évalués deux fois
macro_rules! max_mauvais {
    ($a:expr, $b:expr) => {
        if $a > $b { $a } else { $b }
    };
}

// ✅ Évaluation une seule fois
macro_rules! max_bon {
    ($a:expr, $b:expr) => {{
        let a = $a;
        let b = $b;

        if a > b { a } else { b }
    }};
}
```

Si `$a` est un appel de fonction avec effet de bord, `max_mauvais!` l'appelle deux fois. Toujours lier les arguments à des variables locales.$BLK11031$, 'code', 1),
(11032, 350, 'Hygiène des variables', $BLK11032$```rust
macro_rules! avec_temp {
    ($val:expr) => {{
        let temp = $val;  // 'temp' est locale à la macro, pas de conflit
        println!("temp = {:?}", temp);
    }};
}

fn main() {
    let temp = "extérieur";
    avec_temp!(42);
    println!("temp = {}", temp); // Toujours "extérieur"
}
```

Rust isole les variables introduites par les macros — elles n'entrent pas en conflit avec les variables du code appelant.$BLK11032$, 'code', 2),
(11033, 350, 'Déboguer avec cargo-expand', $BLK11033$```bash
cargo install cargo-expand
cargo expand         # Affiche tout le code après expansion des macros
cargo expand main    # Seulement la fonction main
```

C'est l'outil indispensable pour comprendre ce que ta macro génère réellement.$BLK11033$, 'code', 3);

-- async-tokio.md (cour_id=19)
INSERT INTO chapitre (id, cour_id, slug, title, lead, sort_order) VALUES
(351, 19, 'async-tokio-programmation-asynchrone-en-rust', 'Programmation Asynchrone en Rust', NULL, 1),
(352, 19, 'async-tokio-11-sync-vs-async', '1.1 - Sync vs Async', NULL, 2),
(353, 19, 'async-tokio-13-async-await', '1.3 - async/await', NULL, 3),
(354, 19, 'async-tokio-21-installation-et-setup', '2.1 - Installation et setup', NULL, 4),
(355, 19, 'async-tokio-22-tokiomain', '2.2 - #[tokio::main]', NULL, 5),
(356, 19, 'async-tokio-23-spawning-tasks', '2.3 - Spawning tasks', NULL, 6),
(357, 19, 'async-tokio-31-tokiotime', '3.1 - tokio::time', NULL, 7),
(358, 19, 'async-tokio-32-tokiofs', '3.2 - tokio::fs', NULL, 8),
(359, 19, 'async-tokio-33-tokionet', '3.3 - tokio::net', NULL, 9),
(360, 19, 'async-tokio-41-join-et-select', '4.1 - join! et select!', NULL, 10),
(361, 19, 'async-tokio-42-channels-mpsc', '4.2 - Channels (mpsc)', NULL, 11),
(362, 19, 'async-tokio-43-mutex-et-rwlock', '4.3 - Mutex et RwLock', NULL, 12),
(363, 19, 'async-tokio-51-stream-trait', '5.1 - Stream trait', NULL, 13),
(364, 19, 'async-tokio-52-backpressure', '5.2 - Backpressure', NULL, 14),
(365, 19, 'async-tokio-61-serveur-http-basique', '6.1 - Serveur HTTP basique', NULL, 15),
(366, 19, 'async-tokio-62-routes-et-handlers', '6.2 - Routes et handlers', NULL, 16),
(367, 19, 'async-tokio-7-best-practices', '7. Best practices', NULL, 17),
(368, 19, 'async-tokio-3-limiter-la-concurrence', '• **3. Limiter la concurrence**', NULL, 18),
(369, 19, 'async-tokio-4-gerer-les-timeouts', '• **4. Gérer les timeouts**', NULL, 19),
(370, 19, 'async-tokio-5-cancel-safety', '• **5. Cancel safety**', NULL, 20),
(371, 19, 'async-tokio-6-arc-pour-le-partage', '• **6. Arc pour le partage**', NULL, 21),
(372, 19, 'async-tokio-7-profiler-avec-tokio-console', '• **7. Profiler avec tokio-console**', NULL, 22),
(373, 19, 'async-tokio-8-exercices-pratiques', '8. Exercices pratiques', NULL, 23),
(374, 19, 'async-tokio-aide-memoire', 'Aide-mémoire', NULL, 24);
INSERT INTO cour_block (id, chapitre_id, heading, content, block_type, sort_order) VALUES
(11034, 351, NULL, $BLK11034$Futures, Async Runtime et Concurrence$BLK11034$, 'text', 1),
(11035, 352, NULL, $BLK11035$```
// Code SYNCHRONE (bloquant)
fn fetch_data() -> String {
    std::thread::sleep(Duration::from_secs(2));
    "data".to_string()
}
fn main() {
    let data1 = fetch_data();  // Attend 2s
    let data2 = fetch_data();  // Attend encore 2s
    // Total : 4 secondes
}
// Code ASYNCHRONE (non-bloquant)
async fn fetch_data() -> String {
    tokio::time::sleep(Duration::from_secs(2)).await;
    "data".to_string()
}
#[tokio::main]
async fn main() {
    let future1 = fetch_data();
    let future2 = fetch_data();
    let (data1, data2) = tokio::join!(future1, future2);
    // Total : 2 secondes (parallèle !)
}
```$BLK11035$, 'code', 1),
(11036, 352, 'Async** ≠ **Multi-threading :', $BLK11036$Async permet de faire plusieurs choses **sans bloquer** , mais ne crée pas forcément plusieurs threads. 

**1.2 - Futures en Rust** 

```
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
// Une Future retourne Poll<Output>
trait Future {
    type Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context)
        -> Poll<Self::Output>;
}
// Poll peut être :
enum Poll<T> {
    Ready(T),      // La valeur est prête
    Pending,       // Pas encore prêt, revenir plus tard
}
// Les futures sont LAZY (ne font rien tant qu'on ne les .await pas)
async fn operation() -> i32 {
    42
}
let future = operation();  // Ne fait RIEN
let result = future.await; // MAINTENANT ça s'exécute
```$BLK11036$, 'text', 2),
(11037, 353, NULL, $BLK11037$```
// Fonction async
async fn dire_bonjour() {
    println!("Bonjour !");
}
// Retourne une Future
async fn calculer() -> i32 {
    42
}
// Utiliser .await pour attendre
async fn utiliser() {
    dire_bonjour().await;
    let resultat = calculer().await;
    println!("{}", resultat);
}
// async fn est du sucre syntaxique pour :
fn calculer() -> impl Future<Output = i32> {
    async { 42 }
}
// Chaîner des opérations async
async fn traiter() -> Result<String, Error> {
    let data = fetch_data().await?;
    let processed = process(data).await?;
    let saved = save(processed).await?;
    Ok(saved)
}
```$BLK11037$, 'code', 1),
(11038, 354, NULL, $BLK11038$```
# Dans Cargo.toml
[dependencies]
tokio = { version = "1", features = ["full"] }
```$BLK11038$, 'code', 1),
(11039, 354, NULL, $BLK11039$```
# Features spécifiques (plus léger)
tokio = { version = "1", features = ["rt-multi-thread", "macros", "net"] }
```$BLK11039$, 'code', 2),
(11040, 354, NULL, $BLK11040$```
# Features utiles :
# - rt-multi-thread : Runtime multi-thread
# - rt : Runtime single-thread
# - macros : #[tokio::main] et autres
# - net : TCP/UDP
# - fs : File system async
# - time : Sleep et timers
# - sync : Primitives de sync (Mutex, etc.)
```$BLK11040$, 'code', 3),
(11041, 355, NULL, $BLK11041$```
// Avec la macro (simple)
#[tokio::main]
async fn main() {
    println!("Hello async world!");
}
// Équivalent à :
fn main() {
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async {
            println!("Hello async world!");
        })
}
// Configuration custom du runtime
#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    // ...
}
```$BLK11041$, 'code', 1),
(11042, 355, NULL, $BLK11042$```
// Single-thread runtime
#[tokio::main(flavor = "current_thread")]
async fn main() {
    // Utile pour les tests ou petites apps
}
```$BLK11042$, 'code', 2),
(11043, 356, NULL, $BLK11043$```
use tokio::task;
#[tokio::main]
async fn main() {
    // Spawner une task (comme un thread léger)
    let handle = tokio::spawn(async {
        // Cette tâche s'exécute en parallèle
        println!("Dans une task !");
        42
    });
    // Attendre le résultat
    let result = handle.await.unwrap();
    println!("Résultat : {}", result);
    // Spawner plusieurs tasks
    let mut handles = vec![];
    for i in 0..10 {
        let handle = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(1)).await;
            println!("Task {} terminée", i);
            i
        });
        handles.push(handle);
    }
    // Attendre toutes les tasks
    for handle in handles {
        handle.await.unwrap();
    }
}
```$BLK11043$, 'code', 1),
(11044, 356, NULL, $BLK11044$II **tokio::spawn requiert 'static :** Les tasks doivent posséder leurs données ou utiliser Arc pour partager.$BLK11044$, 'text', 2),
(11045, 357, NULL, $BLK11045$```
use tokio::time::{sleep, interval, timeout, Duration};
// Sleep (non-bloquant)
tokio::time::sleep(Duration::from_secs(1)).await;
// Interval (répétitif)
let mut interval = interval(Duration::from_secs(1));
for _ in 0..5 {
    interval.tick().await;
    println!("Tick !");
}
// Timeout
let result = timeout(
    Duration::from_secs(5),
    longue_operation()
).await;
match result {
    Ok(value) => println!("Terminé : {:?}", value),
    Err(_) => println!("Timeout !"),
}
```$BLK11045$, 'code', 1),
(11046, 357, NULL, $BLK11046$```
// Deadline
use tokio::time::Instant;
let deadline = Instant::now() + Duration::from_secs(10);
tokio::time::sleep_until(deadline).await;
```$BLK11046$, 'code', 2),
(11047, 358, NULL, $BLK11047$```
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
```$BLK11047$, 'code', 1),
(11048, 358, NULL, $BLK11048$```
// Lire un fichier
let contenu = fs::read_to_string("fichier.txt").await?;
```$BLK11048$, 'code', 2),
(11049, 358, NULL, $BLK11049$```
// Écrire un fichier
fs::write("sortie.txt", "Hello async!").await?;
```$BLK11049$, 'code', 3),
(11050, 358, NULL, $BLK11050$```
// Lire avec buffer
let mut file = fs::File::open("data.txt").await?;
let mut buffer = Vec::new();
file.read_to_end(&mut buffer).await?;
```$BLK11050$, 'code', 4),
(11051, 358, NULL, $BLK11051$```
// Écrire avec buffer
let mut file = fs::File::create("output.txt").await?;
file.write_all(b"Hello").await?;
file.flush().await?;
```$BLK11051$, 'code', 5),
(11052, 358, NULL, $BLK11052$```
// Manipulations de fichiers
fs::rename("old.txt", "new.txt").await?;
fs::remove_file("temp.txt").await?;
fs::create_dir_all("path/to/dir").await?;
```$BLK11052$, 'code', 6),
(11053, 359, NULL, $BLK11053$```
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
// Serveur TCP simple
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Serveur démarré sur port 8080");
    loop {
        let (mut socket, addr) = listener.accept().await?;
        println!("Connexion de : {}", addr);
        tokio::spawn(async move {
            let mut buffer = [0; 1024];
            loop {
                let n = socket.read(&mut buffer).await.unwrap();
                if n == 0 {
                    return;
                }
                socket.write_all(&buffer[0..n]).await.unwrap();
            }
        });
    }
}
// Client TCP
async fn connect() -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    stream.write_all(b"Hello server!").await?;
```$BLK11053$, 'code', 1),
(11054, 359, NULL, $BLK11054$```
    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer).await?;
    println!("Reçu : {}", String::from_utf8_lossy(&buffer[..n]));
    Ok(())
}
```$BLK11054$, 'code', 2),
(11055, 360, NULL, $BLK11055$```
use tokio::{join, select};
```$BLK11055$, 'code', 1),
(11056, 360, NULL, $BLK11056$```
// join! attend TOUTES les futures
async fn exemple_join() {
    let (res1, res2, res3) = join!(
        fetch_data1(),
        fetch_data2(),
        fetch_data3()
    );
    // Les 3 s'exécutent en parallèle
}
```$BLK11056$, 'code', 2),
(11057, 360, NULL, $BLK11057$```
// select! prend la PREMIÈRE qui termine
async fn exemple_select() {
    select! {
        result = fetch_data() => {
            println!("Data: {:?}", result);
        }
```$BLK11057$, 'code', 3),
(11058, 360, NULL, $BLK11058$```
        _ = tokio::time::sleep(Duration::from_secs(5)) => {
            println!("Timeout !");
        }
    }
}
// try_join! pour Result
use tokio::try_join;
```$BLK11058$, 'code', 4),
(11059, 360, NULL, $BLK11059$```
async fn exemple_try_join() -> Result<(), Error> {
    let (res1, res2) = try_join!(
        async_operation1(),
        async_operation2()
    )?;
```$BLK11059$, 'code', 5),
(11060, 360, NULL, $BLK11060$```
    Ok(())
}
```$BLK11060$, 'code', 6),
(11061, 361, NULL, $BLK11061$```
use tokio::sync::mpsc;
```$BLK11061$, 'code', 1),
(11062, 361, NULL, $BLK11062$```
#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);  // Buffer de 32
    // Spawner le producteur
    tokio::spawn(async move {
        for i in 0..10 {
            tx.send(i).await.unwrap();
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });
    // Consommateur
    while let Some(value) = rx.recv().await {
        println!("Reçu : {}", value);
    }
}
// unbounded channel (sans limite)
use tokio::sync::mpsc::unbounded_channel;
```$BLK11062$, 'code', 2),
(11063, 361, NULL, $BLK11063$```
let (tx, mut rx) = unbounded_channel();
```$BLK11063$, 'code', 3),
(11064, 361, NULL, $BLK11064$```
// oneshot (un seul message)
use tokio::sync::oneshot;
let (tx, rx) = oneshot::channel();
```$BLK11064$, 'code', 4),
(11065, 361, NULL, $BLK11065$```
tokio::spawn(async move {
    tx.send(42).unwrap();
});
let result = rx.await.unwrap();
```$BLK11065$, 'code', 5),
(11066, 362, NULL, $BLK11066$```
use tokio::sync::{Mutex, RwLock};
use std::sync::Arc;
// Mutex pour async
#[tokio::main]
async fn main() {
    let data = Arc::new(Mutex::new(0));
    let mut handles = vec![];
    for _ in 0..10 {
        let data = Arc::clone(&data);
        let handle = tokio::spawn(async move {
            let mut num = data.lock().await;
            *num += 1;
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.await.unwrap();
    }
    println!("Résultat : {}", *data.lock().await);
}
// RwLock (multiple readers, single writer)
let lock = Arc::new(RwLock::new(String::from("hello")));
// Lecture (multiple simultanés)
let read = lock.read().await;
println!("{}", *read);
// Écriture (exclusif)
let mut write = lock.write().await;
write.push_str(" world");
```$BLK11066$, 'code', 1),
(11067, 363, NULL, $BLK11067$```
use tokio_stream::{Stream, StreamExt};
```$BLK11067$, 'code', 1),
(11068, 363, NULL, $BLK11068$```
// Stream = Iterator async
async fn exemple_stream() {
    let mut stream = tokio_stream::iter(vec![1, 2, 3, 4, 5]);
    while let Some(value) = stream.next().await {
        println!("{}", value);
    }
}
// Créer un stream custom
use async_stream::stream;
fn number_stream() -> impl Stream<Item = i32> {
    stream! {
        for i in 0..10 {
            tokio::time::sleep(Duration::from_millis(100)).await;
            yield i;
        }
    }
}
// Transformer des streams
async fn traiter_stream() {
    let stream = number_stream()
        .filter(|x| x % 2 == 0)
        .map(|x| x * 2);
    tokio::pin!(stream);
    while let Some(value) = stream.next().await {
        println!("{}", value);
    }
}
```$BLK11068$, 'code', 2),
(11069, 364, NULL, $BLK11069$```
use tokio::sync::Semaphore;
use std::sync::Arc;
// Limiter le nombre de requêtes simultanées
#[tokio::main]
async fn main() {
    let semaphore = Arc::new(Semaphore::new(5));  // Max 5 simultanés
    let mut handles = vec![];
    for i in 0..100 {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let handle = tokio::spawn(async move {
            // Faire le travail
            process(i).await;
            drop(permit);  // Libère le slot
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.await.unwrap();
    }
}
// Rate limiting
use tokio::time::{interval, Duration};
```$BLK11069$, 'code', 1),
(11070, 364, NULL, $BLK11070$```
let mut interval = interval(Duration::from_millis(100));
```$BLK11070$, 'code', 2),
(11071, 364, NULL, $BLK11071$```
for request in requests {
    interval.tick().await;  // Attend avant chaque requête
    process(request).await;
}
```$BLK11071$, 'code', 3),
(11072, 365, NULL, $BLK11072$```
# Dans Cargo.toml
[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
# Code
use axum::{
    routing::get,
    Router,
};
#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(handler));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    println!("Serveur sur http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
async fn handler() -> &'static str {
    "Hello, World!"
}
```$BLK11072$, 'code', 1),
(11073, 366, NULL, $BLK11073$```
use axum::{
    extract::{Path, Query, Json},
    response::IntoResponse,
    http::StatusCode,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
#[derive(Deserialize)]
struct CreateUser {
    username: String,
    email: String,
}
#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}
async fn create_user(
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<User>) {
    let user = User {
        id: 1,
        username: payload.username,
    };
    (StatusCode::CREATED, Json(user))
}
async fn get_user(Path(id): Path<u64>) -> Json<User> {
    Json(User {
        id,
        username: "Alice".to_string(),
    })
}
```$BLK11073$, 'code', 1),
(11074, 366, NULL, $BLK11074$```
#[tokio::main]
```$BLK11074$, 'code', 2),
(11075, 366, NULL, $BLK11075$```
async fn main() {
    let app = Router::new()
        .route("/users", post(create_user))
        .route("/users/:id", get(get_user));
```$BLK11075$, 'code', 3),
(11076, 366, NULL, $BLK11076$```
    // ...
}
```$BLK11076$, 'code', 4),
(11077, 367, NULL, $BLK11077$• **1. Ne pas bloquer le runtime** Évite les opérations CPU-intensives dans les tasks async. Utilise `tokio::task::spawn_blocking` .$BLK11077$, 'text', 1),
(11078, 367, NULL, $BLK11078$• **2. Utiliser des channels pour communiquer** Préfère les channels aux Mutex quand c'est possible.$BLK11078$, 'text', 2),
(11079, 368, NULL, $BLK11079$Utilise Semaphore pour éviter de surcharger le système.$BLK11079$, 'text', 1),
(11080, 369, NULL, $BLK11080$Toujours ajouter des timeouts aux opérations réseau.$BLK11080$, 'text', 1),
(11081, 370, NULL, $BLK11081$Assure-toi que tes futures peuvent être annulées proprement.$BLK11081$, 'text', 1),
(11082, 371, NULL, $BLK11082$Utilise `Arc` pour partager des données entre tasks.$BLK11082$, 'text', 1),
(11083, 372, NULL, $BLK11083$Utilise tokio-console pour déboguer les performances.$BLK11083$, 'text', 1),
(11084, 373, 'Exercice 1 : Paralléliser des requêtes', $BLK11084$```
// Écris une fonction qui fetch plusieurs URLs en parallèle
```

```
use reqwest;
```

```
async fn fetch_all(urls: Vec<String>) -> Vec<Result<String, reqwest::Error>> {
    let mut handles = vec![];
```

```
    for url in urls {
        let handle = tokio::spawn(async move {
            reqwest::get(&url)
                .await?
                .text()
                .await
        });
        handles.push(handle);
    }
    let mut results = vec![];
    for handle in handles {
        results.push(handle.await.unwrap());
    }
    results
}
```$BLK11084$, 'code', 1),
(11085, 373, 'Exercice 2 : Rate limiter', $BLK11085$```
// Crée un rate limiter qui limite à N requêtes/seconde
use tokio::time::{interval, Duration};
```

```
struct RateLimiter {
    interval: tokio::time::Interval,
}
impl RateLimiter {
    fn new(requests_per_second: u64) -> Self {
        let duration = Duration::from_secs(1) / requests_per_second as u32;
        Self {
            interval: interval(duration),
        }
    }
    async fn acquire(&mut self) {
        self.interval.tick().await;
    }
}
// Utilisation
#[tokio::main]
async fn main() {
    let mut limiter = RateLimiter::new(10);  // 10 req/s
    for i in 0..100 {
        limiter.acquire().await;
        println!("Requête {}", i);
    }
}
```$BLK11085$, 'code', 2),
(11086, 374, NULL, $BLK11086$|**Concept**|**Usage**|
|---|---|
|**`async fn`**|Fonction asynchrone|
|**`.await`**|Attendre une future|
|**`tokio::spawn`**|Créer une task|
|**`tokio::join!`**|Attendre plusieurs futures|
|**`tokio::select!`**|Première future terminée|
|**`mpsc::channel`**|Communication entre tasks|
|**`Arc<Mutex<T>>`**|Partage mutable|$BLK11086$, 'table', 1),
(11087, 374, NULL, $BLK11087$- **async/await** = programmation non-bloquante$BLK11087$, 'list', 2),
(11088, 374, NULL, $BLK11088$- **Tokio** = runtime pour exécuter les futures$BLK11088$, 'list', 3),
(11089, 374, NULL, $BLK11089$- **spawn** = créer des tâches concurrentes$BLK11089$, 'list', 4),
(11090, 374, NULL, $BLK11090$- **join!** = attendre plusieurs futures$BLK11090$, 'list', 5),
(11091, 374, NULL, $BLK11091$- **channels** = communication entre tasks$BLK11091$, 'list', 6),
(11092, 374, NULL, $BLK11092$- **Semaphore** = limiter la concurrence$BLK11092$, 'list', 7),
(11093, 374, 'INCROYABLE !', $BLK11093$Tu as terminé TOUS les cours Rust ! Tu maîtrises maintenant : I Variables, fonctions, ownership I Structures, enums, pattern matching I Collections et itérateurs I Gestion d'erreurs avancée I Lifetimes I Modules et organisation I Traits avancés I Async/await et Tokio I **TU ES UN EXPERT RUST !** I$BLK11093$, 'text', 8);

-- orm.md (cour_id=20)
INSERT INTO chapitre (id, cour_id, slug, title, lead, sort_order) VALUES
(375, 20, 'orm-creer-une-api-django-like-en-rust', 'Créer une API Django-like en Rust', NULL, 1),
(376, 20, 'orm-1-le-probleme-initial', '1. Le problème initial', NULL, 2),
(377, 20, 'orm-21-les-traits-interfaces', '2.1 - Les traits (interfaces)', NULL, 3),
(378, 20, 'orm-22-les-generiques', '2.2 - Les génériques', NULL, 4),
(379, 20, 'orm-23-phantomdata', '2.3 - PhantomData', NULL, 5),
(380, 20, 'orm-24-const-fn', '2.4 - const fn', NULL, 6),
(381, 20, 'orm-25-les-macros', '2.5 - Les macros', NULL, 7),
(382, 20, 'orm-3-architecture-de-la-solution', '3. Architecture de la solution', NULL, 8),
(383, 20, 'orm-flux-de-donnees', 'Flux de données :', NULL, 9),
(384, 20, 'orm-41-objectsrs-le-manager', '4.1 - objects.rs (le Manager)', NULL, 10),
(385, 20, 'orm-42-queryrs-le-querybuilder', '4.2 - query.rs (le QueryBuilder)', NULL, 11),
(386, 20, 'orm-43-la-macro-impl-objects', '4.3 - La macro impl_objects!', NULL, 12),
(387, 20, 'orm-44-into-la-conversion-magique', '4.4 - Into : La conversion magique', NULL, 13),
(388, 20, 'orm-5-exemples-dutilisation', '5. Exemples d''utilisation', NULL, 14),
(389, 20, 'orm-6-exercices-pratiques', '6. Exercices pratiques', NULL, 15),
(390, 20, 'orm-exercice-1-ajouter-first', 'Exercice 1 : Ajouter first()', NULL, 16),
(391, 20, 'orm-exercice-2-ajouter-exists', 'Exercice 2 : Ajouter exists()', NULL, 17),
(392, 20, 'orm-7-resume-des-concepts', '7. Résumé des concepts', NULL, 18),
(393, 20, 'orm-points-cles-a-retenir', 'Points clés à retenir :', NULL, 19),
(394, 20, 'orm-felicitations', 'Félicitations !', NULL, 20),
(395, 20, 'orm-ressources-pour-aller-plus-loin', 'Ressources pour aller plus loin :', NULL, 21);
INSERT INTO cour_block (id, chapitre_id, heading, content, block_type, sort_order) VALUES
(11094, 375, NULL, $BLK11094$Extension ORM avec Traits, Génériques et Macros$BLK11094$, 'text', 1),
(11095, 375, NULL, $BLK11095$Framework Runique - Documentation Complète$BLK11095$, 'text', 2),
(11096, 375, 'Objectif du cours', $BLK11096$Comprendre comment créer une API Django-like en Rust pour avoir User::objects.filter() au lieu de la syntaxe verbeuse de SeaORM.$BLK11096$, 'text', 3),
(11097, 376, NULL, $BLK11097$Lorsqu'on utilise Django en Python, on a une syntaxe très intuitive pour les requêtes de base de données :$BLK11097$, 'text', 1),
(11098, 376, NULL, $BLK11098$```
# Django (Python) - Simple et intuitif
User.objects.filter(age__gte=18)
User.objects.exclude(status="banned")
User.objects.get(id=1)
```$BLK11098$, 'code', 2),
(11099, 376, NULL, $BLK11099$En revanche, avec SeaORM en Rust, la syntaxe de base est plus verbeuse :$BLK11099$, 'text', 3),
(11100, 376, NULL, $BLK11100$```
// SeaORM (Rust) - Verbeux
User::find()
    .filter(user::Column::Age.gte(18))
    .all(&db)
    .await?
```$BLK11100$, 'code', 4),
(11101, 376, NULL, $BLK11101$**Notre objectif :** Avoir la même syntaxe qu'en Django avec User::objects.filter() en Rust !$BLK11101$, 'text', 5),
(11102, 377, NULL, $BLK11102$Un **trait** en Rust est similaire à une interface : c'est un ensemble de méthodes qu'un type peut implémenter. Les traits permettent d'ajouter des méthodes à des types existants.$BLK11102$, 'text', 1),
(11103, 377, NULL, $BLK11103$```
// Définir un trait
trait Parler {
    fn dire_bonjour(&self);
}
```$BLK11103$, 'code', 2),
(11104, 377, NULL, $BLK11104$```
// Implémenter pour un type
struct Personne {
    nom: String,
}
```$BLK11104$, 'code', 3),
(11105, 377, NULL, $BLK11105$```
impl Parler for Personne {
    fn dire_bonjour(&self) {
        println!("Bonjour, je suis {}", self.nom);
    }
}
```$BLK11105$, 'code', 4),
(11106, 377, NULL, $BLK11106$```
// Utilisation
let p = Personne { nom: "Alice".to_string() };
p.dire_bonjour();  // "Bonjour, je suis Alice"
```$BLK11106$, 'code', 5),
(11107, 377, NULL, $BLK11107$I **Pourquoi c'est important ?** Les traits permettent d'ajouter des méthodes à des types existants sans modifier leur code source !$BLK11107$, 'text', 6),
(11108, 378, NULL, $BLK11108$Les **génériques** permettent d'écrire du code qui fonctionne avec plusieurs types différents.$BLK11108$, 'text', 1),
(11109, 378, NULL, $BLK11109$```
// Sans générique (répétitif)
struct BoiteEntier { contenu: i32 }
struct BoiteString { contenu: String }
```$BLK11109$, 'code', 2),
(11110, 378, NULL, $BLK11110$```
// Avec générique (réutilisable)
struct Boite<T> {
    contenu: T,
}
// Utilisation
let boite_int = Boite { contenu: 42 };
let boite_str = Boite { contenu: "Hello".to_string() };
```$BLK11110$, 'code', 3),
(11111, 378, NULL, $BLK11111$```
// Avec contraintes (bounds)
fn afficher<T: std::fmt::Display>(valeur: T) {
    println!("Valeur: {}", valeur);
}
```$BLK11111$, 'code', 4),
(11112, 379, NULL, $BLK11112$`PhantomData<T>` permet de dire au compilateur "je possède un type T" **sans stocker de données réelles** . C'est un type fantôme de taille zéro.$BLK11112$, 'text', 1),
(11113, 379, NULL, $BLK11113$```
use std::marker::PhantomData;
```$BLK11113$, 'code', 2),
(11114, 379, NULL, $BLK11114$```
struct Manager<E> {
    // On ne stocke PAS de E réellement
    // Mais on dit au compilateur qu'on "possède" un E
    _phantom: PhantomData<E>,
```$BLK11114$, 'code', 3),
(11115, 379, NULL, $BLK11115$```
}
```$BLK11115$, 'code', 4),
(11116, 379, NULL, $BLK11116$```
impl<E> Manager<E> {
    const fn new() -> Self {
        Self { _phantom: PhantomData }
    }
```$BLK11116$, 'code', 5),
(11117, 379, NULL, $BLK11117$```
}
```$BLK11117$, 'code', 6),
(11118, 379, NULL, $BLK11118$I **Avantages :** Le compilateur vérifie les types correctement, mais aucune donnée n'est stockée en mémoire (taille = 0 octets).$BLK11118$, 'text', 7),
(11119, 380, NULL, $BLK11119$`const fn` définit une fonction qui peut être évaluée **à la compilation** plutôt qu'à l'exécution.$BLK11119$, 'text', 1),
(11120, 380, NULL, $BLK11120$```
const fn multiplier(x: i32) -> i32 {
    x * 2
```$BLK11120$, 'code', 2),
(11121, 380, NULL, $BLK11121$```
}
```$BLK11121$, 'code', 3),
(11122, 380, NULL, $BLK11122$```
// Calculé à la compilation !
const RESULTAT: i32 = multiplier(5);
```$BLK11122$, 'code', 4),
(11123, 380, NULL, $BLK11123$```
// Pour notre cas :
pub const objects: Manager<Self> = Manager::new();
//    ^^^^^ constante, pas une fonction
```$BLK11123$, 'code', 5),
(11124, 380, NULL, $BLK11124$I Cela permet de créer `objects` comme une **constante** , accessible sans parenthèses : `User::objects`$BLK11124$, 'text', 6),
(11125, 381, NULL, $BLK11125$Les **macros** permettent de générer du code automatiquement. Elles se terminent par un point d'exclamation `!`$BLK11125$, 'text', 1),
(11126, 381, NULL, $BLK11126$```
// Définir une macro
macro_rules! dire_bonjour {
    ($nom:expr) => {
        println!("Bonjour {}", $nom);
    };
}
```$BLK11126$, 'code', 2),
(11127, 381, NULL, $BLK11127$```
// Utilisation
dire_bonjour!("Alice");
```$BLK11127$, 'code', 3),
(11128, 381, NULL, $BLK11128$```
// Se transforme en :
println!("Bonjour {}", "Alice");
```$BLK11128$, 'code', 4),
(11129, 381, NULL, $BLK11129$```
// Notre macro impl_objects! :
impl_objects!(User);
```$BLK11129$, 'code', 5),
(11130, 381, NULL, $BLK11130$```
// Génère automatiquement :
impl User {
    pub const objects: Objects<Self> = Objects::new();
}
```$BLK11130$, 'code', 6),
(11131, 382, NULL, $BLK11131$Notre solution utilise trois composants principaux qui travaillent ensemble :$BLK11131$, 'text', 1),
(11132, 382, NULL, $BLK11132$**==> picture [328 x 211] intentionally omitted <==**$BLK11132$, 'text', 2),
(11133, 382, NULL, $BLK11133$**----- Start of picture text -----**<br>
IIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIII<br>I   User (entité SeaORM)                    I<br>I   + impl_objects!(Entity)                 I<br>IIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIII<br>I<br>M<br>IIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIII<br>I   Objects<User>                           I<br>I   - Constante créée par la macro          I<br>I   - Méthodes: filter(), exclude(), etc.   I<br>IIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIII<br>I<br>M<br>IIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIII<br>I   RuniqueQueryBuilder<User>                 I<br>I   - Encapsule Select<User>                I<br>I   - Méthodes chainables                   I<br>IIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIII<br>I<br>M<br>IIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIII<br>I   SeaORM Select<User>                     I<br>I   - Query SQL réelle                      I<br>IIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIII<br>**----- End of picture text -----**<br>$BLK11133$, 'text', 3),
(11134, 383, NULL, $BLK11134$1. **Macro** : Génère la constante `objects` pour chaque entité$BLK11134$, 'list', 1),
(11135, 383, NULL, $BLK11135$2. **Objects** : Point d'entrée (comme Django Manager), crée des QueryBuilder$BLK11135$, 'list', 2),
(11136, 383, NULL, $BLK11136$3. **QueryBuilder** : Permet le chaînage de méthodes$BLK11136$, 'list', 3),
(11137, 383, NULL, $BLK11137$4. **Select** : Query SeaORM réelle exécutée sur la base de données$BLK11137$, 'list', 4),
(11138, 384, NULL, $BLK11138$Le fichier `objects.rs` contient la struct `Objects<E>` qui sert de point d'entrée pour toutes les requêtes.$BLK11138$, 'text', 1),
(11139, 384, NULL, $BLK11139$```
use std::marker::PhantomData;
```$BLK11139$, 'code', 2),
(11140, 384, NULL, $BLK11140$```
// Struct générique qui fonctionne avec N'IMPORTE quelle entité
pub struct Objects<E: EntityTrait> {
    //                ^^^^^^^^^^^ E doit être une entité SeaORM
```$BLK11140$, 'code', 3),
(11141, 384, NULL, $BLK11141$```
    _phantom: PhantomData<E>,
```$BLK11141$, 'code', 4),
(11142, 384, NULL, $BLK11142$```
    // ^^^^^^^ On stocke le type E sans données réelles
}
```$BLK11142$, 'code', 5),
(11143, 384, NULL, $BLK11143$```
impl<E: EntityTrait> Objects<E> {
    // Pour chaque type E qui implémente EntityTrait
```$BLK11143$, 'code', 6),
(11144, 384, NULL, $BLK11144$```
    pub const fn new() -> Self {
```$BLK11144$, 'code', 7),
(11145, 384, NULL, $BLK11145$```
        // const fn = peut être appelé à la compilation
        Self { _phantom: PhantomData }
    }
```$BLK11145$, 'code', 8),
(11146, 384, NULL, $BLK11146$```
    pub fn filter<C>(&self, condition: C) -> RuniqueQueryBuilder<E>
    //            ^^ C peut être n'importe quoi convertible en Condition
    where
```$BLK11146$, 'code', 9),
(11147, 384, NULL, $BLK11147$```
        C: Into<Condition>,
```$BLK11147$, 'code', 10),
(11148, 384, NULL, $BLK11148$```
        // ^^^^^^^^^^^^^^ Contrainte : C doit pouvoir devenir Condition
    {
```$BLK11148$, 'code', 11),
(11149, 384, NULL, $BLK11149$```
        // 1. Créer une query SeaORM
        let query = E::find();
```$BLK11149$, 'code', 12),
(11150, 384, NULL, $BLK11150$```
        // 2. L'envelopper dans notre QueryBuilder
        // 3. Appliquer le filtre
        RuniqueQueryBuilder::new(query).filter(condition.into())
        //                                              ^^^^^^ Conversion auto
    }
```$BLK11150$, 'code', 13),
(11151, 384, NULL, $BLK11151$```
}
```$BLK11151$, 'code', 14),
(11152, 384, NULL, $BLK11152$I **Analogie :** `Objects<E>` est comme une **télécommande** pour contrôler `E` .$BLK11152$, 'text', 15),
(11153, 385, NULL, $BLK11153$Le `RuniqueQueryBuilder` encapsule la query SeaORM et permet de chaîner les méthodes.$BLK11153$, 'text', 1),
(11154, 385, NULL, $BLK11154$```
pub struct RuniqueQueryBuilder<E: EntityTrait> {
    select: Select<E>,  // La vraie query SeaORM
}
```$BLK11154$, 'code', 2),
(11155, 385, NULL, $BLK11155$```
impl<E: EntityTrait> RuniqueQueryBuilder<E> {
    pub fn new(select: Select<E>) -> Self {
        Self { select }
    }
    // Méthode chainable
    pub fn filter<C>(mut self, condition: C) -> Self
    //           ^^^ Prend ownership
    where
        C: Into<Condition>,
    {
        // Modifier la query interne
        self.select = self.select.filter(condition.into());
        // Retourner self pour permettre le chaînage
        self
        // ^^^^ Rend ownership
    }
```$BLK11155$, 'code', 3),
(11156, 385, NULL, $BLK11156$```
    // Méthode terminale (consomme self)
    pub async fn all(self, db: &DatabaseConnection)
        -> Result<Vec<E::Model>, DbErr>
    {
        //    ^^^^ Consomme self (pas de chaînage après)
        self.select.all(db).await
    }
}
```$BLK11156$, 'code', 4),
(11157, 385, NULL, $BLK11157$I **Pattern Builder :** Les méthodes qui retournent `Self` sont **chainables** , celles qui consomment `self` sont **terminales** .$BLK11157$, 'text', 5),
(11158, 385, NULL, $BLK11158$```
// Chainable car retourne Self
query.filter(...).exclude(...).limit(10)
```$BLK11158$, 'code', 6),
(11159, 385, NULL, $BLK11159$```
// Terminal car consomme self
```$BLK11159$, 'code', 7),
(11160, 385, NULL, $BLK11160$```
     .all(&db).await
```$BLK11160$, 'code', 8),
(11161, 386, NULL, $BLK11161$La macro génère automatiquement la constante `objects` pour chaque entité.$BLK11161$, 'text', 1),
(11162, 386, NULL, $BLK11162$```
#[macro_export]
//^^^^^^^^^^^ La macro est disponible partout
macro_rules! impl_objects {
    //        ^^^^^^^^^^^^ Nom de la macro
```$BLK11162$, 'code', 2),
(11163, 386, NULL, $BLK11163$```
    ($entity:ty) => {
    // ^^^^^^^ Paramètre : un type
```$BLK11163$, 'code', 3),
(11164, 386, NULL, $BLK11164$```
        impl $entity {
        //   ^^^^^^^ Utilise le paramètre
```$BLK11164$, 'code', 4),
(11165, 386, NULL, $BLK11165$```
            pub const objects: $crate::orm::Objects<Self>
                = $crate::orm::Objects::new();
            //  ^^^^^^
            //  Nom, Type générique, Création const
        }
    };
}
// Utilisation :
impl_objects!(Entity);
```$BLK11165$, 'code', 5),
(11166, 386, NULL, $BLK11166$```
// Se transforme en :
impl Entity {
    pub const objects: rusti::orm::Objects<Self>
        = rusti::orm::Objects::new();
```$BLK11166$, 'code', 6),
(11167, 386, NULL, $BLK11167$```
}
```$BLK11167$, 'code', 7),
(11168, 387, NULL, $BLK11168$Le trait `Into<Condition>` permet la conversion automatique des expressions SeaORM en conditions.$BLK11168$, 'text', 1),
(11169, 387, NULL, $BLK11169$```
pub fn filter<C>(&self, condition: C) -> RuniqueQueryBuilder<E>
where
```$BLK11169$, 'code', 2),
(11170, 387, NULL, $BLK11170$```
    C: Into<Condition>,
    //^^^^^^^^^^^^^^^^ Le secret !
```$BLK11170$, 'code', 3),
(11171, 387, NULL, $BLK11171$```
// SeaORM retourne Expr pour les comparaisons :
Column::Age.gte(18)  // Type: Expr
```$BLK11171$, 'code', 4),
(11172, 387, NULL, $BLK11172$```
// Mais filter() attend Condition
```$BLK11172$, 'code', 5),
(11173, 387, NULL, $BLK11173$```
// Into<Condition> permet la conversion automatique :
```$BLK11173$, 'code', 6),
(11174, 387, NULL, $BLK11174$```
// L'utilisateur écrit :
```$BLK11174$, 'code', 7),
(11175, 387, NULL, $BLK11175$```
.filter(Column::Age.gte(18))
```$BLK11175$, 'code', 8),
(11176, 387, NULL, $BLK11176$```
// Rust convertit automatiquement :
```$BLK11176$, 'code', 9),
(11177, 387, NULL, $BLK11177$```
.filter(Column::Age.gte(18).into())
```$BLK11177$, 'code', 10),
(11178, 387, NULL, $BLK11178$```
//                          ^^^^^^ Ajouté automatiquement
```$BLK11178$, 'code', 11),
(11179, 388, NULL, $BLK11179$Une fois configuré, voici comment utiliser l'API :$BLK11179$, 'text', 1),
(11180, 388, NULL, $BLK11180$```
// 1. Dans ton entité SeaORM
use rusti::impl_objects;
```$BLK11180$, 'code', 2),
(11181, 388, NULL, $BLK11181$```
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub username: String,
    pub age: i32,
}
```$BLK11181$, 'code', 3),
(11182, 388, NULL, $BLK11182$```
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
```$BLK11182$, 'code', 4),
(11183, 388, NULL, $BLK11183$```
impl ActiveModelBehavior for ActiveModel {}
```$BLK11183$, 'code', 5),
(11184, 388, NULL, $BLK11184$**`//`** I **`Ajouter le support objects impl_objects!(Entity);`**$BLK11184$, 'text', 6),
(11185, 388, NULL, $BLK11185$```
// 2. Utilisation dans le code
```$BLK11185$, 'code', 7),
(11186, 388, NULL, $BLK11186$```
// Tous les utilisateurs
let users = User::objects.all().all(&db).await?;
```$BLK11186$, 'code', 8),
(11187, 388, NULL, $BLK11187$```
// Filtrer
let adults = User::objects
    .filter(user::Column::Age.gte(18))
    .all(&db)
    .await?;
// Exclure
let active = User::objects
    .exclude(user::Column::Status.eq("banned"))
    .all(&db)
    .await?;
```$BLK11187$, 'code', 9),
(11188, 388, NULL, $BLK11188$```
// Get par ID
let user = User::objects.get(&db, 1).await?;
// Compter
let count = User::objects.count(&db).await?;
```$BLK11188$, 'code', 10),
(11189, 388, NULL, $BLK11189$```
// Query complexe avec chaînage
let results = User::objects
    .filter(user::Column::Age.gte(18))
    .exclude(user::Column::Status.eq("banned"))
    .order_by_desc(user::Column::CreatedAt)
    .limit(10)
    .offset(20)
    .all(&db)
    .await?;
```$BLK11189$, 'code', 11),
(11190, 389, NULL, $BLK11190$Pour approfondir ta compréhension, voici quelques exercices :$BLK11190$, 'text', 1),
(11191, 390, NULL, $BLK11191$Ajoute une méthode `first()` qui retourne le premier résultat.$BLK11191$, 'text', 1),
(11192, 390, NULL, $BLK11192$```
// Dans objects.rs
```$BLK11192$, 'code', 2),
(11193, 390, NULL, $BLK11193$```
pub async fn first(&self, db: &DatabaseConnection)
    -> Result<Option<E::Model>, DbErr>
{
```$BLK11193$, 'code', 3),
(11194, 390, NULL, $BLK11194$```
    E::find().one(db).await
}
```$BLK11194$, 'code', 4),
(11195, 390, NULL, $BLK11195$```
// Utilisation :
let premier = User::objects.first(&db).await?;
```$BLK11195$, 'code', 5),
(11196, 391, NULL, $BLK11196$Crée une méthode `exists()` qui vérifie si des résultats existent.$BLK11196$, 'text', 1),
(11197, 391, NULL, $BLK11197$```
// Dans query.rs
```$BLK11197$, 'code', 2),
(11198, 391, NULL, $BLK11198$- **`pub async fn exists(self, db: &DatabaseConnection) -> Result<bool, DbErr>`**$BLK11198$, 'list', 3),
(11199, 391, NULL, $BLK11199$```
{
    let count = self.count(db).await?;
    Ok(count > 0)
}
```$BLK11199$, 'code', 4),
(11200, 391, NULL, $BLK11200$```
// Utilisation :
let existe = User::objects
    .filter(user::Column::Username.eq("alice"))
    .exists(&db)
    .await?;
```$BLK11200$, 'code', 5),
(11201, 392, NULL, $BLK11201$Voici un tableau récapitulatif des concepts Rust utilisés :$BLK11201$, 'text', 1),
(11202, 392, NULL, $BLK11202$|**Concept**|**Utilité**|**Exemple**|
|---|---|---|
|**Trait**|Ajouter méthodes à types|impl MonTrait for MaStruct|
|**Générique**|Code réutilisable|struct Box<T>|
|**PhantomData**|Type sans données|PhantomData<E>|
|**const fn**|Eval à compilation|const fn new()|
|**Macro**|Générer code|macro_rules! impl_objects|
|**Into<T>**|Conversion auto|C: Into<Condition>|
|**Builder**|Méthodes chainables|filter().exclude()|$BLK11202$, 'table', 2),
(11203, 393, NULL, $BLK11203$- **Traits** : Permettent d'étendre des types existants$BLK11203$, 'list', 1),
(11204, 393, NULL, $BLK11204$- **Génériques** : Rendent le code réutilisable pour plusieurs types$BLK11204$, 'list', 2),
(11205, 393, NULL, $BLK11205$- **PhantomData** : Type fantôme de taille zéro pour la vérification de types$BLK11205$, 'list', 3),
(11206, 393, NULL, $BLK11206$- **const fn** : Évaluation à la compilation pour créer des constantes$BLK11206$, 'list', 4),
(11207, 393, NULL, $BLK11207$- **Macros** : Génération automatique de code répétitif$BLK11207$, 'list', 5),
(11208, 393, NULL, $BLK11208$- **Into<T>** : Conversions automatiques entre types$BLK11208$, 'list', 6),
(11209, 393, NULL, $BLK11209$- **Builder pattern** : Chaînage de méthodes pour une API fluide$BLK11209$, 'list', 7),
(11210, 394, NULL, $BLK11210$Tu as maintenant compris comment créer une API Django-like en Rust ! Continue à expérimenter, à casser des choses et à apprendre. I **La communauté Rust est là pour t'aider !** I$BLK11210$, 'text', 1),
(11211, 395, NULL, $BLK11211$I The Rust Book : https://doc.rust-lang.org/book/$BLK11211$, 'text', 1),
(11212, 395, NULL, $BLK11212$I Rust by Example : https://doc.rust-lang.org/rust-by-example/$BLK11212$, 'text', 2),
(11213, 395, NULL, $BLK11213$I SeaORM Docs : https://www.sea-ql.org/SeaORM/$BLK11213$, 'text', 3),
(11214, 395, NULL, $BLK11214$I Forum Rust : https://users.rust-lang.org/$BLK11214$, 'text', 4);

-- cours-filtre-admin.md (cour_id=21)
INSERT INTO chapitre (id, cour_id, slug, title, lead, sort_order) VALUES
(396, 21, 'cours-filtre-admin-systeme-de-filtres-admin-comment-ca-marche', 'Système de filtres admin — Comment ça marche', NULL, 1),
(397, 21, 'cours-filtre-admin-vue-densemble', 'Vue d''ensemble', NULL, 2),
(398, 21, 'cours-filtre-admin-etape-1-declarer-un-filtre', 'Étape 1 — Déclarer un filtre', NULL, 3),
(399, 21, 'cours-filtre-admin-etape-2-le-parser', 'Étape 2 — Le parser', NULL, 4),
(400, 21, 'cours-filtre-admin-etape-3-le-generateur', 'Étape 3 — Le générateur', NULL, 5),
(401, 21, 'cours-filtre-admin-la-configuration-daffichage', 'La configuration d''affichage', NULL, 6),
(402, 21, 'cours-filtre-admin-la-closure-de-filtres', 'La closure de filtres', NULL, 7),
(403, 21, 'cours-filtre-admin-etape-4-le-handler', 'Étape 4 — Le handler', NULL, 8),
(404, 21, 'cours-filtre-admin-etape-5-la-pagination-des-filtres', 'Étape 5 — La pagination des filtres', NULL, 9),
(405, 21, 'cours-filtre-admin-etape-6-le-template', 'Étape 6 — Le template', NULL, 10),
(406, 21, 'cours-filtre-admin-etape-7-repli-par-groupe', 'Étape 7 — Repli par groupe', NULL, 11),
(407, 21, 'cours-filtre-admin-etape-8-diagnostic', 'Étape 8 — Diagnostic', NULL, 12),
(408, 21, 'cours-filtre-admin-resume-des-choix-de-conception', 'Résumé des choix de conception', NULL, 13);
INSERT INTO cour_block (id, chapitre_id, heading, content, block_type, sort_order) VALUES
(11215, 396, NULL, $BLK11215$Runique · Axum + SeaORM + Tera · 2026-03-22$BLK11215$, 'text', 1),
(11216, 397, NULL, $BLK11216$Le système de filtres affiche dans une sidebar les valeurs distinctes d'une colonne, et filtre la liste en cliquant dessus. Tout est SQL — aucune donnée en mémoire.$BLK11216$, 'text', 1),
(11217, 397, NULL, $BLK11217$```
Déclaration (admin.rs)
       ↓
   Parser          → lit la macro admin!
       ↓
   Générateur      → produit admin_panel.rs
       ↓
   Handler         → orchestre les requêtes
       ↓
   Template        → rend la sidebar
```$BLK11217$, 'code', 2),
(11218, 398, NULL, $BLK11218$Dans `src/admin.rs` , le dev déclare les colonnes à filtrer :$BLK11218$, 'text', 1),
(11219, 398, NULL, $BLK11219$```
list_filter: [
    ["lang", "Langue"],         // défaut : 10 valeurs par page
    ["block_type", "Type", 5],  // limite explicite : 5 valeurs par page
]
```$BLK11219$, 'code', 2),
(11220, 398, NULL, $BLK11220$Chaque entrée = `["colonne", "Libellé"]` ou `["colonne", "Libellé", limite]` .$BLK11220$, 'text', 3),
(11221, 398, NULL, $BLK11221$**Règle :** ne jamais filtrer sur une FK ou un `id` . Valeurs brutes illisibles. Bons candidats : booléens, énumérations, codes courts.$BLK11221$, 'text', 4),
(11222, 399, NULL, $BLK11222$Le daemon ( `runique start` ) lit `src/admin.rs` token par token et construit une structure intermédiaire :$BLK11222$, 'text', 1),
(11223, 399, NULL, $BLK11223$```
pubstructResourceDef {
pub list_filter: Vec<(String, String, u64)>,
//                    col     label   limit
}
```$BLK11223$, 'code', 2),
(11224, 399, NULL, $BLK11224$Le 3ème élément est optionnel — si absent, la limite par défaut est `10` .$BLK11224$, 'text', 3),
(11225, 400, NULL, $BLK11225$À partir de la structure, le daemon génère deux blocs dans `admin_panel.rs` .$BLK11225$, 'text', 1),
(11226, 401, NULL, $BLK11226$```
let meta = meta.display(
    DisplayConfig::new()
        .list_filter(vec![
            ("lang", "Langue", 10u64),
            ("block_type", "Type", 5u64),
        ])
);
```$BLK11226$, 'code', 1),
(11227, 401, NULL, $BLK11227$Cette config est sérialisée et accessible dans Tera via `resource.display.list_filter` .$BLK11227$, 'text', 2),
(11228, 402, NULL, $BLK11228$Pour chaque colonne, deux requêtes SQL sont générées :$BLK11228$, 'text', 1),
(11229, 402, NULL, $BLK11229$**Comptage** — pour savoir combien de pages existent :$BLK11229$, 'text', 2),
(11230, 402, NULL, $BLK11230$```
SELECTCOUNT(DISTINCT lang) FROM doc_page WHERE lang ISNOTNULL
```$BLK11230$, 'code', 3),
(11231, 402, NULL, $BLK11231$**Valeurs paginées** — seulement ce qui est affiché :$BLK11231$, 'text', 4),
(11232, 402, NULL, $BLK11232$```
SELECTDISTINCTCAST(lang ASTEXT)
FROM doc_page
WHERE lang ISNOTNULL
ORDERBY lang ASC
LIMIT10OFFSET20-- page 2 × 10
```$BLK11232$, 'code', 5),
(11233, 402, NULL, $BLK11233$`CAST(... AS TEXT)` uniformise le type : booléens, entiers et chaînes passent tous par là.$BLK11233$, 'text', 6),
(11234, 402, NULL, $BLK11234$Le résultat est un `HashMap<String, (Vec<String>, u64)>` : chaque colonne → ses valeurs + son total distinct.$BLK11234$, 'text', 7),
(11235, 403, NULL, $BLK11235$Dans `admin_main.rs` , deux séries de paramètres URL sont parsées :$BLK11235$, 'text', 1),
(11236, 403, NULL, $BLK11236$```
filter_lang=fr          → filtre actif sur la colonne lang
fp_lang=2               → page 2 dans la sidebar du groupe lang
```$BLK11236$, 'code', 2),
(11237, 403, NULL, $BLK11237$Les trois requêtes tournent **en parallèle** grâce à `tokio::join!` :$BLK11237$, 'text', 3),
(11238, 403, NULL, $BLK11238$```
tokio::join!(
    list_fn(db, list_params),    // entrées de la table
    count_fn(db, search),        // total pour la pagination principale
    filter_fn(db, filter_pages), // valeurs distinctes par colonne
)
```$BLK11238$, 'code', 4),
(11239, 404, NULL, $BLK11239$C'est la partie centrale. Plutôt que charger 100 valeurs en JS, on pagine côté serveur.$BLK11239$, 'text', 1),
(11240, 404, NULL, $BLK11240$Pour chaque colonne, le handler calcule `filter_meta` :$BLK11240$, 'text', 2),
(11241, 404, NULL, $BLK11241$```
current_page  → page affichée actuellement
total_pages   → nombre total de pages
prev_qs       → query string complet du lien "page précédente"
next_qs       → query string complet du lien "page suivante"
```$BLK11241$, 'code', 3),
(11242, 404, NULL, $BLK11242$`prev_qs` et `next_qs` sont précalculés en Rust car Tera ne peut pas construire des query strings complexes. Le template n'a plus qu'à écrire :$BLK11242$, 'text', 4),
(11243, 404, NULL, $BLK11243$```
<a href="?{{ meta.prev_qs }}&page=1">‹</a>
<a href="?{{ meta.next_qs }}&page=1">›</a>
```$BLK11243$, 'code', 5),
(11244, 404, NULL, $BLK11244$Ces liens préservent automatiquement le tri, la recherche et les autres filtres actifs.$BLK11244$, 'text', 6),
(11245, 405, NULL, $BLK11245$La sidebar s'affiche uniquement si `list_filter` est non vide :$BLK11245$, 'text', 1),
(11246, 405, NULL, $BLK11246$```
{% for filter_entry in resource.display.list_filter %}
  {% set col    = filter_entry[0] %}
  {% set label  = filter_entry[1] %}
  {% set values = filter_values[col] %}
  {% set meta   = filter_meta[col] %}
  {% if values | length > 0 %}
<!-- groupe visible -->
  {% endif %}
{% endfor %}
```$BLK11246$, 'code', 2),
(11247, 405, NULL, $BLK11247$Chaque valeur est un lien qui ajoute `filter_{col}={val}` à l'URL. Cliquer applique un `WHERE col = val` côté SQL.$BLK11247$, 'text', 3),
(11248, 406, NULL, $BLK11248$Chaque groupe peut être replié individuellement. L'état est sauvegardé dans `localStorage` par ressource + colonne :$BLK11248$, 'text', 1),
(11249, 406, NULL, $BLK11249$```
clé : runique_fg_doc_page_lang
      → '1' = ouvert
      → '0' = replié
```$BLK11249$, 'code', 2),
(11250, 406, NULL, $BLK11250$Au chargement, chaque groupe restaure son état. Au clic sur le titre, le corps est caché/montré et l'état est sauvegardé.$BLK11250$, 'text', 3),
(11251, 407, NULL, $BLK11251$Si une colonne n'existe pas en base, la requête échoue. Au lieu de passer silencieusement, le code généré log une erreur :$BLK11251$, 'text', 1),
(11252, 407, NULL, $BLK11252$```
ERROR [runique admin] list_filter `doc_block.lang` :
      colonne introuvable en DB — column "lang" does not exist
```$BLK11252$, 'code', 2),
(11253, 407, NULL, $BLK11253$Le dev voit immédiatement quelle colonne poser dans `list_filter` est invalide.$BLK11253$, 'text', 3),
(11254, 408, NULL, $BLK11254$|**Choix**||**Alternative écartée**|**Pourquoi**|
|---|---|---|---|
|Pagination|serveur|Charger 100 valeurs en<br>JS|Scalable, sans limite arbitraire|$BLK11254$, 'table', 1),
(11255, 408, NULL, $BLK11255$||**Choix**|**Alternative écartée**|**Pourquoi**||
|---|---|---|---|---|
||Limite per-colonne|Limite globale|Chaque colonne a des besoins<br>différents||
||`tokio::join!`|Requêtes séquentielles|Les 3 sont indépendantes||
||`tracing::error!`sur colonne<br>absente|`unwrap_or`silencieux|Diagnostique immédiat||
||localStorage par ressource+colonne|État global|Deux ressources indépendantes||$BLK11255$, 'table', 2);

-- middleware-ordre.md (cour_id=22)
INSERT INTO chapitre (id, cour_id, slug, title, lead, sort_order) VALUES
(409, 22, 'middleware-ordre-1-lordre-inverse-daxum-le-piege-du-builder', '1. L''ordre inversé d''Axum — le piège du builder', NULL, 1),
(410, 22, 'middleware-ordre-2-session-avant-csrf-le-bug-silencieux', '2. Session avant CSRF — le bug silencieux', NULL, 2),
(411, 22, 'middleware-ordre-3-le-body-http-ne-peut-etre-lu-quune-seule-fois', '3. Le body HTTP ne peut être lu qu''une seule fois', NULL, 3),
(412, 22, 'middleware-ordre-4-les-formulaires-doivent-etre-declares-en-get', '4. Les formulaires doivent être déclarés en GET', NULL, 4),
(413, 22, 'middleware-ordre-5-middlewares-actifs-sur-toutes-les-routes', '5. Middlewares actifs sur toutes les routes', NULL, 5),
(414, 22, 'middleware-ordre-6-fausse-route-et-redirection-admin-login', '6. Fausse route et redirection admin/login', NULL, 6),
(415, 22, 'middleware-ordre-7-flash-messages-sur-render-vs-redirect', '7. Flash messages sur render vs redirect', NULL, 7);
INSERT INTO cour_block (id, chapitre_id, heading, content, block_type, sort_order) VALUES
(11256, 409, 'Le comportement contre-intuitif', $BLK11256$Quand tu empiles des middlewares avec Axum via `.layer()`, l'ordre d'exécution est **l'inverse** de l'ordre de déclaration. Le dernier `.layer()` appliqué est le premier exécuté sur la requête entrante.

```rust
// ❌ Ce qu'on écrit
Router::new()
    .layer(SessionLayer::new())     // déclaré en 1er
    .layer(CsrfLayer::new())        // déclaré en 2ème
    .layer(CompressionLayer::new()) // déclaré en 3ème
    .layer(ErrorHandlerLayer::new()) // déclaré en 4ème
```

```
// Ce qui s'exécute réellement sur la requête entrante :
ErrorHandler → Compression → CSRF → Session → Handler
```

La requête traverse les couches **de la dernière déclarée vers la première**. C'est le modèle en oignon de Tower — chaque `.layer()` enveloppe ce qui précède.$BLK11256$, 'text', 1),
(11257, 409, 'Le bug en production', $BLK11257$Ce comportement a causé un bug réel lors de la construction de Runique : le CSRF se retrouvait exécuté **avant** la session, cherchait un token en session inexistante, et rejetait toutes les requêtes POST avec une erreur 403 silencieuse.

> **Important :** Le bug ne se manifeste pas toujours immédiatement. Si le CSRF est désactivé en développement, il peut passer en production sans que le problème ait jamais été visible.$BLK11257$, 'text', 2),
(11258, 409, 'La solution Runique — le système de slots', $BLK11258$Au lieu de dépendre de l'ordre de déclaration, Runique attribue un **slot numéroté fixe** à chaque middleware. Au moment du build, tous les middlewares sont triés par slot et appliqués dans le bon ordre — automatiquement, indépendamment de l'ordre de déclaration du développeur.

```
Slots d'exécution (requête entrante) :
Extensions(0) → Compression(5) → ErrorHandler(10) → Custom(20+)
→ CSP/Headers(30) → Cache(40) → Session(50) → SessionUpgrade(55)
→ CSRF(60) → HostValidation(70) → Handler
```

```rust
// ✅ Avec Runique — l'ordre de déclaration n'a aucune importance
builder::new(config)
    .middleware(|m| {
        m.with_csp(|c| c.policy(SecurityPolicy::strict()))  // slot 30
         .with_allowed_hosts(|h| h.host("monsite.fr"))      // slot 70
         .with_session_memory_limit(5 * 1024 * 1024, 10 * 1024 * 1024) // slot 50
    })
```

Peu importe l'ordre dans le builder, Session sera toujours avant CSRF. Le framework impose la correction structurellement.

---$BLK11258$, 'text', 3),
(11259, 410, 'Pourquoi le CSRF dépend de la session', $BLK11259$La protection CSRF fonctionne en générant un token secret stocké en session, que chaque formulaire doit renvoyer. Le middleware CSRF doit lire ce token en session pour valider la requête.

**Sans session initialisée, le CSRF ne peut pas fonctionner.**$BLK11259$, 'text', 1),
(11260, 410, 'Le bug', $BLK11260$Si CSRF s'exécute avant Session :

```
Requête → CSRF(cherche token en session) → Session(initialise) → Handler
```

Le CSRF tente de lire le token... la session n'existe pas encore. Il ne trouve rien, considère chaque requête invalide, et retourne systématiquement un 403.

Le développeur voit toutes ses requêtes POST rejetées sans message d'erreur clair. L'application est inutilisable, et la cause n'est pas évidente.$BLK11260$, 'text', 2),
(11261, 410, 'La correction dans Runique', $BLK11261$Le bug a été découvert et corrigé en imposant des slots fixes :

```
Session(50) → SessionUpgrade(55) → CSRF(60)
```

Le CSRF est désormais **non configurable** — il s'exécute toujours après la session, et le développeur ne peut pas changer ça même par erreur.

```rust
// Le CSRF est toujours activé, toujours au bon slot
// Aucune configuration requise, aucune erreur possible
builder::new(config)
    .middleware(|m| {
        m.with_session_memory_limit(5 * 1024 * 1024, 10 * 1024 * 1024)
        // CSRF garanti au slot 60, après Session(50) — automatique
    })
```

---$BLK11261$, 'text', 3),
(11262, 411, 'Pourquoi cette contrainte existe', $BLK11262$Le body d'une requête HTTP est un **flux de données** (`Stream`), pas un tableau en mémoire. Les octets arrivent depuis le réseau et sont consommés au fil de la lecture. Il n'y a pas de curseur de retour — une fois lus, ils sont perdus.

```rust
// ❌ Impossible en Rust/Axum
async fn handler(req: Request) -> Response {
    let body1 = axum::body::to_bytes(req.into_body(), usize::MAX).await;
    // req est consommé — body1 contient les données
    // Il n'existe plus de body à lire
}
```$BLK11262$, 'text', 1),
(11263, 411, 'Le problème avec les middlewares', $BLK11263$Si un middleware lit le body (pour logger, valider, parser...), le handler en aval reçoit un body vide.

```
Requête → [Middleware lit le body] → Handler (body vide !)
```

C'est une contrainte du protocole HTTP et de la gestion mémoire Rust — pas un bug corrigeable.$BLK11263$, 'text', 2),
(11264, 411, 'Solution haut niveau — le buffering', $BLK11264$Les frameworks comme Django, Rails ou Express résolvent ça en chargeant **tout le body en mémoire** dès la réception de la requête.

```python
# Django — le body est toujours disponible
def ma_vue(request):
    data1 = request.body  # Disponible
    data2 = request.body  # Toujours disponible — Django a tout bufferisé
```

**Avantage :** simplicité totale.
**Inconvénient :** tout le body est en RAM, même pour un fichier de 500 Mo. Le streaming devient impossible.$BLK11264$, 'text', 3),
(11265, 411, 'Solution Runique — le relais typé (Prisme)', $BLK11265$Runique ne bufferise pas. Le body n'est lu **qu'une seule fois**, directement dans le handler via l'extracteur `Prisme`. Les middlewares en amont ne touchent jamais au body.

```rust
// ✅ Prisme consomme le body une seule fois, au bon endroit
pub async fn login_user(
    mut request: Request,
    Prisme(form): Prisme<LoginForm>, // lecture unique ici
) -> AppResult<Response> {
    // form contient les données parsées
    // Aucun middleware n'a touché au body avant
}
```

Les données dont les middlewares auraient besoin sont transmises via le système d'extensions de la requête — un relais typé en mémoire, pas une relecture du flux réseau.$BLK11265$, 'text', 4),
(11266, 411, 'Conséquence sur les champs password', $BLK11266$`Forms::fill()` ne peut pas remplir les champs `password` automatiquement — ils ne transitent pas par le système de relais (pour des raisons de sécurité). Ils s'utilisent via `add_value()` directement depuis les données de Prisme.

```rust
// ✅ Champs normaux
form.fill(&model);

// ✅ Champs password — directement depuis Prisme
form.add_value("password", &prisme_value);
```

---$BLK11266$, 'text', 5),
(11267, 412, 'Le piège', $BLK11267$En Runique, un formulaire HTML doit être **initialisé et rendu dans un handler GET** avant de pouvoir être soumis en POST. Il est tentant de ne déclarer que le handler POST et de construire le formulaire directement dedans.

```rust
// ❌ Tentant mais incorrect — pas de rendu initial
pub async fn login_user(
    mut request: Request,
    Prisme(form): Prisme<LoginForm>,
) -> AppResult<Response> {
    // Pas de GET handler → le formulaire n'a jamais été rendu
    // Les champs, erreurs et tokens CSRF n'existent pas côté client
}
```$BLK11267$, 'text', 1),
(11268, 412, 'Pourquoi c''est nécessaire', $BLK11268$Le GET sert à :
1. **Injecter le token CSRF** dans le formulaire HTML
2. **Rendre les champs vides** avec leur configuration (labels, types, validation)
3. **Afficher les erreurs** lors d'une soumission invalide (re-render du GET avec erreurs)

```rust
// ✅ Pattern correct — GET pour afficher, POST pour traiter
pub async fn login_page(mut request: Request) -> AppResult<Response> {
    let form = LoginForm::new();
    context_update!(request => { "form" => &form });
    request.render("auth/login.html")
}

pub async fn login_user(
    mut request: Request,
    Prisme(form): Prisme<LoginForm>,
) -> AppResult<Response> {
    // Traitement du POST
}
```

---$BLK11268$, 'text', 2),
(11269, 413, 'La contrainte', $BLK11269$Dans Runique, les middlewares configurés via le builder s'appliquent à **toutes les routes** de l'application. Il n'y a pas de middleware conditionnel par groupe de routes dans le builder.

```rust
// Le rate limiter s'applique à TOUTES les routes
builder::new(config)
    .middleware(|m| {
        m.with_session_memory_limit(...)
    })
```$BLK11269$, 'text', 1),
(11270, 413, 'Pourquoi ce choix', $BLK11270$Les middlewares de sécurité (CSRF, Session, CSP, Host Validation) doivent s'appliquer universellement — une exception est une faille potentielle. Imposer cette contrainte structurellement empêche les oublis.$BLK11270$, 'text', 2),
(11271, 413, 'La solution pour les cas particuliers', $BLK11271$Pour un middleware sur une route spécifique, on utilise `route_layer` directement sur le Router, en dehors du builder :

```rust
let limiter = Arc::new(RateLimiter::new().max_requests(5).retry_after(60));

let upload_route = Router::new()
    .route("/upload", view!(upload_handler))
    .route_layer(middleware::from_fn_with_state(
        limiter,
        rate_limit_middleware,
    ));
```

Le builder gère la sécurité globale, `route_layer` gère les besoins spécifiques.

---$BLK11271$, 'text', 3),
(11272, 414, 'Le bug', $BLK11272$Lors du développement de Runique, une URL inexistante ne retournait pas une page 404 — elle redirectionnait systématiquement vers `/admin/login`.

```
GET /une-page-qui-nexiste-pas → 302 redirect → /admin/login
```$BLK11272$, 'text', 1),
(11273, 414, 'La cause', $BLK11273$Le middleware d'authentification admin interceptait **toutes les requêtes non authentifiées**, y compris les 404. Avant que le router puisse retourner une page d'erreur, le middleware admin court-circuitait la chaîne et redirigait.

L'ordre d'exécution était :

```
Requête → AuthAdmin (redirige si non auth) → Router (jamais atteint)
```$BLK11273$, 'text', 2),
(11274, 414, 'La correction', $BLK11274$Le middleware admin ne doit intercepter que les routes `/admin/*`, pas toutes les routes. La solution est de l'appliquer uniquement sur le groupe de routes admin via `route_layer`, et non comme middleware global.

```rust
// ✅ Middleware admin isolé sur son périmètre
let admin_router = Router::new()
    .route("/admin/*path", view!(admin_handler))
    .route_layer(middleware::from_fn(auth_admin_middleware));
```

Les routes publiques ne sont plus interceptées, les 404 remontent correctement vers l'ErrorHandler.

---$BLK11274$, 'text', 3),
(11275, 415, 'Le comportement attendu', $BLK11275$Les flash messages sont conçus pour **survivre à un redirect** : stockés en session à la fin d'un handler, affichés à la requête suivante après la redirection.

```rust
// Cas nominal — redirect après action
warning!(request.notices => "Mot de passe incorrect.");
return Ok(Redirect::to("/login").into_response());
// → message stocké en session → affiché sur /login
```$BLK11275$, 'text', 1),
(11276, 415, 'Le piège — render sans redirect', $BLK11276$Quand on fait un `render` direct (sans redirect), les flash messages sont déjà consommés ou ne s'affichent pas correctement — ils ont été conçus pour la prochaine requête, pas pour la réponse actuelle.

```rust
// ❌ Le message peut ne pas s'afficher
warning!(request.notices => "Formulaire invalide.");
return request.render("forms/login.html"); // render direct, pas de redirect
```$BLK11276$, 'text', 2),
(11277, 415, 'La solution — flash_now', $BLK11277$Runique introduit `flash_now` pour injecter un message directement dans le contexte du render courant, sans passer par la session.

```rust
// ✅ flash_now — affiché immédiatement dans le render
flash_now!(request => warning "Formulaire invalide.");
return request.render("forms/login.html");
```$BLK11277$, 'text', 3),
(11278, 415, 'Le problème ouvert', $BLK11278$La vraie solution serait une détection automatique : si la réponse est un redirect, stocker en session ; si c'est un render, injecter directement. Mais pour ça il faudrait inspecter la réponse *après* que le handler l'a produite — ce qui ramène au problème de lecture unique et de la chaîne middleware. `flash_now` reste un contournement explicite en attendant.

---$BLK11278$, 'text', 4);

-- macros-export.md (cour_id=23)
INSERT INTO chapitre (id, cour_id, slug, title, lead, sort_order) VALUES
(416, 23, 'macros-export-1-visibilite-par-defaut', '1. Visibilité par défaut', NULL, 1),
(417, 23, 'macros-export-2-macro-export', '2. #[macro_export]', NULL, 2),
(418, 23, 'macros-export-3-crate-reference-absolue', '3. $crate:: — référence absolue', NULL, 3),
(419, 23, 'macros-export-4-reexport-depuis-librs', '4. Réexport depuis lib.rs', NULL, 4),
(420, 23, 'macros-export-5-macro-use-lancienne-facon', '5. #[macro_use] — l''ancienne façon', NULL, 5);
INSERT INTO cour_block (id, chapitre_id, heading, content, block_type, sort_order) VALUES
(11279, 416, NULL, $BLK11279$Contrairement aux fonctions et types, une macro déclarée dans un module n'est **pas automatiquement accessible** dans les sous-modules ou depuis l'extérieur.$BLK11279$, 'text', 1),
(11280, 416, NULL, $BLK11280$```rust
// src/utils.rs
macro_rules! dire {

    ($msg:expr) => { println!("{}", $msg); };

}

// src/main.rs
mod utils;

fn main() {
    dire!("Bonjour"); // ❌ Erreur : macro non trouvée
}
```$BLK11280$, 'code', 2),
(11281, 416, NULL, $BLK11281$Les macros suivent leur propre système de portée, basé sur l'ordre de déclaration dans le fichier, pas sur la hiérarchie des modules.$BLK11281$, 'text', 3),
(11282, 416, NULL, $BLK11282$---$BLK11282$, 'text', 4),
(11283, 417, NULL, $BLK11283$`#[macro_export]` rend une macro disponible à la racine de la crate, comme si elle était déclarée dans `lib.rs`.$BLK11283$, 'text', 1),
(11284, 417, NULL, $BLK11284$```rust
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
```$BLK11284$, 'code', 2),
(11285, 417, NULL, $BLK11285$> **Important :** `#[macro_export]` place la macro à la racine de la crate, peu importe où elle est définie. Si ta crate s'appelle `mon_crate`, la macro est accessible via `mon_crate::dire!`.$BLK11285$, 'warning', 3),
(11286, 417, NULL, $BLK11286$---$BLK11286$, 'text', 4),
(11287, 418, NULL, $BLK11287$Le problème : dans une macro, tu veux appeler d'autres éléments de ta crate (types, fonctions). Si tu écris juste `MaStruct`, ça peut ne pas se résoudre dans le contexte de l'appelant.$BLK11287$, 'text', 1),
(11288, 418, NULL, $BLK11288$```rust
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
```$BLK11288$, 'code', 2),
(11289, 418, NULL, $BLK11289$`$crate` se résout à la crate qui contient la définition de la macro, pas celle qui l'utilise. C'est la façon correcte de référencer des items internes.$BLK11289$, 'text', 3),
(11290, 418, 'Exemple concret', $BLK11290$```rust
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

---$BLK11290$, 'code', 4),
(11291, 419, NULL, $BLK11291$Pour une lib qui expose des macros, la convention est de les réexporter depuis `lib.rs` afin que l'utilisateur n'ait qu'un seul point d'import.$BLK11291$, 'text', 1),
(11292, 419, NULL, $BLK11292$```rust
// ma_lib/src/lib.rs

mod macros; // Fichier qui contient les macro_rules!

// Réexport explicite
pub use macros::*; // ⚠️ N'exporte que les items pub — les macros #[macro_export] sont déjà à la racine

// Ou via le module lui-même
pub mod macros;
```$BLK11292$, 'code', 2),
(11293, 419, NULL, $BLK11293$Avec `#[macro_export]`, les macros sont automatiquement à la racine — l'utilisateur importe juste :$BLK11293$, 'text', 3),
(11294, 419, NULL, $BLK11294$```rust
use ma_lib::ma_macro; // Fonctionne sans pub use supplémentaire
// ou
ma_lib::ma_macro!();  // Chemin complet
```$BLK11294$, 'code', 4),
(11295, 419, NULL, $BLK11295$---$BLK11295$, 'text', 5),
(11296, 420, NULL, $BLK11296$Avant Rust 2018, on utilisait `#[macro_use]` pour importer toutes les macros d'une crate :$BLK11296$, 'text', 1),
(11297, 420, NULL, $BLK11297$```rust
// Ancien style (pre-2018)
#[macro_use]
extern crate serde;

// Nouveau style (2018+)
use serde::{Serialize, Deserialize};
```$BLK11297$, 'code', 2),
(11298, 420, NULL, $BLK11298$Même chose pour les modules internes :$BLK11298$, 'text', 3),
(11299, 420, NULL, $BLK11299$```rust
// Ancien style
#[macro_use]
mod macros;

// Nouveau style — préférer #[macro_export] + use explicite
```$BLK11299$, 'code', 4),
(11300, 420, NULL, $BLK11300$> **À retenir :** `#[macro_use]` est encore valide mais déconseillé. Utilise `use crate::ma_macro` ou `use ma_lib::ma_macro` à la place. C'est plus explicite et compatible avec rust-analyzer.$BLK11300$, 'warning', 5);

-- macros-derive.md (cour_id=24)
INSERT INTO chapitre (id, cour_id, slug, title, lead, sort_order) VALUES
(421, 24, 'macros-derive-1-quest-ce-quune-derive-macro', '1. Qu''est-ce qu''une derive macro ?', NULL, 1),
(422, 24, 'macros-derive-2-structure-dune-crate-proc-macro', '2. Structure d''une crate proc-macro', NULL, 2),
(423, 24, 'macros-derive-3-syn-parser-le-code', '3. syn — parser le code', NULL, 3),
(424, 24, 'macros-derive-4-quote-generer-du-code', '4. quote — générer du code', NULL, 4),
(425, 24, 'macros-derive-5-exemple-complet-describe', '5. Exemple complet — Describe', NULL, 5),
(426, 24, 'macros-derive-6-acceder-aux-champs-de-la-struct', '6. Accéder aux champs de la struct', NULL, 6),
(427, 24, 'macros-derive-7-attributs-helper-sur-les-champs', '7. Attributs helper sur les champs', NULL, 7);
INSERT INTO cour_block (id, chapitre_id, heading, content, block_type, sort_order) VALUES
(11301, 421, NULL, $BLK11301$Une derive macro s'utilise comme ça :$BLK11301$, 'text', 1),
(11302, 421, NULL, $BLK11302$```rust
#[derive(Debug, Clone, MaMacro)]
struct Personne {
    nom: String,
    age: u32,
}
```$BLK11302$, 'code', 2),
(11303, 421, NULL, $BLK11303$`Debug` et `Clone` sont des derives de la stdlib. `MaMacro` est une derive personnalisée. Elle reçoit la définition complète de `Personne` et génère du code Rust supplémentaire — généralement une implémentation de trait.$BLK11303$, 'text', 3),
(11304, 421, NULL, $BLK11304$---$BLK11304$, 'text', 4),
(11305, 422, NULL, $BLK11305$Les macros procédurales **doivent** être dans leur propre crate séparée.$BLK11305$, 'text', 1),
(11306, 422, NULL, $BLK11306$```toml
# ma_lib_derive/Cargo.toml
[package]
name = "ma_lib_derive"
version = "0.1.0"

[lib]
proc-macro = true

[dependencies]
syn   = { version = "2.0", features = ["full"] }
quote = "1.0"
proc-macro2 = "1.0"
```$BLK11306$, 'code', 2),
(11307, 422, NULL, $BLK11307$```toml
# ma_lib/Cargo.toml
[dependencies]
ma_lib_derive = { path = "../ma_lib_derive" }
```$BLK11307$, 'code', 3),
(11308, 422, NULL, $BLK11308$```rust
// ma_lib_derive/src/lib.rs
use proc_macro::TokenStream;

#[proc_macro_derive(MaTrait)]
pub fn derive_ma_trait(input: TokenStream) -> TokenStream {
    // input = la struct/enum sur laquelle #[derive(MaTrait)] est appliqué
    // retour = code Rust à ajouter
    todo!()
}
```$BLK11308$, 'code', 4),
(11309, 422, NULL, $BLK11309$---$BLK11309$, 'text', 5),
(11310, 423, NULL, $BLK11310$`syn` transforme les tokens bruts en arbre syntaxique utilisable.$BLK11310$, 'text', 1),
(11311, 423, NULL, $BLK11311$```rust
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(MaTrait)]
pub fn derive_ma_trait(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let nom_struct = &input.ident;       // Identifiant : Personne, Config, etc.
    let generics   = &input.generics;    // Les paramètres génériques <T, U>
    let data       = &input.data;        // Contenu : struct, enum, union

    todo!()
}
```$BLK11311$, 'code', 2),
(11312, 423, NULL, $BLK11312$Les champs utiles de `DeriveInput` :$BLK11312$, 'text', 3),
(11313, 423, NULL, $BLK11313$```rust
input.ident      // nom de la struct/enum
input.generics   // <T: Clone, U>
input.attrs      // attributs #[...] posés sur la struct
input.data       // Data::Struct, Data::Enum, Data::Union
```$BLK11313$, 'code', 4),
(11314, 423, NULL, $BLK11314$---$BLK11314$, 'text', 5),
(11315, 424, NULL, $BLK11315$`quote!` produit des tokens Rust à partir d'un template. Les variables sont interpolées avec `#`.$BLK11315$, 'text', 1),
(11316, 424, NULL, $BLK11316$```rust
use quote::quote;

let nom = &input.ident;

let expanded = quote! {
    impl #nom {
        pub fn hello(&self) -> String {
            format!("Je suis une instance de {}", stringify!(#nom))
        }
    }
};

TokenStream::from(expanded)
```$BLK11316$, 'code', 2),
(11317, 424, NULL, $BLK11317$Règles d'interpolation :$BLK11317$, 'text', 3),
(11318, 424, NULL, $BLK11318$```rust
let name: &Ident = ...;
let ty: &Type = ...;
let items: &[...] = ...;

quote! {
    #name          // Interpolation simple
    #ty            // Un type
    #(#items),*   // Répétition (comme $(...),* en macro_rules!)
}
```$BLK11318$, 'code', 4),
(11319, 424, NULL, $BLK11319$---$BLK11319$, 'text', 5),
(11320, 425, NULL, $BLK11320$Une derive macro qui génère une méthode `describe()` sur n'importe quelle struct.$BLK11320$, 'text', 1),
(11321, 425, NULL, $BLK11321$```rust
// ma_lib_derive/src/lib.rs
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Describe)]
pub fn derive_describe(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl #name {
            pub fn describe(&self) -> String {
                format!("Instance de {}", stringify!(#name))
            }
        }
    };

    TokenStream::from(expanded)
}
```$BLK11321$, 'code', 2),
(11322, 425, NULL, $BLK11322$```rust
// utilisation
use ma_lib_derive::Describe;

#[derive(Describe)]
struct Config {
    host: String,
    port: u16,
}

fn main() {
    let c = Config { host: "localhost".to_string(), port: 8080 };
    println!("{}", c.describe()); // "Instance de Config"
}
```$BLK11322$, 'code', 3),
(11323, 425, NULL, $BLK11323$---$BLK11323$, 'text', 4),
(11324, 426, NULL, $BLK11324$Pour itérer sur les champs et générer du code par champ :$BLK11324$, 'text', 1),
(11325, 426, NULL, $BLK11325$```rust
use syn::{Data, Fields};

#[proc_macro_derive(ListeChamps)]
pub fn derive_liste_champs(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Extraire les champs nommés
    let champs = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("ListeChamps ne supporte que les structs avec champs nommés"),
        },
        _ => panic!("ListeChamps ne supporte que les structs"),
    };

    // Noms des champs comme strings
    let noms_champs: Vec<_> = champs
        .iter()
        .map(|f| f.ident.as_ref().unwrap().to_string())
        .collect();

    let expanded = quote! {
        impl #name {
            pub fn champs() -> &'static [&'static str] {
                &[ #(#noms_champs),* ]
            }
        }
    };

    TokenStream::from(expanded)
}
```$BLK11325$, 'code', 2),
(11326, 426, NULL, $BLK11326$```rust
#[derive(ListeChamps)]
struct Personne {
    nom: String,
    age: u32,
    email: String,
}

fn main() {
    println!("{:?}", Personne::champs()); // ["nom", "age", "email"]
}
```$BLK11326$, 'code', 3),
(11327, 426, NULL, $BLK11327$---$BLK11327$, 'text', 4),
(11328, 427, NULL, $BLK11328$On peut définir des attributs personnalisés à placer sur les champs :$BLK11328$, 'text', 1),
(11329, 427, NULL, $BLK11329$```rust
#[proc_macro_derive(Validable, attributes(valider))]
pub fn derive_validable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let champs = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(f) => &f.named,
            _ => panic!(),
        },
        _ => panic!(),
    };

    // Regarde si le champ a #[valider(min_len = 3)]
    for champ in champs {
        for attr in &champ.attrs {
            if attr.path().is_ident("valider") {
                // Parser les arguments de l'attribut
            }
        }
    }

    // Génère la validation...
    todo!()
}
```$BLK11329$, 'code', 2),
(11330, 427, NULL, $BLK11330$```rust
#[derive(Validable)]
struct RegisterForm {
    #[valider(min_len = 3, max_len = 50)]
    username: String,
    #[valider(email)]
    email: String,
    #[valider(min_len = 8)]
    password: String,
}
```$BLK11330$, 'code', 3),
(11331, 427, NULL, $BLK11331$C'est exactement comme ça que `derive_form` fonctionne dans Runique pour générer la validation des formulaires.$BLK11331$, 'text', 4);

-- macros-attribut.md (cour_id=25)
INSERT INTO chapitre (id, cour_id, slug, title, lead, sort_order) VALUES
(428, 25, 'macros-attribut-1-derive-vs-attribute-la-difference', '1. Derive vs Attribute — la différence', NULL, 1),
(429, 25, 'macros-attribut-2-signature-dune-attribute-macro', '2. Signature d''une attribute macro', NULL, 2),
(430, 25, 'macros-attribut-3-exemple-mesure-du-temps-dexecution', '3. Exemple — mesure du temps d''exécution', NULL, 3),
(431, 25, 'macros-attribut-4-lire-les-arguments-de-lattribut', '4. Lire les arguments de l''attribut', NULL, 4),
(432, 25, 'macros-attribut-5-transformer-une-struct', '5. Transformer une struct', NULL, 5),
(433, 25, 'macros-attribut-6-cas-dusage-reels', '6. Cas d''usage réels', NULL, 6);
INSERT INTO cour_block (id, chapitre_id, heading, content, block_type, sort_order) VALUES
(11332, 428, NULL, $BLK11332$| | Derive macro | Attribute macro |
|---|---|---|
| Syntaxe | `#[derive(Trait)]` | `#[mon_attribut]` |
| Ce qu'elle reçoit | La struct/enum, inchangée | L'item entier |
| Ce qu'elle retourne | Du code **en plus** | Le remplacement complet |
| S'applique à | Struct, enum, union | Tout item (fn, struct, impl, mod...) |$BLK11332$, 'table', 1),
(11333, 428, NULL, $BLK11333$Une derive macro **ajoute** du code. Une attribute macro **remplace** l'item par ce qu'elle retourne — elle peut modifier, décorer, ou entièrement transformer.$BLK11333$, 'text', 2),
(11334, 428, NULL, $BLK11334$---$BLK11334$, 'text', 3),
(11335, 429, NULL, $BLK11335$```rust
// ma_lib_derive/src/lib.rs
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn mon_attribut(attr: TokenStream, item: TokenStream) -> TokenStream {
    // attr = les arguments entre parenthèses : #[mon_attribut(arg1, arg2)]
    // item = le code sur lequel l'attribut est posé (la fonction, struct, etc.)
    // retour = le code qui remplace l'item
    item // Retourner item sans modification = attribut no-op
}
```$BLK11335$, 'code', 1),
(11336, 429, NULL, $BLK11336$---$BLK11336$, 'text', 2),
(11337, 430, NULL, $BLK11337$```rust
// ma_lib_derive/src/lib.rs
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn timed(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let name    = &input.sig.ident;
    let vis     = &input.vis;
    let sig     = &input.sig;
    let block   = &input.block;

    let expanded = quote! {
        #vis #sig {
            let __start = std::time::Instant::now();
            let __result = (|| #block)();
            println!(
                "[timed] '{}' : {:?}",
                stringify!(#name),
                __start.elapsed()
            );
            __result
        }
    };

    TokenStream::from(expanded)
}
```$BLK11337$, 'code', 1),
(11338, 430, NULL, $BLK11338$```rust
// utilisation
use ma_lib_derive::timed;

#[timed]
fn calcul(n: u64) -> u64 {
    (0..n).sum()
}

fn main() {
    let r = calcul(1_000_000);
    // Affiche : [timed] 'calcul' : 1.2ms
    println!("résultat: {}", r);
}
```$BLK11338$, 'code', 2),
(11339, 430, NULL, $BLK11339$La fonction `calcul` est remplacée par une version identique mais entourée d'un chronomètre. L'appelant n'y voit rien.$BLK11339$, 'text', 3),
(11340, 430, NULL, $BLK11340$---$BLK11340$, 'text', 4),
(11341, 431, NULL, $BLK11341$```rust
#[timed(prefix = "MON_APP")]
fn calcul(n: u64) -> u64 { ... }
```$BLK11341$, 'code', 1),
(11342, 431, NULL, $BLK11342$Pour parser les arguments, on utilise `syn::parse` :$BLK11342$, 'text', 2),
(11343, 431, NULL, $BLK11343$```rust
use syn::{parse_macro_input, LitStr, Token};
use syn::parse::{Parse, ParseStream};

struct TimedArgs {
    prefix: Option<String>,
}

impl Parse for TimedArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(TimedArgs { prefix: None });
        }
        // Attend : prefix = "valeur"
        let _: syn::Ident = input.parse()?; // "prefix"
        let _: Token![=] = input.parse()?;
        let val: LitStr = input.parse()?;
        Ok(TimedArgs { prefix: Some(val.value()) })
    }
}

#[proc_macro_attribute]
pub fn timed(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as TimedArgs);
    let prefix = args.prefix.unwrap_or_else(|| "timed".to_string());

    let input = parse_macro_input!(item as ItemFn);
    let name  = &input.sig.ident;
    let sig   = &input.sig;
    let vis   = &input.vis;
    let block = &input.block;

    let expanded = quote! {
        #vis #sig {
            let __start = std::time::Instant::now();
            let __result = (|| #block)();
            println!("[{}] '{}' : {:?}", #prefix, stringify!(#name), __start.elapsed());
            __result
        }
    };

    TokenStream::from(expanded)
}
```$BLK11343$, 'code', 3),
(11344, 431, NULL, $BLK11344$---$BLK11344$, 'text', 4),
(11345, 432, NULL, $BLK11345$Une attribute macro peut aussi s'appliquer à une struct et la modifier :$BLK11345$, 'text', 1),
(11346, 432, NULL, $BLK11346$```rust
#[proc_macro_attribute]
pub fn avec_id(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as syn::ItemStruct);

    // Ajoute un champ `id: u32` à la struct
    if let syn::Fields::Named(ref mut fields) = input.fields {
        let id_field: syn::Field = syn::parse_quote! {
            pub id: u32
        };
        fields.named.push(id_field);
    }

    quote! { #input }.into()
}
```$BLK11346$, 'code', 2),
(11347, 432, NULL, $BLK11347$```rust
#[avec_id]
struct Article {
    titre: String,
    contenu: String,
}

// Après expansion, la struct a 3 champs : id, titre, contenu
fn main() {
    let a = Article { id: 1, titre: "Hello".to_string(), contenu: "...".to_string() };
}
```$BLK11347$, 'code', 3),
(11348, 432, NULL, $BLK11348$---$BLK11348$, 'text', 4),
(11349, 433, NULL, $BLK11349$Les attribute macros sont utilisées partout dans l'écosystème Rust :$BLK11349$, 'text', 1),
(11350, 433, NULL, $BLK11350$**Axum — définir des handlers HTTP**$BLK11350$, 'text', 2),
(11351, 433, NULL, $BLK11351$```rust
// Simplifié — Axum utilise des extracteurs, pas d'attribute macro
// Mais des frameworks comme Actix-web utilisent ce pattern
#[get("/users")]
async fn list_users() -> impl Responder { ... }
```$BLK11351$, 'code', 3),
(11352, 433, NULL, $BLK11352$**Tokio — transformer une fn en runtime async**$BLK11352$, 'text', 4),
(11353, 433, NULL, $BLK11353$```rust
#[tokio::main]
async fn main() {
    // Tokio injecte le runtime autour de ce bloc
}
```$BLK11353$, 'code', 5),
(11354, 433, NULL, $BLK11354$Ce qui se génère (simplifié) :$BLK11354$, 'text', 6),
(11355, 433, NULL, $BLK11355$```rust
fn main() {
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async {
            // ton code ici
        });
}
```$BLK11355$, 'code', 7),
(11356, 433, NULL, $BLK11356$**Serde — contrôle fin de la sérialisation**$BLK11356$, 'text', 8),
(11357, 433, NULL, $BLK11357$```rust
#[derive(Serialize, Deserialize)]
struct Config {
    #[serde(rename = "host_name")]
    host: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    port: Option<u16>,
}
```$BLK11357$, 'code', 9),
(11358, 433, NULL, $BLK11358$`serde` et `rename` sont des attributs helper enregistrés par la derive macro Serialize/Deserialize.$BLK11358$, 'text', 10);

-- macros-function-like.md (cour_id=26)
INSERT INTO chapitre (id, cour_id, slug, title, lead, sort_order) VALUES
(434, 26, 'macros-function-like-1-macro-rules-vs-function-like-proc-macro', '1. macro_rules! vs function-like proc macro', NULL, 1),
(435, 26, 'macros-function-like-2-signature-dune-function-like-macro', '2. Signature d''une function-like macro', NULL, 2),
(436, 26, 'macros-function-like-3-exemple-sql-avec-validation-a-la-compilation', '3. Exemple — SQL avec validation à la compilation', NULL, 3),
(437, 26, 'macros-function-like-4-exemple-dsl-de-configuration', '4. Exemple — DSL de configuration', NULL, 4),
(438, 26, 'macros-function-like-5-parser-des-structures-complexes', '5. Parser des structures complexes', NULL, 5),
(439, 26, 'macros-function-like-6-comparaison-des-3-types-de-proc-macros', '6. Comparaison des 3 types de proc macros', NULL, 6);
INSERT INTO cour_block (id, chapitre_id, heading, content, block_type, sort_order) VALUES
(11359, 434, NULL, $BLK11359$Les deux s'appellent avec `ma_macro!(...)`. La différence est dans ce qu'on peut faire à l'intérieur.$BLK11359$, 'text', 1),
(11360, 434, NULL, $BLK11360$| | `macro_rules!` | Function-like proc macro |
|---|---|---|
| Parsing | Pattern matching limité | Parsing Rust arbitraire via `syn` |
| Erreurs | Messages basiques | Messages précis avec `span` |
| Logique | Patterns déclaratifs | Code Rust complet |
| Validation | Limitée | Validation complète à la compilation |
| Complexité | Simple | Nécessite crate proc-macro |$BLK11360$, 'table', 2),
(11361, 434, NULL, $BLK11361$Utilise `macro_rules!` tant que tu peux. Passe aux function-like proc macros quand le parsing devient trop complexe ou quand tu veux des erreurs de compilation précises.$BLK11361$, 'text', 3),
(11362, 434, NULL, $BLK11362$---$BLK11362$, 'text', 4),
(11363, 435, NULL, $BLK11363$```rust
// ma_lib_derive/src/lib.rs
use proc_macro::TokenStream;

#[proc_macro]
pub fn ma_macro(input: TokenStream) -> TokenStream {
    // input = tout ce qui est entre les parenthèses de ma_macro!(...)
    // retour = le code qui remplace l'appel entier
    input
}
```$BLK11363$, 'code', 1),
(11364, 435, NULL, $BLK11364$---$BLK11364$, 'text', 2),
(11365, 436, NULL, $BLK11365$Valider qu'une requête SQL commence par SELECT/INSERT/UPDATE **au moment de la compilation**, pas à l'exécution.$BLK11365$, 'text', 1),
(11366, 436, NULL, $BLK11366$```rust
// ma_lib_derive/src/lib.rs
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr};

#[proc_macro]
pub fn sql(input: TokenStream) -> TokenStream {
    let sql_str = parse_macro_input!(input as LitStr);
    let sql = sql_str.value();

    let upper = sql.trim().to_uppercase();
    let valide = upper.starts_with("SELECT")
        || upper.starts_with("INSERT")
        || upper.starts_with("UPDATE")
        || upper.starts_with("DELETE");

    if !valide {
        // Erreur de compilation avec position précise
        return syn::Error::new(
            sql_str.span(),
            format!("Requête SQL invalide : doit commencer par SELECT/INSERT/UPDATE/DELETE")
        )
        .to_compile_error()
        .into();
    }

    quote! { #sql_str }.into()
}
```$BLK11366$, 'code', 2),
(11367, 436, NULL, $BLK11367$```rust
// utilisation
use ma_lib_derive::sql;

fn main() {
    let q1 = sql!("SELECT * FROM users WHERE age > 18"); // ✅ OK

    let q2 = sql!("DROP TABLE users"); // ❌ Erreur de compilation
    // error: Requête SQL invalide : doit commencer par SELECT/INSERT/UPDATE/DELETE
}
```$BLK11367$, 'code', 3),
(11368, 436, NULL, $BLK11368$Le `DROP TABLE` est refusé **avant même que le programme compile**. C'est impossible à faire avec `macro_rules!`.$BLK11368$, 'text', 4),
(11369, 436, NULL, $BLK11369$---$BLK11369$, 'text', 5),
(11370, 437, NULL, $BLK11370$```rust
use std::collections::HashMap;

// Avec macro_rules! — fonctionne mais limité
macro_rules! config {
    ($($section:ident { $($key:ident = $value:expr),* $(,)? })*) => {{
        let mut cfg: HashMap<&str, HashMap<&str, String>> = HashMap::new();
        $(
            let mut section = HashMap::new();
            $(
                section.insert(stringify!($key), $value.to_string());
            )*
            cfg.insert(stringify!($section), section);
        )*
        cfg
    }};
}

fn main() {
    let cfg = config! {
        database {
            host = "localhost",
            port = 5432,
        }
        server {
            host = "0.0.0.0",
            port = 8080,
        }
    };

    println!("{:?}", cfg["database"]["host"]); // "localhost"
}
```$BLK11370$, 'code', 1),
(11371, 437, NULL, $BLK11371$Avec une function-like proc macro, on pourrait ajouter : validation des types, erreurs précises si une clé est manquante, génération d'une struct typée plutôt qu'un HashMap.$BLK11371$, 'text', 2),
(11372, 437, NULL, $BLK11372$---$BLK11372$, 'text', 3),
(11373, 438, NULL, $BLK11373$Pour un DSL non-standard, on implémente le trait `Parse` de `syn` :$BLK11373$, 'text', 1),
(11374, 438, NULL, $BLK11374$```rust
use syn::parse::{Parse, ParseStream};
use syn::{Ident, Token, LitStr, braced};

// Représente : section_name { key = "value", ... }
struct Section {
    name: Ident,
    entries: Vec<(Ident, LitStr)>,
}

impl Parse for Section {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;

        let content;
        braced!(content in input); // Consomme les { }

        let mut entries = vec![];
        while !content.is_empty() {
            let key: Ident = content.parse()?;
            let _: Token![=] = content.parse()?;
            let val: LitStr = content.parse()?;
            let _: Option<Token![,]> = content.parse()?; // virgule optionnelle
            entries.push((key, val));
        }

        Ok(Section { name, entries })
    }
}

#[proc_macro]
pub fn config_typed(input: TokenStream) -> TokenStream {
    // Parser toutes les sections
    let sections = syn::parse::Parser::parse(
        |s: ParseStream| {
            let mut sections = vec![];
            while !s.is_empty() {
                sections.push(s.parse::<Section>()?);
            }
            Ok(sections)
        },
        input,
    ).unwrap();

    // Générer une struct par section
    let structs = sections.iter().map(|s| {
        let name = &s.name;
        let fields = s.entries.iter().map(|(k, v)| {
            quote::quote! { pub #k: &'static str }
        });
        let values = s.entries.iter().map(|(k, v)| {
            quote::quote! { #k: #v }
        });
        quote::quote! {
            pub struct #name { #(#fields,)* }
            pub static CONFIG_#name: #name = #name { #(#values,)* };
        }
    });

    quote::quote! { #(#structs)* }.into()
}
```$BLK11374$, 'code', 2),
(11375, 438, NULL, $BLK11375$---$BLK11375$, 'text', 3),
(11376, 439, NULL, $BLK11376$```
#[derive(MaTrait)]    → Derive macro
struct Foo { ... }      → Ajoute du code, ne modifie pas la struct

#[mon_attribut]       → Attribute macro
fn ma_fn() { ... }      → Remplace l'item par ce que la macro retourne

ma_macro!(...)        → Function-like proc macro
                        → Remplace l'appel par ce que la macro retourne
```$BLK11376$, 'code', 1),
(11377, 439, NULL, $BLK11377$En pratique dans l'écosystème :$BLK11377$, 'text', 2),
(11378, 439, NULL, $BLK11378$| Cas d'usage | Type |
|---|---|
| Implémenter un trait automatiquement | Derive |
| Décorer une fonction (log, auth, retry) | Attribute |
| DSL avec parsing custom | Function-like |
| Validation à la compilation | Function-like |
| Génération de code depuis données externes | Function-like |$BLK11378$, 'table', 3),
(11379, 439, NULL, $BLK11379$Dans Runique, `derive_form` est une **derive macro** — elle génère les méthodes de formulaire depuis la définition de struct. `admin!`, `model!`, `view!` sont des **`macro_rules!`** classiques — assez puissantes pour leur usage sans nécessiter une proc-macro.$BLK11379$, 'text', 4);

-- traits-basics.md (cour_id=27)
INSERT INTO chapitre (id, cour_id, slug, title, lead, sort_order) VALUES
(440, 27, 'traits-basics-1-quest-ce-quun-trait', '1. Qu''est-ce qu''un trait', NULL, 1),
(441, 27, 'traits-basics-2-definir-un-trait', '2. Définir un trait', NULL, 2),
(442, 27, 'traits-basics-3-implementer-un-trait-sur-une-struct', '3. Implémenter un trait sur une struct', NULL, 3),
(443, 27, 'traits-basics-4-implementation-par-defaut', '4. Implémentation par défaut', NULL, 4),
(444, 27, 'traits-basics-5-impl-trait-en-parametre-et-en-retour', '5. `impl Trait` en paramètre et en retour', NULL, 5),
(445, 27, 'traits-basics-6-les-derives-communes', '6. Les derives communes', NULL, 6),
(446, 27, 'traits-basics-7-impl-multiple-et-coherence', '7. Impl multiple et cohérence', NULL, 7),
(447, 27, 'traits-basics-8-exercices-pratiques', '8. Exercices pratiques', NULL, 8),
(448, 27, 'traits-basics-9-aide-memoire', '9. Aide-mémoire', NULL, 9);
INSERT INTO cour_block (id, chapitre_id, heading, content, block_type, sort_order) VALUES
(11380, 440, NULL, $BLK11380$Un **trait** est un contrat : il définit un ensemble de méthodes qu'un type doit implémenter.
C'est l'équivalent des interfaces dans d'autres langages, avec des fonctionnalités en plus.$BLK11380$, 'text', 1),
(11381, 440, NULL, $BLK11381$```rust
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
```$BLK11381$, 'code', 2),
(11382, 440, NULL, $BLK11382$Les traits permettent de :
- Écrire du code générique réutilisable
- Définir des interfaces sans héritage
- Garantir des comportements à la compilation$BLK11382$, 'text', 3),
(11383, 440, NULL, $BLK11383$---$BLK11383$, 'text', 4),
(11384, 441, NULL, $BLK11384$Un trait déclare des signatures de méthodes. Les types qui l'implémentent doivent fournir le corps.$BLK11384$, 'text', 1),
(11385, 441, NULL, $BLK11385$```rust
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
```$BLK11385$, 'code', 2),
(11386, 441, NULL, $BLK11386$Un trait peut aussi définir des **méthodes associées** (sans `&self`) :$BLK11386$, 'text', 3),
(11387, 441, NULL, $BLK11387$```rust
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
```$BLK11387$, 'code', 4),
(11388, 441, NULL, $BLK11388$---$BLK11388$, 'text', 5),
(11389, 442, NULL, $BLK11389$La syntaxe est `impl NomTrait for NomType`.$BLK11389$, 'text', 1),
(11390, 442, NULL, $BLK11390$```rust
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
```$BLK11390$, 'code', 2),
(11391, 442, NULL, $BLK11391$---$BLK11391$, 'text', 3),
(11392, 443, NULL, $BLK11392$Une implémentation par défaut s'applique automatiquement si le type ne la redéfinit pas.
Elle peut appeler d'autres méthodes du même trait.$BLK11392$, 'text', 1),
(11393, 443, NULL, $BLK11393$```rust
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
```$BLK11393$, 'code', 2),
(11394, 443, NULL, $BLK11394$---$BLK11394$, 'text', 3),
(11395, 444, NULL, $BLK11395$`impl Trait` est un raccourci syntaxique pour les trait bounds. Il rend le code plus lisible.$BLK11395$, 'text', 1),
(11396, 444, 'En paramètre', $BLK11396$```rust
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
```$BLK11396$, 'code', 2),
(11397, 444, 'En retour', $BLK11397$`impl Trait` en position de retour cache le type concret tout en gardant le dispatch statique.

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

---$BLK11397$, 'text', 3),
(11398, 445, NULL, $BLK11398$L'attribut `#[derive(...)]` génère automatiquement des implémentations de traits standard.$BLK11398$, 'text', 1),
(11399, 445, '`Debug`', $BLK11399$Permet l'affichage avec `{:?}` et `{:#?}` (pretty-print).

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
```$BLK11399$, 'text', 2),
(11400, 445, '`Clone` et `Copy`', $BLK11400$```rust
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

> **Règle :** `Copy` ne peut s'appliquer qu'aux types dont tous les champs sont `Copy`. `String`, `Vec`, `Box` ne peuvent pas être `Copy` car ils possèdent de la mémoire sur le tas.$BLK11400$, 'code', 3),
(11401, 445, '`PartialEq` et `Eq`', $BLK11401$```rust
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
```$BLK11401$, 'code', 4),
(11402, 445, '`Hash`', $BLK11402$`Hash` est nécessaire pour utiliser un type comme clé de `HashMap` ou dans un `HashSet`.
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

---$BLK11402$, 'text', 5),
(11403, 446, 'Plusieurs traits sur un même type', $BLK11403$Un type peut implémenter autant de traits que nécessaire.

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
```$BLK11403$, 'text', 1),
(11404, 446, 'La règle de cohérence (orphan rule)', $BLK11404$Rust impose une contrainte : pour implémenter un trait sur un type, **au moins l'un des deux** doit être défini dans le crate courant.

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

---$BLK11404$, 'text', 2),
(11405, 447, 'Exercice 1 — Trait `Convertible`', $BLK11405$Créez un trait `Convertible` avec une méthode `en_chaine(&self) -> String` et implémentez-le
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
```$BLK11405$, 'text', 1),
(11406, 447, 'Exercice 2 — Tri générique', $BLK11406$Écrivez une fonction `plus_grand` générique qui fonctionne avec tout type comparable et affichable.

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

---$BLK11406$, 'text', 2),
(11407, 448, NULL, $BLK11407$| Syntaxe | Signification |
|---|---|
| `trait Foo { fn bar(&self); }` | Définir un trait |
| `impl Foo for MaStruct { ... }` | Implémenter un trait |
| `fn f(x: impl Foo)` | Paramètre avec trait bound |
| `fn f() -> impl Foo` | Retour avec type opaque |
| `fn f<T: Foo>(x: T)` | Générique explicite |
| `where T: Foo + Bar` | Clause where (bounds multiples) |
| `#[derive(Debug, Clone)]` | Implémentation automatique |$BLK11407$, 'table', 1),
(11408, 448, NULL, $BLK11408$**Derives et leurs usages :**$BLK11408$, 'text', 2),
(11409, 448, NULL, $BLK11409$| Derive | Permet |
|---|---|
| `Debug` | `{:?}` et `{:#?}` |
| `Clone` | `.clone()` explicite |
| `Copy` | Copie implicite (types légers) |
| `PartialEq` | `==` et `!=` |
| `Eq` | Égalité totale (+ `PartialEq`) |
| `Hash` | Clé de `HashMap` / `HashSet` |
| `Default` | `T::default()` |
| `PartialOrd` / `Ord` | `<`, `>`, tri |$BLK11409$, 'table', 3),
(11410, 448, NULL, $BLK11410$**Points clés à retenir :**$BLK11410$, 'text', 4),
(11411, 448, NULL, $BLK11411$- Un trait = un contrat de comportement
- `impl Trait` en paramètre = syntaxe courte pour un bound
- `impl Trait` en retour = type concret opaque (statique, pas de box)
- `Box<dyn Trait>` = dispatch dynamique (pour types hétérogènes)
- La règle de cohérence protège l'écosystème des conflits
- `Copy` requiert `Clone`, et tous les champs doivent être `Copy`$BLK11411$, 'list', 5);

-- concurrence.md (cour_id=28)
INSERT INTO chapitre (id, cour_id, slug, title, lead, sort_order) VALUES
(449, 28, 'concurrence-1-pourquoi-la-concurrence-est-difficile', '1. Pourquoi la concurrence est difficile', NULL, 1),
(450, 28, 'concurrence-2-mutext-exclusion-mutuelle', '2. `Mutex<T>` — exclusion mutuelle', NULL, 2),
(451, 28, 'concurrence-3-rwlockt-lecture-ecriture', '3. `RwLock<T>` — lecture/écriture', NULL, 3),
(452, 28, 'concurrence-4-arct-reference-comptee-thread-safe', '4. `Arc<T>` — référence comptée thread-safe', NULL, 4),
(453, 28, 'concurrence-5-arcmutext-pattern-classique', '5. `Arc<Mutex<T>>` — pattern classique', NULL, 5),
(454, 28, 'concurrence-6-lazylockt-initialisation-paresseuse', '6. `LazyLock<T>` — initialisation paresseuse', NULL, 6),
(455, 28, 'concurrence-7-oncelockt-valeur-initialisee-une-seule-fois', '7. `OnceLock<T>` — valeur initialisée une seule fois', NULL, 7),
(456, 28, 'concurrence-8-comparaison-et-quand-utiliser-quoi', '8. Comparaison et quand utiliser quoi', NULL, 8),
(457, 28, 'concurrence-9-exemples-concrets-avec-runique', '9. Exemples concrets avec Runique', NULL, 9),
(458, 28, 'concurrence-10-exercices-pratiques', '10. Exercices pratiques', NULL, 10),
(459, 28, 'concurrence-11-aide-memoire', '11. Aide-mémoire', NULL, 11);
INSERT INTO cour_block (id, chapitre_id, heading, content, block_type, sort_order) VALUES
(11412, 449, NULL, $BLK11412$En programmation concurrente, plusieurs threads accèdent aux mêmes données simultanément.
Cela génère deux catégories de bugs :$BLK11412$, 'text', 1),
(11413, 449, NULL, $BLK11413$**Data race** — deux threads modifient la même mémoire sans synchronisation. Le résultat est
imprévisible et peut être différent à chaque exécution.$BLK11413$, 'text', 2),
(11414, 449, NULL, $BLK11414$**Deadlock** — deux threads attendent chacun un verrou que l'autre détient. Ils se bloquent
mutuellement pour toujours.$BLK11414$, 'text', 3),
(11415, 449, NULL, $BLK11415$```rust
// Ce code ne compile PAS — Rust empêche le data race à la compilation
use std::thread;

let mut compteur = 0;

thread::spawn(|| compteur += 1); // erreur : compteur emprunté depuis un autre thread
thread::spawn(|| compteur += 1); // erreur : idem
```$BLK11415$, 'code', 4),
(11416, 449, NULL, $BLK11416$Rust résout ces problèmes grâce aux traits `Send` et `Sync` vérifiés à la compilation,
et aux primitives de synchronisation de la bibliothèque standard.$BLK11416$, 'text', 5),
(11417, 449, NULL, $BLK11417$---$BLK11417$, 'text', 6),
(11418, 450, NULL, $BLK11418$`Mutex<T>` (*Mutual Exclusion*) garantit qu'un seul thread à la fois peut accéder aux données.
Pour lire ou modifier la valeur, il faut d'abord acquérir le **verrou**.$BLK11418$, 'text', 1),
(11419, 450, NULL, $BLK11419$```rust
use std::sync::Mutex;

let m = Mutex::new(5);

{
    // lock() bloque jusqu'à ce que le verrou soit disponible
    let mut val = m.lock().unwrap();
    *val += 1;
    println!("{val}"); // 6
} // le verrou est libéré ici automatiquement (drop du MutexGuard)

// On peut de nouveau accéder
println!("{:?}", m.lock().unwrap()); // 6
```$BLK11419$, 'code', 2),
(11420, 450, 'Gestion des erreurs avec `lock()`', $BLK11420$`lock()` retourne `Err` si un thread a paniqué en tenant le verrou (*poisoned mutex*).

```rust
use std::sync::Mutex;

let mutex = Mutex::new(vec![1, 2, 3]);

match mutex.lock() {
    Ok(mut guard) => {
        guard.push(4);
        println!("{:?}", *guard);
    }

    Err(poisoned) => {
        // Récupérer quand même les données
        let mut guard = poisoned.into_inner();
        guard.push(99);
        println!("Récupéré : {:?}", *guard);
    }
}
```$BLK11420$, 'text', 3),
(11421, 450, '`try_lock()` — tentative non bloquante', $BLK11421$```rust
use std::sync::Mutex;

let mutex = Mutex::new(0);

match mutex.try_lock() {
    Ok(mut val) => *val += 1,
    Err(_) => println!("Verrou occupé, on continue"),
}
```

---$BLK11421$, 'code', 4),
(11422, 451, NULL, $BLK11422$`RwLock<T>` (*Read-Write Lock*) permet **plusieurs lecteurs simultanés** ou **un seul écrivain**.
C'est plus efficace que `Mutex` quand les lectures sont fréquentes et les écritures rares.$BLK11422$, 'text', 1),
(11423, 451, NULL, $BLK11423$```rust
use std::sync::RwLock;

let verrou = RwLock::new(vec![1, 2, 3]);

// Plusieurs lectures simultanées — OK
let lecture1 = verrou.read().unwrap();
let lecture2 = verrou.read().unwrap();
println!("{:?} {:?}", *lecture1, *lecture2);
drop(lecture1);
drop(lecture2);

// Écriture exclusive — bloque si des lecteurs sont actifs
{
    let mut ecriture = verrou.write().unwrap();
    ecriture.push(4);
} // verrou d'écriture libéré

println!("{:?}", verrou.read().unwrap()); // [1, 2, 3, 4]
```$BLK11423$, 'code', 2),
(11424, 451, 'Différence avec `Mutex`', $BLK11424$```rust
// Mutex : un seul accès à la fois, même pour la lecture
// RwLock : plusieurs lecteurs simultanés, un seul écrivain

// Choisir selon le ratio lecture/écriture :
// - Beaucoup de lectures, peu d'écritures → RwLock
// - Équilibré ou données petites → Mutex (moins de surcharge)
```

---$BLK11424$, 'code', 3),
(11425, 452, NULL, $BLK11425$`Arc<T>` (*Atomically Reference Counted*) permet à **plusieurs threads de posséder** la même
valeur. Chaque clone incrémente un compteur atomique ; la valeur est libérée quand le compteur
atteint zéro.$BLK11425$, 'text', 1),
(11426, 452, NULL, $BLK11426$```rust
use std::sync::Arc;
use std::thread;

let donnees = Arc::new(vec![1, 2, 3, 4, 5]);
let mut handles = vec![];

for i in 0..3 {
    let clone = Arc::clone(&donnees);

    let handle = thread::spawn(move || {
        println!("Thread {i} : {:?}", clone);
    });

    handles.push(handle);
}

for h in handles {
    h.join().unwrap();
}

// donnees est toujours accessible ici
println!("Total : {}", donnees.len());
```$BLK11426$, 'code', 2),
(11427, 452, NULL, $BLK11427$> `Arc<T>` seul ne permet que la lecture. Pour modifier les données partagées entre threads,
> combinez avec `Mutex<T>` ou `RwLock<T>`.$BLK11427$, 'warning', 3),
(11428, 452, NULL, $BLK11428$```rust
// Rc<T> vs Arc<T>
use std::rc::Rc;
use std::sync::Arc;

let rc  = Rc::new(42);   // thread unique — compteur ordinaire, plus rapide
let arc = Arc::new(42);  // multi-thread — compteur atomique, légèrement plus lent

// Rc ne peut PAS être envoyé entre threads (erreur de compilation)
// Arc peut traverser les frontières de threads
```$BLK11428$, 'code', 4),
(11429, 452, NULL, $BLK11429$---$BLK11429$, 'text', 5),
(11430, 453, NULL, $BLK11430$C'est la combinaison standard pour **partager et modifier** des données entre plusieurs threads.$BLK11430$, 'text', 1),
(11431, 453, NULL, $BLK11431$```rust
use std::sync::{Arc, Mutex};
use std::thread;

let compteur = Arc::new(Mutex::new(0));
let mut handles = vec![];

for _ in 0..10 {
    let clone = Arc::clone(&compteur);

    let handle = thread::spawn(move || {
        let mut val = clone.lock().unwrap();
        *val += 1;
    });

    handles.push(handle);
}

for h in handles {
    h.join().unwrap();
}

println!("Résultat : {}", compteur.lock().unwrap()); // 10
```$BLK11431$, 'code', 2),
(11432, 453, 'Pattern avec état applicatif', $BLK11432$```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct Cache {
    donnees: Arc<Mutex<HashMap<String, String>>>,
}

impl Cache {
    fn new() -> Self {
        Cache {
            donnees: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn inserer(&self, cle: &str, valeur: &str) {
        let mut map = self.donnees.lock().unwrap();
        map.insert(cle.to_string(), valeur.to_string());
    }

    fn obtenir(&self, cle: &str) -> Option<String> {
        let map = self.donnees.lock().unwrap();
        map.get(cle).cloned()
    }
}

let cache = Cache::new();
let cache2 = cache.clone(); // partage le même Arc intérieur

cache.inserer("cle1", "valeur1");
println!("{:?}", cache2.obtenir("cle1")); // Some("valeur1")
```

---$BLK11432$, 'code', 3),
(11433, 454, NULL, $BLK11433$`LazyLock<T>` (stable depuis Rust 1.80) initialise une valeur **la première fois qu'on y accède**,
de façon thread-safe. Idéal pour les ressources globales coûteuses à initialiser.$BLK11433$, 'text', 1),
(11434, 454, NULL, $BLK11434$```rust
use std::sync::LazyLock;
use std::collections::HashMap;

// Initialisé au premier accès, jamais avant
static CODES_PAYS: LazyLock<HashMap<&str, &str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("FR", "France");
    m.insert("DE", "Allemagne");
    m.insert("JP", "Japon");
    m
});

fn main() {
    // La HashMap est créée ici, au premier accès
    println!("{:?}", CODES_PAYS.get("FR")); // Some("France")
    println!("{:?}", CODES_PAYS.get("JP")); // Some("Japon")
}
```$BLK11434$, 'code', 2),
(11435, 454, 'Comparaison avec `once_cell` (avant Rust 1.80)', $BLK11435$```rust
// Avant Rust 1.80, on utilisait la crate once_cell
// once_cell::sync::Lazy est identique à std::sync::LazyLock

// Depuis Rust 1.80, LazyLock est dans la stdlib — pas de dépendance externe nécessaire
use std::sync::LazyLock;

static CONFIG: LazyLock<String> = LazyLock::new(|| {
    std::env::var("APP_CONFIG").unwrap_or_else(|_| "defaut".to_string())
});
```$BLK11435$, 'code', 3),
(11436, 454, '`LazyLock` avec un type complexe', $BLK11436$```rust
use std::sync::LazyLock;

struct Connexion {
    url: String,
}

impl Connexion {
    fn nouvelle(url: &str) -> Self {
        println!("Connexion établie vers {url}");
        Connexion { url: url.to_string() }
    }

    fn ping(&self) -> bool {
        println!("Ping vers {}", self.url);
        true
    }
}

static DB: LazyLock<Connexion> = LazyLock::new(|| {
    Connexion::nouvelle("postgres://localhost/mabase")
});

// La connexion n'est créée que lors du premier appel à DB
fn main() {
    println!("Démarrage...");
    DB.ping(); // connexion créée ici
    DB.ping(); // déjà initialisé, réutilisé directement
}
```

---$BLK11436$, 'code', 4),
(11437, 455, NULL, $BLK11437$`OnceLock<T>` est similaire à `LazyLock` mais l'initialisation est **manuelle** — vous choisissez
quand et comment initialiser la valeur.$BLK11437$, 'text', 1),
(11438, 455, NULL, $BLK11438$```rust
use std::sync::OnceLock;

static INSTANCE: OnceLock<String> = OnceLock::new();

fn obtenir_instance() -> &'static String {
    INSTANCE.get_or_init(|| {
        println!("Initialisation unique...");
        "valeur globale".to_string()
    })
}

fn main() {
    println!("{}", obtenir_instance()); // initialise
    println!("{}", obtenir_instance()); // réutilise, pas de réinitialisation
}
```$BLK11438$, 'code', 2),
(11439, 455, 'Initialisation depuis une fonction externe', $BLK11439$```rust
use std::sync::OnceLock;

static PORT: OnceLock<u16> = OnceLock::new();

fn configurer(port: u16) -> Result<(), u16> {
    PORT.set(port) // retourne Err(port) si déjà initialisé
}

fn port() -> u16 {
    *PORT.get().expect("port non configuré")
}

fn main() {
    configurer(8080).unwrap();

    match configurer(9090) {
        Ok(_)  => println!("configuré"),
        Err(p) => println!("déjà initialisé avec {p}"),
    }

    println!("Port actif : {}", port()); // 8080
}
```$BLK11439$, 'code', 3),
(11440, 455, 'Différence `LazyLock` vs `OnceLock`', $BLK11440$```rust
// LazyLock — initialisation automatique à la closure définie à la déclaration
static A: LazyLock<String> = LazyLock::new(|| "automatique".to_string());

// OnceLock — initialisation manuelle, peut être faite depuis n'importe où
static B: OnceLock<String> = OnceLock::new();

fn main() {
    let _ = &*A;              // A s'initialise ici
    B.set("manuel".to_string()).unwrap(); // B initialisé explicitement
}
```

---$BLK11440$, 'code', 4),
(11441, 456, NULL, $BLK11441$| Type | Thread-safe | Propriétaires | Mutation | Cas d'usage |
|---|---|---|---|---|
| `Mutex<T>` | ✅ | 1 | oui (lock exclusif) | Compteur, état partagé |
| `RwLock<T>` | ✅ | 1 | oui (1 écrivain ou N lecteurs) | Cache lu souvent, écrit rarement |
| `Arc<T>` | ✅ | N | non (seul) | Partage en lecture seule |
| `Arc<Mutex<T>>` | ✅ | N | oui (lock) | État partagé entre threads |
| `Arc<RwLock<T>>` | ✅ | N | oui (lock) | Config partagée, lectures fréquentes |
| `LazyLock<T>` | ✅ | — | non (init une fois) | Ressource globale paresseuse |
| `OnceLock<T>` | ✅ | — | non (init une fois) | Valeur globale initialisée manuellement |$BLK11441$, 'table', 1),
(11442, 456, NULL, $BLK11442$**Règles pratiques :**$BLK11442$, 'text', 2),
(11443, 456, NULL, $BLK11443$- Vous partagez entre threads sans modifier → `Arc<T>`
- Vous partagez et modifiez → `Arc<Mutex<T>>`
- Lectures très fréquentes, écritures rares → `Arc<RwLock<T>>`
- Ressource globale à initialiser une seule fois → `LazyLock<T>` ou `OnceLock<T>`
- Thread unique avec mutation partagée → `Rc<RefCell<T>>`$BLK11443$, 'list', 3),
(11444, 456, NULL, $BLK11444$---$BLK11444$, 'text', 4),
(11445, 457, NULL, $BLK11445$Dans Runique, plusieurs primitives de concurrence sont utilisées pour gérer l'état global
du framework (environnement, token CSS, configuration de session, nettoyage de tâches).$BLK11445$, 'text', 1),
(11446, 457, '`LazyLock` pour l''environnement global', $BLK11446$```rust
use std::sync::LazyLock;

// Lecture du .env une seule fois au démarrage
static ENV: LazyLock<RuniqueEnv> = LazyLock::new(|| {
    dotenvy::dotenv().ok();
    match std::env::var("DEBUG").as_deref() {
        Ok("true") => RuniqueEnv::Development,
        _          => RuniqueEnv::Production,
    }
});

pub fn is_debug() -> bool {
    matches!(*ENV, RuniqueEnv::Development)
}

pub fn css_token() -> String {
    static TOKEN: LazyLock<String> = LazyLock::new(|| {
        use std::time::{SystemTime, UNIX_EPOCH};
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_millis();
        format!("{:04}", ts % 10_000)
    });
    TOKEN.clone()
}
```$BLK11446$, 'code', 2),
(11447, 457, '`Arc<Mutex<T>>` pour le nettoyage de sessions', $BLK11447$```rust
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::Instant;

#[derive(Clone)]
struct SessionStore {
    donnees: Arc<Mutex<HashMap<String, (Vec<u8>, Instant)>>>,
}

impl SessionStore {
    fn new() -> Self {
        SessionStore {
            donnees: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn nettoyer_expirees(&self, duree_max: std::time::Duration) {
        let mut map = self.donnees.lock().unwrap();
        let maintenant = Instant::now();

        map.retain(|_cle, (_valeur, horodatage)| {
            maintenant.duration_since(*horodatage) < duree_max
        });
    }
}
```$BLK11447$, 'code', 3),
(11448, 457, '`Arc<RwLock<T>>` pour une configuration partagée', $BLK11448$```rust
use std::sync::{Arc, RwLock};

#[derive(Clone)]
struct AppConfig {
    interne: Arc<RwLock<ConfigInterne>>,
}

struct ConfigInterne {
    page_size: usize,
    site_title: String,
}

impl AppConfig {
    fn page_size(&self) -> usize {
        // Lecture légère — plusieurs threads peuvent lire simultanément
        self.interne.read().unwrap().page_size
    }

    fn definir_page_size(&self, taille: usize) {
        // Écriture exclusive
        self.interne.write().unwrap().page_size = taille;
    }
}
```

---$BLK11448$, 'code', 4),
(11449, 458, 'Exercice 1 — Compteur concurrent', $BLK11449$Implémentez un compteur thread-safe que plusieurs threads peuvent incrémenter simultanément.

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn compteur_concurrent(nb_threads: usize, increments: usize) -> usize {
    let compteur = Arc::new(Mutex::new(0usize));
    let mut handles = vec![];

    for _ in 0..nb_threads {
        let c = Arc::clone(&compteur);

        handles.push(thread::spawn(move || {
            for _ in 0..increments {
                *c.lock().unwrap() += 1;
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    *compteur.lock().unwrap()
}

fn main() {
    let resultat = compteur_concurrent(8, 100);
    println!("Résultat : {resultat}"); // toujours 800
}
```$BLK11449$, 'text', 1),
(11450, 458, 'Exercice 2 — Cache avec `RwLock`', $BLK11450$Implémentez un cache thread-safe utilisant `RwLock` pour maximiser les lectures concurrentes.

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

struct CacheRW<K, V> {
    donnees: Arc<RwLock<HashMap<K, V>>>,
}

impl<K, V> CacheRW<K, V>
where
    K: Eq + std::hash::Hash + Clone,
    V: Clone,
{
    fn new() -> Self {
        CacheRW { donnees: Arc::new(RwLock::new(HashMap::new())) }
    }

    fn inserer(&self, cle: K, valeur: V) {
        self.donnees.write().unwrap().insert(cle, valeur);
    }

    fn obtenir(&self, cle: &K) -> Option<V> {
        self.donnees.read().unwrap().get(cle).cloned()
    }

    fn taille(&self) -> usize {
        self.donnees.read().unwrap().len()
    }
}
```$BLK11450$, 'text', 2),
(11451, 458, 'Exercice 3 — Singleton avec `OnceLock`', $BLK11451$Implémentez un pattern singleton thread-safe pour une configuration d'application.

```rust
use std::sync::OnceLock;

struct AppSettings {
    port: u16,
    debug: bool,
    max_connexions: usize,
}

static SETTINGS: OnceLock<AppSettings> = OnceLock::new();

fn init_settings(port: u16, debug: bool, max_connexions: usize) {
    SETTINGS.set(AppSettings { port, debug, max_connexions })
        .expect("Settings déjà initialisés");
}

fn settings() -> &'static AppSettings {
    SETTINGS.get().expect("Settings non initialisés — appeler init_settings d'abord")
}

fn main() {
    init_settings(8080, true, 100);

    println!("Port : {}", settings().port);
    println!("Debug : {}", settings().debug);
}
```

---$BLK11451$, 'text', 3),
(11452, 459, NULL, $BLK11452$| Primitive | Import | Usage principal |
|---|---|---|
| `Mutex<T>` | `std::sync::Mutex` | Accès exclusif (lecture + écriture) |
| `RwLock<T>` | `std::sync::RwLock` | N lecteurs OU 1 écrivain |
| `Arc<T>` | `std::sync::Arc` | Propriété partagée entre threads |
| `LazyLock<T>` | `std::sync::LazyLock` | Global initialisé paresseusement |
| `OnceLock<T>` | `std::sync::OnceLock` | Global initialisé une seule fois |$BLK11452$, 'table', 1),
(11453, 459, NULL, $BLK11453$**Patterns fréquents :**$BLK11453$, 'text', 2),
(11454, 459, NULL, $BLK11454$```rust
// Partagé + mutable entre threads
let etat = Arc::new(Mutex::new(valeur));

// Partagé + mutable, lectures fréquentes
let config = Arc::new(RwLock::new(valeur));

// Global paresseux
static X: LazyLock<T> = LazyLock::new(|| { ... });

// Global initialisé manuellement
static Y: OnceLock<T> = OnceLock::new();
Y.set(valeur).unwrap();
```$BLK11454$, 'code', 3),
(11455, 459, NULL, $BLK11455$**Points clés :**$BLK11455$, 'text', 4),
(11456, 459, NULL, $BLK11456$- `Mutex` bloque tous les accès — simple, sûr, légèrement moins performant sous forte lecture
- `RwLock` autorise plusieurs lectures simultanées — gain réel si lectures >> écritures
- `Arc` ne permet pas la mutation seul — combinez avec `Mutex` ou `RwLock`
- `LazyLock` remplace `once_cell::sync::Lazy` depuis Rust 1.80
- `OnceLock` remplace `once_cell::sync::OnceCell` depuis Rust 1.70
- Un `Mutex` verrouillé dans un `await` peut bloquer des threads Tokio — préférez `tokio::sync::Mutex` en code async$BLK11456$, 'list', 5);

-- Reset séquences
SELECT setval('chapitre_id_seq', (SELECT MAX(id) FROM chapitre));
