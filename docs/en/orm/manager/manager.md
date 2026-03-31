# Manager & SeaORM helpers

## SeaORM + Objects Manager

Runique uses **SeaORM** with a Django-like manager via the `impl_objects!` macro.

The macro is **automatically generated** by the daemon in each entity file (`runique makemigrations`). No manual declaration needed.

> **Note**: `impl_objects!` is pure syntactic sugar. It does not modify the generated queries — the SQL produced is identical to native SeaORM.

```rust
use demo_app::entities::users;

// Retrieve all users
let all_users = users::Entity::objects
    .all()
    .all(&db)
    .await?;

// With filter
let active_users = users::Entity::objects
    .filter(users::Column::Active.eq(true))
    .all(&db)
    .await?;

// Single result
let user = users::Entity::objects
    .filter(users::Column::Id.eq(user_id))
    .first(&db)
    .await?;
```

---

## Without the macro (native SeaORM)

```rust
// All records
let all_users = users::Entity::find().all(&db).await?;

// With filter
let active_users = users::Entity::find()
    .filter(users::Column::Active.eq(true))
    .all(&db)
    .await?;

// Single result by ID
let user = users::Entity::find_by_id(user_id)
    .one(&*db)
    .await?;
```

---

## Available helpers

- `all()`: entry point for chaining `filter`, `order_by_asc/desc`, `limit`, `offset`.
- `filter(...)` / `exclude(...)`: add conditions.
- `first(db)` / `count(db)`: execute the query.
- `get(db, id)` / `get_optional(db, id)`: direct primary key access.
- `get_or_404(db, ctx, msg)`: returns a 404/500 response with Tera rendering if missing.

---

## `search!` macro — Filter DSL

For common cases, the `search!` macro provides a more concise syntax than native SeaORM chaining:

```rust
use runique::search;

// Equivalent to objects.filter(Col::Active.eq(true))
let active = search!(users::Entity => Active eq true)
    .all(&db).await?;

// Multi-condition (AND)
let results = search!(users::Entity =>
    Active eq true,
    Age gte 18,
    Status in ["active", "verified"],
)
.limit(20)
.all(&db)
.await?;
```

See [CRUD Queries — search! macro](/docs/en/orm/queries#search-macro--filter-dsl) for the full reference.

---

## Access via form — `FormEntity`

When a form is annotated with `#[form(model = Entity)]`, Runique automatically generates:

- The `FormEntity` trait implementation (linking the form to its entity)
- A `pub const objects` constant on the form, identical to `Entity::objects`

```rust
#[form(schema = user_schema, model = users::Entity)]
pub struct UserForm;

// Access objects via the form — equivalent to users::Entity::objects
let user = UserForm::objects
    .filter(users::Column::Id.eq(id))
    .first(&db)
    .await?;

let all = UserForm::objects
    .all()
    .all(&db)
    .await?;
```

The `@Form` syntax in `search!` automatically delegates to the linked entity:

```rust
// Equivalent to search!(users::Entity => Active eq true)
let active = search!(@UserForm => Active eq true)
    .all(&db).await?;

// All search! syntaxes are supported
let results = search!(@UserForm =>
    Active eq true,
    Age gte 18,
    Status in ["active", "verified"],
)
.limit(20)
.all(&db)
.await?;
```

> The `FormEntity` trait is defined in `runique::forms`. It is implemented automatically — no manual declaration required.

---

## See also

| Section | Description |
| --- | --- |
| [CRUD Queries](/docs/en/orm/queries) | SELECT, COUNT, WHERE, INSERT, UPDATE, DELETE |
| [Advanced](/docs/en/orm/advanced) | Transactions, relations |
| [Forms](/docs/en/model/forms/forms) | `#[form(model = ...)]`, `FormEntity`, `Form::objects` |

## Back to summary

- [ORM](/docs/en/orm)
