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
use sea_orm::Order;
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

## WHERE — Filtering

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

// Multiple conditions
let users = users::Entity::objects
    .filter(users::Column::Active.eq(true))
    .filter(users::Column::Age.gte(18))
    .all(&*db)
    .await?;

// OR
let users = users::Entity::objects
    .filter(
        users::Column::Email.eq("a@test.com")
            .or(users::Column::Email.eq("b@test.com"))
    )
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
| --- | --- |
| [Manager & helpers](https://github.com/seb-alliot/runique/blob/main/docs/en/orm/manager/manager.md) | `impl_objects!`, helpers |
| [Advanced](https://github.com/seb-alliot/runique/blob/main/docs/en/orm/advanced/advanced.md) | Transactions, relations |

## Back to summary

- [ORM](https://github.com/seb-alliot/runique/blob/main/docs/en/orm/07-orm.md)
