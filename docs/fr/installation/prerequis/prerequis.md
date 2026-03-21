# Prérequis & Setup initial

## Prérequis

- **Rust 1.85+** — [Installer rustup](https://rustup.rs/)
- **PostgreSQL 12+** (ou SQLite pour dev)
- **Git**

### Vérifier les versions

```bash
rustc --version    # Rust 1.85+
cargo --version    # Cargo 1.85+
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

Créer un fichier `.env` dans le répertoire `demo-app/` :

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
```

### 3. Compiler le projet

```bash
cargo build

# Ou pour le mode release (optimisé) :
cargo build --release
```

### 4. Lancer le serveur

```bash
cargo run -p demo-app
```

**Output attendu :**

```
🦀 Runique Framework opérationnel
   Serveur lancé sur http://127.0.0.1:3000
```

### Outils recommandés

```bash
# Rust analyzer pour l'IDE
rustup component add rust-analyzer

# Linter & formatter
rustup component add clippy rustfmt

# SeaORM CLI (requis pour les migrations)
cargo install sea-orm-cli
```

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Base de données](/docs/fr/installation/base-de-donnees) | SQLite, PostgreSQL |
| [Migrations](/docs/fr/installation/migrations) | Workflow de migration |

## Retour au sommaire

- [Installation](/docs/fr/installation)
