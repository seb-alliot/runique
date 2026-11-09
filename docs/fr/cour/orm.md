# ORM — Une API à la Django sur SeaORM

> Comment Runique offre `Entity::objects.filter(...).all(db)` au-dessus de SeaORM : les notions Rust utilisées (traits, génériques, `PhantomData`, `const fn`, macros, `Into`), le code réel du gestionnaire et du constructeur de requêtes, et les différentes façons d'écrire la même requête.

### Objectif du cours

Comprendre, ligne par ligne, comment est construite l'API de requêtes de Runique, et savoir l'utiliser : `objects`, `RuniqueQueryBuilder`, `search!`, et SeaORM directement.

## Table des matières

1. Le problème de départ
2. Les notions Rust nécessaires
3. Architecture
4. Le gestionnaire : `Objects<E>`
5. La macro `impl_objects!`
6. Le constructeur de requêtes : `RuniqueQueryBuilder<E>`
7. Lire un seul enregistrement : `get`, `first`, `one`, `get_or_404`
8. La macro `search!`
9. La même requête écrite de plusieurs façons
10. Sécurité des requêtes
11. Étendre l'API dans son projet
12. Exercices pratiques
13. Résumé des concepts

## 1. Le problème de départ

En SeaORM, une requête s'écrit ainsi :

```rust
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

let actifs = users::Entity::find()
    .filter(users::Column::Active.eq(true))
    .order_by_asc(users::Column::Username)
    .all(&db)
    .await?;
```

C'est complet et typé, mais il faut importer les bons traits (`QueryFilter`, `QueryOrder`...), et la forme diffère de celle de Django, que beaucoup de développeurs web connaissent :

```python
User.objects.filter(active=True).order_by("username")
```

Runique propose une forme proche, **sans rien perdre** de la vérification par le compilateur :

```rust
let actifs = users::Entity::objects
    .filter(users::Column::Active.eq(true))
    .order_by_asc(users::Column::Username)
    .all(&db)
    .await?;
```

`objects` est une **constante** (pas de parenthèses), comme l'attribut `objects` de Django. Tout ce qui suit est du SeaORM : la requête SQL produite est exactement la même.

## 2. Les notions Rust nécessaires

### 2.1 Les traits : un comportement commun

SeaORM définit le trait `EntityTrait`, implémenté par chaque entité : il fournit `find()`, `find_by_id()`, le type `Model`, le type de clé primaire. Le gestionnaire de Runique est écrit **une fois** pour tout type qui implémente ce trait (cours `traits-basics`).

```rust
impl<E: EntityTrait> Objects<E> {
    // ces méthodes existent pour toutes les entités
}
```

### 2.2 Les génériques : un code pour toutes les entités

```rust
pub struct Objects<E: EntityTrait> { /* ... */ }
```

`Objects<users::Entity>` et `Objects<article::Entity>` sont deux types différents, générés à la compilation (monomorphisation, cours `generics`) : `Objects<users::Entity>::all()` renvoie des `users::Model`, jamais des `article::Model`.

### 2.3 `PhantomData` : porter un type sans le stocker

```rust
use std::marker::PhantomData;

pub struct Objects<E: EntityTrait> {
    _phantom: PhantomData<E>,
}
```

Le gestionnaire n'a **aucune donnée** à garder : il a seulement besoin de savoir de quelle entité il s'agit. Rust refuse un paramètre de type inutilisé ; `PhantomData<E>` l'utilise sans occuper un seul octet. `Objects<E>` a donc une taille nulle.

### 2.4 `const fn` : construire à la compilation

```rust
impl<E: EntityTrait> Objects<E> {
    pub const fn new() -> Self {
        Self { _phantom: PhantomData }
    }
}
```

Une `const fn` peut être appelée pour initialiser une **constante**. C'est ce qui permet de déclarer `objects` comme une `const` associée (section 5), sans `static` ni initialisation au démarrage.

### 2.5 Les macros : générer la même déclaration pour chaque entité

Écrire à la main `pub const objects: Objects<Self> = Objects::new();` dans chaque entité serait répétitif : une macro `macro_rules!` le fait (section 5, cours `macros-declaratives`).

### 2.6 `Into` : accepter plusieurs formes

```rust
pub fn filter<C>(&self, condition: C) -> RuniqueQueryBuilder<E>
where
    C: Into<Condition>,
```

`Column::Active.eq(true)` produit une `SimpleExpr` ; `Condition::any().add(..)` produit une `Condition`. Grâce à `Into<Condition>`, `filter` accepte les deux (cours `borrow-avance`, section 5).

### 2.7 Le builder par valeur

```rust
pub fn filter<C: Into<Condition>>(mut self, condition: C) -> Self {
    self.query = self.query.filter(condition.into());
    self
}
```

Chaque méthode **prend** le constructeur (`self`), le modifie et le **rend**. On enchaîne donc `.filter(..).order_by_asc(..).limit(..)` sans variable intermédiaire, et le compilateur empêche de réutiliser une requête déjà consommée.

## 3. Architecture

```
users::Entity::objects               Objects<users::Entity>        (constante, taille nulle)
        │ .filter(..) / .all() / .exclude(..)
        ▼
RuniqueQueryBuilder<users::Entity>   { query: Select<users::Entity> }   (enveloppe une requête SeaORM)
        │ .filter(..) .order_by_asc(..) .limit(..)     → renvoient Self
        │ .all(db) .first(db) .one(db) .count(db)      → exécutent (async)
        ▼
SeaORM Select<E>  →  SQL  →  base
```

| Fichier | Rôle |
|---|---|
| `runique/src/macros/bdd/objects.rs` | `Objects<E>` : le point d'entrée |
| `runique/src/macros/bdd/query.rs` | `RuniqueQueryBuilder<E>` : la requête qu'on construit |
| `runique/src/macros/bdd/impl_objects.rs` | `impl_objects!` : ajoute `objects` à une entité |
| `runique/src/macros/bdd/filter.rs` | `search!` et ses opérateurs |

## 4. Le gestionnaire : `Objects<E>`

```rust
// runique/src/macros/bdd/objects.rs (extrait)
pub struct Objects<E: EntityTrait> {
    _phantom: PhantomData<E>,
}

impl<E: EntityTrait> Objects<E> {
    pub const fn new() -> Self {
        Self { _phantom: PhantomData }
    }

    /// Returns a builder over every row of the entity — equivalent to `E::find()`.
    pub fn all(&self) -> RuniqueQueryBuilder<E> {
        RuniqueQueryBuilder::new(E::find())
    }

    /// Starts a query builder with a `WHERE` condition already applied.
    pub fn filter<C>(&self, condition: C) -> RuniqueQueryBuilder<E>
    where
        C: Into<Condition>,
    {
        RuniqueQueryBuilder::new(E::find()).filter(condition.into())
    }

    /// Starts a query builder that excludes rows matching `condition`.
    pub fn exclude<C>(&self, condition: C) -> RuniqueQueryBuilder<E>
    where
        C: Into<Condition>,
    {
        RuniqueQueryBuilder::new(E::find()).exclude(condition.into())
    }

    /// Fetches the row with primary key `id`, or `DbErr::RecordNotFound` if none exists.
    pub async fn get(
        &self,
        db: &ADb,
        id: impl Into<<E::PrimaryKey as sea_orm::PrimaryKeyTrait>::ValueType>,
    ) -> Result<E::Model, DbErr> {
        E::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Record not found".to_string()))
    }
}

impl<E: EntityTrait> Copy for Objects<E> {}
```

À lire :

- `all`, `filter`, `exclude` ne font **aucune** requête : ils créent un `RuniqueQueryBuilder` ;
- `get` exécute directement (`async`) ;
- le type de l'identifiant est **celui de la clé primaire de l'entité** (`<E::PrimaryKey as PrimaryKeyTrait>::ValueType`) : passer un texte à une entité dont la clé est un `i32` est une erreur de compilation ;
- `Copy` est implémenté à la main : le `#[derive(Copy)]` exigerait `E: Copy`, inutile puisque rien de `E` n'est stocké.

| Méthode de `Objects<E>` | Exécute ? | Renvoie |
|---|---|---|
| `all()` | non | `RuniqueQueryBuilder<E>` |
| `filter(cond)` / `exclude(cond)` | non | `RuniqueQueryBuilder<E>` |
| `filter_many(paires)` / `exclude_many(paires)` | non | `RuniqueQueryBuilder<E>` |
| `get(db, id)` | oui | `Result<Model, DbErr>` (`RecordNotFound` si absent) |
| `get_optional(db, id)` | oui | `Result<Option<Model>, DbErr>` |
| `count(db)` | oui | `Result<u64, DbErr>` |
| `get_or_404(db, id, &request, message)` | oui | `Result<Model, Response>` (page 404 ou 500) |

## 5. La macro `impl_objects!`

```rust
// runique/src/macros/bdd/impl_objects.rs
#[macro_export]
macro_rules! impl_objects {
    ($entity:ty) => {
        impl $entity {
            #[allow(non_upper_case_globals)]
            pub const objects: $crate::macros::bdd::objects::Objects<Self> =
                $crate::macros::bdd::objects::Objects::new();
        }
    };
}
```

- Une **constante associée** à l'entité : `users::Entity::objects`, sans parenthèses, comme en Django.
- `#[allow(non_upper_case_globals)]` : une constante s'écrit normalement en majuscules ; le nom `objects` est gardé pour la ressemblance avec Django.
- `$crate::` : la macro fonctionne depuis le projet de l'utilisateur (cours `macros-export`).
- `Objects::new()` est une `const fn` : sans elle, cette constante serait impossible.

### Qui l'appelle

Dans un projet Runique, on n'écrit jamais `impl_objects!` soi-même : le code généré par `model!{}` l'appelle pour chaque entité (`runique/derive_form/src/model/utils/active_model.rs`). Pour une entité SeaORM écrite à la main, une ligne suffit :

```rust
runique::impl_objects!(Entity);
```

## 6. Le constructeur de requêtes : `RuniqueQueryBuilder<E>`

### 6.1 La structure

```rust
// runique/src/macros/bdd/query.rs (extrait)
pub struct RuniqueQueryBuilder<E: EntityTrait> {
    query: Select<E>,                // la requête SeaORM sous-jacente
}

impl<E: EntityTrait> RuniqueQueryBuilder<E> {
    pub fn filter<C: Into<Condition>>(mut self, condition: C) -> Self {
        self.query = self.query.filter(condition.into());
        self
    }

    pub fn exclude<C: Into<Condition>>(mut self, condition: C) -> Self {
        self.query = self.query.filter(condition.into().not());   // NOT (condition)
        self
    }

    pub async fn all(self, db: &ADb) -> Result<Vec<E::Model>, DbErr> {
        self.query.all(db).await
    }
}
```

Un **newtype** autour de `Select<E>` (cours `structures-enums`, section 4) : il ajoute des noms à la Django sans réécrire la logique de SeaORM.

### 6.2 Toutes les méthodes

| Méthode | Effet SQL |
|---|---|
| `filter(cond)` | `AND cond` |
| `exclude(cond)` | `AND NOT cond` |
| `filter_many([(col, val), ..])` | `AND col = val` pour chaque paire |
| `exclude_many([(col, val), ..])` | `AND col <> val` pour chaque paire |
| `order_by_asc(col)` / `asc(col)` | `ORDER BY col ASC` |
| `order_by_desc(col)` / `desc(col)` | `ORDER BY col DESC` |
| `order_by_random()` | ordre aléatoire |
| `limit(n)` / `offset(n)` | `LIMIT` / `OFFSET` |
| `join(rel)` / `left_join(rel)` | `INNER JOIN` / `LEFT JOIN` |
| `also_related(Entite)` | charge l'entité liée en même temps : `Vec<(Model, Option<Lie>)>` |
| `into_select()` | rend le `Select<E>` SeaORM, pour toute opération non prévue |
| `all(db)` | exécute, `Vec<Model>` |
| `first(db)` | exécute, `Option<Model>`, le premier |
| `one(db)` | exécute, `Option<Model>`, **erreur si plus d'une ligne** |
| `count(db)` | exécute, `u64` |
| `get_or_404(db, &request, message)` | exécute, la ligne ou une page 404 / 500 |

### 6.3 Exemples

```rust
use sea_orm::ColumnTrait;

// Plusieurs conditions : ET
let adultes_actifs = users::Entity::objects
    .filter(users::Column::Active.eq(true))
    .filter(users::Column::Age.gte(18))
    .order_by_desc(users::Column::CreatedAt)
    .limit(20)
    .all(&db)
    .await?;

// OU : une Condition
use sea_orm::Condition;
let deux = users::Entity::objects
    .filter(
        Condition::any()
            .add(users::Column::Email.eq("a@ex.com"))
            .add(users::Column::Email.eq("b@ex.com")),
    )
    .all(&db)
    .await?;

// Pagination
let page = 3u64;
let par_page = 20u64;
let articles = article::Entity::objects
    .all()
    .order_by_desc(article::Column::Id)
    .limit(par_page)
    .offset((page - 1) * par_page)
    .all(&db)
    .await?;

// Avec l'auteur, en une requête
let avec_auteur = article::Entity::objects
    .all()
    .also_related(users::Entity)
    .all(&db)
    .await?;                        // Vec<(article::Model, Option<users::Model>)>
```

### 6.4 Sortir vers SeaORM

```rust
use sea_orm::{QuerySelect, EntityTrait};

let select = article::Entity::objects
    .filter(article::Column::Publie.eq(true))
    .into_select();                 // Select<article::Entity>

let titres: Vec<String> = select
    .select_only()
    .column(article::Column::Titre)
    .into_tuple()
    .all(&db)
    .await?;
```

Le constructeur ne cache rien : tout ce que SeaORM sait faire reste accessible.

## 7. Lire un seul enregistrement : `get`, `first`, `one`, `get_or_404`

```rust
// Par la clé primaire
let u = users::Entity::objects.get(&db, 5).await?;              // Err(RecordNotFound) si absent
let u = users::Entity::objects.get_optional(&db, 5).await?;     // None si absent

// Par une condition
let u = users::Entity::objects
    .filter(users::Column::Email.eq("ada@ex.com"))
    .first(&db).await?;                                          // le premier, même s'il y en a plusieurs

let u = users::Entity::objects
    .filter(users::Column::Email.eq("ada@ex.com"))
    .one(&db).await?;                                            // Err si deux lignes correspondent
```

### `first` ou `one`

```rust
// runique/src/macros/bdd/query.rs (extrait)
pub async fn one(self, db: &ADb) -> Result<Option<E::Model>, DbErr> {
    let mut results = self.query.paginate(db, 2).fetch_page(0).await?;   // au plus 2 lignes
    match results.len() {
        0 => Ok(None),
        1 => Ok(Some(results.remove(0))),
        _ => Err(DbErr::Custom("search!.one(): multiple rows returned".to_string())),
    }
}
```

`one` lit **deux** lignes au plus pour vérifier qu'il n'y en a qu'une. À utiliser quand l'unicité est une hypothèse de sécurité (retrouver un compte par son e-mail, un jeton par sa valeur) : si la base contient un doublon, mieux vaut une erreur qu'un compte pris au hasard. `first` convient quand plusieurs résultats sont normaux et que l'ordre décide.

### Dans un handler : `get_or_404`

```rust
use runique::prelude::*;

pub async fn detail(request: Request) -> AppResult<Response> {
    let id = request.get_path_as::<Pk>("id").unwrap_or_default();

    let article = match article::Entity::objects
        .get_or_404(request.db(), id, &request, "Article introuvable")
        .await
    {
        Ok(a) => a,
        Err(reponse) => return Ok(reponse),       // page 404 (ou 500 si la requête a échoué)
    };
    // ...
}
```

Deux formes existent : `Objects::get_or_404(db, id, &request, message)` par la clé primaire, et `RuniqueQueryBuilder::get_or_404(db, &request, message)` après des filtres.

## 8. La macro `search!`

`search!` écrit les conditions avec des opérateurs inspirés de Django et renvoie un `RuniqueQueryBuilder` :

```rust
use runique::search;

let actifs = search!(users::Entity => Active eq true).all(&db).await?;
let adultes = search!(users::Entity => Age gte 18).all(&db).await?;
let sauf = search!(users::Entity => ! Level eq 99).all(&db).await?;
let rust = search!(posts::Entity => Title icontains "rust").all(&db).await?;
let choisis = search!(users::Entity => Id in [1, 2, 3]).all(&db).await?;
let tous = search!(users::Entity).order_by_asc(users::Column::Username).all(&db).await?;
```

| Opérateur | SQL |
|---|---|
| `eq`, `ne`, `gt`, `gte`, `lt`, `lte` | `=`, `<>`, `>`, `>=`, `<`, `<=` |
| `contains`, `icontains`, `startswith`, `endswith`, `iexact` | `LIKE` / `ILIKE` avec jokers placés par la macro, valeur prise littéralement |
| `like`, `ilike` | motif écrit par toi, jokers compris |
| `isnull`, `not_null` | `IS NULL`, `IS NOT NULL` |
| `in [..]`, `not_in [..]`, `in (expr)` | `IN`, `NOT IN` |
| `range (a, b)` | `BETWEEN` |
| `! Col op val` | `NOT (...)` |
| `or(C1 op v, C2 op v)` | `... OR ...` |

Les noms de colonnes (`Active`, `Age`) sont ceux de l'enum `Column` de l'entité : une faute de frappe est une erreur de compilation. La documentation complète est dans la page « Requêtes » de l'ORM.

## 9. La même requête écrite de plusieurs façons

Les articles publiés dont le titre contient « rust », du plus récent au plus ancien, 10 au maximum.

```rust
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};

// 1. SeaORM direct
let a = article::Entity::find()
    .filter(article::Column::Publie.eq(true))
    .filter(article::Column::Titre.contains("rust"))
    .order_by_desc(article::Column::CreatedAt)
    .limit(10)
    .all(&db)
    .await?;

// 2. objects (gestionnaire Runique)
let b = article::Entity::objects
    .filter(article::Column::Publie.eq(true))
    .filter(article::Column::Titre.contains("rust"))
    .order_by_desc(article::Column::CreatedAt)
    .limit(10)
    .all(&db)
    .await?;

// 3. filter_many pour les égalités
let c = article::Entity::objects
    .filter_many([(article::Column::Publie, true)])
    .filter(article::Column::Titre.contains("rust"))
    .desc(article::Column::CreatedAt)
    .limit(10)
    .all(&db)
    .await?;

// 4. search!
let d = search!(article::Entity => Publie eq true, Titre icontains "rust")
    .order_by_desc(article::Column::CreatedAt)
    .limit(10)
    .all(&db)
    .await?;
```

| Forme | Imports nécessaires | Lisibilité | Pour |
|---|---|---|---|
| SeaORM | traits `QueryFilter`, `QueryOrder`, `QuerySelect` | bonne | tout, y compris l'avancé |
| `objects` | aucun trait de requête | bonne, proche de Django | le cas courant |
| `filter_many` | aucun | courte pour plusieurs égalités | filtres simples en lot |
| `search!` | `search` | la plus courte | filtres avec opérateurs Django (`icontains`, `in`, `range`) |

Les quatre produisent du SQL équivalent (la version `search!` utilise `ILIKE` pour `icontains`, insensible à la casse). On peut mélanger : partir de `search!` et continuer avec `.order_by_desc(..)`, ou sortir vers SeaORM avec `.into_select()`.

## 10. Sécurité des requêtes

- **Pas de SQL écrit à la main** : toutes les valeurs passent par des paramètres liés. Une saisie comme `'; DROP TABLE users; --` reste une simple chaîne comparée à une colonne.
- **Opérateurs de recherche** : `contains`, `icontains`, `startswith`, `endswith` et `iexact` placent eux-mêmes leurs jokers et prennent la valeur **littéralement** : un `%` saisi par un visiteur ne transforme pas une recherche précise en « tout correspond ». Pour un motif avec jokers, `like` / `ilike` sont explicites.
- **Unicité** : `one` plutôt que `first` quand la présence de deux lignes doit être traitée comme une anomalie (section 7).
- **Droits** : dans l'admin, les lignes visibles par un compte sont limitées par `filter_by_gate(&preuve)`, qui ajoute à la requête les conditions décidées par la porte d'autorisation. La preuve `Authorized` ne peut être fabriquée que par la porte : impossible d'appeler cette méthode avec des droits inventés.

## 11. Étendre l'API dans son projet

On ne peut pas ajouter de méthode à `RuniqueQueryBuilder` depuis son projet (le type appartient à Runique), mais un **trait d'extension** le permet (cours `traits-basics`, section 13.3) :

```rust
use runique::prelude::*;
use sea_orm::{DbErr, EntityTrait};

pub trait RequeteExt<E: EntityTrait> {
    async fn exists(self, db: &ADb) -> Result<bool, DbErr>;
}

impl<E: EntityTrait> RequeteExt<E> for RuniqueQueryBuilder<E>
where
    E::Model: Sync,
{
    async fn exists(self, db: &ADb) -> Result<bool, DbErr> {
        Ok(self.limit(1).count(db).await? > 0)
    }
}

// Utilisation
let deja_inscrit = users::Entity::objects
    .filter(users::Column::Email.eq(email))
    .exists(&db)
    .await?;
```

Un trait avec `async fn` est utilisable ici car on l'appelle sur un type concret (cours `traits-avances`, section 11).

## 12. Exercices pratiques

### Exercice 1 : `last()`

Écris, dans un trait d'extension, `last(self, colonne, db)` qui renvoie le dernier enregistrement selon une colonne.

```rust
// Solution :
pub trait DernierExt<E: EntityTrait> {
    async fn last<C: sea_orm::ColumnTrait + Send>(self, colonne: C, db: &ADb)
        -> Result<Option<E::Model>, DbErr>;
}

impl<E: EntityTrait> DernierExt<E> for RuniqueQueryBuilder<E> {
    async fn last<C: sea_orm::ColumnTrait + Send>(self, colonne: C, db: &ADb)
        -> Result<Option<E::Model>, DbErr>
    {
        self.order_by_desc(colonne).first(db).await
    }
}
```

### Exercice 2 : `exists()`

Pourquoi `self.limit(1).count(db)` plutôt que `self.all(db).await?.is_empty()` ?

Réponse : `all` charge toutes les lignes correspondantes en mémoire pour n'en regarder que la présence ; `count` avec `limit(1)` ne demande à la base qu'un nombre. Sur une table d'un million de lignes, la différence est énorme.

### Exercice 3 : quatre écritures

Écris de quatre façons (SeaORM, `objects`, `filter_many`, `search!`) la requête « utilisateurs actifs de niveau 2 ».

```rust
// Solution :
let a = users::Entity::find()
    .filter(users::Column::Active.eq(true))
    .filter(users::Column::Level.eq(2))
    .all(&db).await?;
let b = users::Entity::objects
    .filter(users::Column::Active.eq(true))
    .filter(users::Column::Level.eq(2))
    .all(&db).await?;
let c = users::Entity::objects
    .filter_many([(users::Column::Active, sea_orm::Value::from(true)), (users::Column::Level, 2.into())])
    .all(&db).await?;
let d = search!(users::Entity => Active eq true, Level eq 2).all(&db).await?;
```

Dans `filter_many`, toutes les valeurs doivent avoir le même type : on les convertit en `sea_orm::Value` quand les colonnes sont de types différents.

## 13. Résumé des concepts

| Notion Rust | Où dans l'ORM | Ce qu'elle apporte |
|---|---|---|
| trait (`EntityTrait`) | `impl<E: EntityTrait> Objects<E>` | un code pour toutes les entités |
| générique | `Objects<E>`, `RuniqueQueryBuilder<E>` | les résultats ont le type exact de l'entité |
| `PhantomData` | champ unique de `Objects<E>` | un type porté sans donnée, taille nulle |
| `const fn` | `Objects::new()` | `objects` peut être une constante |
| `macro_rules!` | `impl_objects!` | une ligne par entité, générée par `model!{}` |
| `Into<Condition>` | `filter`, `exclude` | accepter expressions et conditions composées |
| builder par valeur | toutes les méthodes du constructeur | enchaînement sans variable intermédiaire |
| newtype | `RuniqueQueryBuilder` autour de `Select` | ajouter une API sans réécrire SeaORM |
| type associé | `<E::PrimaryKey as PrimaryKeyTrait>::ValueType` | l'identifiant a le type de la clé |
| trait d'extension | `RequeteExt` | ajouter ses propres méthodes |

### Points clés à retenir

- `Entity::objects` est une constante de taille nulle ; `all`, `filter` et `exclude` ne font aucune requête.
- Les méthodes `async` (`all`, `first`, `one`, `count`, `get`) exécutent.
- `one` refuse les doublons, `first` prend le premier.
- `search!` pour les opérateurs à la Django, `into_select()` pour tout le reste de SeaORM.
- Jamais de SQL écrit à la main : les valeurs sont toujours des paramètres.
