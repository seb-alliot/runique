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

La macro `search!` offre une syntaxe inspirée de Django pour construire des filtres SeaORM.
Elle retourne un `RuniqueQueryBuilder` chainable (`.limit()`, `.order_by_asc()`, `.all()`, etc.).

### Tableau de référence complet

| Syntaxe | SQL généré | Équivalent Django |
| ------- | ---------- | ----------------- |
| `search!(Entity)` | *(aucun filtre)* | `.objects.all()` |
| `Col eq val` | `WHERE col = val` | `filter(col=val)` |
| `Col exact val` | `WHERE col = val` | `filter(col__exact=val)` |
| `Col ne val` | `WHERE col != val` | — |
| `Col gt val` | `WHERE col > val` | `filter(col__gt=val)` |
| `Col lt val` | `WHERE col < val` | `filter(col__lt=val)` |
| `Col gte val` | `WHERE col >= val` | `filter(col__gte=val)` |
| `Col lte val` | `WHERE col <= val` | `filter(col__lte=val)` |
| `Col like val` | `WHERE col LIKE val` | — |
| `Col ilike val` | `WHERE col ILIKE val` | — |
| `Col not_like val` | `WHERE col NOT LIKE val` | — |
| `Col not_ilike val` | `WHERE col NOT ILIKE val` | — |
| `Col contains val` | `WHERE col LIKE '%val%'` | `filter(col__contains=val)` |
| `Col icontains val` | `WHERE col ILIKE '%val%'` | `filter(col__icontains=val)` |
| `Col startswith val` | `WHERE col LIKE 'val%'` | `filter(col__startswith=val)` |
| `Col endswith val` | `WHERE col LIKE '%val'` | `filter(col__endswith=val)` |
| `Col iexact val` | `WHERE col ILIKE val` | `filter(col__iexact=val)` |
| `Col isnull` | `WHERE col IS NULL` | `filter(col__isnull=True)` |
| `Col not_null` | `WHERE col IS NOT NULL` | `filter(col__isnull=False)` |
| `Col in [v1, v2]` | `WHERE col IN (v1, v2)` *(littéral)* | `filter(col__in=[v1, v2])` |
| `Col not_in [v1, v2]` | `WHERE col NOT IN (v1, v2)` | `exclude(col__in=[v1, v2])` |
| `Col in (expr)` | `WHERE col IN (...)` *(Vec/itérateur)* | `filter(col__in=qs)` |
| `Col not_in (expr)` | `WHERE col NOT IN (...)` | `exclude(col__in=qs)` |
| `Col range (a, b)` | `WHERE col BETWEEN a AND b` | `filter(col__range=(a, b))` |
| `Col not_range (a, b)` | `WHERE col NOT BETWEEN a AND b` | — |
| `! Col op val` | exclusion (NOT) | `.exclude(col__op=val)` |
| `or(C1 op v, C2 op v)` | `WHERE c1 op v OR c2 op v` | `Q(c1__op=v) \| Q(c2__op=v)` |

### Fetch all

```rust
let all = search!(users::Entity)
    .order_by_asc(users::Column::Name)
    .all(&*db).await?;
```

### Opérateurs de base

```rust
use runique::search;

let actifs     = search!(users::Entity => Active eq true).all(&*db).await?;
let adultes    = search!(users::Entity => Age gte 18).all(&*db).await?;
let non_admins = search!(users::Entity => ! Level eq 99).all(&*db).await?;

// LIKE / ILIKE / NOT LIKE / NOT ILIKE
let rust          = search!(users::Entity => Bio like "%rust%").all(&*db).await?;
let rust_ci       = search!(users::Entity => Bio ilike "%rust%").all(&*db).await?;
let pas_rust      = search!(users::Entity => Bio not_like "%rust%").all(&*db).await?;
let pas_rust_ci   = search!(users::Entity => Bio not_ilike "%rust%").all(&*db).await?;

// NULL
let sans_bio = search!(users::Entity => Bio isnull).all(&*db).await?;
let avec_bio = search!(users::Entity => Bio not_null).all(&*db).await?;
```

### contains / icontains / startswith / endswith / iexact

Ces opérateurs gèrent les `%` automatiquement.

```rust
let rust  = search!(posts::Entity => Title contains "rust").all(&*db).await?;
let rust_ci = search!(posts::Entity => Title icontains "rust").all(&*db).await?;
let hello = search!(posts::Entity => Title startswith "Hello").all(&*db).await?;
let rs    = search!(posts::Entity => Filename endswith ".rs").all(&*db).await?;
let user  = search!(users::Entity => Username iexact "Alice").first(&*db).await?;
```

### IN / NOT IN

```rust
// IN littéral
let selected = search!(users::Entity => Id in [1, 2, 3]).all(&*db).await?;

// NOT IN littéral
let others = search!(users::Entity => Status not_in ["banned", "deleted"]).all(&*db).await?;

// IN dynamique (Vec, itérateur)
let ids: Vec<i32> = get_allowed_ids();
let users = search!(users::Entity => Id in (ids)).all(&*db).await?;

// NOT IN dynamique
let blocked: Vec<i32> = get_blocked_ids();
let clean = search!(users::Entity => Id not_in (blocked)).all(&*db).await?;
```

### BETWEEN / NOT BETWEEN

```rust
let mid  = search!(users::Entity => Age range (18, 30)).all(&*db).await?;
let hors = search!(users::Entity => Age not_range (18, 30)).all(&*db).await?;
```

### OR multi-colonnes — `or(...)`

```rust
// Recherche sur plusieurs colonnes
let results = search!(posts::Entity => or(Title icontains "rust", Content icontains "rust"))
    .all(&*db).await?;

// Avec variable runtime
let results = search!(posts::Entity => or(Title icontains term, Summary icontains term))
    .all(&*db).await?;
```

### Multi-conditions (AND chainé)

Séparer les conditions par des virgules — chaque condition est un AND.

```rust
let results = search!(users::Entity =>
    Active eq true,
    Age gte 18,
    Role in ["admin", "moderator"],
)
.order_by_desc(users::Column::CreatedAt)
.limit(20)
.all(&*db)
.await?;
```

Tous les opérateurs sont disponibles en multi-conditions :

```rust
search!(users::Entity =>
    Role in ["admin", "moderator"],        // IN littéral
    Id in (ids_dynamiques),                // IN dynamique
    CreatedAt range (date_a, date_b),      // BETWEEN
    Bio not_null,                           // IS NOT NULL
    Username icontains "alice",            // ILIKE %alice%
    or(Title icontains q, Bio icontains q), // OR multi-colonnes
)
```

### `.into_select()` — projection partielle

Pour les cas où `select_only()`, `column()`, `distinct()` ou `into_tuple()` sont nécessaires,
`.into_select()` expose le `Select<E>` SeaORM interne.

```rust
// Récupérer une seule colonne distincte
let difficultes: Vec<String> = search!(cour::Entity => Lang eq "fr")
    .into_select()
    .select_only()
    .column(cour::Column::Difficulte)
    .distinct()
    .into_tuple::<String>()
    .all(db.as_ref())  // ← .as_ref() obligatoire ici
    .await?;
```

> **Note — `.as_ref()` requis :** une fois sorti du `RuniqueQueryBuilder`, les méthodes SeaORM
> utilisent un bound générique `C: ConnectionTrait`. `Arc<DatabaseConnection>` ne satisfait pas
> ce bound directement — `db.as_ref()` convertit explicitement en `&DatabaseConnection`.
> Les méthodes du `RuniqueQueryBuilder` (`.all()`, `.first()`, etc.) n'ont pas ce problème
> car elles prennent `&DatabaseConnection` en paramètre concret.

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
