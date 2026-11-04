# Migrations

## Le crate `migration`

Les migrations vivent dans un crate Rust à part, `migration/`, qui fait partie du workspace du projet :

```
migration/
├── Cargo.toml        runique + sea-orm-migration, même moteur que l'application
└── src/
    ├── main.rs       lance la CLI de SeaORM (up, down, status)
    ├── lib.rs        la liste des migrations, tenue à jour par makemigrations
    ├── m<date>_*.rs  les migrations générées
    └── snapshots/    l'état du schéma déjà migré, modèle par modèle
```

**Vous n'avez pas à le créer** : `runique new` l'écrit avec le projet, et `runique makemigrations` le crée s'il manque (projet non créé par `runique new`), avec le moteur de `DB_ENGINE`. Il n'y a rien à écrire à la main dans `lib.rs`.

### Le `Cargo.toml` attendu

Runique l'écrit une seule fois, à la création ; ensuite il est à vous et Runique n'y touche plus. Voici le fichier tel qu'il est généré (exemple Postgres) :

```toml
[package]
name = "migration"
version = "0.1.0"
edition = "2024"
publish = false

[lib]
name = "migration"
path = "src/lib.rs"

[dependencies]
runique = { version = "4.0.0", features = ["postgres"] }
sea-orm-migration = { version = "=2.0.4", features = [
    "runtime-tokio-rustls",
    "sqlx-postgres",
] }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
async-trait = "0.1"
```

Ce qui est obligatoire :

- **`runique`** avec la même feature de moteur que l'application (`sqlite`, `postgres`, `mysql` ou `mariadb`) : il fournit les tables du framework.
- **`sea-orm-migration`** à la **même version exacte** que celle de Runique, avec `runtime-tokio-rustls` et le pilote du moteur : `sqlx-sqlite`, `sqlx-postgres`, ou `sqlx-mysql` (MySQL comme MariaDB).
- **Les features par défaut de `sea-orm-migration`** : elles apportent `cli::run_cli`, appelé par `src/main.rs`. Avec `default-features = false`, ajoutez `"cli"` à la liste, sinon le crate ne compile plus.
- **`tokio`** avec `macros` et `rt-multi-thread` (`#[tokio::main]` de `main.rs`), et **`async-trait`** (les migrations générées).
- **`[lib] name = "migration"`** : `main.rs` appelle `migration::Migrator`.

---

## Workflow en deux étapes

### 1. Générer les fichiers de migration

```bash
runique makemigrations
```

`makemigrations` lit vos modèles dans `src/entities/`, les compare aux snapshots, et écrit ce qui a changé :

| Passe | Source | Résultat |
| --- | --- | --- |
| 1 — Tables de l'application | Blocs `model!{}` | `CREATE TABLE` / `ALTER TABLE` |
| 2 — Extensions du framework | Blocs `extend!{}` | `ALTER TABLE ADD / DROP COLUMN` sur une table `eihwaz_*` |
| 3 — Tables du framework | Automatique | Migrations `eihwaz_*` placées en tête de `lib.rs` |

La commande se termine par le nombre de fichiers écrits, ou « No changes detected » si rien n'a changé, puis rappelle la commande suivante.

### 2. Appliquer les migrations

```bash
runique migration up
```

### Annuler ou consulter

```bash
runique migration down -n 1    # annule la dernière migration (son down() s'exécute)
runique migration status       # liste les migrations et leur état
```

Ces commandes lancent le crate `migration` (`cargo run`), qui lit `DATABASE_URL` dans `.env` : lancez-les depuis la racine du projet. `sea-orm-cli` n'est pas nécessaire, et la version de SeaORM utilisée est celle de `migration/Cargo.toml`, alignée sur Runique.

> `runique makemigrations` est le seul outil à utiliser pour **générer** les migrations. N'utilisez pas `sea-orm-cli migrate generate` : Runique tient ses propres snapshots et l'ordre de `lib.rs`.

---

## Changements destructifs — `--force`

Avant d'écrire quoi que ce soit, `makemigrations` détecte les changements destructifs et **refuse** de générer la migration, sauf avec `--force` :

- `DROP COLUMN` (perte de données)
- changement de type d'une colonne (voir les limites plus bas)
- passage `nullable` → obligatoire (nécessite une valeur par défaut ou un remplissage préalable)
- réduction de longueur (`max_length` plus petit) : des valeurs plus longues seraient tronquées ou refusées
- suppression d'une clé étrangère (risque d'enregistrements orphelins)
- ajout d'une FK `ON DELETE CASCADE` sur des données existantes

```bash
runique makemigrations --force
```

Le contrôle couvre les tables de l'application (`model!{}`) comme les extensions des tables du framework (`extend!{}`).

---

## Clés étrangères et moteur cible

Les fichiers générés sont **communs à tous les moteurs** : ce qui ne concerne qu'un moteur est choisi à l'exécution, dans le fichier.

- Les FK sont déclarées **dans le `CREATE TABLE`**, et les tables référencées sont créées avant celles qui les référencent.
- **Cycle** (deux nouvelles tables qui se référencent l'une l'autre) : sous SQLite, la clé reste dans le `CREATE TABLE` ; sous PostgreSQL / MySQL / MariaDB, la clé qui ferme le cycle est ajoutée par un `ALTER TABLE` une fois la table visée créée.
- Chaque colonne de `belongs_to` reçoit un index `idx_<table>_<colonne>`.
- Les enums deviennent un `CREATE TYPE … AS ENUM` sous PostgreSQL, un `VARCHAR` ailleurs.

Changer de moteur ne demande donc pas de régénérer les migrations.

---

## Tables du framework — fournies automatiquement

Cinq migrations du framework sont placées en tête de `lib.rs`, sans rien à déclarer :

| Migration | Crée | Ordre |
| --- | --- | --- |
| `EihwazUsersMigration` | `eihwaz_users` (id, username, email, password, is_active, is_superuser, is_dirigeant, is_admin, activated_at, created_at, updated_at) | 1er |
| `EihwazSessionsMigration` | `eihwaz_sessions` (sessions authentifiées persistées) | 2e |
| `AdminTableMigration` | `eihwaz_groupes`, `eihwaz_groupes_droits`, `eihwaz_users_groupes` | 3e |
| `EihwazHistoryMigration` | `eihwaz_history` (historique des modifications de l'admin) | 4e |
| `EihwazResetTokensMigration` | `eihwaz_reset_tokens` (jetons de réinitialisation hashés, un seul actif par compte) | 5e |

> Ne déclarez pas `eihwaz_users` dans vos entités : pour lui ajouter des colonnes, utilisez `extend!{}`.

---

## Étendre une table du framework — `extend!{}`

```rust
use runique::prelude::*;

extend! {
    table: "eihwaz_users",
    fields: {
        avatar: image [upload_to: "avatars/"],
        bio: textarea,
        website: url [required],
    }
}
```

`makemigrations` génère les `ALTER TABLE ADD COLUMN` correspondants. Le snapshot de l'extension est stocké dans `migration/src/snapshots/runique/<table>.rs`.

**Tables extensibles :** `eihwaz_users` et `eihwaz_groupes`, et elles seules. Étendre une autre table est une erreur de **compilation**.

---

## Génération atomique

Les changements des modèles et des blocs `extend!{}` sont d'abord planifiés en mémoire, validés (contrôle destructif, longueur des identifiants), puis écrits en un seul lot. En cas d'erreur d'écriture, les fichiers déjà écrits sont annulés, et les snapshots comme `lib.rs` sont restaurés.

---

## Limites actuelles

- **Le changement de type d'une colonne n'est pas encore migré.** `makemigrations` le bloque ; avec `--force`, la migration écrite ne contient qu'un commentaire d'avertissement, aucun `ALTER`, alors que le snapshot enregistre déjà le nouveau type. La console l'affiche en jaune. Avant `runique migration up`, écrivez à la main le `up()` et le `down()` de cette migration (`alter_table` / `modify_column` de SeaORM), ou ajoutez une nouvelle colonne, recopiez les données, puis supprimez l'ancienne. Le passage vers ou depuis un enum, lui, est migré.
- **La suppression d'une table n'est pas détectée.** Retirer un modèle ne génère rien : `makemigrations` répond « aucun changement », et la table reste en base. Pour la supprimer, écrivez une migration à la main (`drop_table`, à enregistrer dans `lib.rs`), retirez son snapshot de `migration/src/snapshots/`, et enlevez d'abord tout `belongs_to` qui la vise. Prévu pour la 4.1.
- **La génération n'est pas encore protégée contre un arrêt brutal** (coupure, `kill -9`) en plein milieu de l'écriture : une erreur ordinaire est annulée, une interruption peut laisser des fichiers partiellement mis à jour. Le dossier `migration/` étant versionné, `git` permet de le restaurer.
- **Ne copiez pas les snapshots d'un autre projet sans ses migrations** : un snapshot signifie « déjà migré ». `makemigrations` ne génère alors rien pour ces modèles, alors qu'aucune migration ne crée leurs tables.
- **Les identifiants SQL sont limités à 64 caractères sous MySQL / MariaDB** (noms d'index et de clés étrangères générés à partir des noms de tables et de colonnes) : `makemigrations` refuse un identifiant trop long plutôt que de le tronquer ; raccourcissez les noms en cause.
- **`runique migration up` n'applique une migration de façon atomique que sous Postgres.** sea-orm-migration n'enveloppe chaque migration (et son inscription dans `seaql_migrations`) dans une transaction que sur ce moteur. Sous SQLite et MySQL / MariaDB, une migration qui échoue en cours de route peut rester à moitié appliquée ; sous MySQL / MariaDB, aucune transaction n'y changerait rien (chaque instruction DDL valide d'elle-même). En développement sous SQLite, supprimez le fichier de base et relancez `runique migration up` ; ailleurs, corrigez la base à la main avant de relancer.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Base de données](/docs/fr/installation/base-de-donnees) | SQLite, PostgreSQL, MySQL / MariaDB |
| [CLI Runique](/docs/fr/installation/cli) | Commandes disponibles |
| [Modèles](/docs/fr/model) | DSL `model!{}` |
| [Fonctionnement de makemigrations](/docs/fr/architecture/makemigrations) | Snapshots, diff, génération |

## Retour au sommaire

- [Installation](/docs/fr/installation)
