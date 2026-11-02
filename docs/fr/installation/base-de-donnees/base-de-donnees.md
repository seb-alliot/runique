# Base de données

## Choisir le moteur (features Cargo)

| Feature | Ce qu'elle apporte |
|---|---|
| `orm` (par défaut) | SeaORM : entités, requêtes, migrations, sessions. **Aucun pilote** : le code compile et `makemigrations` fonctionne, mais l'application ne peut se connecter à aucune base. |
| `sqlite`, `postgres`, `mysql` | Le pilote du moteur. Chacun active aussi `orm`. |
| `mariadb` | Alias de `mysql`. |
| `all-databases` | Les trois pilotes à la fois, pour les outils multi-moteurs (CLI, CI). |

Une application active **un seul** moteur : en activer deux est une erreur de compilation (sauf via `all-databases`). `orm` n'a pas besoin d'être écrite.

---

## Versions de SeaORM

Runique impose une version **exacte** de SeaORM. Les `Cargo.toml` générés par `runique new` la reprennent automatiquement.

| Runique | SeaORM / sea-orm-migration |
|---|---|
| 4.0.0 | `=2.0.4` |

Ne changez pas cette version à la main : quand vous mettez Runique à jour, alignez `Cargo.toml` et `migration/Cargo.toml` sur la version de la nouvelle release. Une version différente dans le projet est refusée par Cargo à la résolution des dépendances.

---

## Changer de moteur : trois endroits à modifier ensemble

Un projet créé par `runique new` est en SQLite. Pour passer à un autre moteur, modifiez **les trois** :

**1. `Cargo.toml`** — la feature de `runique` :

```toml
[dependencies]
runique = { version = "4.0.0", features = ["postgres"] }
```

**2. `migration/Cargo.toml`** — la feature de `runique` **et** le pilote de `sea-orm-migration` :

```toml
[dependencies]
runique = { version = "4.0.0", features = ["postgres"] }
sea-orm-migration = { version = "=2.0.4", features = [
    "runtime-tokio-rustls",
    "sqlx-postgres",
] }
```

| Moteur | Feature `runique` | Feature `sea-orm-migration` |
|---|---|---|
| SQLite | `sqlite` | `sqlx-sqlite` |
| PostgreSQL | `postgres` | `sqlx-postgres` |
| MySQL | `mysql` | `sqlx-mysql` |
| MariaDB | `mariadb` | `sqlx-mysql` |

**3. `.env`** — `DB_ENGINE` et `DATABASE_URL` :

```env
DB_ENGINE=postgres
DATABASE_URL=postgres://runique_user:secure_password@localhost:5432/mon_projet
```

Si `.env` désigne un moteur qui n'est pas compilé, l'application refuse de démarrer avec « driver not enabled » ; le message indique le moteur compilé, pour aligner l'un ou l'autre.

Puis appliquez les migrations sur la nouvelle base : `runique migration up`.

### Exemples de `DATABASE_URL`

```env
DATABASE_URL="sqlite://data.db?mode=rwc"                        # fichier créé s'il n'existe pas
DATABASE_URL=postgres://user:password@localhost:5432/ma_base
DATABASE_URL=mysql://user:password@localhost:3306/ma_base       # MySQL et MariaDB
```

Un chemin SQLite relatif est lu depuis le dossier où l'application est lancée : lancez `cargo run` et `runique migration` depuis la racine du projet.

---

## PostgreSQL

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

**Windows :** [télécharger l'installeur](https://www.postgresql.org/download/windows/) et suivre l'assistant.

### 2. Créer l'utilisateur et la base de données

```sql
-- En tant qu'administrateur : psql -U postgres
CREATE USER runique_user WITH PASSWORD 'secure_password';
CREATE DATABASE mon_projet OWNER runique_user;
GRANT ALL PRIVILEGES ON DATABASE mon_projet TO runique_user;
GRANT ALL PRIVILEGES ON SCHEMA public TO runique_user;
```

### 3. Vérifier la connexion

```bash
psql -U runique_user -d mon_projet -h localhost
```

---

## Limites actuelles

- **Un moteur par compilation.** Une application ne peut pas parler à deux moteurs différents en même temps (sauf `all-databases`, réservé aux outils).
- **SQLite n'accepte qu'un écrivain à la fois** sur toute la base : sous forte charge en écriture, une requête peut attendre puis échouer (`SQLITE_BUSY`). C'est le moteur de développement ; préférez PostgreSQL en production.
- **Les migrations générées sont communes à tous les moteurs** : ce qui ne concerne qu'un moteur (type natif `CREATE TYPE` des enums sous PostgreSQL, clés étrangères d'un cycle) est choisi à l'exécution, dans le fichier. Changer de moteur ne demande pas de régénérer les migrations, mais une **base neuve** : les données ne sont pas transférées d'un moteur à l'autre.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Prérequis](/docs/fr/installation/prerequis) | Setup initial |
| [Migrations](/docs/fr/installation/migrations) | Workflow de migration |

## Retour au sommaire

- [Installation](/docs/fr/installation)
