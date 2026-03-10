# Configuration Base de Données

## SQLite (Développement)

### 1. Modifier `demo-app/Cargo.toml`

```toml
[dependencies]
runique = { version = "1.1.45", features = ["orm", "sqlite"] }
```

### 2. Mettre à jour `.env`

```env
DATABASE_URL=sqlite:runique.db?mode=rwc
```

### 3. Relancer

```bash
cargo run -p demo-app
```

SQLite créera le fichier `runique.db` automatiquement.

---

## PostgreSQL (Production)

### 1. Installer PostgreSQL

**macOS :**

```bash
brew install postgresql
brew services start postgresql
```

**Linux (Debian/Ubuntu) :**

```bash
sudo apt-get install postgresql postgresql-contrib
sudo systemctl start postgresql
```

**Windows :**

- [Télécharger l'installer](https://www.postgresql.org/download/windows/)
- Suivre l'assistant d'installation

### 2. Créer l'utilisateur et la base de données

```sql
-- Connecter en tant qu'admin
psql -U postgres

-- Créer l'utilisateur
CREATE USER runique_user WITH PASSWORD 'secure_password';

-- Créer la base de données
CREATE DATABASE runique OWNER runique_user;

-- Accorder les permissions
GRANT ALL PRIVILEGES ON DATABASE runique TO runique_user;
GRANT ALL PRIVILEGES ON SCHEMA public TO runique_user;
```

### 3. Configurer `.env`

```env
IP_SERVER=127.0.0.1
PORT=3000
DEBUG=true
SECRET_KEY=votre-cle-secrete-a-changer-en-production
ALLOWED_HOSTS=localhost,127.0.0.1

DB_ENGINE=postgres
DB_USER=runique_user
DB_PASSWORD=secure_password
DB_HOST=localhost
DB_PORT=5432
DB_NAME=runique
DATABASE_URL=postgres://runique_user:secure_password@localhost:5432/runique
```

### 4. Vérifier la connexion

```bash
psql -U runique_user -d runique -h localhost
```

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Prérequis](https://github.com/seb-alliot/runique/blob/main/docs/fr/installation/prerequis/prerequis.md) | Setup initial |
| [Migrations](https://github.com/seb-alliot/runique/blob/main/docs/fr/installation/migrations/migrations.md) | Workflow de migration |

## Retour au sommaire

- [Installation](https://github.com/seb-alliot/runique/blob/main/docs/fr/installation/01-installation.md)
