
---
# 💾 Installation & Setup

## Prerequisites

* **Rust 1.75+** - [Install rustup](https://rustup.rs/)
* **PostgreSQL 12+** (or SQLite for dev)
* **Git**

### Check Versions

```bash
rustc --version    # Rust 1.75+
cargo --version    # Cargo 1.75+
postgres --version # PostgreSQL 12+
```

---

## Project Installation

### 1. Clone the repository

```bash
git clone https://github.com/seb-alliot/runique.git
cd runique
```

### 2. Configure .env

Create a `.env` file in the `demo-app/` directory:

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
SECRET_KEY=your_secret_key_change_in_production
ALLOWED_HOSTS=localhost,127.0.0.1
```

### 3. Create the Database

```bash
# PostgreSQL
createdb runique

# Or via psql:
psql -U postgres
CREATE DATABASE runique;
```

### 4. Configure the Database (REQUIRED)

Run the migrations:

```bash
cd demo-app/migration
cargo run
cd ..
```

**Note:** The database is mandatory – the framework cannot function without it.

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

**Expected Output:**

```rust
🦀 Runique Framework operational
   Server running at http://127.0.0.1:3000
   Connected to sqlite: runique
```

Access **[http://127.0.0.1:3000](http://127.0.0.1:3000)** 🎉

---

## SQLite Configuration (Development)

To use SQLite in development:

### 1. Update `demo-app/Cargo.toml`

```toml
[dependencies]
runique = { version = "1.1.11", features = ["orm", "sqlite"] }
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

### 2. Create User and Database

```sql
-- Connect as admin
psql -U postgres

-- Create user
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

# Debug mode (disable in production)
DEBUG=true

# Database configuration (PostgreSQL example)
DB_ENGINE=postgres
DB_USER=myuser
DB_PASSWORD=mypassword
DB_HOST=localhost
DB_PORT=5432
DB_NAME=mydatabase

# Or SQLite (default)
DB_ENGINE=sqlite
DATABASE_URL="sqlite://mydatabase.db?mode=rwc"
```

### 4. Verify Connection

```bash
psql -U runique_user -d runique -h localhost
```

---

## Migrations (SeaORM)

### View Existing Migrations

```bash
cd demo-app/migration
ls -la
```

### Run Migrations

Migrations are not automatic – follow the procedure explained in the README after `cargo new your_project`.

---

## Troubleshooting

### ❌ "Connection refused" PostgreSQL

```bash
# Check that PostgreSQL is running
sudo systemctl status postgresql

# Or macOS:
brew services list
```

### ❌ "Permission denied" on Database

```bash
# Check permissions
psql -U postgres -d runique -c "\dp"

# Reapply permissions
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO runique_user;
```

### "SQLite driver should normally be enabled by default – modify the supported database in SeaORM in your Cargo"

Check that the feature is enabled in `Cargo.toml`:

```toml
runique = { version = "1.1.11", features = ["orm", "postgres"] }
```

### ❌ Compilation Error "sea_orm"

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

# SeaORM CLI (optional)
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
4. Check out the [**Examples**](https://github.com/seb-alliot/runique/blob/main/docs/en/10-examples.md)
