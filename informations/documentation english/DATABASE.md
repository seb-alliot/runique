# 🗄️ Database Guide - Rusti Framework

Rusti offers integration with SeaORM providing a Django-like API.

## Table of Contents

1. [Configuration](#configuration)
2. [Defining Models](#defining-models)
3. [Django-like API](#django-like-api)
4. [Migrations](#migrations)
5. [Advanced Queries](#advanced-queries)
6. [Relations](#relations)
7. [Transactions](#transactions)

---

## Configuration

### Supported Databases

| Database | Cargo Feature | Connection URL |
|----------|---------------|----------------|
| **SQLite** | `sqlite` (default) | `sqlite://database.db?mode=rwc` |
| **PostgreSQL** | `postgres` | `postgres://user:pass@host:5432/db` |
| **MySQL** | `mysql` | `mysql://user:pass@host:3306/db` |
| **MariaDB** | `mariadb` | `mariadb://user:pass@host:3306/db` |

### Installation with Feature

```toml
# Cargo.toml

# SQLite (enabled by default)
[dependencies]
rusti = "0.1"

# PostgreSQL
[dependencies]
rusti = { version = "0.1", features = ["postgres"] }

# MySQL
[dependencies]
rusti = { version = "0.1", features = ["mysql"] }

# All databases
[dependencies]
rusti = { version = "0.1", features = ["all-databases"] }
```

### Configuration via `.env` File

```env
# PostgreSQL
DB_ENGINE=postgres
DB_USER=myuser
DB_PASSWORD=mypassword
DB_HOST=localhost
DB_PORT=5432
DB_NAME=mydb

# MySQL / MariaDB
DB_ENGINE=mysql
DB_USER=root
DB_PASSWORD=secret
DB_HOST=localhost
DB_PORT=3306
DB_NAME=mydb

# SQLite
DB_ENGINE=sqlite
DB_NAME=database.sqlite
```

### Programmatic Configuration

```rust
use rusti::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::from_env();
    
    // Automatic detection from .env
    let db_config = DatabaseConfig::from_env()?.build();
    let db = db_config.connect().await?;
    
    RustiApp::new(settings).await?
        .with_database(db)
        .routes(routes())
        .run()
        .await?;
    
    Ok(())
}
```

---

## Defining Models

### Basic Model

```rust
use sea_orm::entity::prelude::*;
use rusti::impl_objects;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub username: String,
    pub email: String,
    pub is_active: bool,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// ✨ Enable Django-like API
impl_objects!(Entity);
```

### Columns with Options

```rust
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "posts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    
    #[sea_orm(unique)]
    pub slug: String,
    
    #[sea_orm(column_type = "Text")]
    pub content: String,
    
    #[sea_orm(nullable)]
    pub published_at: Option<DateTime>,
    
    #[sea_orm(default_value = "true")]
    pub is_draft: bool,
    
    #[sea_orm(indexed)]
    pub author_id: i32,
}

impl_objects!(Entity);
```

---

## Django-like API

Thanks to `impl_objects!(Entity)`, you get a Django ORM-like API.

### Basic Retrieval

```rust
use crate::models::{users, Entity as User};
use rusti::prelude::*;

pub async fn examples(db: &DatabaseConnection) -> Result<(), DbErr> {
    // All records
    let all_users = User::objects.all().all(db).await?;
    
    // One record by ID (error if absent)
    let user = User::objects.get(db, 1).await?;
    
    // One record by ID (None if absent)
    let maybe_user = User::objects.get_optional(db, 999).await?;
    
    // Count
    let count = User::objects.count(db).await?;
    
    Ok(())
}
```

### Filtering

```rust
// Simple filter
let adults = User::objects
    .filter(users::Column::Age.gte(18))
    .all(db)
    .await?;

// Multiple filters (AND)
let active_adults = User::objects
    .filter(users::Column::Age.gte(18))
    .filter(users::Column::IsActive.eq(true))
    .all(db)
    .await?;

// Complex filters
use sea_orm::Condition;

let users = User::objects
    .filter(
        Condition::all()
            .add(users::Column::Age.gte(18))
            .add(users::Column::Email.like("%@example.com"))
    )
    .all(db)
    .await?;
```

### Exclusion

```rust
// Exclude records
let active_users = User::objects
    .exclude(users::Column::IsActive.eq(false))
    .all(db)
    .await?;
```

### Sorting and Limiting

```rust
// Ascending sort
let users = User::objects
    .order_by_asc(users::Column::Username)
    .all(db)
    .await?;

// Descending sort
let recent_users = User::objects
    .order_by_desc(users::Column::CreatedAt)
    .all(db)
    .await?;

// Limit
let top_10 = User::objects
    .order_by_desc(users::Column::Score)
    .limit(10)
    .all(db)
    .await?;

// Pagination
let page_2 = User::objects
    .order_by_asc(users::Column::Id)
    .offset(20)
    .limit(10)
    .all(db)
    .await?;

// First result
let first_user = User::objects
    .order_by_asc(users::Column::CreatedAt)
    .first(db)
    .await?;
```

### Complete Chaining

```rust
let result = User::objects
    .filter(users::Column::IsActive.eq(true))
    .exclude(users::Column::IsBanned.eq(true))
    .filter(users::Column::Age.gte(18))
    .order_by_desc(users::Column::CreatedAt)
    .limit(20)
    .all(db)
    .await?;
```

---

## Advanced Queries

### Comparison Operators

```rust
use sea_orm::prelude::*;

// Equality
.filter(users::Column::Role.eq("admin"))

// Greater/Less than
.filter(users::Column::Age.gte(18))
.filter(users::Column::Score.lt(100))

// LIKE
.filter(users::Column::Email.like("%@gmail.com"))
.filter(users::Column::Username.starts_with("admin"))

// IN
.filter(users::Column::Status.is_in(["active", "pending"]))

// BETWEEN
.filter(users::Column::Age.between(18, 65))

// IS NULL / IS NOT NULL
.filter(users::Column::DeletedAt.is_null())
.filter(users::Column::VerifiedAt.is_not_null())
```

### OR Conditions

```rust
use sea_orm::Condition;

let users = User::objects
    .filter(
        Condition::any()
            .add(users::Column::Role.eq("admin"))
            .add(users::Column::Role.eq("moderator"))
    )
    .all(db)
    .await?;
```

---

## Relations

### OneToMany (One-to-Many)

```rust
// User model (parent)
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct UserModel {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub username: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum UserRelation {
    #[sea_orm(has_many = "super::posts::Entity")]
    Posts,
}

// Post model (child)
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "posts")]
pub struct PostModel {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub title: String,
    pub author_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum PostRelation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::AuthorId",
        to = "super::users::Column::Id"
    )]
    Author,
}

// Usage
use sea_orm::ModelTrait;

let user = User::objects.get(db, 1).await?;
let posts = user.find_related(Post).all(db).await?;
```

---

## Transactions

### Basic Transaction

```rust
use sea_orm::TransactionTrait;

let txn = db.begin().await?;

// Operations in the transaction
let user = users::ActiveModel {
    username: Set("alice".to_string()),
    ..Default::default()
};
let inserted_user = user.insert(&txn).await?;

let post = posts::ActiveModel {
    title: Set("My first post".to_string()),
    author_id: Set(inserted_user.id),
    ..Default::default()
};
post.insert(&txn).await?;

// Commit
txn.commit().await?;
```

---

## Migrations

### Creating Migrations

Use `sea-orm-cli`:

```bash
cargo install sea-orm-cli

# Generate a migration
sea-orm-cli migrate generate create_users_table

# Apply migrations
sea-orm-cli migrate up

# Rollback
sea-orm-cli migrate down
```

---

## Complete Example in a Handler

```rust
use rusti::prelude::*;
use crate::models::{users, Entity as User};

pub async fn create_user(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    Json(payload): Json<CreateUserRequest>,
    mut message: Message,
) -> Response {
    // Check if user exists
    let existing = User::objects
        .filter(users::Column::Email.eq(&payload.email))
        .first(&db)
        .await;
    
    if existing.is_ok() {
        let _ = message.error("This email is already in use").await;
        return redirect("/register");
    }
    
    // Create user
    let user = users::ActiveModel {
        username: Set(payload.username),
        email: Set(payload.email),
        created_at: Set(chrono::Utc::now()),
        ..Default::default()
    };
    
    match user.insert(&*db).await {
        Ok(inserted) => {
            let _ = message.success("Account created successfully!").await;
            redirect(&format!("/user/{}", inserted.id))
        }
        Err(_) => {
            let _ = message.error("Error during creation").await;
            redirect("/register")
        }
    }
}
```

---

## Best Practices

### 1. Use transactions for multiple operations

```rust
// ✅ Good
db.transaction(|txn| {
    Box::pin(async move {
        user.insert(txn).await?;
        post.insert(txn).await?;
        Ok(())
    })
}).await?;

// ❌ Bad (risk of inconsistency)
user.insert(db).await?;
post.insert(db).await?; // If this fails, user already exists
```

### 2. Handle errors properly

```rust
match User::objects.get(db, id).await {
    Ok(user) => {
        // Processing
    }
    Err(DbErr::RecordNotFound(_)) => {
        return (StatusCode::NOT_FOUND, "User not found").into_response();
    }
    Err(e) => {
        tracing::error!("Database error: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, "Error").into_response();
    }
}
```

### 3. Use indexes for frequent queries

```rust
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    
    #[sea_orm(indexed)] // ✅ Index for fast search
    pub email: String,
    
    #[sea_orm(indexed)]
    pub username: String,
}
```

---

## See Also

- 📖 [SeaORM Documentation](https://www.sea-ql.org/SeaORM/)
- 🚀 [Getting Started](GETTING_STARTED.md)
- 🔧 [Configuration](CONFIGURATION.md)

**Develop efficiently with Rusti! 🦀**
