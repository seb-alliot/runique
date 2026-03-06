# 💾 Installation & Setup

## Prerequisites

* **Rust 1.75+** – [Install rustup](https://rustup.rs/)
* **PostgreSQL 12+** (or SQLite for development)
* **Git**

### Check Versions

```bash
rustc --version    # Rust 1.75+
cargo --version    # Cargo 1.75+
postgres --version # PostgreSQL 12+
```

---

## Project Installation

### 1. Clone the Repository

```bash
git clone https://github.com/seb-alliot/runique.git
cd runique
```

### 2. Configure .env

Create a `.env` file inside the `demo-app/` directory:

```env
# Server
IP_SERVER=127.0.0.1
PORT=3000
DEBUG=true

# Database (PostgreSQL)
DB_ENGINE=postgres
DB_USER=postgres
DB_PASSWORD=your_password_here
DB_HOST=localhost
DB_PORT=5432
DB_NAME=runique
DATABASE_URL=postgres://postgres:your_password_here@localhost:5432/runique

# Templates & Static Files
TEMPLATES_DIR=templates
STATICFILES_DIRS=static
MEDIA_ROOT=media

# Security
SECRETE_KEY=your_secret_key_change_in_production
RUNIQUE_ALLOWED_HOSTS=localhost,127.0.0.1
```

### 3. Create the Database

```bash
# PostgreSQL
createdb runique

# Or from psql:
psql -U postgres
CREATE DATABASE runique;
```

### 4. Apply Migrations

Apply the existing migrations via the SeaORM CLI:

```bash
cd demo-app/migration
sea-orm-cli migrate up
cd ..
```

**Note:** Migrations initialize the database schema required by the framework.

### 5. Build the Project

```bash
cargo build

# Or for release mode (optimized):
cargo build --release
```

### 6. Start the Server

```bash
cargo run -p demo-app
```

**Expected output:**

```rust
🦀 Runique Framework operational
   Server running at http://127.0.0.1:3000
   Connected to sqlite: runique
```

Visit **[http://127.0.0.1:3000](http://127.0.0.1:3000)** 🎉

---

## SQLite Configuration (Development)

To use SQLite in development:

### 1. Modify `demo-app/Cargo.toml`

```toml
[dependencies]
runique = { version = "1.1.41", features = ["orm", "sqlite"] }

```

### 2. Update `.env`

```env
# SQLite
DATABASE_URL=sqlite:runique.db?mode=rwc
```

### 3. Restart

```bash
cargo run -p demo-app
```

SQLite will automatically create the `runique.db` file.

---

## PostgreSQL Configuration (Production)

### 1. Install PostgreSQL

**macOS:**

```bash
brew install postgresql
brew services start postgresql
```

**Linux (Debian/Ubuntu):**

```bash
sudo apt-get install postgresql postgresql-contrib
sudo systemctl start postgresql
```

**Windows:**

* [Download the installer](https://www.postgresql.org/download/windows/)
* Follow the installation wizard

### 2. Create the User and Database

```sql
-- Connect as admin
psql -U postgres

-- Create the user
CREATE USER runique_user WITH PASSWORD 'secure_password';

-- Create the database
CREATE DATABASE runique OWNER runique_user;

-- Grant permissions
GRANT ALL PRIVILEGES ON DATABASE runique TO runique_user;
GRANT ALL PRIVILEGES ON SCHEMA public TO runique_user;
```

### 3. Configure `.env`

```env

IP_SERVER=127.0.0.1
PORT=3000
SECRET_KEY=your-secret-key-change-in-production
ALLOWED_HOSTS=localhost,127.0.0.1

# Debug Mode (disable in production)
DEBUG=true

# Database configuration (sqlite example)
DB_ENGINE=sqlite
DB_USER=monuser
DB_PASSWORD=monmotdepasse
DB_HOST=localhost
DB_PORT=5432
DB_NAME=mabase

# Or SQLite (default)
DB_ENGINE=sqlite
DATABASE_URL="sqlite://mabase.db?mode=rwc"

```

### 4. Verify the Connection

```bash
psql -U runique_user -d runique -h localhost
```

---

## Migrations (SeaORM)

The migration workflow involves two steps:

### 1. Generate Migration Files

`runique makemigrations` reads your entities declared in `src/entities` and generates the corresponding migration files:

```bash
runique makemigrations --entities src/entities --migrations migration/src
```

### 2. Apply Migrations

Migrations are run via the SeaORM CLI:

```bash
sea-orm-cli migrate up --migration-dir migration/src
```

Or via the Runique wrapper:

```bash
runique migration up --migrations migration/src
```

### Other Migration Commands

```bash
runique migration down --migrations migration/src    # Revert the last migration
runique migration status --migrations migration/src  # Check migration status
```

---

## Runique CLI

### Create a Superuser

```bash
runique create-superuser
```

The command is fully interactive and step-by-step guided:

```
=== Create Superuser ===  [Ctrl+C to quit]

[1/5] Hash algorithm:
  1) Argon2  (recommended)
  2) Bcrypt
  3) Scrypt
  4) Custom provider
Choice [1-4] (default: 1):

[2/5] Username:
[3/5] Email:
[4/5] Password:
[5/5] Confirm password:

──────────────────────────────────
  Algorithm : Argon2
  Username  : admin
  Email     : admin@example.com
  Password  : ••••••••
──────────────────────────────────
[Enter] Confirm  [A] Change algorithm  [Ctrl+C] Cancel
```

**Navigation:** `ESC` goes back to the previous step at any time. Since the algorithm step comes first, it can also be changed at the end via `[A]` without starting over.

> The CLI runs without the application runtime — it has no access to the `PasswordConfig` configured in `main.rs`. The algorithm is chosen explicitly at each run.
>
> For the `Custom` case, provide a binary or script that reads the password from **stdin** and returns the hash on **stdout** — select option `4) Custom provider` and provide the path.

### Other Commands

```bash
runique new <name>                                                    # Create a new project
runique start [--main src/main.rs] [--admin src/admin.rs]           # Start with admin daemon
runique makemigrations --entities src/entities --migrations migration/src  # Generate migrations
runique migration up|down|status --migrations migration/src         # Manage migrations
```

---

## Troubleshooting

### ❌ "Connection refused" PostgreSQL

```bash
# Check that PostgreSQL is running
sudo systemctl status postgresql

# Or on macOS:
brew services list
```

### ❌ "Permission denied" on the database

```bash
# Check permissions
psql -U postgres -d runique -c "\dp"

# Reapply permissions
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO runique_user;
```

### ❌ SQLite feature not enabled

Make sure the feature is enabled in `Cargo.toml`:

```toml
runique = { version = "1.1.41", features = ["orm", "postgres"] }

```

### ❌ Compilation error "sea_orm"

```bash
# Clean and rebuild
cargo clean
cargo build
```

---

## Development Setup

### Install Recommended Tools

```bash
# Rust analyzer for IDE
rustup component add rust-analyzer

# Linter & formatter
rustup component add clippy rustfmt

# SeaORM CLI (required for migrations)
cargo install sea-orm-cli
```

### Pre-commit Hooks (optional)

```bash
# Install pre-commit
pip install pre-commit

# Setup hooks
pre-commit install

# Test hooks
pre-commit run --all-files
```

---

## Next Steps

✅ Installation complete! Now:

1. Read the [**Architecture**](https://github.com/seb-alliot/runique/blob/main/docs/en/02-architecture.md)
2. Create your first [**Routing**](https://github.com/seb-alliot/runique/blob/main/docs/en/04-routing.md)
3. Define your [**Forms**](https://github.com/seb-alliot/runique/blob/main/docs/en/05-forms.md)
4. Explore the [**Examples**](https://github.com/seb-alliot/runique/blob/main/docs/en/10-examples.md)

---
