Voici la traduction complète en anglais de ton chapitre sur l’ORM et la base de données :

# 🗄️ ORM & Database

## SeaORM + Objects Manager

Runique uses **SeaORM** with a Django-like manager via the `impl_objects!` macro:

```rust
use demo_app::models::users;
use runique::impl_objects; // adds <Entity>::objects

// Add the manager once in your model
impl_objects!(users::Entity);

// Retrieve all users
let all_users = users::Entity::objects
    .all()
    .all(&*db)
    .await?;

// With a filter
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

### Without Macro (Native SeaORM)

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

### Available Helpers

* `all()`: entry point to chain `filter`, `order_by_asc/desc`, `limit`, `offset`.
* `filter(...)` / `exclude(...)`: add conditions.
* `first(db)` / `count(db)`: executes the query (alias for `one` / quick client-side count).
* `get(db, id)` / `get_optional(db, id)`: direct access by primary key.
* `get_or_404(db, ctx, msg)`: returns a 404/500 response with Tera rendering if missing.

---

## Common Queries

### SELECT - Retrieve

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

// Multiple columns
let users = users::Entity::objects
    .all()
    .order_by_asc(users::Column::CreatedAt)
    .order_by_desc(users::Column::Id)
    .all(&*db)
    .await?;
```

### COUNT - Count records

```rust
let count = users::Entity::objects
    .filter(users::Column::Active.eq(true))
    .count(&*db)
    .await?;

println!("Active users: {}", count);
```

### WHERE - Filtering

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

let users = users::Entity::objects
    .filter(users::Column::Name.like("%john%"))
    .all(&*db)
    .await?;

// Multiple conditions
use sea_orm::sea_query::Expr;
let users = users::Entity::objects
    .filter(users::Column::Active.eq(true))
    .filter(users::Column::Age.gte(18))
    .all(&*db)
    .await?;

// OR condition
use sea_orm::sea_query::IntoCondition;
let users = users::Entity::objects
    .filter(
        users::Column::Email.eq("a@test.com")
            .or(users::Column::Email.eq("b@test.com"))
    )
    .all(&*db)
    .await?;
```

---

## INSERT - Create

```rust
use sea_orm::Set;

let new_user = users::ActiveModel {
    email: Set("john@example.com".to_string()),
    username: Set("john".to_string()),
    password: Set(hash_password("password123")),
    ..Default::default()
};

let user = new_user.insert(&*db).await?;
println!("Created: {:?}", user);
```

---

## UPDATE - Modify

```rust
use sea_orm::{Set, Unchanged};

let mut user = users::Entity::find_by_id(1)
    .one(&*db)
    .await?
    .ok_or("User not found")?;

let mut user = user.into_active_model();
user.email = Set("newemail@example.com".to_string());
user.username = Set("newname".to_string());

let updated = user.update(&*db).await?;
println!("Updated: {:?}", updated);
```

---

## DELETE - Remove

```rust
// Delete single
let result = users::Entity::delete_by_id(1)
    .exec(&*db)
    .await?;

println!("Deleted: {} row(s)", result.rows_affected);

// Delete multiple
let result = users::Entity::delete_many()
    .filter(users::Column::Active.eq(false))
    .exec(&*db)
    .await?;
```

---

## Transactions

```rust
use sea_orm::TransactionTrait;

let mut transaction = db.begin().await?;

let user = users::ActiveModel {
    email: Set("test@example.com".to_string()),
    ..Default::default()
}.insert(&mut *transaction).await?;

let profile = profiles::ActiveModel {
    user_id: Set(user.id),
    ..Default::default()
}.insert(&mut *transaction).await?;

transaction.commit().await?;
```

---

## Relations

### One-to-Many

```rust
// User has many Posts
let user = users::Entity::objects.get_optional(&*db, 1).await?;

if let Some(user) = user {
    let posts = user.find_related(posts::Entity)
        .all(&*db)
        .await?;
}
```

### Many-to-Many

```rust
let user = users::Entity::objects.get_optional(&*db, 1).await?;

if let Some(user) = user {
    let roles = user.find_related(roles::Entity)
        .all(&*db)
        .await?;
}
```

---

## Complete Pattern

```rust
use demo_app::models::users;
use runique::prelude::*;
use axum::extract::Path;

// CRUD Handler
// Using macro (Entity::objects)
async fn list_users() -> Response {
    let db = /* access db from app state */;

    match users::Entity::find()
        .all(&*db)
        .await
    {
        Ok(users) => Json(json!({"users": users})),
        Err(e) => error_response(format!("Database error: {}", e)),
    }
}

// Without macro (native SeaORM)
async fn list_users_raw() -> Response {
    let db = /* access db from app state */;

    match users::Entity::find()
        .all(&*db)
        .await
    {
        Ok(users) => Json(json!({"users": users})),
        Err(e) => error_response(format!("Database error: {}", e)),
    }
}

// Using find_by_id
async fn get_user(
    Path(id): Path<i32>,
) -> Response {
    let db = /* access db from app state */;

    match users::Entity::find_by_id(id)
        .one(&*db)
        .await
    {
        Ok(Some(user)) => Json(json!({"user": user})),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

// Without macro (find_by_id)
async fn get_user_raw(
    Path(id): Path<i32>,
) -> Response {
    let db = /* access db from app state */;

    match users::Entity::find_by_id(id)
        .one(&*db)
        .await
    {
        Ok(Some(user)) => Json(json!({"user": user})),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn create_user(
    Message(mut messages): Message,
    Json(payload): Json<CreateUserRequest>,
) -> Response {
    let db = /* access db from app state */;

    let user = users::ActiveModel {
        email: Set(payload.email),
        username: Set(payload.username),
        ..Default::default()
    };

    match user.insert(&*db).await {
        Ok(created) => {
            messages.success("User created!");
            (StatusCode::CREATED, Json(json!({"user": created}))).into_response()
        }
        Err(e) => {
            messages.error(format!("Error: {}", e));
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn delete_user(
    Path(id): Path<i32>,
    Message(mut messages): Message,
) -> Response {
    let db = /* access db from app state */;

    match users::Entity::delete_by_id(id)
        .exec(&*db)
        .await
    {
        Ok(result) if result.rows_affected > 0 => {
            messages.success("User deleted");
            Redirect::to("/users").into_response()
        }
        _ => {
            messages.error("Deletion failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
```

---

## Next Steps

← [**Templates**](https://github.com/seb-alliot/runique/blob/main/docs/en/06-templates.md) | [**Middleware & Security**](https://github.com/seb-alliot/runique/blob/main/docs/en/08-middleware.md) →
