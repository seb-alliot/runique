# Database Guide - Runique Framework

Runique provides integration with SeaORM offering a Django ORM-inspired API.

## Table of Contents

1. [Configuration](#configuration)
2. [Model Definition](#model-definition)
3. [Django-like API](#django-like-api)
4. [Migrations](#migrations)
5. [Advanced Queries](#advanced-queries)
6. [Relations](#relations)
7. [Transactions](#transactions)

---

## Configuration

### Supported Databases

Runique supports multiple database engines via SeaORM:

| Database | Cargo Feature | Connection URL |
|----------|---------------|----------------|
| **SQLite** | `sqlite` (default) | `sqlite://database.db?mode=rwc` |
| **PostgreSQL** | `postgres` | `postgres://user:pass@host:5432/db` |
| **MySQL** | `mysql` | `mysql://user:pass@host:3306/db` |
| **MariaDB** | `mariadb` | `mariadb://user:pass@host:3306/db` |

### Installation with Features

```toml
# Cargo.toml

# SQLite (enabled by default with 'orm' feature)
[dependencies]
runique = "0.1"

# PostgreSQL
[dependencies]
runique = { version = "0.1", features = ["postgres"] }

# MySQL
[dependencies]
runique = { version = "0.1", features = ["mysql"] }

# MariaDB (uses MySQL driver)
[dependencies]
runique = { version = "0.1", features = ["mariadb"] }

# All databases
[dependencies]
runique = { version = "0.1", features = ["all-databases"] }
```

**Feature Notes:**
- `default = ["orm"]` - ORM feature is enabled by default
- `orm` - Enables SeaORM support
- `sqlite`, `postgres`, `mysql`, `mariadb` - Specific drivers
- `all-databases` - Enables all drivers simultaneously

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

#### Method 1: From `.env`

```rust
use runique::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::from_env();

    // Automatic detection from .env
    let db_config = DatabaseConfig::from_env()?.build();
    let db = db_config.connect().await?;

    RuniqueApp::new(settings).await?
        .with_database(db)
        .routes(routes())
        .run()
        .await?;

    Ok(())
}
```

#### Method 2: Manual Configuration

```rust
use runique::DatabaseConfig;
use std::time::Duration;

let db_config = DatabaseConfig::from_url("postgres://user:pass@localhost/mydb")?
    .max_connections(20)
    .min_connections(5)
    .connect_timeout(Duration::from_secs(10))
    .pool_size(5, 20)
    .logging(true)
    .build();

let db = db_config.connect().await?;
```

### Connection Pool Configuration

#### Default Values

**Important:** Connection pool parameters have default values defined in the code (`runique/src/database/config.rs`):

```rust
pub struct DatabaseConfig {
    pub max_connections: u32,        // Default: 20
    pub min_connections: u32,        // Default: 5
    pub connect_timeout: Duration,   // Default: 8 seconds
    pub acquire_timeout: Duration,   // Default: 8 seconds
    pub idle_timeout: Duration,      // Default: 300 seconds (5 minutes)
    pub max_lifetime: Duration,      // Default: 3600 seconds (1 hour)
    pub sqlx_logging: bool,          // Default: true
}
```

**These values are automatically applied** when creating a `DatabaseConfig` via `from_env()` or `from_url()`.

#### Customization via Builder

You can modify these values using builder methods:

```rust
let db_config = DatabaseConfig::from_url("postgres://localhost/mydb")?
    .max_connections(50)              // Modify maximum
    .min_connections(10)              // Modify minimum
    .connect_timeout(Duration::from_secs(30))  // Custom timeout
    .pool_size(10, 50)               // Or both at once
    .logging(false)                   // Disable SQL logs
    .build();
```

**Available Methods:**
- `.max_connections(u32)` - Maximum number of connections in the pool
- `.min_connections(u32)` - Minimum number of connections maintained
- `.connect_timeout(Duration)` - Timeout to establish a connection
- `.pool_size(min: u32, max: u32)` - Configure min and max simultaneously
- `.logging(bool)` - Enable/disable SQL logs

#### Special Behavior for SQLite

**SQLite automatically forces `max_connections: 1` and `min_connections: 1`** because SQLite doesn't support native multi-threading the same way PostgreSQL/MySQL do.

```rust
// Configuration for SQLite
let db_config = DatabaseConfig::from_url("sqlite://local.db")?
    .max_connections(10)  // Will be ignored and replaced with 1
    .build();

// The SQLite pool will automatically have:
// - max_connections: 1
// - min_connections: 1
```

This limitation is **normal and expected** for SQLite.

#### Example: High-Load Configuration

```rust
// For a high-traffic application (PostgreSQL/MySQL)
let db_config = DatabaseConfig::from_env()?
    .max_connections(100)
    .min_connections(20)
    .connect_timeout(Duration::from_secs(10))
    .acquire_timeout(Duration::from_secs(8))
    .build();
```

#### Example: Small Application Configuration

```rust
// For a small application
let db_config = DatabaseConfig::from_env()?
    .max_connections(10)
    .min_connections(2)
    .build();
```

---

## Model Definition

### Basic Model

```rust
use sea_orm::entity::prelude::*;
use runique::impl_objects;

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

// Enable Django-like API
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

### Enumerations

```rust
#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(Some(20))")]
pub enum UserRole {
    #[sea_orm(string_value = "admin")]
    Admin,
    #[sea_orm(string_value = "moderator")]
    Moderator,
    #[sea_orm(string_value = "user")]
    User,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub username: String,
    pub role: UserRole,
}

impl_objects!(Entity);
```

---

## Django-like API

Thanks to `impl_objects!(Entity)`, you get a Django ORM-like API.

### Basic Retrieval

```rust
use crate::models::{users, Entity as User};
use runique::prelude::*;

pub async fn examples(db: &DatabaseConnection) -> Result<(), DbErr> {
    // All records
    let all_users = User::objects.all().all(db).await?;

    // One record by ID (error if not found)
    let user = User::objects.get(db, 1).await?;

    // One record by ID (None if not found)
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

// Multiple exclusions
let valid_users = User::objects
    .exclude(users::Column::Email.like("%@spam.com"))
    .exclude(users::Column::IsBanned.eq(true))
    .all(db)
    .await?;
```

### Ordering and Limiting

```rust
// Ascending order
let users = User::objects
    .order_by_asc(users::Column::Username)
    .all(db)
    .await?;

// Descending order
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

// Inequality
.filter(users::Column::Status.ne("banned"))

// Greater / Less
.filter(users::Column::Age.gte(18))
.filter(users::Column::Score.lt(100))

// LIKE
.filter(users::Column::Email.like("%@gmail.com"))
.filter(users::Column::Username.starts_with("admin"))
.filter(users::Column::Slug.ends_with("-draft"))

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

### Raw SQL Queries

```rust
use sea_orm::{FromQueryResult, Statement, DatabaseBackend};

#[derive(Debug, FromQueryResult)]
struct CustomResult {
    username: String,
    post_count: i64,
}

let results = CustomResult::find_by_statement(
    Statement::from_sql_and_values(
        DatabaseBackend::Postgres,  // ✅ Use DatabaseBackend::
        r#"
        SELECT u.username, COUNT(p.id) as post_count
        FROM users u
        LEFT JOIN posts p ON p.author_id = u.id
        GROUP BY u.username
        ORDER BY post_count DESC
        LIMIT $1
        "#,
        [10.into()],
    )
)
.all(db)
.await?;
```

**Note:** Use `DatabaseBackend::Postgres`, `DatabaseBackend::MySql`, or `DatabaseBackend::Sqlite` depending on your database.

---

## Relations

### OneToMany (One to Many)

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

### ManyToMany (Many to Many)

```rust
// Junction table
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "post_tags")]
pub struct PostTagModel {
    #[sea_orm(primary_key)]
    pub post_id: i32,
    #[sea_orm(primary_key)]
    pub tag_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum PostTagRelation {
    #[sea_orm(belongs_to = "super::posts::Entity")]
    Post,
    #[sea_orm(belongs_to = "super::tags::Entity")]
    Tag,
}

// Get tags for a post
let post = Post::objects.get(db, 1).await?;
let tags = post
    .find_linked(post_tags::PostToTag)
    .all(db)
    .await?;
```

---

## Transactions

### Basic Transaction

```rust
use sea_orm::TransactionTrait;

let txn = db.begin().await?;

// Operations within transaction
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

### Transaction with Closure

```rust
db.transaction::<_, (), DbErr>(|txn| {
    Box::pin(async move {
        // Your operations here
        let user = users::ActiveModel {
            username: Set("bob".to_string()),
            ..Default::default()
        };
        user.insert(txn).await?;

        Ok(())
    })
}).await?;
```

---

## Migrations

### Using sea-orm-cli

Runique doesn't yet include a migration wrapper. Use `sea-orm-cli` directly:

```bash
# Install sea-orm-cli
cargo install sea-orm-cli

# Generate entities from existing database
sea-orm-cli generate entity \
    -u postgresql://user:pass@localhost/mydb \
    -o src/entities

# Create a migration
sea-orm-cli migrate generate create_users_table

# Apply migrations
sea-orm-cli migrate up

# Rollback
sea-orm-cli migrate down
```

### Migration Structure

```
migrations/
├── mod.rs
├── m20220101_000001_create_users_table.rs
├── m20220102_000001_create_posts_table.rs
└── m20220103_000001_add_email_index.rs
```

### Migration Example

```rust
// migration/m20250101_000001_create_users.rs
use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250101_000001_create_users"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Users::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Users::Username).string().not_null())
                    .col(ColumnDef::new(Users::Email).string().not_null().unique_key())
                    .col(ColumnDef::new(Users::CreatedAt).timestamp().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
    Username,
    Email,
    CreatedAt,
}
```

---

## Complete Handler Example

```rust
use runique::prelude::*;
use crate::models::{users, Entity as User};

pub async fn create_user(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    Json(payload): Json<CreateUserRequest>,
    mut message: Message,
    template: Template,
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
        Err(e) => {
            let _ = message.error("Error creating account").await;
            redirect("/register")
        }
    }
}
```

---

## Best Practices

### 1. Use Transactions for Multiple Operations

```rust
// Good
db.transaction(|txn| {
    Box::pin(async move {
        user.insert(txn).await?;
        post.insert(txn).await?;
        Ok(())
    })
}).await?;

// Bad (risk of inconsistency)
user.insert(db).await?;
post.insert(db).await?; // If this fails, user already exists
```

### 2. Handle Errors Properly

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

### 3. Use Indexes for Frequent Queries

```rust
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    #[sea_orm(indexed)] // Index for fast search
    pub email: String,

    #[sea_orm(indexed)]
    pub username: String,
}
```

### 4. Optimize Connection Pool Based on Your Needs

```rust
// For high-traffic application
let db_config = DatabaseConfig::from_env()?
    .max_connections(100)
    .min_connections(20)
    .build();

// For small application
let db_config = DatabaseConfig::from_env()?
    .max_connections(10)
    .min_connections(2)
    .build();
```

---

## See Also

- [SeaORM Documentation](https://www.sea-ql.org/SeaORM/)
- [Getting Started Guide](informations/documentation_english/GETTING_STARTED.md)
- [Configuration](informations/documentation_english/CONFIGURATION.md)

Develop efficiently with Runique!

---

**Version:** 1.0.6 (Corrected - January 2, 2026)
**License:** MIT

*Documentation created with ❤️ by Claude for Itsuki*
