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
    .all(&*db)
    .await?;

// With filter
let active_users = users::Entity::objects
    .filter(users::Column::Active.eq(true))
    .all(&*db)
    .await?;

// Single result
let user = users::Entity::objects
    .filter(users::Column::Id.eq(user_id))
    .first(&*db)
    .await?;
```

---

## Without the macro (native SeaORM)

```rust
// All records
let all_users = users::Entity::find().all(&*db).await?;

// With filter
let active_users = users::Entity::find()
    .filter(users::Column::Active.eq(true))
    .all(&*db)
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

## See also

| Section | Description |
| --- | --- |
| [CRUD Queries](/docs/en/orm/queries) | SELECT, COUNT, WHERE, INSERT, UPDATE, DELETE |
| [Advanced](/docs/en/orm/advanced) | Transactions, relations |

## Back to summary

- [ORM](/docs/en/orm)
