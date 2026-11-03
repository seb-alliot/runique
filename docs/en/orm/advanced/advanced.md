# Advanced — Transactions, Relations & Complete Pattern

## Transactions

A transaction groups several writes: they all become real at `commit()`, or none does. Without `commit()`, the transaction is rolled back on its own when the handler leaves — a `return`, an error raised by `?` —, with no `rollback()` to write.

### In a handler — `request.db_atomic()`

`request.db_atomic()` is only a syntax shortcut: it does exactly `request.engine.db.begin()`, SeaORM's transaction, without importing `TransactionTrait`. It is the form to use in a handler.

`ValidationForm::save` and `delete` take the connection (`&request.engine.db`) as well as the transaction (`&txn`). One form maps to one model and one `save`; the transaction ties that `save` to your other writes:

```rust
pub async fn publish(mut request: Request) -> AppResult<Response> {
    let template = "article/new.html";
    let form: ArticleForm = request.form();
    let form = match ValidationForm::try_form(form, &request, "article").await {
        Ok(validated) => validated,
        Err(form) => {
            context_update!(request => { "article_form" => &form });
            return request.render(template); // nothing was opened
        }
    };

    let txn = request.db_atomic().await?;

    if let Err(form) = form.save(&txn).await {
        context_update!(request => { "article_form" => &form });
        return request.render(template); // left without commit: rollback
    }
    log::ActiveModel {
        action: Set("article published".to_string()),
        ..Default::default()
    }
    .insert(&txn)
    .await?; // error: rollback, the article is undone too

    txn.commit().await?; // the article and the log, or neither
    Ok(Redirect::to("/articles").into_response())
}
```

Inside, `save` opens its own transaction; on one already open, SeaORM makes it a savepoint. A failing `save` only cancels its own write and gives the form back with its errors; leaving without `commit()` is what cancels the rest.

**Files.** `save` moves the file between its write and the end of its savepoint. If the handler then leaves without `commit()`, the row is rolled back but the moved file stays on disk, orphaned — never a row pointing at a missing file. See [Fields](/docs/en/formulaire/fields).

**`admin` routes.** On a route declared `admin` in `urlpatterns!`, `save` writes through the admin's gate, which reads and writes through the connection: a form on those routes refuses a transaction (the usual refusal message). Hand it `&request.engine.db`.

### Outside a request — CLI, tasks

Without a `Request`, open the transaction on the connection directly, with `TransactionTrait` imported:

```rust
use sea_orm::TransactionTrait;

let txn = engine.db.begin().await?;

let user = users::ActiveModel {
    email: Set("test@example.com".to_string()),
    ..Default::default()
}.insert(&txn).await?;

profiles::ActiveModel {
    user_id: Set(user.id),
    ..Default::default()
}.insert(&txn).await?;

txn.commit().await?;
```

This suits a command that creates an account and its linked rows. Migrations don't go through this: `runique migration up` leaves the decision to sea-orm-migration, which only wraps each migration in a transaction under Postgres. See [Migrations](/docs/en/installation/migrations).

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
        .all(request.db())
        .await?;

    Ok(Json(json!({ "users": users })).into_response())
}

async fn get_user(
    Path(id): Path<i32>,
    request: Request,
) -> AppResult<Response> {
    match users::Entity::find_by_id(id).one(request.db()).await? {
        Some(user) => Ok(Json(json!({ "user": user })).into_response()),
        None => Ok(StatusCode::NOT_FOUND.into_response()),
    }
}

async fn create_user(
    Json(payload): Json<CreateUserRequest>,
    mut request: Request,  // Request must be last
) -> AppResult<Response> {
    let user = users::ActiveModel {
        email: Set(payload.email),
        username: Set(payload.username),
        ..Default::default()
    }.insert(request.db()).await?;

    success!(request.notices => "User created!");
    Ok((StatusCode::CREATED, Json(json!({ "user": user }))).into_response())
}

async fn delete_user(
    Path(id): Path<i32>,
    mut request: Request,
) -> AppResult<Response> {
    let result = users::Entity::delete_by_id(id)
        .exec(request.db())
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
