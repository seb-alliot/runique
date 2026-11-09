# ORM — A Django-like API on top of SeaORM

> How Runique offers `Entity::objects.filter(...).all(db)` on top of SeaORM: the Rust notions used (traits, generics, `PhantomData`, `const fn`, macros, `Into`), the real code of the manager and the query builder, and the different ways to write the same query.

### Course objective

Understand, line by line, how Runique's query API is built, and know how to use it: `objects`, `RuniqueQueryBuilder`, `search!`, and SeaORM directly.

## Table of contents

1. The starting problem
2. The Rust notions needed
3. Architecture
4. The manager: `Objects<E>`
5. The `impl_objects!` macro
6. The query builder: `RuniqueQueryBuilder<E>`
7. Reading a single record: `get`, `first`, `one`, `get_or_404`
8. The `search!` macro
9. The same query written in several ways
10. Query security
11. Extending the API in your project
12. Practical exercises
13. Summary of concepts

## 1. The starting problem

In SeaORM, a query is written like this:

```rust
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

let active = users::Entity::find()
    .filter(users::Column::Active.eq(true))
    .order_by_asc(users::Column::Username)
    .all(&db)
    .await?;
```

It is complete and typed, but you have to import the right traits (`QueryFilter`, `QueryOrder`...), and the shape differs from Django's, which many web developers know:

```python
User.objects.filter(active=True).order_by("username")
```

Runique offers a close form, **without losing anything** of the compiler's checking:

```rust
let active = users::Entity::objects
    .filter(users::Column::Active.eq(true))
    .order_by_asc(users::Column::Username)
    .all(&db)
    .await?;
```

`objects` is a **constant** (no parentheses), like Django's `objects` attribute. Everything that follows is SeaORM: the SQL produced is exactly the same.

## 2. The Rust notions needed

### 2.1 Traits: a shared behaviour

SeaORM defines the `EntityTrait` trait, implemented by each entity: it provides `find()`, `find_by_id()`, the `Model` type, the primary key type. Runique's manager is written **once** for any type that implements this trait (course `traits-basics`).

```rust
impl<E: EntityTrait> Objects<E> {
    // these methods exist for all entities
}
```

### 2.2 Generics: one piece of code for all entities

```rust
pub struct Objects<E: EntityTrait> { /* ... */ }
```

`Objects<users::Entity>` and `Objects<article::Entity>` are two different types, generated at compile time (monomorphisation, course `generics`): `Objects<users::Entity>::all()` returns `users::Model`, never `article::Model`.

### 2.3 `PhantomData`: carry a type without storing it

```rust
use std::marker::PhantomData;

pub struct Objects<E: EntityTrait> {
    _phantom: PhantomData<E>,
}
```

The manager has **no data** to keep: it only needs to know which entity it is about. Rust refuses an unused type parameter; `PhantomData<E>` uses it without taking up a single byte. So `Objects<E>` has a size of zero.

### 2.4 `const fn`: build at compile time

```rust
impl<E: EntityTrait> Objects<E> {
    pub const fn new() -> Self {
        Self { _phantom: PhantomData }
    }
}
```

A `const fn` can be called to initialise a **constant**. This is what allows `objects` to be declared as an associated `const` (section 5), without a `static` or start-up initialisation.

### 2.5 Macros: generate the same declaration for each entity

Writing `pub const objects: Objects<Self> = Objects::new();` by hand in each entity would be repetitive: a `macro_rules!` macro does it (section 5, course `macros-declaratives`).

### 2.6 `Into`: accept several forms

```rust
pub fn filter<C>(&self, condition: C) -> RuniqueQueryBuilder<E>
where
    C: Into<Condition>,
```

`Column::Active.eq(true)` produces a `SimpleExpr`; `Condition::any().add(..)` produces a `Condition`. Thanks to `Into<Condition>`, `filter` accepts both (course `borrow-avance`, section 5).

### 2.7 The by-value builder

```rust
pub fn filter<C: Into<Condition>>(mut self, condition: C) -> Self {
    self.query = self.query.filter(condition.into());
    self
}
```

Each method **takes** the builder (`self`), modifies it and **gives it back**. So you chain `.filter(..).order_by_asc(..).limit(..)` without an intermediate variable, and the compiler prevents reusing a query that has already been consumed.

## 3. Architecture

```
users::Entity::objects               Objects<users::Entity>        (constant, zero size)
        │ .filter(..) / .all() / .exclude(..)
        ▼
RuniqueQueryBuilder<users::Entity>   { query: Select<users::Entity> }   (wraps a SeaORM query)
        │ .filter(..) .order_by_asc(..) .limit(..)     → return Self
        │ .all(db) .first(db) .one(db) .count(db)      → execute (async)
        ▼
SeaORM Select<E>  →  SQL  →  database
```

| File | Role |
|---|---|
| `runique/src/macros/bdd/objects.rs` | `Objects<E>`: the entry point |
| `runique/src/macros/bdd/query.rs` | `RuniqueQueryBuilder<E>`: the query being built |
| `runique/src/macros/bdd/impl_objects.rs` | `impl_objects!`: adds `objects` to an entity |
| `runique/src/macros/bdd/filter.rs` | `search!` and its operators |

## 4. The manager: `Objects<E>`

```rust
// runique/src/macros/bdd/objects.rs (excerpt)
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

To note:

- `all`, `filter`, `exclude` make **no** query: they create a `RuniqueQueryBuilder`;
- `get` executes directly (`async`);
- the type of the identifier is **that of the entity's primary key** (`<E::PrimaryKey as PrimaryKeyTrait>::ValueType`): passing a string to an entity whose key is an `i32` is a compile error;
- `Copy` is implemented by hand: `#[derive(Copy)]` would require `E: Copy`, which is useless since nothing of `E` is stored.

| Method of `Objects<E>` | Executes? | Returns |
|---|---|---|
| `all()` | no | `RuniqueQueryBuilder<E>` |
| `filter(cond)` / `exclude(cond)` | no | `RuniqueQueryBuilder<E>` |
| `filter_many(pairs)` / `exclude_many(pairs)` | no | `RuniqueQueryBuilder<E>` |
| `get(db, id)` | yes | `Result<Model, DbErr>` (`RecordNotFound` if missing) |
| `get_optional(db, id)` | yes | `Result<Option<Model>, DbErr>` |
| `count(db)` | yes | `Result<u64, DbErr>` |
| `get_or_404(db, id, &request, message)` | yes | `Result<Model, Response>` (404 or 500 page) |

## 5. The `impl_objects!` macro

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

- A **constant associated** with the entity: `users::Entity::objects`, without parentheses, like in Django.
- `#[allow(non_upper_case_globals)]`: a constant is normally written in capitals; the name `objects` is kept to resemble Django.
- `$crate::`: the macro works from the user's project (course `macros-export`).
- `Objects::new()` is a `const fn`: without it, this constant would be impossible.

### Who calls it

In a Runique project, you never write `impl_objects!` yourself: the code generated by `model!{}` calls it for each entity (`runique/derive_form/src/model/utils/active_model.rs`). For a hand-written SeaORM entity, one line is enough:

```rust
runique::impl_objects!(Entity);
```

## 6. The query builder: `RuniqueQueryBuilder<E>`

### 6.1 The structure

```rust
// runique/src/macros/bdd/query.rs (excerpt)
pub struct RuniqueQueryBuilder<E: EntityTrait> {
    query: Select<E>,                // the underlying SeaORM query
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

A **newtype** around `Select<E>` (course `structures-enums`, section 4): it adds Django-style names without rewriting SeaORM's logic.

### 6.2 All the methods

| Method | SQL effect |
|---|---|
| `filter(cond)` | `AND cond` |
| `exclude(cond)` | `AND NOT cond` |
| `filter_many([(col, val), ..])` | `AND col = val` for each pair |
| `exclude_many([(col, val), ..])` | `AND col <> val` for each pair |
| `order_by_asc(col)` / `asc(col)` | `ORDER BY col ASC` |
| `order_by_desc(col)` / `desc(col)` | `ORDER BY col DESC` |
| `order_by_random()` | random order |
| `limit(n)` / `offset(n)` | `LIMIT` / `OFFSET` |
| `join(rel)` / `left_join(rel)` | `INNER JOIN` / `LEFT JOIN` |
| `also_related(Entity)` | loads the related entity at the same time: `Vec<(Model, Option<Related>)>` |
| `into_select()` | gives back the SeaORM `Select<E>`, for any operation not provided |
| `all(db)` | executes, `Vec<Model>` |
| `first(db)` | executes, `Option<Model>`, the first one |
| `one(db)` | executes, `Option<Model>`, **error if more than one row** |
| `count(db)` | executes, `u64` |
| `get_or_404(db, &request, message)` | executes, the row or a 404 / 500 page |

### 6.3 Examples

```rust
use sea_orm::ColumnTrait;

// Several conditions: AND
let active_adults = users::Entity::objects
    .filter(users::Column::Active.eq(true))
    .filter(users::Column::Age.gte(18))
    .order_by_desc(users::Column::CreatedAt)
    .limit(20)
    .all(&db)
    .await?;

// OR: a Condition
use sea_orm::Condition;
let two = users::Entity::objects
    .filter(
        Condition::any()
            .add(users::Column::Email.eq("a@ex.com"))
            .add(users::Column::Email.eq("b@ex.com")),
    )
    .all(&db)
    .await?;

// Pagination
let page = 3u64;
let per_page = 20u64;
let articles = article::Entity::objects
    .all()
    .order_by_desc(article::Column::Id)
    .limit(per_page)
    .offset((page - 1) * per_page)
    .all(&db)
    .await?;

// With the author, in one query
let with_author = article::Entity::objects
    .all()
    .also_related(users::Entity)
    .all(&db)
    .await?;                        // Vec<(article::Model, Option<users::Model>)>
```

### 6.4 Dropping down to SeaORM

```rust
use sea_orm::{QuerySelect, EntityTrait};

let select = article::Entity::objects
    .filter(article::Column::Published.eq(true))
    .into_select();                 // Select<article::Entity>

let titles: Vec<String> = select
    .select_only()
    .column(article::Column::Title)
    .into_tuple()
    .all(&db)
    .await?;
```

The builder hides nothing: everything SeaORM can do stays accessible.

## 7. Reading a single record: `get`, `first`, `one`, `get_or_404`

```rust
// By primary key
let u = users::Entity::objects.get(&db, 5).await?;              // Err(RecordNotFound) if missing
let u = users::Entity::objects.get_optional(&db, 5).await?;     // None if missing

// By a condition
let u = users::Entity::objects
    .filter(users::Column::Email.eq("ada@ex.com"))
    .first(&db).await?;                                          // the first one, even if there are several

let u = users::Entity::objects
    .filter(users::Column::Email.eq("ada@ex.com"))
    .one(&db).await?;                                            // Err if two rows match
```

### `first` or `one`

```rust
// runique/src/macros/bdd/query.rs (excerpt)
pub async fn one(self, db: &ADb) -> Result<Option<E::Model>, DbErr> {
    let mut results = self.query.paginate(db, 2).fetch_page(0).await?;   // at most 2 rows
    match results.len() {
        0 => Ok(None),
        1 => Ok(Some(results.remove(0))),
        _ => Err(DbErr::Custom("search!.one(): multiple rows returned".to_string())),
    }
}
```

`one` reads **two** rows at most to check that there is only one. Use it when uniqueness is a security assumption (finding an account by its email, a token by its value): if the database contains a duplicate, an error is better than an account picked at random. `first` fits when several results are normal and the order decides.

### In a handler: `get_or_404`

```rust
use runique::prelude::*;

pub async fn detail(request: Request) -> AppResult<Response> {
    let id = request.get_path_as::<Pk>("id").unwrap_or_default();

    let article = match article::Entity::objects
        .get_or_404(request.db(), id, &request, "Article not found")
        .await
    {
        Ok(a) => a,
        Err(response) => return Ok(response),       // 404 page (or 500 if the query failed)
    };
    // ...
}
```

Two forms exist: `Objects::get_or_404(db, id, &request, message)` by primary key, and `RuniqueQueryBuilder::get_or_404(db, &request, message)` after filters.

## 8. The `search!` macro

`search!` writes conditions with Django-inspired operators and returns a `RuniqueQueryBuilder`:

```rust
use runique::search;

let active = search!(users::Entity => Active eq true).all(&db).await?;
let adults = search!(users::Entity => Age gte 18).all(&db).await?;
let except = search!(users::Entity => ! Level eq 99).all(&db).await?;
let rust = search!(posts::Entity => Title icontains "rust").all(&db).await?;
let chosen = search!(users::Entity => Id in [1, 2, 3]).all(&db).await?;
let everyone = search!(users::Entity).order_by_asc(users::Column::Username).all(&db).await?;
```

| Operator | SQL |
|---|---|
| `eq`, `ne`, `gt`, `gte`, `lt`, `lte` | `=`, `<>`, `>`, `>=`, `<`, `<=` |
| `contains`, `icontains`, `startswith`, `endswith`, `iexact` | `LIKE` / `ILIKE` with wildcards placed by the macro, value taken literally |
| `like`, `ilike` | pattern written by you, wildcards included |
| `isnull`, `not_null` | `IS NULL`, `IS NOT NULL` |
| `in [..]`, `not_in [..]`, `in (expr)` | `IN`, `NOT IN` |
| `range (a, b)` | `BETWEEN` |
| `! Col op val` | `NOT (...)` |
| `or(C1 op v, C2 op v)` | `... OR ...` |

The column names (`Active`, `Age`) are those of the entity's `Column` enum: a typo is a compile error. The full documentation is in the "Queries" page of the ORM.

## 9. The same query written in several ways

The published articles whose title contains "rust", from newest to oldest, 10 at most.

```rust
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};

// 1. SeaORM directly
let a = article::Entity::find()
    .filter(article::Column::Published.eq(true))
    .filter(article::Column::Title.contains("rust"))
    .order_by_desc(article::Column::CreatedAt)
    .limit(10)
    .all(&db)
    .await?;

// 2. objects (Runique manager)
let b = article::Entity::objects
    .filter(article::Column::Published.eq(true))
    .filter(article::Column::Title.contains("rust"))
    .order_by_desc(article::Column::CreatedAt)
    .limit(10)
    .all(&db)
    .await?;

// 3. filter_many for equalities
let c = article::Entity::objects
    .filter_many([(article::Column::Published, true)])
    .filter(article::Column::Title.contains("rust"))
    .desc(article::Column::CreatedAt)
    .limit(10)
    .all(&db)
    .await?;

// 4. search!
let d = search!(article::Entity => Published eq true, Title icontains "rust")
    .order_by_desc(article::Column::CreatedAt)
    .limit(10)
    .all(&db)
    .await?;
```

| Form | Imports needed | Readability | For |
|---|---|---|---|
| SeaORM | traits `QueryFilter`, `QueryOrder`, `QuerySelect` | good | everything, including advanced cases |
| `objects` | no query trait | good, close to Django | the common case |
| `filter_many` | none | short for several equalities | simple filters in bulk |
| `search!` | `search` | the shortest | filters with Django operators (`icontains`, `in`, `range`) |

All four produce equivalent SQL (the `search!` version uses `ILIKE` for `icontains`, which is case-insensitive). You can mix them: start from `search!` and continue with `.order_by_desc(..)`, or drop down to SeaORM with `.into_select()`.

## 10. Query security

- **No hand-written SQL**: all values go through bound parameters. An input such as `'; DROP TABLE users; --` stays a simple string compared with a column.
- **Search operators**: `contains`, `icontains`, `startswith`, `endswith` and `iexact` place their own wildcards and take the value **literally**: a `%` typed by a visitor does not turn a precise search into "everything matches". For a pattern with wildcards, `like` / `ilike` are explicit.
- **Uniqueness**: `one` rather than `first` when the presence of two rows must be treated as an anomaly (section 7).
- **Permissions**: in the admin, the rows visible to an account are limited by `filter_by_gate(&proof)`, which adds to the query the conditions decided by the authorisation gate. The `Authorized` proof can only be built by the gate: it is impossible to call this method with made-up permissions.

## 11. Extending the API in your project

You cannot add a method to `RuniqueQueryBuilder` from your project (the type belongs to Runique), but an **extension trait** allows it (course `traits-basics`, section 13.3):

```rust
use runique::prelude::*;
use sea_orm::{DbErr, EntityTrait};

pub trait QueryExt<E: EntityTrait> {
    async fn exists(self, db: &ADb) -> Result<bool, DbErr>;
}

impl<E: EntityTrait> QueryExt<E> for RuniqueQueryBuilder<E>
where
    E::Model: Sync,
{
    async fn exists(self, db: &ADb) -> Result<bool, DbErr> {
        Ok(self.limit(1).count(db).await? > 0)
    }
}

// Usage
let already_registered = users::Entity::objects
    .filter(users::Column::Email.eq(email))
    .exists(&db)
    .await?;
```

A trait with `async fn` is usable here because it is called on a concrete type (course `traits-avances`, section 11).

## 12. Practical exercises

### Exercise 1: `last()`

Write, in an extension trait, `last(self, column, db)` that returns the last record according to a column.

```rust
// Solution:
pub trait LastExt<E: EntityTrait> {
    async fn last<C: sea_orm::ColumnTrait + Send>(self, column: C, db: &ADb)
        -> Result<Option<E::Model>, DbErr>;
}

impl<E: EntityTrait> LastExt<E> for RuniqueQueryBuilder<E> {
    async fn last<C: sea_orm::ColumnTrait + Send>(self, column: C, db: &ADb)
        -> Result<Option<E::Model>, DbErr>
    {
        self.order_by_desc(column).first(db).await
    }
}
```

### Exercise 2: `exists()`

Why `self.limit(1).count(db)` rather than `self.all(db).await?.is_empty()`?

Answer: `all` loads all the matching rows into memory just to check whether any is present; `count` with `limit(1)` only asks the database for a number. On a table of a million rows, the difference is huge.

### Exercise 3: four ways to write it

Write the query "active users of level 2" in four ways (SeaORM, `objects`, `filter_many`, `search!`).

```rust
// Solution:
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

In `filter_many`, all the values must have the same type: convert them to `sea_orm::Value` when the columns have different types.

## 13. Summary of concepts

| Rust notion | Where in the ORM | What it brings |
|---|---|---|
| trait (`EntityTrait`) | `impl<E: EntityTrait> Objects<E>` | one piece of code for all entities |
| generic | `Objects<E>`, `RuniqueQueryBuilder<E>` | results have the exact type of the entity |
| `PhantomData` | only field of `Objects<E>` | a type carried without data, zero size |
| `const fn` | `Objects::new()` | `objects` can be a constant |
| `macro_rules!` | `impl_objects!` | one line per entity, generated by `model!{}` |
| `Into<Condition>` | `filter`, `exclude` | accept expressions and composed conditions |
| by-value builder | all the builder's methods | chaining without an intermediate variable |
| newtype | `RuniqueQueryBuilder` around `Select` | add an API without rewriting SeaORM |
| associated type | `<E::PrimaryKey as PrimaryKeyTrait>::ValueType` | the identifier has the type of the key |
| extension trait | `QueryExt` | add your own methods |

### Key points to remember

- `Entity::objects` is a zero-size constant; `all`, `filter` and `exclude` make no query.
- The `async` methods (`all`, `first`, `one`, `count`, `get`) execute.
- `one` rejects duplicates, `first` takes the first one.
- `search!` for Django-style operators, `into_select()` for the rest of SeaORM.
- Never hand-written SQL: values are always parameters.
