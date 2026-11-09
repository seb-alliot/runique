# Les type aliases en Rust

> Donner un nom court à un type long, sans créer de nouveau type : ce qu'un alias permet, ce qu'il ne permet pas, et quand passer à un newtype.

## Table des matières

1. À quoi sert un alias
2. Syntaxe
3. Un alias n'est pas un nouveau type
4. Alias génériques
5. Les alias de `Result`
6. Alias de types complexes
7. Alias choisis à la compilation
8. Alias, newtype ou `use ... as`
9. Limites et pièges
10. Organiser ses alias
11. Exemples réels dans Runique
12. Exercices
13. Points clés à retenir

## 1. À quoi sert un alias

Un alias donne un **nom** à un type existant. Il sert à :

- **raccourcir** un type long qui revient souvent (`Arc<RwLock<HashMap<String, String>>>`) ;
- **centraliser** une décision : changer le type en un seul endroit (le type de clé primaire) ;
- **documenter** l'intention (`type Octets = Vec<u8>`) ;
- **fixer** un paramètre d'un type générique (`type Result<T> = std::result::Result<T, MonErreur>`).

Il n'ajoute **aucune** vérification : c'est un synonyme, rien de plus.

## 2. Syntaxe

### 2.1 Forme générale

```rust
type Nom = TypeExistant;

type Kilometres = f64;
type Octets = Vec<u8>;
type Grille = [[u8; 9]; 9];
type Rappel = fn(i32) -> i32;
```

Le nom suit la convention des types : `PascalCase`. Un alias peut être `pub`, `pub(crate)` ou privé, comme tout élément.

### 2.2 Utilisation

```rust
type Kilometres = f64;

fn distance(a: Kilometres, b: Kilometres) -> Kilometres {
    (a - b).abs()
}

let trajet: Kilometres = 42.0;
let d = distance(trajet, 10.0);
```

### 2.3 Dans une struct

```rust
use std::collections::HashMap;

type Scores = HashMap<String, u32>;

struct Partie {
    scores: Scores,
}

impl Partie {
    fn new() -> Self {
        Self { scores: Scores::new() }     // les fonctions associées passent par l'alias
    }
}
```

## 3. Un alias n'est pas un nouveau type

```rust
type Metres = f64;
type Secondes = f64;

fn vitesse(d: Metres, t: Secondes) -> f64 {
    d / t
}

let d: Metres = 100.0;
let t: Secondes = 9.58;
vitesse(t, d);              // compile : les arguments sont inversés, aucune erreur
```

`Metres`, `Secondes` et `f64` sont **le même type** : le compilateur ne voit aucune différence. Pour qu'il refuse l'inversion, il faut des newtypes (section 8).

Conséquences :

- toutes les méthodes et tous les traits de `f64` sont disponibles sur `Metres` ;
- on peut passer un `f64` partout où un `Metres` est attendu, et inversement ;
- les messages d'erreur affichent souvent le type réel, pas l'alias.

## 4. Alias génériques

### 4.1 Avec paramètres

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

type Partage<T> = Arc<Mutex<T>>;
type Index<V> = HashMap<String, V>;

let compteur: Partage<u32> = Arc::new(Mutex::new(0));
let villes: Index<u32> = Index::new();
```

### 4.2 Fixer une partie des paramètres

```rust
use std::collections::HashMap;

type Table<K, V> = HashMap<K, V>;
type TableTexte<V> = Table<String, V>;        // K fixé
type TableComptes = TableTexte<u64>;           // K et V fixés
```

Chaque niveau fixe un paramètre de plus : on part d'un alias générique et on le spécialise.

### 4.3 Les contraintes ne sont pas vérifiées

```rust
// type Affichable<T: std::fmt::Display> = Vec<T>;
// avertissement : bounds on generic parameters are not enforced in type aliases
```

Une contrainte dans un alias est **ignorée** (le compilateur avertit). On la met sur les fonctions et les `impl` qui utilisent le type.

## 5. Les alias de `Result`

### 5.1 Le motif le plus courant

```rust
#[derive(Debug)]
pub enum Erreur {
    Introuvable,
    Base(String),
}

pub type Result<T> = std::result::Result<T, Erreur>;

pub fn charger(id: u32) -> Result<String> {
    if id == 0 {
        return Err(Erreur::Introuvable);
    }
    Ok("Ada".into())
}
```

La bibliothèque standard fait de même : `std::io::Result<T>`, `std::fmt::Result`. SeaORM, anyhow (`anyhow::Result<T>`) et Runique (`AppResult<T>`) aussi.

### 5.2 Le piège du masquage

```rust
use std::io;

type Result<T> = std::result::Result<T, io::Error>;

// fn analyser(s: &str) -> Result<i32, std::num::ParseIntError> { ... }
// ERREUR : ce Result-là n'a qu'un paramètre
fn analyser(s: &str) -> std::result::Result<i32, std::num::ParseIntError> {
    s.parse()
}
```

Un alias nommé `Result` **masque** celui du prélude dans tout le module. Deux façons d'éviter la confusion :

```rust
// 1. Lui donner un nom distinct
type IoResult<T> = std::result::Result<T, std::io::Error>;

// 2. L'utiliser préfixé depuis un autre module : io::Result<T>, ma_lib::Result<T>
```

### 5.3 Un alias de `Result` avec une erreur par défaut

```rust
pub type Resultat<T, E = Erreur> = std::result::Result<T, E>;

fn a() -> Resultat<u32> { Ok(1) }                         // E = Erreur
fn b() -> Resultat<u32, String> { Err("autre".into()) }   // E choisi
```

## 6. Alias de types complexes

### 6.1 Closures et trait objects

```rust
use std::future::Future;
use std::pin::Pin;

type Ecouteur = Box<dyn Fn(&str) + Send + Sync>;
type FuturEnvoyable<T> = Pin<Box<dyn Future<Output = T> + Send>>;
type GestionnaireAsync = Box<dyn Fn(String) -> FuturEnvoyable<String> + Send + Sync>;

struct Bus {
    ecouteurs: Vec<Ecouteur>,
}
```

Sans alias, `Vec<Box<dyn Fn(&str) + Send + Sync>>` revient dans chaque signature.

### 6.2 Pointeurs de fonction

```rust
type Operation = fn(f64, f64) -> f64;

fn appliquer(op: Operation, a: f64, b: f64) -> f64 {
    op(a, b)
}

let table: [(&str, Operation); 2] = [("+", |a, b| a + b), ("*", |a, b| a * b)];
```

### 6.3 État partagé

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

type Cache = Arc<RwLock<HashMap<String, Vec<u8>>>>;

fn lire(cache: &Cache, cle: &str) -> Option<Vec<u8>> {
    cache.read().unwrap().get(cle).cloned()
}
```

### 6.4 Alias de variante d'enum

```rust
type Note = Option<u8>;

let a: Note = Note::Some(15);       // les variantes passent par l'alias
let b = Note::None;
```

## 7. Alias choisis à la compilation

Un alias peut dépendre de la plateforme ou d'une feature : le même nom désigne un type différent selon le build.

```rust
#[cfg(feature = "grands-nombres")]
pub type Entier = i64;

#[cfg(not(feature = "grands-nombres"))]
pub type Entier = i32;

pub fn doubler(x: Entier) -> Entier { x * 2 }
```

Le reste du code écrit `Entier` et ne change pas. Attention : les deux versions doivent compiler, donc le code ne doit utiliser que ce que les deux types ont en commun. Il faut tester chaque combinaison de features (voir `cargo-dependances`).

```rust
#[cfg(unix)]
pub type Descripteur = std::os::unix::io::RawFd;      // i32
#[cfg(windows)]
pub type Descripteur = std::os::windows::io::RawHandle;
```

## 8. Alias, newtype ou `use ... as`

### 8.1 La même idée, trois écritures

```rust
// 1. Alias : un synonyme
type IdUtilisateur = u32;

// 2. Newtype : un nouveau type
struct IdCommande(u32);

// 3. Renommage à l'import : un autre nom pour un élément existant
use std::collections::HashMap as Table;
```

### 8.2 Comparaison

| | Alias `type` | Newtype `struct X(T)` | `use ... as` |
|---|---|---|---|
| Crée un nouveau type | non | oui | non |
| Empêche de mélanger avec le type d'origine | non | oui | non |
| Méthodes du type d'origine | toutes | à réexposer (ou `Deref`) | toutes |
| Implémenter un trait étranger | non (sauf si le type d'origine est local) | oui | non |
| Peut fixer des paramètres génériques | oui | oui | non |
| Coût à l'exécution | aucun | aucun | aucun |

### 8.3 Quand utiliser quoi

- **Alias** : raccourcir, centraliser une décision de type, fixer un paramètre d'erreur.
- **Newtype** : empêcher une confusion (identifiants, unités), garantir une validation (`Email` construit par `parse`), cacher une représentation, implémenter un trait étranger.
- **`use ... as`** : résoudre un conflit de noms entre deux imports (`fmt::Result` et `io::Result`).

### 8.4 Le même code, de l'alias au newtype

```rust
// Alias : rien n'empêche l'erreur
type Euros = i64;
type Centimes = i64;
fn payer(montant: Centimes) {}
let prix: Euros = 12;
payer(prix);                              // compile : bug (12 centimes au lieu de 12 euros)

// Newtype : l'erreur devient une erreur de compilation
struct EnEuros(i64);
struct EnCentimes(i64);
impl From<EnEuros> for EnCentimes {
    fn from(e: EnEuros) -> Self { EnCentimes(e.0 * 100) }
}
fn payer2(montant: EnCentimes) {}
// payer2(EnEuros(12));                   // ERREUR : expected EnCentimes
payer2(EnEuros(12).into());               // conversion explicite
```

## 9. Limites et pièges

### 9.1 Aucune vérification supplémentaire

Voir section 3 : un alias ne protège jamais contre une confusion de valeurs.

### 9.2 Pas de constructeur de tuple struct par l'alias

```rust
struct Point(i32, i32);
type P = Point;

// let p = P(1, 2);               // ERREUR E0423 : l'alias n'existe que comme type
let p = Point(1, 2);
let q: P = Point(3, 4);
let r = P { 0: 5, 1: 6 };         // la forme avec accolades passe par l'alias
```

Pour une struct à champs nommés, `P { x, y }` fonctionne.

### 9.3 Implémenter un trait

```rust
type Liste = Vec<i32>;

// impl std::fmt::Display for Liste { ... }
// ERREUR E0117 : Display et Vec viennent de la bibliothèque standard (règle de l'orphelin)
```

L'alias **est** `Vec<i32>` : l'implémentation viserait `Vec<i32>`, ce que seule la bibliothèque standard peut faire. Un newtype (`struct Liste(Vec<i32>)`) le permet.

À l'inverse, sur un alias d'un type **local**, `impl` fonctionne normalement, puisqu'il vise le type local.

### 9.4 Messages d'erreur

Le compilateur affiche souvent le type réel (`HashMap<String, Vec<u8>>`) plutôt que l'alias (`Cache`). Avec beaucoup d'alias imbriqués, retrouver l'origine d'une erreur demande de les déplier.

### 9.5 Trop d'alias

Un alias par usage (`FormData`, `Erreurs`, `Parametres` pour le même `HashMap<String, String>`) fait croire à trois types différents. Préfère un alias par **forme** de type, nommé d'après sa structure (`StrMap`), sauf si l'usage porte une vraie décision.

## 10. Organiser ses alias

### 10.1 Centraliser

```rust
// src/types.rs
use std::collections::HashMap;
use std::sync::Arc;

pub type StrMap = HashMap<String, String>;
pub type Partage<T> = Arc<T>;
pub type Result<T> = std::result::Result<T, crate::Erreur>;
```

```rust
// src/lib.rs
pub mod types;
pub use types::*;          // ou via un prelude
```

### 10.2 Ou les garder près de leur usage

Un alias utilisé dans un seul module reste dans ce module, privé. Seuls les alias partagés vont dans un fichier commun.

### 10.3 Conventions de nommage observées

| Préfixe / forme | Signification | Exemple |
|---|---|---|
| `A...` | dans un `Arc` | `AEngine = Arc<RuniqueEngine>` |
| `O...` | dans une `Option` | `OCurrentUser = Option<CurrentUser>` |
| `...Map` | une table | `StrMap`, `JsonMap` |
| `...Result<T>` | un `Result` à erreur fixée | `AppResult<T>`, `IoResult<T>` |

## 11. Exemples réels dans Runique

Les alias de Runique sont regroupés dans `runique/src/utils/aliases/definition.rs`, et la plupart sont réexportés par `runique::prelude`.

### 11.1 Un alias choisi par feature : `Pk`

```rust
// runique/src/utils/config/pk.rs
#[cfg(all(feature = "big-pk", feature = "pk-uuid"))]
compile_error!("features `big-pk` and `pk-uuid` are mutually exclusive — enable at most one of them");

#[cfg(feature = "pk-uuid")]
pub type Pk = uuid::Uuid;

#[cfg(all(feature = "big-pk", not(feature = "pk-uuid")))]
pub type Pk = i64;

#[cfg(not(any(feature = "big-pk", feature = "pk-uuid")))]
pub type Pk = i32;
```

Tout le framework écrit `Pk`. Changer de type de clé primaire, c'est changer une feature dans `Cargo.toml`, pas des centaines de signatures. Le `compile_error!` empêche d'activer deux choix contradictoires.

### 11.2 Un alias interne choisi par feature : `Inner` de `ADb`

```rust
// runique/src/db/adb.rs
#[cfg(not(feature = "test-utils"))]
type Inner = sea_orm::DatabaseConnection;
#[cfg(feature = "test-utils")]
type Inner = crate::db::RuniqueDb;      // connexion réelle ou transaction de test

#[derive(Clone, Debug)]
pub struct ADb(Arc<Inner>);
```

L'alias est privé : l'utilisateur ne voit que `ADb`, quelle que soit la feature.

### 11.3 Quand un alias ne suffit plus : `ADb`

La base de données a d'abord été un alias, `Arc<DatabaseConnection>`. Problème : chaque appel SeaORM (`.insert(db)`, `.one(db)`, `db.begin()`) attend un type qui implémente `ConnectionTrait`, et `Arc<DatabaseConnection>` ne l'implémente pas. Il fallait écrire `.as_ref()` partout.

Impossible de corriger avec un alias : `impl ConnectionTrait for Arc<DatabaseConnection>` est refusé par la règle de l'orphelin (le trait et le type viennent d'autres crates, et `Arc` n'est pas `#[fundamental]`). D'où un newtype :

```rust
impl ConnectionTrait for ADb { /* délègue à la connexion */ }
impl TransactionTrait for ADb { /* idem */ }
```

`ADb` reste aussi bon marché à cloner qu'un `Arc`, et `&ADb` s'utilise directement : `.insert(db)`, sans `.as_ref()`. C'est l'illustration exacte de la ligne « implémenter un trait étranger » du tableau de la section 8.2.

### 11.4 Alias de collections et de résultats

```rust
// runique/src/utils/aliases/definition.rs
pub type AEngine = Arc<RuniqueEngine>;
pub type StrMap = HashMap<String, String>;
pub type StrVecMap = HashMap<String, Vec<String>>;
pub type JsonMap = HashMap<String, serde_json::Value>;
pub type AppResult<T> = Result<T, Box<AppError>>;

// Dans un handler
pub async fn contact(mut request: Request) -> AppResult<Response> {
    // ...
}
```

### 11.5 Alias de type d'état d'un builder

```rust
// runique/src/app/builder/mod.rs
pub mod state {
    pub struct No;
    pub struct Yes;
    pub type Fresh = (No, No, No, No, No, No, No);
}
```

Le builder de Runique note dans son type quels réglages uniques ont déjà été faits (`No` ou `Yes` par réglage). `Fresh` nomme l'état de départ : sans l'alias, il faudrait écrire le tuple de sept `No` dans chaque signature qui en parle.

## 12. Exercices

### Exercice 1 : refactoring

Simplifie ces signatures avec des alias.

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

fn lire(c: &Arc<Mutex<HashMap<String, Vec<String>>>>, k: &str) -> Option<Vec<String>> { None }
fn ecrire(c: &Arc<Mutex<HashMap<String, Vec<String>>>>, k: String, v: Vec<String>) {}
fn charger() -> Result<Arc<Mutex<HashMap<String, Vec<String>>>>, std::io::Error> { todo!() }
```

```rust
// Solution :
type Groupes = HashMap<String, Vec<String>>;
type Partage<T> = Arc<Mutex<T>>;
type IoResult<T> = std::result::Result<T, std::io::Error>;

fn lire(c: &Partage<Groupes>, k: &str) -> Option<Vec<String>> {
    c.lock().unwrap().get(k).cloned()
}
fn ecrire(c: &Partage<Groupes>, k: String, v: Vec<String>) {
    c.lock().unwrap().insert(k, v);
}
fn charger() -> IoResult<Partage<Groupes>> {
    Ok(Arc::new(Mutex::new(Groupes::new())))
}
```

### Exercice 2 : alias ou newtype

Pour chaque cas, choisis alias ou newtype : (a) le type d'erreur de toute ta bibliothèque ; (b) des identifiants d'utilisateur et de produit, tous deux `i64` ; (c) `Box<dyn Fn(&Event) + Send + Sync>` répété partout ; (d) une adresse e-mail toujours valide.

Réponses : (a) alias `type Result<T> = std::result::Result<T, Erreur>` ; (b) newtypes, pour qu'on ne puisse pas les confondre ; (c) alias ; (d) newtype avec un constructeur qui valide.

### Exercice 3 : alias par feature

Écris un alias `Horodatage` qui vaut `chrono::DateTime<chrono::Utc>` avec la feature `chrono`, et `u64` (secondes) sans.

```rust
// Solution :
#[cfg(feature = "chrono")]
pub type Horodatage = chrono::DateTime<chrono::Utc>;

#[cfg(not(feature = "chrono"))]
pub type Horodatage = u64;
```

Le code qui manipule `Horodatage` ne peut utiliser que ce qui existe pour les deux types (comparaison, copie) : tout le reste doit être placé sous le même `#[cfg]`.

## 13. Points clés à retenir

- Un alias est un **synonyme** : aucun nouveau type, aucune vérification.
- Il sert à raccourcir, centraliser et fixer des paramètres (`type Result<T> = ...`).
- Les contraintes génériques d'un alias sont ignorées.
- Un alias nommé `Result` masque celui du prélude dans son module.
- Pour empêcher une confusion, valider ou implémenter un trait étranger : un **newtype**.
- `use ... as` renomme un import, il ne crée pas de type.
- Un alias sous `#[cfg]` change le type selon le build : teste chaque combinaison.
