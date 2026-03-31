# Advanced — Transactions, Relations & Complete Pattern

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
let user = users::Entity::objects.get_optional(&db, 1).await?;

if let Some(user) = user {
    let posts = user.find_related(posts::Entity)
        .all(&db)
        .await?;
}
```

### Many-to-Many

```rust
let user = users::Entity::objects.get_optional(&db, 1).await?;

if let Some(user) = user {
    let roles = user.find_related(roles::Entity)
        .all(&db)
        .await?;
}
```

---

## Complete CRUD pattern

```rust
use demo_app::entities::users;
use runique::prelude::*;
use axum::extract::Path;

async fn list_users(request: Request) -> AppResult<Response> {
    let users = users::Entity::find()
        .all(&request.engine.db)
        .await?;

    Ok(Json(json!({ "users": users })).into_response())
}

async fn get_user(
    Path(id): Path<i32>,
    request: Request,
) -> AppResult<Response> {
    match users::Entity::find_by_id(id).one(&request.engine.db).await? {
        Some(user) => Ok(Json(json!({ "user": user })).into_response()),
        None => Ok(StatusCode::NOT_FOUND.into_response()),
    }
}

async fn create_user(
    mut request: Request,
    Json(payload): Json<CreateUserRequest>,
) -> AppResult<Response> {
    let user = users::ActiveModel {
        email: Set(payload.email),
        username: Set(payload.username),
        ..Default::default()
    }.insert(&request.engine.db).await?;

    success!(request.notices => "User created!");
    Ok((StatusCode::CREATED, Json(json!({ "user": user }))).into_response())
}

async fn delete_user(
    Path(id): Path<i32>,
    mut request: Request,
) -> AppResult<Response> {
    let result = users::Entity::delete_by_id(id)
        .exec(&request.engine.db)
        .await?;

    if result.rows_affected > 0 {
        success!(request.notices => "User deleted");
        Ok(Redirect::to("/users").into_response())
    } else {
        Ok(StatusCode::NOT_FOUND.into_response())
    }
}
```

---

## See also

| Section | Description |
| --- | --- |
| [Manager & helpers](/docs/en/orm/manager) | `impl_objects!`, helpers |
| [CRUD Queries](/docs/en/orm/queries) | SELECT, INSERT, UPDATE, DELETE |

## Back to summary

- [ORM](/docs/en/orm)
