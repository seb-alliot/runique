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
<summary>💡 Installation plus rapide (SQLite uniquement)</summary>

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
```
migration/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs
    ├── main.rs
    └── m20220101_000001_create_table.rs
```

---

### Étape 3 : Configurer les fichiers de migration

#### 3.1 - Modifier `migration/Cargo.toml`

Remplace le contenu de `migration/Cargo.toml` par :

```toml
[package]
name = "migration"
version = "1.1.11"
edition = "2021"

[dependencies]
sea-orm-migration = { version = "2.0.0-rc.18", features = [
    "runtime-tokio-rustls",
    "sqlx-postgres",
    "sqlx-sqlite",
] }
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"
```

>  **Important** : Utilise la version `2.0.0-rc.18` pour matcher avec Runique !

<details>
<summary> Personnaliser les features selon ta base de données</summary>

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

> 💡 **Note** : Cette version assure la compatibilité avec Runique v1.0.855+

---

### Étape 4 : Créer la migration de la table users
### Modifier create_table.rs initialement créé ou supprimer et créer une nouvelle migration
### Attention, si tu en crée une nouvelle, supprime l'ancienne et modifie le numéro de timestamp dans lib.rs

```bash
# Créer le fichier de migration
sea-orm-cli migrate generate create_users_table
```

Cela génère un fichier `migration/src/m{TIMESTAMP}_create_users_table.rs`

---

### Étape 5 : Compléter le fichier de migration

Modifier le fichier choisis et remplace **tout son contenu** par :

```rust
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Users::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Users::Username)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Users::Email)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Users::Password).string().not_null())
                    .col(ColumnDef::new(Users::Age).integer().not_null())
                    .col(
                        ColumnDef::new(Users::CreatedAt)
                            .timestamp()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    Username,
    Email,
    Password,
    Age,
    CreatedAt,
}
```

---

### Étape 6 : Appliquer la migration

```bash
# Applique la migration
sea-orm-cli migrate up
```

Tu devrais voir :
```
Applying migration 'm{TIMESTAMP}_create_users_table'
Migration 'm{TIMESTAMP}_create_users_table' has been applied
```

 **La table `users` est maintenant créée !**

---

### Étape 7 : Lancer l'application

```bash
cargo run
```

🎉 Ouvre ton navigateur sur **http://127.0.0.1:3000**

---

##  Structure finale du projet

```
{}/
├── migration/                           ← Nouveau dossier créé
│   ├── Cargo.toml                       ← Configuré à l'étape 3.1
│   └── src/
│       ├── lib.rs
│       ├── main.rs                      ← Modifié à l'étape 3.2
│       └── m{TIMESTAMP}_create_users_table.rs  ← Créé à l'étape 5
├── src/
│   ├── models/
│   │   ├── mod.rs
│   │   └── users.rs                     ←  Correspond à la migration
│   ├── static/css/
│   ├── forms.rs
│   ├── main.rs
│   ├── url.rs
│   └── views.rs
├── templates/
│   ├── index.html
│   ├── about/
│   └── profile/
├── .env
├── Cargo.toml
└── README.md
```

---

##  Commandes utiles

```bash
# Voir le statut des migrations
sea-orm-cli migrate status

# Rollback de la dernière migration
sea-orm-cli migrate down

# Créer une nouvelle migration
sea-orm-cli migrate generate nom_de_la_migration

# Réinitialiser la base de données (supprime toutes les données)
sea-orm-cli migrate down -n 999
sea-orm-cli migrate up
```

---

##  Dépannage

###  Erreur : "table users doesn't exist"
→ Tu n'as pas appliqué la migration. Exécute :
```bash
sea-orm-cli migrate up
```

###  Erreur : "sea-orm-cli: command not found"
→ Installe sea-orm-cli :
```bash
cargo install sea-orm-cli
```

###  Erreur : "version mismatch"
→ Vérifie que `migration/Cargo.toml` utilise bien `sea-orm-migration = "2.0.0-rc.18"`

###  Erreur : "no such table: users"
→ Vérifie que :
1. La migration a bien été appliquée (`sea-orm-cli migrate status`)
2. Le chemin `DATABASE_URL` dans `.env` est correct
3. Tu lances `cargo run` depuis la racine du projet (pas depuis `migration/`)

###  Port 3000 déjà utilisé
→ Change le port dans `.env` :
```env
PORT=8080
```

###  Erreur : "workspace" lors de `sea-orm-cli migrate up`
→ Vérifie que `migration/Cargo.toml` contient bien la section `[workspace]`

---

## 🎯 Fonctionnalités incluses

-  Inscription utilisateur avec validation
-  Recherche d'utilisateur par nom
-  Protection CSRF automatique
-  Flash messages (success, error, info, warning)
-  Thème sombre moderne et responsive
-  Templates Tera avec héritage

---

##  Documentation

- [Runique Framework](https://docs.rs/runique)
- [SeaORM Migrations](https://www.sea-ql.org/SeaORM/docs/migration/setting-up-migration/)
- [Tera Templates](https://keats.github.io/tera/)

---

Généré par **Runique CLI v1.0.85** 🦀