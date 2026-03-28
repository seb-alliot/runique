# CRUD Queries

## SELECT — Retrieve

```rust
// All records
let users: Vec<users::Model> = users::Entity::objects
    .all()
    .all(&*db)
    .await?;

// With limit and offset
let users = users::Entity::objects
    .all()
    .limit(10)
    .offset(0)
    .all(&*db)
    .await?;

// With ordering
let users = users::Entity::objects
    .all()
    .order_by_asc(users::Column::Name)
    .all(&*db)
    .await?;
```

---

## COUNT — Count records

```rust
let count = users::Entity::objects
    .filter(users::Column::Active.eq(true))
    .count(&*db)
    .await?;
```

---

## WHERE — Filtering (native SeaORM)

```rust
use sea_orm::ColumnTrait;

// Equality
let user = users::Entity::objects
    .filter(users::Column::Email.eq("test@example.com"))
    .first(&*db)
    .await?;

// Comparisons
let users = users::Entity::objects
    .filter(users::Column::Age.gt(18))
    .all(&*db)
    .await?;

// Multiple conditions (AND)
let users = users::Entity::objects
    .filter(users::Column::Active.eq(true))
    .filter(users::Column::Age.gte(18))
    .all(&*db)
    .await?;

// OR
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

## `search!` macro — Filter DSL

The `search!` macro provides a concise, Django-inspired syntax for building filters.
It returns a chainable `RuniqueQueryBuilder` (`.limit()`, `.order_by_asc()`, `.all()`, etc.).

### Basic operators

| Syntax | Generated SQL |
| ------ | ------------- |
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
| `Col ~~ val` | `WHERE col ILIKE val` *(case-insensitive)* |
| `Col !~~ val` | `WHERE col NOT ILIKE val` |
| `Col = null` | `WHERE col IS NULL` |
| `Col != null` | `WHERE col IS NOT NULL` |

```rust
use runique::search;

// Include / exclude
let active   = search!(users::Entity => +Active = true).all(&*db).await?;
let no_alice = search!(users::Entity => -Username = "alice").all(&*db).await?;

// Comparisons
let adults     = search!(users::Entity => Age >= 18).all(&*db).await?;
let non_admins = search!(users::Entity => Level != 99).all(&*db).await?;

// LIKE / NOT LIKE
let rust      = search!(users::Entity => Bio ~  "%rust%").all(&*db).await?;
let no_spam   = search!(users::Entity => Bio !~ "%spam%").all(&*db).await?;

// ILIKE / NOT ILIKE (case-insensitive)
let rust_ci   = search!(users::Entity => Bio ~~  "%rust%").all(&*db).await?;
let no_spam_ci = search!(users::Entity => Bio !~~ "%spam%").all(&*db).await?;

// NULL / NOT NULL
let no_bio   = search!(users::Entity => Bio =  null).all(&*db).await?;
let with_bio = search!(users::Entity => Bio != null).all(&*db).await?;
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
// BETWEEN (inclusive)
let mid = search!(users::Entity => +Age = between(18, 30))
    .all(&*db).await?;

// NOT BETWEEN
let outside = search!(users::Entity => -Age = between(18, 30))
    .all(&*db).await?;
```

### OR — same column

```rust
// Col = (v1 | v2 | v3)
let statuses = search!(users::Entity => Status = ("active" | "pending" | "review"))
    .all(&*db).await?;

// LIKE on multiple values
let themes = search!(posts::Entity => Title ~ ("%rust%" | "%sea%"))
    .all(&*db).await?;

// ILIKE on multiple values (case-insensitive)
let themes_ci = search!(posts::Entity => Title ~~ ("%rust%" | "%sea%"))
    .all(&*db).await?;

// NOT ILIKE on multiple values
let no_spam = search!(posts::Entity => Title !~~ ("%spam%" | "%ad%"))
    .all(&*db).await?;
```

### OR — multiple columns

```rust
// LIKE across multiple columns
let results = search!(posts::Entity => (Title ~ "%rust%" | Content ~ "%rust%"))
    .all(&*db).await?;

// ILIKE across multiple columns (case-insensitive)
let results_ci = search!(posts::Entity => (Title ~~ "%rust%" | Content ~~ "%rust%"))
    .all(&*db).await?;

// NOT ILIKE across multiple columns
let no_spam = search!(posts::Entity => (Title !~~ "%spam%" | Content !~~ "%spam%"))
    .all(&*db).await?;
```

### Multiple conditions (chained AND)

Separate conditions with commas — each one is an AND.

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

All operators work in multi-condition mode:

```rust
search!(users::Entity =>
    +Role = ["admin", "moderator"],          // IN
    +CreatedAt = between(date_a, date_b),    // BETWEEN
    Bio != null,                              // IS NOT NULL
    (Title ~~ "%rust%" | Bio ~~ "%rust%"),   // OR ILIKE
)
```

### OR of AND groups — `[(...) | (...)]`

To express `(A AND B) OR (C AND D)`:

```rust
let results = search!(users::Entity => [
    (Status = "active", Level >= 2) |
    (Status = "vip",    Level >= 1)
])
.all(&*db).await?;
```

Also works inside multi-condition mode:

```rust
search!(posts::Entity =>
    [(Title ~ "%rust%", Status = "published") | (Title ~ "%sea%", Status = "draft")],
    AuthorId = author_id,
)
```

> **Note:** `~~` and `!~~` (ILIKE) are two-token operators. In multi-column OR patterns
> `(Col1 op v1 | Col2 op v2)`, all members must use the same operator. To mix `~~` and `=`
> in the same OR, use multi-condition mode instead.

### Via `@Form` — FormEntity

If the form is linked to an entity via `#[form(model = ...)]`, use `@Form` as a shorthand:

```rust
#[form(schema = user_schema, model = users::Entity)]
pub struct UserForm;

// Identical to search!(users::Entity => ...)
let active = search!(@UserForm => Active = true)
    .all(&*db).await?;

// All syntaxes are supported
search!(@UserForm =>
    +Role = ["admin", "moderator"],
    Age >= 18,
)
.limit(10)
.all(&*db)
.await?;
```

---

## INSERT — Create

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

## UPDATE — Modify

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

## DELETE — Remove

```rust
// Delete single record
let result = users::Entity::delete_by_id(1)
    .exec(&*db)
    .await?;

// Delete multiple records
let result = users::Entity::delete_many()
    .filter(users::Column::Active.eq(false))
    .exec(&*db)
    .await?;
```

---

## See also

| Section | Description |
| ------- | ----------- |
| [Manager & helpers](/docs/en/orm/manager) | `impl_objects!`, helpers |
| [Advanced](/docs/en/orm/advanced) | Transactions, relations |

## Back to summary

- [ORM](/docs/en/orm)
