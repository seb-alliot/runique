# Les type aliases en Rust

**Guide complet et pratique** — pour développeurs Rust intermédiaires.

## Table des matières

1. Introduction aux type aliases
2. Syntaxe et utilisation de base
3. Cas d'usage courants
4. Type alias ou newtype
5. Type aliases génériques
6. Organisation et bonnes pratiques
7. Limitations et pièges
8. Exemples réels (framework Runique)
9. Patterns avancés
10. Exercices

## 1. Introduction aux type aliases

Un **type alias** donne un autre nom à un type existant. Il ne crée **pas** de nouveau type : c'est un *synonyme*, que le compilateur remplace par le type réel.

### Pourquoi utiliser des type aliases ?

- **Lisibilité** : raccourcir des types longs ou imbriqués
- **Maintenabilité** : définir un type à un seul endroit
- **Documentation** : donner un sens métier à un type technique
- **Abstraction** : pouvoir changer l'implémentation sans toucher aux signatures

> **Note :** un alias n'a aucun coût à l'exécution. Il disparaît à la compilation.

## 2. Syntaxe et utilisation de base

### 2.1 Syntaxe générale

```rust
type NomAlias = TypeExistant;

// Exemples
type UserId = i32;
type Username = String;
type Result<T> = std::result::Result<T, std::io::Error>;
```

### 2.2 Premier exemple concret

```rust
// Sans alias
fn create_user(id: i32, name: String) -> i32 { id }
fn get_user(id: i32) -> Option<String> { None }

// Avec alias
type UserId = i32;
type Username = String;

fn create_user(id: UserId, name: Username) -> UserId { id }
fn get_user(id: UserId) -> Option<Username> { None }
```

L'intention devient lisible : on manipule un identifiant d'utilisateur et un nom, pas un entier et une chaîne quelconques.

### 2.3 Dans une structure

```rust
type Timestamp = i64;
type JsonData = serde_json::Value;

struct Event {
    id: UserId,
    created_at: Timestamp,
    data: JsonData,
}

let event = Event {
    id: 42,
    created_at: 1_706_400_000,
    data: serde_json::json!({ "action": "login" }),
};
```

## 3. Cas d'usage courants

### 3.1 Simplifier un type de retour

```rust
use std::collections::HashMap;
use std::error::Error;

// Avant
fn process_data(input: &str) -> Result<HashMap<String, Vec<i32>>, Box<dyn Error>> { todo!() }

// Après
type DataMap = HashMap<String, Vec<i32>>;
type ProcessResult = Result<DataMap, Box<dyn Error>>;

fn process_data(input: &str) -> ProcessResult { todo!() }
```

### 3.2 Types récurrents

```rust
use std::sync::Arc;
use sea_orm::{DatabaseConnection, DbErr};

type DbPool = Arc<DatabaseConnection>;
type DbResult<T> = Result<T, DbErr>;

async fn get_user(pool: &DbPool, id: UserId) -> DbResult<Option<User>> { todo!() }
async fn create_user(pool: &DbPool, user: User) -> DbResult<()> { todo!() }
async fn delete_user(pool: &DbPool, id: UserId) -> DbResult<()> { todo!() }
```

> **Attention :** trop d'alias rendent le code *moins* lisible — il faut aller chercher la définition. N'en créez que s'ils apportent quelque chose.

### 3.3 Abstraire un détail d'implémentation

```rust
// API publique
pub type Cache = std::collections::HashMap<String, String>;

// Plus tard : pub type Cache = dashmap::DashMap<String, String>;
// Le code qui utilise `Cache` ne change pas — tant que les méthodes appelées existent
// sur le nouveau type.
```

## 4. Type alias ou newtype

### 4.1 La différence fondamentale

```rust
// Alias : PAS un nouveau type
type UserId = i32;

// Newtype : un NOUVEAU type
struct UserIdN(i32);

let a: UserId = 42;
let b: i32 = a;          // OK : même type

let c = UserIdN(42);
// let d: i32 = c;       // ERREUR : types différents
let e: i32 = c.0;        // OK : accès explicite
```

### 4.2 Quand utiliser quoi ?

| Critère | Type alias | Newtype |
|---|---|---|
| Sûreté de type | Aucune (synonyme) | Forte |
| Coût à l'exécution | Aucun | Aucun |
| Méthodes propres | Non | Oui |
| Implémenter un trait | Non (c'est le type d'origine) | Oui |
| Verbosité | Faible | Moyenne |
| Interopérabilité | Transparente | Conversion explicite |

### 4.3 Recommandations

- **Alias** : simplifier une écriture sans ajouter de garantie — `Result<T>` d'un module, collections, types de callback.
- **Newtype** : créer un type distinct avec ses règles — unités (`Meters`, `Seconds`), identifiants validés, ou implémenter un trait étranger sur un type étranger (voir 8.2).

## 5. Type aliases génériques

### 5.1 Alias générique

```rust
type AppResult<T> = Result<T, AppError>;

fn create_user(name: &str) -> AppResult<User> { todo!() }
fn delete_user(id: UserId) -> AppResult<()> { todo!() }
```

### 5.2 Spécialisation progressive

```rust
type GenericResult<T, E> = Result<T, E>;   // tout générique
type AppResult<T> = Result<T, AppError>;   // erreur fixée
type UserResult = AppResult<User>;         // tout fixé

type DbResult<T> = Result<T, DbErr>;
type UserDbResult = DbResult<User>;
```

### 5.3 Types complexes

```rust
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

// Callbacks
type EventHandler = Box<dyn Fn(&Event) + Send + Sync>;
type EventHandlers = Vec<EventHandler>;

// État partagé
type SharedState<T> = Arc<Mutex<T>>;

// Future en boîte
type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
```

> **Astuce :** les alias génériques donnent une API cohérente dans tout le projet. Définissez-les une fois, dans un module central.

## 6. Organisation et bonnes pratiques

### 6.1 Centraliser

```rust
// Mauvais : dispersé
mod user { type UserId = i32; }
mod product { type ProductId = i32; }

// Bon : un module dédié
// src/types.rs
pub type UserId = i32;
pub type ProductId = i32;
pub type Timestamp = i64;
pub type JsonData = serde_json::Value;

// ailleurs
use crate::types::*;
```

### 6.2 Structure recommandée

```rust
// src/types/mod.rs
pub mod db;
pub mod api;
pub mod errors;
pub use db::*;
pub use api::*;
pub use errors::*;

// src/types/db.rs
pub type DbPool = std::sync::Arc<sea_orm::DatabaseConnection>;
pub type DbResult<T> = Result<T, sea_orm::DbErr>;

// src/types/api.rs
pub type ApiResult<T> = Result<axum::Json<T>, ApiError>;

// src/types/errors.rs
pub type AppError = Box<dyn std::error::Error + Send + Sync>;
pub type AppResult<T> = Result<T, AppError>;
```

### 6.3 Conventions de nommage

- **PascalCase**, comme tout type Rust
- **Suffixes descriptifs** : `UserId`, `UserResult`
- **Contexte métier** : `OrderId` plutôt que `Id`, `Price` plutôt que `Decimal`
- **Pas d'abréviations obscures**, sauf convention établie dans le projet

## 7. Limitations et pièges

### 7.1 Aucune vérification supplémentaire

```rust
type UserId = i32;
type ProductId = i32;

fn get_user(id: UserId) -> User { todo!() }

let product_id: ProductId = 123;
let user = get_user(product_id); // compile : bug silencieux

// Solution : des newtypes
struct UserIdN(i32);
struct ProductIdN(i32);

fn get_user_n(id: UserIdN) -> User { todo!() }
// get_user_n(ProductIdN(123)); // ERREUR de compilation
```

### 7.2 Messages d'erreur

```rust
type ComplexType = HashMap<String, Vec<Result<i32, String>>>;

fn process(data: ComplexType) {}
// Le compilateur affiche le type complet, pas l'alias :
// expected `HashMap<String, Vec<Result<i32, String>>>`, found ...
```

> **Limitation :** les erreurs montrent le type réel. Avec beaucoup d'alias imbriqués, elles deviennent plus difficiles à relier au code.

### 7.3 Pas d'implémentation de trait propre à l'alias

```rust
use std::fmt;

type UserId = i32;

// ERREUR : c'est `impl Display for i32`, interdit (trait et type étrangers)
// impl fmt::Display for UserId { ... }

// Solution : un newtype
struct UserIdN(i32);

impl fmt::Display for UserIdN {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "User #{}", self.0)
    }
}
```

## 8. Exemples réels (framework Runique)

Les alias de Runique sont regroupés dans `runique/src/utils/aliases/definition.rs`, et la plupart sont réexportés par `runique::prelude`.

### 8.1 Un alias choisi par feature : `Pk`

```rust
// runique/src/utils/config/pk.rs
#[cfg(feature = "pk-uuid")]
pub type Pk = uuid::Uuid;

#[cfg(all(feature = "big-pk", not(feature = "pk-uuid")))]
pub type Pk = i64;

#[cfg(not(any(feature = "big-pk", feature = "pk-uuid")))]
pub type Pk = i32;
```

Tout le framework écrit `Pk` (`CurrentUser.id`, `BuiltinUserEntity::find_by_id(&db, id: Pk)`…). Changer de type de clé primaire, c'est changer une feature dans `Cargo.toml`, pas des centaines de signatures. C'est le cas d'école de l'alias : aucune garantie à ajouter, juste un seul endroit qui décide.

### 8.2 Quand un alias ne suffit plus : `ADb`

La base de données a d'abord été un alias, `Arc<DatabaseConnection>`. Problème : chaque appel SeaORM (`.insert(db)`, `.one(db)`, `db.begin()`) attend un type qui implémente `ConnectionTrait`, et `Arc<DatabaseConnection>` ne l'implémente pas. Il fallait écrire `.as_ref()` partout.

Impossible de corriger ça avec un alias : `impl ConnectionTrait for Arc<DatabaseConnection>` est refusé par la règle de l'orphelin (trait et type viennent tous deux d'autres crates — voir 7.3). D'où un newtype :

```rust
// runique/src/db/adb.rs
#[derive(Clone, Debug)]
pub struct ADb(Arc<Inner>);

impl ConnectionTrait for ADb { /* délègue à la connexion */ }
impl TransactionTrait for ADb { /* idem */ }
```

`ADb` reste aussi bon marché à cloner qu'un `Arc`, et `&ADb` s'utilise directement : `.insert(db)`, sans `.as_ref()`.

### 8.3 Alias de collections et de résultats

```rust
// runique/src/utils/aliases/definition.rs
pub type AEngine = Arc<RuniqueEngine>;
pub type StrMap = HashMap<String, String>;          // données de formulaire, erreurs…
pub type JsonMap = HashMap<String, serde_json::Value>;
pub type AppResult<T> = Result<T, Box<AppError>>;

// Dans un handler
pub async fn contact(mut request: Request) -> AppResult<Response> {
    // ...
}
```

La convention suivie : **un alias par type concret**, nommé d'après sa structure (`StrMap`) plutôt que d'après un usage particulier — sinon le même `HashMap<String, String>` finit avec cinq noms.

## 9. Patterns avancés

### 9.1 Alias conditionnels

`Pk` (8.1) en est un exemple : le même nom désigne un type différent selon la configuration de compilation.

```rust
#[cfg(feature = "async")]
pub type Handler = Box<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

#[cfg(not(feature = "async"))]
pub type Handler = Box<dyn Fn() + Send + Sync>;

fn register_handler(handler: Handler) { /* identique dans les deux cas */ }
```

### 9.2 Chaîne de traitement

```rust
type RawData = Vec<u8>;
type ParsedData = Result<Record, ParseError>;
type ValidatedData = Result<Record, ValidationError>;

fn parse(raw: RawData) -> ParsedData { todo!() }
fn validate(parsed: Record) -> ValidatedData { todo!() }
```

### 9.3 Trait objects

```rust
type EventListener = Box<dyn Fn(&Event) + Send + Sync>;
type AsyncHandler = Box<dyn Fn(Request) -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync>;

type EventHandlers = Vec<EventListener>;
```

> **Pattern :** combinez alias et génériques pour des API souples. La bibliothèque standard le fait elle-même : `std::io::Result<T>` est un alias de `Result<T, std::io::Error>`.

## 10. Exercices

### Exercice 1 : refactoring

Introduisez des alias adaptés :

```rust
fn get_user(id: i32, db: &Arc<DatabaseConnection>) -> Result<Option<User>, Box<dyn Error>> { todo!() }
fn create_user(name: String, email: String, db: &Arc<DatabaseConnection>) -> Result<User, Box<dyn Error>> { todo!() }
```

### Exercice 2 : organisation

Rangez ces types dans une hiérarchie de modules :

```rust
type UserId = i32;
type ProductId = i32;
type OrderId = i32;
type UserResult = Result<User, DbErr>;
type ProductResult = Result<Product, DbErr>;
type ApiError = Box<dyn Error + Send + Sync>;
type JsonPayload = serde_json::Value;
```

### Exercice 3 : généricité

Créez des alias pour ce cache :

```rust
struct Cache<K, V> {
    data: HashMap<K, V>,
}
// 1. Un cache de chaînes vers chaînes
// 2. Un résultat de cache avec erreur
// 3. Un cache partagé entre tâches async
```

## Solutions

### Solution 1

```rust
type UserId = i32;
type DbPool = Arc<DatabaseConnection>;
type AppError = Box<dyn Error>;
type AppResult<T> = Result<T, AppError>;

fn get_user(id: UserId, db: &DbPool) -> AppResult<Option<User>> { todo!() }
fn create_user(name: String, email: String, db: &DbPool) -> AppResult<User> { todo!() }
```

### Solution 2

```rust
// src/types/mod.rs
pub mod ids;
pub mod db;
pub mod api;

// src/types/ids.rs
pub type UserId = i32;
pub type ProductId = i32;
pub type OrderId = i32;

// src/types/db.rs
pub type UserResult = Result<User, DbErr>;
pub type ProductResult = Result<Product, DbErr>;

// src/types/api.rs
pub type ApiError = Box<dyn Error + Send + Sync>;
pub type JsonPayload = serde_json::Value;
pub type ApiResult<T> = Result<T, ApiError>;
```

### Solution 3

```rust
// 1.
type StringCache = Cache<String, String>;
// 2.
type CacheResult<V> = Result<V, CacheError>;
// 3.
type SharedCache<K, V> = Arc<tokio::sync::RwLock<Cache<K, V>>>;
```

## Points clés à retenir

- Un alias est un *synonyme*, pas un nouveau type, et ne coûte rien à l'exécution.
- Il sert à nommer et centraliser des types récurrents ou longs.
- Il n'apporte aucune sûreté : deux alias du même type sont interchangeables.
- Dès qu'il faut une garantie, des méthodes ou un trait étranger, on passe au newtype — comme `ADb` dans Runique.
- Les erreurs du compilateur montrent le type réel, pas l'alias.

## Ressources complémentaires

- *The Rust Programming Language*, section « Advanced Types » : type aliases et newtype
- *Rust by Example*, section « Aliasing »
- *Rust API Guidelines* : conventions de nommage
