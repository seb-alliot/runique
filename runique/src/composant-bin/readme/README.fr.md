# Runique — Django-inspired Rust Framework

---

## Démarrage rapide

### Étape 1 : Installation de sea-orm-cli

```bash
# Installation globale (recommandé)
cargo install sea-orm-cli

#  L'installation prend environ 5-10 minutes, c'est normal !
```

<details>
<summary>Installation plus rapide (SQLite uniquement)</summary>

Si tu utilises uniquement SQLite, tu peux installer une version allégée :

```bash
cargo install sea-orm-cli --no-default-features \
    --features cli,runtime-tokio-rustls,sqlite
```

Cela prend quelques minutes.

</details>

---

### Étape 2 : Initialiser les migrations

```bash
# Créer le dossier de migrations
sea-orm-cli migrate init
```

Cela crée la structure suivante :

```text
migration/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs
    ├── main.rs
    └── m20220101_000001_create_table.rs
```

Supprime `m20220101_000001_create_table.rs` — `runique makemigrations` générera les vrais fichiers.

---

### Étape 3 : Configurer les fichiers de migration

#### 3.1 - Modifier `migration/Cargo.toml`

Remplace le contenu de `migration/Cargo.toml` par :

```toml
[package]
name = "migration"
version = "0.1.0"
edition = "2024"

[dependencies]
sea-orm-migration = { version = "2.0.0-rc.18", features = [
    "runtime-tokio-rustls",
    "sqlx-postgres",
    "sqlx-sqlite",
] }
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"
```

> **Important** : Utilise la version `2.0.0-rc.18` pour matcher avec Runique !

<details>
<summary>Personnaliser les features selon ta base de données</summary>

**Pour PostgreSQL uniquement** :

```toml
sea-orm-migration = { version = "2.0.0-rc.18", features = [
    "runtime-tokio-rustls",
    "sqlx-postgres"
] }
```

**Pour SQLite uniquement** :

```toml
sea-orm-migration = { version = "2.0.0-rc.18", features = [
    "runtime-tokio-rustls",
    "sqlx-sqlite"
] }
```

**Pour MySQL uniquement** :

```toml
sea-orm-migration = { version = "2.0.0-rc.18", features = [
    "runtime-tokio-rustls",
    "sqlx-mysql"
] }
```

</details>

#### 3.2 - Modifier `migration/src/main.rs`

Remplace le contenu de `migration/src/main.rs` par :

```rust
use sea_orm_migration::prelude::*;

#[tokio::main]
async fn main() {
    cli::run_cli(migration::Migrator).await;
}
```

---

### Étape 4 : Définir tes entités

Tes entités se trouvent dans `src/entities/`. Chaque fichier utilise la macro `model!` :

```rust
// src/entities/users.rs
use runique::prelude::*;

model! {
    Users,
    table: "users",
    pk: id => i32,
    fields: {
        username: String [required, max_len(150), unique],
        email: String [required, unique],
        password: String [required, max_len(128)],
        created_at: datetime [auto_now],
        updated_at: datetime [auto_now_update],
    }
}
```

Runique lit ces fichiers pour générer les migrations automatiquement.

---

### Étape 5 : Générer les migrations

```bash
runique makemigrations
```

Cela scanne `src/entities/`, compare avec les snapshots, et génère :

```text
migration/src/
├── lib.rs                                      ← mis à jour automatiquement
├── main.rs
├── m{TIMESTAMP}_create_users_table.rs          ← généré
└── snapshot/
    └── users.rs                                ← snapshot du schéma
```

> Pour faire évoluer le schéma (ALTER), modifie ton entité et relance `runique makemigrations`.
> Utilise `--force` pour ignorer la confirmation sur les changements destructifs.
>
> ⚠️ **Avertissement**
> La commande `makemigrations` permet de générer les tables SeaORM tout en
> respectant la chronologie du système de migrations.
> Pour garantir la cohérence du suivi des migrations, utilisez uniquement
> la CLI de SeaORM pour appliquer ou gérer les migrations.
> L'utilisation des commandes peut entraîner une désynchronisation.

---

### Étape 6 : Appliquer la migration

```bash
sea-orm-cli migrate up
```

Tu devrais voir :

```text
Applying migration 'm{TIMESTAMP}_create_users_table'
Migration 'm{TIMESTAMP}_create_users_table' has been applied
```

**La table `users` est maintenant créée !**

---

### Étape 7 : Lancer l'application

```bash
cargo run
```

Ouvre ton navigateur sur **`http://127.0.0.1:3000`**

---

## Faire évoluer le schéma

Quand tu dois ajouter, supprimer ou modifier une colonne :

1. Modifie le `model!` dans `src/entities/ton_modele.rs`
2. Lance `runique makemigrations` — détecte le diff, génère une migration ALTER
3. Lance `sea-orm-cli migrate up` — l'applique

Pour les changements destructifs (changement de type, nullable → not null) :

```bash
runique makemigrations --force
```

---

## Structure finale du projet

```text
{}/
├── migration/                           <- Créé à l'étape 2
│   ├── Cargo.toml                       <- Configuré à l'étape 3.1
│   └── src/
│       ├── lib.rs                       <- Géré automatiquement par makemigrations
│       ├── main.rs                      <- Configuré à l'étape 3.2
│       ├── m{TIMESTAMP}_create_users_table.rs
│       └── snapshot/
│           └── users.rs
├── src/
│   ├── entities/
│   │   ├── mod.rs
│   │   └── users.rs                     <- Définit le schéma model!
│   ├── formulaire/
│   ├── main.rs
│   ├── url.rs
│   └── views.rs
├── templates/
├── static/
├── .env
├── Cargo.toml
└── README.md
```

---

## Commandes utiles

```bash
# Générer les migrations depuis les entités
runique makemigrations

# Forcer la génération (ignorer la confirmation sur changements destructifs)
runique makemigrations --force

# Chemins personnalisés
runique makemigrations --entities src/entities --migrations migration/src

# Appliquer les migrations
sea-orm-cli migrate up

# Voir le statut des migrations
sea-orm-cli migrate status

# Lister les rollbacks disponibles
runique migration status

# Rollback d'une migration spécifique
runique migration down --files users/20240101_120000

# Rollback d'un batch complet
runique migration down --batch 20240101_120000

# Réinitialiser la base de données (supprime toutes les données)
sea-orm-cli migrate down -n 999
sea-orm-cli migrate up
```

---

## Dépannage

### Erreur : "table users doesn't exist"

Tu n'as pas appliqué la migration. Exécute :

```bash
sea-orm-cli migrate up
```

### Erreur : "sea-orm-cli: command not found"

Installe sea-orm-cli :

```bash
cargo install sea-orm-cli
```

### Erreur : "version mismatch"

Vérifie que `migration/Cargo.toml` utilise bien `sea-orm-migration = "2.0.0-rc.18"`

### Erreur : "no such table: users"

Vérifie que :

1. La migration a bien été appliquée (`sea-orm-cli migrate status`)
2. Le chemin `DATABASE_URL` dans `.env` est correct
3. Tu lances `cargo run` depuis la racine du projet (pas depuis `migration/`)

### Port 3000 déjà utilisé

Change le port dans `.env` :

```env
PORT=8080
```

### Erreur : "workspace" lors de `sea-orm-cli migrate up`

Vérifie que `migration/Cargo.toml` contient bien la section `[workspace]`

---

## Fonctionnalités incluses

- Inscription utilisateur avec validation
- Recherche d'utilisateur par nom
- Protection CSRF automatique
- Flash messages (success, error, info, warning)
- Thème sombre moderne et responsive
- Templates Tera avec héritage

---

## Documentation

- [Runique Framework](https://docs.rs/runique)
- [SeaORM Migrations](https://www.sea-ql.org/SeaORM/docs/migration/setting-up-migration/)
- [Tera Templates](https://keats.github.io/tera/)

---

Généré par **Runique CLI v1.1.54**
