Voici la version sans icônes :

```markdown
# Automatically Generated Runique Project

A modern web framework inspired by Django, built with Rust, Axum, SeaORM and Tera.

---

## Quick Start

### Step 1: Install sea-orm-cli

```bash
# Global installation (recommended)
cargo install sea-orm-cli

# Installation takes about 5-10 minutes, this is normal!
```

<details>
<summary>Faster installation (SQLite only)</summary>

If you're only using SQLite, you can install a lighter version:

```bash
cargo install sea-orm-cli --no-default-features \
    --features cli,runtime-tokio-rustls,sqlite
```

This takes about 2-3 minutes instead of 5-10 minutes.

</details>

---

### Step 2: Initialize migrations

```bash
# Create the migrations folder
sea-orm-cli migrate init
```

This creates the following structure:
```
migration/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs
    ├── main.rs
    └── m20220101_000001_create_table.rs
```

---

### Step 3: Configure migration files

#### 3.1 - Modify `migration/Cargo.toml`

Replace the content of `migration/Cargo.toml` with:

```toml
[package]
name = "migration"
version = "0.1.0"
edition = "2021"

[dependencies]
sea-orm-migration = { version = "2.0.0-rc.18", features = [
    "runtime-tokio-rustls",
    "sqlx-postgres",
    "sqlx-sqlite",
] }
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"
```

> **Important**: Use version `2.0.0-rc.18` to match with Runique!

<details>
<summary>Customize features based on your database</summary>

**For PostgreSQL only**:
```toml
sea-orm-migration = { version = "2.0.0-rc.18", features = [
    "runtime-tokio-rustls",
    "sqlx-postgres"
] }
```

**For SQLite only**:
```toml
sea-orm-migration = { version = "2.0.0-rc.18", features = [
    "runtime-tokio-rustls",
    "sqlx-sqlite"
] }
```

**For MySQL only**:
```toml
sea-orm-migration = { version = "2.0.0-rc.18", features = [
    "runtime-tokio-rustls",
    "sqlx-mysql"
] }
```

</details>

#### 3.2 - Modify `migration/src/main.rs`

Replace the content of `migration/src/main.rs` with:

```rust
use sea_orm_migration::prelude::*;

#[tokio::main]
async fn main() {
    cli::run_cli(migration::Migrator).await;
}
```

> **Note**: This version ensures compatibility with Runique v1.0.85+

---

### Step 4: Create the users table migration
### Modify the initially created create_table.rs or delete and create a new migration
### Warning: if you create a new one, delete the old one and modify the timestamp number in lib.rs

```bash
# Create the migration file
sea-orm-cli migrate generate create_users_table
```

This generates a file `migration/src/m{TIMESTAMP}_create_users_table.rs`

---

### Step 5: Complete the migration file

Modify the chosen file and replace **all its content** with:

```rust
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

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
                    .col(
                        ColumnDef::new(Users::Username)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Users::Email)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Users::Password).string().not_null())
                    .col(ColumnDef::new(Users::Age).integer().not_null())
                    .col(
                        ColumnDef::new(Users::CreatedAt)
                            .timestamp()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
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

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    Username,
    Email,
    Password,
    Age,
    CreatedAt,
}
```

---

### Step 6: Apply the migration

```bash
# Apply the migration
sea-orm-cli migrate up
```

You should see:
```
Applying migration 'm{TIMESTAMP}_create_users_table'
Migration 'm{TIMESTAMP}_create_users_table' has been applied
```

**The `users` table is now created!**

---

### Step 7: Run the application

```bash
cargo run
```

Open your browser at **http://127.0.0.1:3000**

---

## Final project structure

```
{}/
├── migration/                           <- New folder created
│   ├── Cargo.toml                       <- Configured in step 3.1
│   └── src/
│       ├── lib.rs
│       ├── main.rs                      <- Modified in step 3.2
│       └── m{TIMESTAMP}_create_users_table.rs  <- Created in step 5
├── src/
│   ├── models/
│   │   ├── mod.rs
│   │   └── users.rs                     <- Matches the migration
│   ├── static/css/
│   ├── forms.rs
│   ├── main.rs
│   ├── url.rs
│   └── views.rs
├── templates/
│   ├── index.html
│   ├── about/
│   └── profile/
├── .env
├── Cargo.toml
└── README.md
```

---

## Useful commands

```bash
# Check migrations status
sea-orm-cli migrate status

# Rollback the last migration
sea-orm-cli migrate down

# Create a new migration
sea-orm-cli migrate generate migration_name

# Reset the database (deletes all data)
sea-orm-cli migrate down -n 999
sea-orm-cli migrate up
```

---

## Troubleshooting

### Error: "table users doesn't exist"
You haven't applied the migration. Run:
```bash
sea-orm-cli migrate up
```

### Error: "sea-orm-cli: command not found"
Install sea-orm-cli:
```bash
cargo install sea-orm-cli
```

### Error: "version mismatch"
Check that `migration/Cargo.toml` is using `sea-orm-migration = "2.0.0-rc.18"`

### Error: "no such table: users"
Verify that:
1. The migration has been applied (`sea-orm-cli migrate status`)
2. The `DATABASE_URL` path in `.env` is correct
3. You're running `cargo run` from the project root (not from `migration/`)

### Port 3000 already in use
Change the port in `.env`:
```env
PORT=8080
```

### Error: "workspace" when running `sea-orm-cli migrate up`
Check that `migration/Cargo.toml` contains the `[workspace]` section

---

## Included Features

- User registration with validation
- User search by name
- Automatic CSRF protection
- Flash messages (success, error, info, warning)
- Modern dark theme and responsive
- Tera templates with inheritance

---

## Documentation

- [Runique Framework](https://docs.rs/runique)
- [SeaORM Migrations](https://www.sea-ql.org/SeaORM/docs/migration/setting-up-migration/)
- [Tera Templates](https://keats.github.io/tera/)

---

Generated by **Runique CLI v1.0.85**
```
