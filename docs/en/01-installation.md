# 💾 Installation & Setup

## Prerequisites

- **Rust 1.70+** - [Install rustup](https://rustup.rs/)
- **PostgreSQL 12+** (or SQLite for dev)
- **Git**

### Verify versions:

```bash
rustc --version    # Rust 1.70+
cargo --version    # Cargo 1.70+
postgres --version # PostgreSQL 12+
```

---

## Project Installation

### 1. Clone repository

```bash
git clone https://github.com/seb-alliot/runique.git
cd runique
```

### 2. Configure .env

Create `.env` file in `demo-app/` directory:

```env
# Server
IP_SERVER=127.0.0.1
PORT=3000
SECRET_KEY=your-secret-key-change-this-in-production
ALLOWED_HOSTS=localhost,127.0.0.1

# Debug Mode (set to false in production)
DEBUG=true

# Database Configuration (PostgreSQL example)
DB_ENGINE=postgres
DB_USER=myuser
DB_PASSWORD=mypassword
DB_HOST=localhost
DB_PORT=5432
DB_NAME=mydb

# Or SQLite (default)
DB_ENGINE=sqlite
DB_NAME=app.db
```

### 3. Create database

```bash
# PostgreSQL
createdb runique

# Or from psql:
psql -U postgres
CREATE DATABASE runique;
```

### 4. Build project

```bash
cargo build

# Or release (optimized):
cargo build --release
```

### 5. Run server

```bash
cargo run -p demo-app
```

**Expected output:**
```
🦀 Runique Framework opérationnel
   Serveur lancé sur http://127.0.0.1:3000
   Connected to Sqlite : runique
```

Visit **http://127.0.0.1:3000** 🎉

---

## SQLite Configuration (Development)

For SQLite development:

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

SQLite will create `runique.db` automatically.

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
- [Download installer](https://www.postgresql.org/download/windows/)
- Follow installation wizard

### 2. Create user and database

```sql
-- Connect as admin
psql -U postgres

-- Create user
CREATE USER runique_user WITH PASSWORD 'secure_password';

-- Create database
CREATE DATABASE runique OWNER runique_user;

-- Grant permissions
GRANT ALL PRIVILEGES ON DATABASE runique TO runique_user;
GRANT ALL PRIVILEGES ON SCHEMA public TO runique_user;
```

### 3. Configure `.env`

```env

DB_ENGINE=postgres
DB_USER=runique_user
DB_PASSWORD=secure_password
DB_HOST=localhost
DB_PORT=5432
DB_NAME=runique
DATABASE_URL=postgresql://username:password@localhost:5432/database_name

```

### 4. Test connection

```bash
psql -U runique_user -d runique -h localhost
```

---

## Migrations (SeaORM)

### View migrations

```bash
cd demo-app/migration
ls -la
```

### Run migrations

Migrations are not automatic. Please follow the procedure explained in the README provided after running cargo new your_project.

## Troubleshooting

### ❌ "Connection refused" PostgreSQL

```bash
# Check if PostgreSQL is running
sudo systemctl status postgresql

# Or macOS:
brew services list
```

### ❌ "Permission denied" database

```bash
# Check permissions
psql -U postgres -d runique -c "\dp"

# Reapply permissions
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO runique_user;
```

### ❌ "SQLite driver not enabled"

Check feature is enabled in `Cargo.toml`:
```toml
runique = { version = "1.1.11", features = ["orm", "postgres"] }
```

### ❌ Compilation error "sea_orm"

```bash
# Clean and rebuild
cargo clean
cargo build
```

---

## Development Setup

### Install recommended tools

```bash
# Rust analyzer for IDE
rustup component add rust-analyzer

# Linter & formatter
rustup component add clippy rustfmt

# SeaORM CLI (optional)
cargo install sea-orm-cli
```

### Pre-commit hooks (optional)

```bash
# Install pre-commit
pip install pre-commit

# Setup hooks
pre-commit install

# Test hooks
pre-commit run --all-files
```

---

## Next steps

✅ Installation complete! Now:

1. Read [**Architecture**](https://github.com/seb-alliot/runique/blob/main/docs/en/02-architecture.md)
2. Create your first [**Routes**](https://github.com/seb-alliot/runique/blob/main/docs/en/04-routing.md)
3. Define your [**Forms**](https://github.com/seb-alliot/runique/blob/main/docs/en/05-forms.md)
4. Check [**Examples**](https://github.com/seb-alliot/runique/blob/main/docs/en/10-examples.md)
