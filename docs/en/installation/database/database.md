# Database Configuration

## SQLite (Development)

### 1. Modify `demo-app/Cargo.toml`

```toml
[dependencies]
runique = { version = "1.1.54", features = ["orm", "sqlite"] }
```

### 2. Update `.env`

```env
DATABASE_URL=sqlite:runique.db?mode=rwc
```

### 3. Restart

```bash
cargo run -p demo-app
```

SQLite by default, PostgreSQL recommended for production.

---

## PostgreSQL (Production)

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

- [Download the installer](https://www.postgresql.org/download/windows/)
- Follow the installation wizard

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
DEBUG=true
SECRET_KEY=your-secret-key-change-in-production

DB_ENGINE=postgres
DB_USER=runique_user
DB_PASSWORD=secure_password
DB_HOST=localhost
DB_PORT=5432
DB_NAME=runique
DATABASE_URL=postgres://runique_user:secure_password@localhost:5432/runique
```

### 4. Verify the Connection

```bash
psql -U runique_user -d runique -h localhost
```

---

## Create a superuser

After applying migrations, create the first admin account:

```bash
runique create-superuser
```

The CLI guides through an interactive wizard: hash algorithm, username, email, password, then a confirmation step before creation.

---

## See also

| Section | Description |
| --- | --- |
| [Prerequisites](/docs/en/installation/prerequisites) | Initial setup |
| [Migrations](/docs/en/installation/migrations) | Migration workflow |

## Back to summary

- [Installation](/docs/en/installation)
