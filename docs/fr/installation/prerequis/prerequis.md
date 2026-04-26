# Prérequis & Setup initial

## Prérequis

- **Rust 1.88+** — [Installer rustup](https://rustup.rs/)
- **PostgreSQL 12+** (ou SQLite pour le développement)

### Vérifier les versions

```bash
rustc --version    # Rust 1.88+
cargo --version    # Cargo 1.88+
```

---

## Installation

### 1. Installer Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Installer la CLI Runique

```bash
cargo install runique
```

### 3. Créer un nouveau projet

```bash
runique new mon-projet
cd mon-projet
```

### 4. Configurer l'environnement

Créer un fichier `.env` à la racine du projet :

```env
# Serveur
IP_SERVER=127.0.0.1
PORT=3000
DEBUG=true

# Base de données (PostgreSQL)
DB_ENGINE=postgres
DB_USER=postgres
DB_PASSWORD=your_password_here
DB_HOST=localhost
DB_PORT=5432
DB_NAME=mon-projet
DATABASE_URL=postgres://postgres:your_password_here@localhost:5432/mon-projet

# Templates & Static Files
TEMPLATES_DIR=templates
STATICFILES_DIRS=static
MEDIA_ROOT=media

# Sécurité
SECRET_KEY=your_secret_key_change_in_production

# Les hosts autorisés se configurent dans le builder (main.rs), pas en variable d'env
```

### 5. Lancer le serveur

```bash
cargo run
```

**Output attendu :**

```
🦀 Runique Framework opérationnel
   Serveur lancé sur http://127.0.0.1:3000
```

> `runique start` est réservé aux projets utilisant le panneau d'administration — il active le daemon admin en parallèle. Pour un projet sans admin, `cargo run` suffit.

---

## Outils recommandés

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
| [CLI Runique](/docs/fr/installation/cli) | Toutes les commandes disponibles |

## Retour au sommaire

- [Installation](/docs/fr/installation)
