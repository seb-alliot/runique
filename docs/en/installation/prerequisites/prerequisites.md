# Prerequisites & Initial Setup

## Prerequisites

- **Rust 1.88+** — [Install rustup](https://rustup.rs/)
- **PostgreSQL 12+** (or SQLite for development)

### Check Versions

```bash
rustc --version    # Rust 1.88+
cargo --version    # Cargo 1.88+
```

---

## Installation

### 1. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Install the Runique CLI

```bash
cargo install runique
```

### 3. Create a New Project

```bash
runique new my-project
cd my-project
```

### 4. Configure the Environment

Create a `.env` file at the project root:

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
DB_NAME=my-project
DATABASE_URL=postgres://postgres:your_password_here@localhost:5432/my-project

# Templates & Static Files
TEMPLATES_DIR=templates
STATICFILES_DIRS=static
MEDIA_ROOT=media

# Security
SECRET_KEY=your_secret_key_change_in_production
```

### 5. Start the Server

```bash
cargo run
```

**Expected output:**

```
🦀 Runique Framework operational
   Server running at http://127.0.0.1:3000
```

> `runique start` is reserved for projects using the administration panel — it starts the admin daemon alongside the server. For a project without admin, `cargo run` is sufficient.

---

## Recommended Tools

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
| [Runique CLI](/docs/en/installation/cli) | All available commands |

## Back to summary

- [Installation](/docs/en/installation)
