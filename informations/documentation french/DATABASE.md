# 🗄️ Guide de la base de données - Rusti Framework

Rusti propose une intégration avec SeaORM qui offre une API inspirée de Django ORM.

## Table des matières

1. [Configuration](#configuration)
2. [Définition des modèles](#définition-des-modèles)
3. [API Django-like](#api-django-like)
4. [Migrations](#migrations)
5. [Requêtes avancées](#requêtes-avancées)
6. [Relations](#relations)
7. [Transactions](#transactions)

---

## Configuration

### Bases de données supportées

Rusti supporte plusieurs moteurs de bases de données via SeaORM :

| Base de données | Feature Cargo | URL de connexion |
|-----------------|---------------|------------------|
| **SQLite** | `sqlite` (défaut) | `sqlite://database.db?mode=rwc` |
| **PostgreSQL** | `postgres` | `postgres://user:pass@host:5432/db` |
| **MySQL** | `mysql` | `mysql://user:pass@host:3306/db` |
| **MariaDB** | `mariadb` | `mariadb://user:pass@host:3306/db` |

### Installation avec feature

```toml
# Cargo.toml

# SQLite (activé par défaut)
[dependencies]
rusti = "0.1"

# PostgreSQL
[dependencies]
rusti = { version = "0.1", features = ["postgres"] }

# MySQL
[dependencies]
rusti = { version = "0.1", features = ["mysql"] }

# Toutes les bases
[dependencies]
rusti = { version = "0.1", features = ["all-databases"] }
```

### Configuration via fichier `.env`

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

### Configuration programmatique

#### Méthode 1 : Depuis `.env`

```rust
use rusti::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::from_env();
    
    // Détection automatique depuis .env
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

#### Méthode 2 : Configuration manuelle

```rust
use rusti::DatabaseConfig;
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

---

## Définition des modèles

### Modèle de base

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

// ✨ Active l'API Django-like
impl_objects!(Entity);
```

### Colonnes avec options

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

### Énumérations

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

## API Django-like

Grâce à `impl_objects!(Entity)`, vous bénéficiez d'une API similaire à Django ORM.

### Récupération basique

```rust
use crate::models::{users, Entity as User};
use rusti::prelude::*;

pub async fn examples(db: &DatabaseConnection) -> Result<(), DbErr> {
    // Tous les enregistrements
    let all_users = User::objects.all().all(db).await?;
    
    // Un enregistrement par ID (erreur si absent)
    let user = User::objects.get(db, 1).await?;
    
    // Un enregistrement par ID (None si absent)
    let maybe_user = User::objects.get_optional(db, 999).await?;
    
    // Compter
    let count = User::objects.count(db).await?;
    
    Ok(())
}
```

### Filtrage

```rust
// Filtre simple
let adults = User::objects
    .filter(users::Column::Age.gte(18))
    .all(db)
    .await?;

// Filtres multiples (AND)
let active_adults = User::objects
    .filter(users::Column::Age.gte(18))
    .filter(users::Column::IsActive.eq(true))
    .all(db)
    .await?;

// Filtres complexes
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
// Exclure des enregistrements
let active_users = User::objects
    .exclude(users::Column::IsActive.eq(false))
    .all(db)
    .await?;

// Exclure plusieurs conditions
let valid_users = User::objects
    .exclude(users::Column::Email.like("%@spam.com"))
    .exclude(users::Column::IsBanned.eq(true))
    .all(db)
    .await?;
```

### Tri et limitation

```rust
// Tri croissant
let users = User::objects
    .order_by_asc(users::Column::Username)
    .all(db)
    .await?;

// Tri décroissant
let recent_users = User::objects
    .order_by_desc(users::Column::CreatedAt)
    .all(db)
    .await?;

// Limitation
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

// Premier résultat
let first_user = User::objects
    .order_by_asc(users::Column::CreatedAt)
    .first(db)
    .await?;
```

### Chaînage complet

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

## Requêtes avancées

### Opérateurs de comparaison

```rust
use sea_orm::prelude::*;

// Égalité
.filter(users::Column::Role.eq("admin"))

// Inégalité
.filter(users::Column::Status.ne("banned"))

// Supérieur / Inférieur
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

### Conditions OR

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

### Requêtes brutes SQL

```rust
use sea_orm::{FromQueryResult, Statement};

#[derive(Debug, FromQueryResult)]
struct CustomResult {
    username: String,
    post_count: i64,
}

let results = CustomResult::find_by_statement(
    Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
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

---

## Relations

### OneToMany (Un à plusieurs)

```rust
// Modèle User (parent)
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

// Modèle Post (enfant)
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

// Utilisation
use sea_orm::ModelTrait;

let user = User::objects.get(db, 1).await?;
let posts = user.find_related(Post).all(db).await?;
```

### ManyToMany (Plusieurs à plusieurs)

```rust
// Table intermédiaire
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

// Récupérer les tags d'un post
let post = Post::objects.get(db, 1).await?;
let tags = post
    .find_linked(post_tags::PostToTag)
    .all(db)
    .await?;
```

---

## Transactions

### Transaction basique

```rust
use sea_orm::TransactionTrait;

let txn = db.begin().await?;

// Opérations dans la transaction
let user = users::ActiveModel {
    username: Set("alice".to_string()),
    ..Default::default()
};
let inserted_user = user.insert(&txn).await?;

let post = posts::ActiveModel {
    title: Set("Mon premier post".to_string()),
    author_id: Set(inserted_user.id),
    ..Default::default()
};
post.insert(&txn).await?;

// Valider
txn.commit().await?;
```

### Transaction avec closure

```rust
db.transaction::<_, (), DbErr>(|txn| {
    Box::pin(async move {
        // Vos opérations ici
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

### Création de migrations

Utilisez `sea-orm-cli` :

```bash
cargo install sea-orm-cli

# Générer une migration
sea-orm-cli migrate generate create_users_table

# Appliquer les migrations
sea-orm-cli migrate up

# Rollback
sea-orm-cli migrate down
```

### Exemple de migration

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

## Exemple complet dans un handler

```rust
use rusti::prelude::*;
use crate::models::{users, Entity as User};

pub async fn create_user(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    Json(payload): Json<CreateUserRequest>,
    mut message: Message,
    template: Template,
) -> Response {
    // Vérifier si l'utilisateur existe
    let existing = User::objects
        .filter(users::Column::Email.eq(&payload.email))
        .first(&db)
        .await;
    
    if existing.is_ok() {
        let _ = message.error("Cet email est déjà utilisé").await;
        return redirect("/register");
    }
    
    // Créer l'utilisateur
    let user = users::ActiveModel {
        username: Set(payload.username),
        email: Set(payload.email),
        created_at: Set(chrono::Utc::now()),
        ..Default::default()
    };
    
    match user.insert(&*db).await {
        Ok(inserted) => {
            let _ = message.success("Compte créé avec succès !").await;
            redirect(&format!("/user/{}", inserted.id))
        }
        Err(e) => {
            let _ = message.error("Erreur lors de la création").await;
            redirect("/register")
        }
    }
}
```

---

## Bonnes pratiques

### 1. Utilisez des transactions pour les opérations multiples

```rust
// ✅ Bon
db.transaction(|txn| {
    Box::pin(async move {
        user.insert(txn).await?;
        post.insert(txn).await?;
        Ok(())
    })
}).await?;

// ❌ Mauvais (risque d'incohérence)
user.insert(db).await?;
post.insert(db).await?; // Si ça échoue, user existe déjà
```

### 2. Gérez les erreurs proprement

```rust
match User::objects.get(db, id).await {
    Ok(user) => {
        // Traitement
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

### 3. Utilisez des index pour les requêtes fréquentes

```rust
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    
    #[sea_orm(indexed)] // ✅ Index pour recherche rapide
    pub email: String,
    
    #[sea_orm(indexed)]
    pub username: String,
}
```

---

## Voir aussi

- 📖 [SeaORM Documentation](https://www.sea-ql.org/SeaORM/)
- 🚀 [Guide de démarrage](GETTING_STARTED.md)
- 🔧 [Configuration](CONFIGURATION.md)

**Développez efficacement avec Rusti ! 🦀**
