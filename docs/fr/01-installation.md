# 💾 Installation & Setup

## Prérequis

- **Rust 1.75+** - [Installer rustup](https://rustup.rs/)
- **PostgreSQL 12+** (ou SQLite pour dev)
- **Git**

### Vérifier les versions

```bash
rustc --version    # Rust 1.75+
cargo --version    # Cargo 1.75+
postgres --version # PostgreSQL 12+
```

---

## Installation du Projet

### 1. Cloner le repository

```bash
git clone https://github.com/seb-alliot/runique.git
cd runique
```

### 2. Configuration .env

Créer un fichier `.env` dans le répertoire `demo-app/`:

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

### 3. Créer la base de données

```bash
# PostgreSQL
createdb runique

# Ou depuis psql:
psql -U postgres
CREATE DATABASE runique;
```

### 4. Configurer la base de données (REQUIS)

Lancer les migrations:

```bash
cd demo-app/migration
cargo run
cd ..
```

**Note:** La base de données est obligatoire - le framework ne peut pas fonctionner sans.

### 5. Compiler le projet

```bash
cargo build

# Ou pour le mode release (optimisé):
cargo build --release
```

### 6. Lancer le serveur

```bash
cargo run -p demo-app
```

**Output attendu:**

```rust
🦀 Runique Framework opérationnel
   Serveur lancé sur http://127.0.0.1:3000
   Connected to sqlite: runique
```

Accédez à **<http://127.0.0.1:3000>** 🎉

---

## Configuration SQLite (Développement)

Pour utiliser SQLite en développement:

### 1. Modifier `demo-app/Cargo.toml`

```toml
[dependencies]
runique = { version = "1.1.30", features = ["orm", "sqlite"] }

```

### 2. Mettre à jour `.env`

```env
# SQLite
DATABASE_URL=sqlite:runique.db?mode=rwc
```

### 3. Relancer

```bash
cargo run -p demo-app
```

SQLite créera le fichier `runique.db` automatiquement.

---

## Configuration PostgreSQL (Production)

### 1. Installer PostgreSQL

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
SECRET_KEY=votre-cle-secrete-a-changer-en-production
ALLOWED_HOSTS=localhost,127.0.0.1

# Mode Debug (désactiver en production)
DEBUG=true

# Configuration base de données (exemple PostgreSQL)
DB_ENGINE=postgres
DB_USER=monuser
DB_PASSWORD=monmotdepasse
DB_HOST=localhost
DB_PORT=5432
DB_NAME=mabase

# Ou SQLite (par défaut)
DB_ENGINE=sqlite
DATABASE_URL="sqlite://mabase.db?mode=rwc"

```

### 4. Vérifier la connexion

```bash
psql -U runique_user -d runique -h localhost
```

---

## Migrations (SeaORM)

### Voir les migrations existantes

```bash
cd demo-app/migration
ls -la
```

### Exécuter les migrations

Les migrations ne sont pas automatique, suivez la procédure expliqué dans le readme fournis apres cargo new votre_projet

## Troubleshooting

### ❌ "Connection refused" PostgreSQL

```bash
# Vérifier que PostgreSQL est running
sudo systemctl status postgresql

# Ou macOS:
brew services list
```

### ❌ "Permission denied" sur la base de données

```bash
# Vérifier les permissions
psql -U postgres -d runique -c "\dp"

# Réappliquer les permissions
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO runique_user;
```

### "SQLite driver est normalement activé de base modifier la base de donnée prise en charge par sea-orm dans votre cargo"

Vérifier que la feature est activée dans `Cargo.toml`:

```toml
runique = { version = "1.1.30", features = ["orm", "postgres"] }

```

### ❌ Erreur de compilation "sea_orm"

```bash
# Nettoyer et reconstruire
cargo clean
cargo build
```

---

## Development Setup

### Installer les outils recommandés

```bash
# Rust analyzer pour l'IDE
rustup component add rust-analyzer

# Linter & formatter
rustup component add clippy rustfmt

# SeaORM CLI (optionnel)
cargo install sea-orm-cli
```

### Pre-commit hooks (optionnel)

```bash
# Installer pre-commit
pip install pre-commit

# Setup hooks
pre-commit install

# Test hooks
pre-commit run --all-files
```

---

## Prochaines étapes

✅ Installation complète! Maintenant:

1. Lire la [**Architecture**](https://github.com/seb-alliot/runique/blob/main/docs/fr/02-architecture.md)
2. Créer votre premier [**Routing**](https://github.com/seb-alliot/runique/blob/main/docs/fr/04-routing.md)
3. Définir vos [**Formulaires**](https://github.com/seb-alliot/runique/blob/main/docs/fr/05-forms.md)
4. Consulter les [**Exemples**](https://github.com/seb-alliot/runique/blob/main/docs/fr/10-examples.md)
