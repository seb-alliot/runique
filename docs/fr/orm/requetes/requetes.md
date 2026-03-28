# Requêtes CRUD

## SELECT — Récupérer

```rust
// Tous les enregistrements
let users: Vec<users::Model> = users::Entity::objects
    .all()
    .all(&*db)
    .await?;

// Avec limite et offset
let users = users::Entity::objects
    .all()
    .limit(10)
    .offset(0)
    .all(&*db)
    .await?;

// Avec tri
let users = users::Entity::objects
    .all()
    .order_by_asc(users::Column::Name)
    .all(&*db)
    .await?;
```

---

## COUNT — Compter

```rust
let count = users::Entity::objects
    .filter(users::Column::Active.eq(true))
    .count(&*db)
    .await?;
```

---

## WHERE — Filtrage (SeaORM natif)

```rust
use sea_orm::ColumnTrait;

// Égalité
let user = users::Entity::objects
    .filter(users::Column::Email.eq("test@example.com"))
    .first(&*db)
    .await?;

// Comparaisons
let users = users::Entity::objects
    .filter(users::Column::Age.gt(18))
    .all(&*db)
    .await?;

// Multiples conditions (AND)
let users = users::Entity::objects
    .filter(users::Column::Active.eq(true))
    .filter(users::Column::Age.gte(18))
    .all(&*db)
    .await?;

// OU
use sea_orm::Condition;
let users = users::Entity::objects
    .filter(
        Condition::any()
            .add(users::Column::Email.eq("a@test.com"))
            .add(users::Column::Email.eq("b@test.com"))
    )
    .all(&*db)
    .await?;
```

---

## Macro `search!` — DSL de filtrage

La macro `search!` offre une syntaxe concise inspirée de Django pour construire des filtres.
Elle retourne un `RuniqueQueryBuilder` chainable (`.limit()`, `.order_by_asc()`, `.all()`, etc.).

### Opérateurs de base

| Syntaxe | SQL généré |
| ------- | --------- |
| `+Col = val` | `WHERE col = val` |
| `-Col = val` | `WHERE col != val` |
| `Col = val` | `WHERE col = val` |
| `Col != val` | `WHERE col != val` |
| `Col > val` | `WHERE col > val` |
| `Col < val` | `WHERE col < val` |
| `Col >= val` | `WHERE col >= val` |
| `Col <= val` | `WHERE col <= val` |
| `Col ~ val` | `WHERE col LIKE val` |
| `Col !~ val` | `WHERE col NOT LIKE val` |
| `Col ~~ val` | `WHERE col ILIKE val` *(insensible à la casse)* |
| `Col !~~ val` | `WHERE col NOT ILIKE val` |
| `Col = null` | `WHERE col IS NULL` |
| `Col != null` | `WHERE col IS NOT NULL` |

```rust
use runique::search;

// Inclusion / exclusion
let actifs = search!(users::Entity => +Active = true)
    .all(&*db).await?;

let sans_alice = search!(users::Entity => -Username = "alice")
    .all(&*db).await?;

// Comparaisons
let adultes    = search!(users::Entity => Age >= 18).all(&*db).await?;
let non_admins = search!(users::Entity => Level != 99).all(&*db).await?;

// LIKE / NOT LIKE
let rust      = search!(users::Entity => Bio ~  "%rust%").all(&*db).await?;
let pas_spam  = search!(users::Entity => Bio !~ "%spam%").all(&*db).await?;

// ILIKE / NOT ILIKE (insensible à la casse)
let rust_ci      = search!(users::Entity => Bio ~~  "%rust%").all(&*db).await?;
let pas_spam_ci  = search!(users::Entity => Bio !~~ "%spam%").all(&*db).await?;

// NULL / NOT NULL
let sans_bio  = search!(users::Entity => Bio =  null).all(&*db).await?;
let avec_bio  = search!(users::Entity => Bio != null).all(&*db).await?;
```

### IN / NOT IN

```rust
// IN
let selected = search!(users::Entity => +Id = [1, 2, 3])
    .all(&*db).await?;

// NOT IN
let others = search!(users::Entity => -Status = ["banned", "deleted"])
    .all(&*db).await?;
```

### BETWEEN / NOT BETWEEN

```rust
// BETWEEN (inclusif)
let mid = search!(users::Entity => +Age = between(18, 30))
    .all(&*db).await?;

// NOT BETWEEN
let hors = search!(users::Entity => -Age = between(18, 30))
    .all(&*db).await?;
```

### OR — même colonne

```rust
// Col = (v1 | v2 | v3)
let statuts = search!(users::Entity => Status = ("active" | "pending" | "review"))
    .all(&*db).await?;

// LIKE sur plusieurs valeurs
let themes = search!(posts::Entity => Title ~ ("%rust%" | "%sea%"))
    .all(&*db).await?;

// ILIKE sur plusieurs valeurs (insensible à la casse)
let themes_ci = search!(posts::Entity => Title ~~ ("%rust%" | "%sea%"))
    .all(&*db).await?;

// NOT ILIKE sur plusieurs valeurs
let sans_spam = search!(posts::Entity => Title !~~ ("%spam%" | "%pub%"))
    .all(&*db).await?;
```

### OR — multi-colonnes

```rust
// LIKE multi-colonnes
let results = search!(posts::Entity => (Title ~ "%rust%" | Content ~ "%rust%"))
    .all(&*db).await?;

// ILIKE multi-colonnes (insensible à la casse)
let results_ci = search!(posts::Entity => (Title ~~ "%rust%" | Content ~~ "%rust%"))
    .all(&*db).await?;

// NOT ILIKE multi-colonnes
let sans_spam = search!(posts::Entity => (Title !~~ "%spam%" | Content !~~ "%spam%"))
    .all(&*db).await?;
```

### Multi-conditions (AND chainé)

Séparer les conditions par des virgules — chaque condition est un AND.

```rust
let results = search!(users::Entity =>
    Active = true,
    Age >= 18,
    Status = ("active" | "verified"),
)
.order_by_desc(users::Column::CreatedAt)
.limit(20)
.all(&*db)
.await?;
```

Les bras multi-conditions supportent tous les opérateurs :

```rust
search!(users::Entity =>
    +Role = ["admin", "moderator"],          // IN
    +CreatedAt = between(date_a, date_b),    // BETWEEN
    Bio != null,                              // IS NOT NULL
    (Title ~~ "%rust%" | Bio ~~ "%rust%"),   // OR ILIKE
)
```

### OR de groupes AND — `[(...) | (...)]`

Pour exprimer `(A AND B) OR (C AND D)` :

```rust
let results = search!(users::Entity => [
    (Status = "active", Level >= 2) |
    (Status = "vip",    Level >= 1)
])
.all(&*db).await?;
```

Fonctionne aussi en multi-conditions :

```rust
search!(posts::Entity =>
    [(Title ~ "%rust%", Status = "published") | (Title ~ "%sea%", Status = "draft")],
    AuthorId = author_id,
)
```

> **Note :** `~~` et `!~~` (ILIKE) sont des opérateurs à deux tokens. Dans les patterns OR
> multi-colonnes `(Col1 op v1 | Col2 op v2)`, tous les membres doivent utiliser le même
> opérateur. Pour mélanger `~~` et `=` dans un même OR, utiliser le multi-conditions.

### Via `@Form` — FormEntity

Si le form est lié à une entité via `#[form(model = ...)]`, utiliser `@Form` comme raccourci :

```rust
#[form(schema = user_schema, model = users::Entity)]
pub struct UserForm;

// Identique à search!(users::Entity => ...)
let actifs = search!(@UserForm => Active = true)
    .all(&*db).await?;

// Toutes les syntaxes sont supportées
search!(@UserForm =>
    +Role = ["admin", "moderator"],
    Age >= 18,
)
.limit(10)
.all(&*db)
.await?;
```

---

## INSERT — Créer

```rust
use sea_orm::Set;

let new_user = users::ActiveModel {
    email: Set("john@example.com".to_string()),
    username: Set("john".to_string()),
    password: Set(hash_password("password123")),
    ..Default::default()
};

let user = new_user.insert(&*db).await?;
```

---

## UPDATE — Modifier

```rust
use sea_orm::{Set, Unchanged};

let mut user = users::Entity::find_by_id(1)
    .one(&*db)
    .await?
    .ok_or("User not found")?;

let mut user = user.into_active_model();
user.email = Set("newemail@example.com".to_string());

let updated = user.update(&*db).await?;
```

---

## DELETE — Supprimer

```rust
// Supprimer un seul
let result = users::Entity::delete_by_id(1)
    .exec(&*db)
    .await?;

// Supprimer multiples
let result = users::Entity::delete_many()
    .filter(users::Column::Active.eq(false))
    .exec(&*db)
    .await?;
```

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Manager & helpers](/docs/fr/orm/manager) | `impl_objects!`, helpers |
| [Avancé](/docs/fr/orm/avance) | Transactions, relations |

## Retour au sommaire

- [ORM](/docs/fr/orm)
