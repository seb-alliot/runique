# Runique — Django-inspired Rust Framework
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

This takes 5-10 minutes.

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

Delete `m20220101_000001_create_table.rs` — `runique makemigrations` will generate the real files.

---

### Step 3: Configure migration files

#### 3.1 - Modify `migration/Cargo.toml`

Replace the content of `migration/Cargo.toml` with:

```toml
[package]
name = "migration"
version = "0.1.0"
edition = "2024"

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

---

### Step 4: Define your entities

Your entities are already in `src/entities/`. Each file uses the `model!` macro:

```rust
// src/entities/users.rs
use runique::prelude::*;

model! {
    Users,
    table: "users",
    pk: id => i32,
    fields: {
        username: String [required, max_len(150), unique],
        email: String [required, unique],
        password: String [required, max_len(128)],
        created_at: datetime [auto_now],
        updated_at: datetime [auto_now_update],
    }
}
```

Runique reads these to generate migrations automatically.

---

### Step 5: Generate migrations

```bash
runique makemigrations
```

This scans `src/entities/`, compares against snapshots, and generates:

```text
migration/src/
├── lib.rs                                      ← auto-updated
├── main.rs
├── m{TIMESTAMP}_create_users_table.rs          ← generated
└── snapshot/
    └── users.rs                                ← schema snapshot
```

> For schema changes (ALTER), just modify your entity and run `runique makemigrations` again.
> Use `--force` to skip the destructive change prompt.
>
> ⚠️ **Warning**
> The `makemigrations` command generates SeaORM tables while preserving the
> chronological order of the migration system.
> To ensure migration tracking remains consistent, only use the SeaORM CLI
> to apply or manage migrations.
> Using other commands may lead to migration desynchronization.

---

### Step 6: Apply the migration

```bash
sea-orm-cli migrate up
```

You should see:

```text
Applying migration 'm{TIMESTAMP}_create_users_table'
Migration 'm{TIMESTAMP}_create_users_table' has been applied
```

**The `users` table is now created!**

---

### Step 7: Run the application

```bash
cargo run
```

Open your browser at **`http://127.0.0.1:3000`**

---

## Evolving the schema

When you need to add, remove, or modify a column:

1. Edit the `model!` in `src/entities/your_model.rs`
2. Run `runique makemigrations` — detects the diff, generates an ALTER migration
3. Run `sea-orm-cli migrate up` — applies it

For destructive changes (type change, nullable → not null):

```bash
runique makemigrations --force
```

---

## Final project structure

```
{}/
├── migration/                           <- Created in step 2
│   ├── Cargo.toml                       <- Configured in step 3.1
│   └── src/
│       ├── lib.rs                       <- Auto-managed by makemigrations
│       ├── main.rs                      <- Configured in step 3.2
│       ├── m{TIMESTAMP}_create_users_table.rs
│       └── snapshot/
│           └── users.rs
├── src/
│   ├── entities/
│   │   ├── mod.rs
│   │   └── users.rs                     <- Defines model! schema
│   ├── formulaire/
│   ├── main.rs
│   ├── url.rs
│   └── views.rs
├── templates/
├── static/
├── .env
├── Cargo.toml
└── README.md
```

---

## Useful commands

```bash
# Generate migrations from entities
runique makemigrations

# Force generation (skip destructive change prompt)
runique makemigrations --force

# Custom paths
runique makemigrations --entities src/entities --migrations migration/src

# Apply migrations
sea-orm-cli migrate up

# Check migration status
sea-orm-cli migrate status

# List available rollbacks
runique migration status

# Rollback a specific migration
runique migration down --files users/20240101_120000

# Rollback a full batch
runique migration down --batch 20240101_120000

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

Generated by **Runique CLI v1.1.52**
