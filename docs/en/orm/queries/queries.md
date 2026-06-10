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

The `search!` macro provides a Django-inspired syntax for building SeaORM filters.
It returns a chainable `RuniqueQueryBuilder` (`.limit()`, `.order_by_asc()`, `.all()`, etc.).

### Full reference table

| Syntax | Generated SQL | Django equivalent |
| ------ | ------------- | ----------------- |
| `search!(Entity)` | *(no filter)* | `.objects.all()` |
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
| `Col in [v1, v2]` | `WHERE col IN (v1, v2)` *(literal)* | `filter(col__in=[v1, v2])` |
| `Col not_in [v1, v2]` | `WHERE col NOT IN (v1, v2)` | `exclude(col__in=[v1, v2])` |
| `Col in (expr)` | `WHERE col IN (...)` *(Vec/iterator)* | `filter(col__in=qs)` |
| `Col not_in (expr)` | `WHERE col NOT IN (...)` | `exclude(col__in=qs)` |
| `?Col in (expr)` | `WHERE col IN (...)` *(skipped if empty)* | — |
| `?Col not_in (expr)` | `WHERE col NOT IN (...)` *(skipped if empty)* | — |
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

### Basic operators

```rust
use runique::search;

let active     = search!(users::Entity => Active eq true).all(&*db).await?;
let adults     = search!(users::Entity => Age gte 18).all(&*db).await?;
let non_admins = search!(users::Entity => ! Level eq 99).all(&*db).await?;

// LIKE / ILIKE / NOT LIKE / NOT ILIKE
let rust        = search!(users::Entity => Bio like "%rust%").all(&*db).await?;
let rust_ci     = search!(users::Entity => Bio ilike "%rust%").all(&*db).await?;
let not_rust    = search!(users::Entity => Bio not_like "%rust%").all(&*db).await?;
let not_rust_ci = search!(users::Entity => Bio not_ilike "%rust%").all(&*db).await?;

// NULL
let no_bio   = search!(users::Entity => Bio isnull).all(&*db).await?;
let with_bio = search!(users::Entity => Bio not_null).all(&*db).await?;
```

### contains / icontains / startswith / endswith / iexact

These operators handle `%` wildcards automatically.

```rust
let rust    = search!(posts::Entity => Title contains "rust").all(&*db).await?;
let rust_ci = search!(posts::Entity => Title icontains "rust").all(&*db).await?;
let hello   = search!(posts::Entity => Title startswith "Hello").all(&*db).await?;
let rs      = search!(posts::Entity => Filename endswith ".rs").all(&*db).await?;
let user    = search!(users::Entity => Username iexact "Alice").first(&*db).await?;
```

### IN / NOT IN

```rust
// Literal IN
let selected = search!(users::Entity => Id in [1, 2, 3]).all(&*db).await?;

// Literal NOT IN
let others = search!(users::Entity => Status not_in ["banned", "deleted"]).all(&*db).await?;

// Dynamic IN (Vec, iterator)
let ids: Vec<i32> = get_allowed_ids();
let users = search!(users::Entity => Id in (ids)).all(&*db).await?;

// Dynamic NOT IN
let blocked: Vec<i32> = get_blocked_ids();
let clean = search!(users::Entity => Id not_in (blocked)).all(&*db).await?;
```

> **Note — type inference with `in (expr)`**
>
> The macro internally converts `$val` to `Vec<_>` before passing it to `is_in`.
> This prevents inference errors in contexts where the overall return type is complex
> (e.g. `HashMap<i32, String>`).
> If the compiler cannot infer the element type, annotate the variable explicitly:
>
> ```rust
> let ids: Vec<i32> = lines.iter().filter_map(|l| l.item_id).collect();
> let items = search!(item::Entity => Id in (ids)).all(db).await?;
> ```

### BETWEEN / NOT BETWEEN

```rust
let mid     = search!(users::Entity => Age range (18, 30)).all(&*db).await?;
let outside = search!(users::Entity => Age not_range (18, 30)).all(&*db).await?;
```

### OR across columns — `or(...)`

```rust
// Search across multiple columns
let results = search!(posts::Entity => or(Title icontains "rust", Content icontains "rust"))
    .all(&*db).await?;

// With a runtime variable
let results = search!(posts::Entity => or(Title icontains term, Summary icontains term))
    .all(&*db).await?;
```

### Multiple conditions (chained AND)

Separate conditions with commas — each one is an AND.

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

All operators work in multi-condition mode:

```rust
search!(users::Entity =>
    Role in ["admin", "moderator"],         // literal IN
    Id in (dynamic_ids),                    // dynamic IN
    CreatedAt range (date_a, date_b),       // BETWEEN
    Bio not_null,                            // IS NOT NULL
    Username icontains "alice",             // ILIKE %alice%
    or(Title icontains q, Bio icontains q), // OR across columns
)
```

### `order_by_random()` — random order

```rust
let picks = search!(product::Entity => Available eq true)
    .order_by_random()
    .limit(5)
    .all(&*db).await?;
```

### `order_by_expr(expr, order)` — custom expression

Accepts any SeaORM `IntoSimpleExpr` — useful for computed columns, COALESCE, CASE, etc.

```rust
use sea_orm::Order;
use sea_orm::sea_query::Expr;

let results = search!(product::Entity)
    .order_by_expr(Expr::col(product::Column::Price), Order::Desc)
    .all(&*db).await?;
```

### `.one()` — expect exactly one result

Returns `Ok(None)` if no row matches, `Ok(Some(model))` if exactly one, and `Err` if more than one row matches. Analogous to Django's `.get()`.

Internally fetches at most 2 rows to detect the ambiguous case without a full scan.

```rust
// Returns Err if more than one active admin exists
let admin = search!(users::Entity => IsStaff eq true, Username eq "alice")
    .one(&*db)
    .await?; // Result<Option<users::Model>, DbErr>
```

Use `.first()` instead when you only want the first row of a potentially multi-row result without erroring.

### `.into_select()` — partial projection

For cases where `select_only()`, `column()`, `distinct()` or `into_tuple()` are needed,
`.into_select()` exposes the underlying SeaORM `Select<E>`.

```rust
// Retrieve a single distinct column
let difficulties: Vec<String> = search!(course::Entity => Lang eq "en")
    .into_select()
    .select_only()
    .column(course::Column::Difficulty)
    .distinct()
    .into_tuple::<String>()
    .all(db.as_ref())  // ← .as_ref() required here
    .await?;
```

> **Note — `.as_ref()` required:** once outside `RuniqueQueryBuilder`, SeaORM methods use a
> generic bound `C: ConnectionTrait`. `Arc<DatabaseConnection>` does not satisfy this bound
> directly — `db.as_ref()` explicitly converts to `&DatabaseConnection`.
> `RuniqueQueryBuilder` methods (`.all()`, `.first()`, etc.) do not have this issue
> because they take `&DatabaseConnection` as a concrete parameter.

### Via `@Form` — FormEntity

If the form is linked to an entity via `#[form(model = ...)]`, use `@Form` as a shorthand:

```rust
#[form(schema = user_schema, model = users::Entity)]
pub struct UserForm;

// Identical to search!(users::Entity => ...)
let active = search!(@UserForm => Active eq true)
    .all(&*db).await?;

// All syntaxes are supported
search!(@UserForm =>
    Role in ["admin", "moderator"],
    Age gte 18,
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

## `search_cond!` — Raw Condition

Returns a `sea_orm::Condition` to inject into an existing query builder, instead of creating a new one.

| Syntax | Result |
| ------ | ------ |
| `all_columns icontains val` | `OR ILIKE` across all columns |
| `or("col1" icontains val, "col2" icontains val)` | `OR ILIKE` across named columns |
| `?Col in (expr)` | `Condition::all().add(col IN (...))` — no-op if empty |
| `?Col not_in (expr)` | `Condition::all().add(col NOT IN (...))` — no-op if empty |

```rust
// Typically used to inject a condition into an already-in-progress query
let cond = search_cond!(commande::Entity => or("numero" icontains q, "statut" icontains q));
let results = commande::Entity::find()
    .filter(cond)
    .all(db).await?;

// Conditional filter on a Vec (no-op if empty)
let statuts: Vec<StatutCommande> = get_active_statuts();
let cond = search_cond!(commande::Entity => ?Statut in (statuts));
let results = commande::Entity::find()
    .filter(cond)
    .all(db).await?;
```

---

## Current limitations of `search!`

### `in (expr)` with an empty vec

`Col in (vec)` generates `WHERE col IN (...)`. If the vec is empty, the produced SQL is invalid
or always false depending on the engine. Keep the manual guard:

```rust
let ids: Vec<i32> = compute_ids();
if !ids.is_empty() {
    query = query.filter(col.is_in(ids));
}
```

**Built-in workaround — `?Col in (expr)` :** if the vec may be empty, use the conditional syntax that skips the filter automatically:

```rust
let ids: Vec<i32> = compute_ids();  // may be empty
let results = search!(item::Entity => ?Id in (ids)).all(db).await?;
```

Likewise `?Col not_in (expr)` for conditional exclusion.

### No inline conditional filter (boolean condition)

It is not possible to skip a filter inside a `search!` call based on an arbitrary boolean condition.
If some filters are optional on a predicate unrelated to an empty vec, apply `.filter()` manually after the macro.

### OR on the same field — multiple enum variants

`or(...)` operates on multiple **different columns**. To filter one column against several
enum values, use `Col in [V1, V2]` (literal) or `.filter(Col.is_in(vec))`:

```rust
// OK — literal IN
search!(commande::Entity => Statut in ["en_attente", "accepte"])

// OK — enum Vec
let statuts = vec![StatutCommande::EnAttente, StatutCommande::Accepte];
query.filter(commande::Column::Statut.is_in(statuts))

// Not available — planned future syntax:
// search!(commande::Entity => Statut any [EnAttente, Accepte])
```

### No aggregates

`.avg()`, `.sum()`, `.count_by()` are not available on `RuniqueQueryBuilder`.
For aggregates, use SeaORM directly or `.into_select()`.

---

## See also

| Section | Description |
| ------- | ----------- |
| [Manager & helpers](/docs/en/orm/manager) | `impl_objects!`, helpers |
| [Advanced](/docs/en/orm/advanced) | Transactions, relations |

## Back to summary

- [ORM](/docs/en/orm)
