# 🗄️ ORM & Base de Données

## SeaORM + Objects Manager

Runique utilise **SeaORM** pour l'ORM, avec un wrapper Django-like:

```rust
use demo_app::models::users;
use runique::data_base_runique::Objects;

// Récupérer tous les users
let all_users = users::Entity::find()
    .all(&*db)
    .await?;

// Avec filtre
let active_users = users::Entity::find()
    .filter(users::Column::Active.eq(true))
    .all(&*db)
    .await?;

// Un seul résultat
let user = users::Entity::find()
    .filter(users::Column::Id.eq(user_id))
    .one(&*db)
    .await?;
```

---

## Requêtes Courantes

### SELECT - Récupérer

```rust
use runique::data_base_runique::Objects;

// Tous les enregistrements
let users: Vec<users::Model> = users::Entity::find()
    .all(&*db)
    .await?;

// Avec limite et offset
let users = users::Entity::find()
    .limit(10)
    .offset(0)
    .all(&*db)
    .await?;

// Avec tri
use sea_orm::Order;
let users = users::Entity::find()
    .order_by_asc(users::Column::Name)
    .all(&*db)
    .await?;

// Multiple colonnes
let users = users::Entity::find()
    .order_by_asc(users::Column::CreatedAt)
    .order_by_desc(users::Column::Id)
    .all(&*db)
    .await?;
```

### COUNT - Compter

```rust
let count = users::Entity::find()
    .filter(users::Column::Active.eq(true))
    .count(&*db)
    .await?;

println!("Utilisateurs actifs: {}", count);
```

### WHERE - Filtrage

```rust
use sea_orm::ColumnTrait;

// Égalité
let user = users::Entity::find()
    .filter(users::Column::Email.eq("test@example.com"))
    .one(&*db)
    .await?;

// Comparaisons
let users = users::Entity::find()
    .filter(users::Column::Age.gt(18))
    .all(&*db)
    .await?;

let users = users::Entity::find()
    .filter(users::Column::Name.like("%john%"))
    .all(&*db)
    .await?;

// Multiples conditions
use sea_orm::sea_query::Expr;
let users = users::Entity::find()
    .filter(users::Column::Active.eq(true))
    .filter(users::Column::Age.gte(18))
    .all(&*db)
    .await?;

// OU
use sea_orm::sea_query::IntoCondition;
let users = users::Entity::find()
    .filter(
        users::Column::Email.eq("a@test.com")
            .or(users::Column::Email.eq("b@test.com"))
    )
    .all(&*db)
    .await?;
```

---

## INSERT - Créer

```rust
use sea_orm::Set;

let new_user = users::ActiveModel {
    email: Set("john@example.com".to_string()),
    username: Set("john".to_string()),
    password: Set(hash_password("password123")),
    ..Default::default()
};

let user = new_user.insert(&*db).await?;
println!("Créé: {:?}", user);
```

---

## UPDATE - Modifier

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
println!("Modifié: {:?}", updated);
```

---

## DELETE - Supprimer

```rust
// Supprimer un seul
let result = users::Entity::delete_by_id(1)
    .exec(&*db)
    .await?;

println!("Supprimés: {} ligne(s)", result.rows_affected);

// Supprimer multiples
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
// User a plusieurs Posts
let user = users::Entity::find_by_id(1)
    .one(&*db)
    .await?;

if let Some(user) = user {
    let posts = user.find_related(posts::Entity)
        .all(&*db)
        .await?;
}
```

### Many-to-Many

```rust
let user = users::Entity::find_by_id(1)
    .one(&*db)
    .await?;

if let Some(user) = user {
    let roles = user.find_related(roles::Entity)
        .all(&*db)
        .await?;
}
```

---

## Pattern Complet

```rust
use demo_app::models::users;
use runique::prelude::*;
use axum::extract::Path;

// Handler CRUD
async fn list_users(
    ctx: RuniqueContext,
) -> Response {
    let db = ctx.engine.db.clone();
    
    match users::Entity::find()
        .all(&*db)
        .await
    {
        Ok(users) => {
            Json(json!({"users": users}))
        }
        Err(e) => {
            error_response(format!("Database error: {}", e))
        }
    }
}

async fn get_user(
    Path(id): Path<i32>,
    ctx: RuniqueContext,
) -> Response {
    let db = ctx.engine.db.clone();
    
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
    mut ctx: RuniqueContext,
    Json(payload): Json<CreateUserRequest>,
) -> Response {
    let db = ctx.engine.db.clone();

    let user = users::ActiveModel {
        email: Set(payload.email),
        username: Set(payload.username),
        ..Default::default()
    };

    match user.insert(&*db).await {
        Ok(created) => {
            success!(ctx.flash => "Utilisateur créé!");
            (StatusCode::CREATED, Json(json!({"user": created})))
        }
        Err(e) => {
            error!(ctx.flash => format!("Erreur: {}", e));
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn delete_user(
    Path(id): Path<i32>,
    mut ctx: RuniqueContext,
) -> Response {
    let db = ctx.engine.db.clone();

    match users::Entity::delete_by_id(id)
        .exec(&*db)
        .await
    {
        Ok(result) if result.rows_affected > 0 => {
            success!(ctx.flash => "Utilisateur supprimé");
            Redirect::to("/users").into_response()
        }
        _ => {
            error!(ctx.flash => "Erreur lors de la suppression");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
```

---

## Prochaines étapes

← [**Templates**](./06-templates.md) | [**Middleware & Sécurité**](./08-middleware.md) →
