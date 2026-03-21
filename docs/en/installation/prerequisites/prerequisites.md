# Prerequisites & Initial Setup

## Prerequisites

- **Rust 1.85+** — [Install rustup](https://rustup.rs/)
- **PostgreSQL 12+** (or SQLite for development)
- **Git**

### Check Versions

```bash
rustc --version    # Rust 1.85+
cargo --version    # Cargo 1.85+
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

### 3. Build the Project

```bash
cargo build

# Or for release mode (optimized):
cargo build --release
```

### 4. Start the Server

```bash
cargo run -p demo-app
```

**Expected output:**

```
🦀 Runique Framework operational
   Server running at http://127.0.0.1:3000
```

### Recommended Tools

```bash
# Rust analyzer for IDE
rustup component add rust-analyzer

# Linter & formatter
rustup component add clippy rustfmt

# SeaORM CLI (required for migrations)
cargo install sea-orm-cli
```

---

## See also

| Section | Description |
| --- | --- |
| [Database](/docs/en/installation/database) | SQLite, PostgreSQL |
| [Migrations](/docs/en/installation/migrations) | Migration workflow |

## Back to summary

- [Installation](/docs/en/installation)
