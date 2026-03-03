# 🗄️ ORM & Base de Données

## SeaORM + Objects Manager

Runique utilise **SeaORM** avec un manager Django-like via la macro `impl_objects!` :

```rust
use demo_app::models::users;
use runique::impl_objects; // ajoute <Entity>::objects

// Ajoutez le manager une fois dans votre modèle
impl_objects!(users::Entity);

// Récupérer tous les users
let all_users = users::Entity::objects
    .all()
    .all(&*db)
    .await?;

// Avec filtre
let active_users = users::Entity::objects
    .filter(users::Column::Active.eq(true))
    .all(&*db)
    .await?;

// Un seul résultat
let user = users::Entity::objects
    .filter(users::Column::Id.eq(user_id))
    .first(&*db)
    .await?;
```

### Sans macro (SeaORM natif)

```rust
// Tous les enregistrements
let all_users = users::Entity::find().all(&*db).await?;

// Avec filtre
let active_users = users::Entity::find()
    .filter(users::Column::Active.eq(true))
    .all(&*db)
    .await?;

// Un seul résultat par ID
let user = users::Entity::find_by_id(user_id)
    .one(&*db)
    .await?;
```

### Helpers disponibles
- `all()` : point d'entrée pour chaîner `filter`, `order_by_asc/desc`, `limit`, `offset`.
- `filter(...)` / `exclude(...)` : ajoutent des conditions.
- `first(db)` / `count(db)` : exécutent la requête (alias pour `one` / comptage rapide côté client).
- `get(db, id)` / `get_optional(db, id)` : accès direct par clé primaire.
- `get_or_404(db, ctx, msg)` : retourne une réponse 404/500 avec rendu Tera si manquant.

---

## Requêtes Courantes

### SELECT - Récupérer

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
use sea_orm::Order;
let users = users::Entity::objects
    .all()
    .order_by_asc(users::Column::Name)
    .all(&*db)
    .await?;

// Multiple colonnes
let users = users::Entity::objects
    .all()
    .order_by_asc(users::Column::CreatedAt)
    .order_by_desc(users::Column::Id)
    .all(&*db)
    .await?;
```

### COUNT - Compter

```rust
let count = users::Entity::objects
    .filter(users::Column::Active.eq(true))
    .count(&*db)
    .await?;

println!("Utilisateurs actifs: {}", count);
```

### WHERE - Filtrage

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

let users = users::Entity::objects
    .filter(users::Column::Name.like("%john%"))
    .all(&*db)
    .await?;

// Multiples conditions
use sea_orm::sea_query::Expr;
let users = users::Entity::objects
    .filter(users::Column::Active.eq(true))
    .filter(users::Column::Age.gte(18))
    .all(&*db)
    .await?;

// OU
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

## Pattern Complet

```rust
use demo_app::models::users;
use runique::prelude::*;
use axum::extract::Path;

// Handler CRUD
// Variante avec macro (Entity::objects)
async fn list_users() -> AppResult<Response> {
    let db = /* access db from app state */;

    match users::Entity::find()
        .all(&*db)
        .await
    {
        Ok(users) => Json(json!({"users": users})),
        Err(e) => error_response(format!("Database error: {}", e)),
    }
}

// Variante sans macro (SeaORM natif)
async fn list_users_raw() -> AppResult<Response> {
    let db = /* access db from app state */;

    match users::Entity::find()
        .all(&*db)
        .await
    {
        Ok(users) => Json(json!({"users": users})),
        Err(e) => error_response(format!("Database error: {}", e)),
    }
}

// Variante avec find_by_id
async fn get_user(
    Path(id): Path<i32>,
) -> AppResult<Response> {
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

// Variante sans macro (find_by_id)
async fn get_user_raw(
    Path(id): Path<i32>,
) -> AppResult<Response> {
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
    mut request: Request,
    Json(payload): Json<CreateUserRequest>,
) -> AppResult<Response> {
    let db = /* access db from app state */;

    let user = users::ActiveModel {
        email: Set(payload.email),
        username: Set(payload.username),
        ..Default::default()
    };

    match user.insert(&*db).await {
        Ok(created) => {
            success!(request.notices => "Utilisateur créé !");
            (StatusCode::CREATED, Json(json!({"user": created}))).into_response()
        }
        Err(e) => {
            error!(request.notices => format!("Erreur : {}", e));
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn delete_user(
    Path(id): Path<i32>,
    mut request: Request,
) -> AppResult<Response> {
    let db = /* access db from app state */;

    match users::Entity::delete_by_id(id)
        .exec(&*db)
        .await
    {
        Ok(result) if result.rows_affected > 0 => {
            success!(request.notices => "Utilisateur supprimé");
            Redirect::to("/users").into_response()
        }
        _ => {
            error!(request.notices => "Erreur lors de la suppression");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
```

---

## Prochaines étapes

← [**Templates**](https://github.com/seb-alliot/runique/blob/main/docs/fr/06-templates.md) | [**Middleware & Sécurité**](https://github.com/seb-alliot/runique/blob/main/docs/fr/08-middleware.md) →
