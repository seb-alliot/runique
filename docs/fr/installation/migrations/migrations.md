# Migrations (SeaORM)

## Workflow en deux étapes

### 1. Générer les fichiers de migration

`runique makemigrations` lit vos entités déclarées dans `src/entities` et génère les fichiers de migration correspondants :

```bash
runique makemigrations --entities src/entities --migrations migration/src
```

**Ce que `makemigrations` fait en une seule commande :**

| Passe | Source | Résultat |
| --- | --- | --- |
| 1 — Tables app | Blocs `model!{}` dans `src/entities` | `CREATE TABLE` / `ALTER TABLE` |
| 2 — Extensions framework | Blocs `extend!{}` dans `src/entities` | `ALTER TABLE ADD/DROP COLUMN` |
| 3 — Positionnement admin | `RUNIQUE_USER_TABLE` dans `.env` | Ordre garanti dans `lib.rs` |

### 2. Appliquer les migrations

```bash
sea-orm-cli migrate up --migration-dir migration/src
```

---

## Tables framework — fournies automatiquement

Runique injecte automatiquement deux migrations dans votre `lib.rs` sans que vous ayez à les définir :

| Migration | Crée | Ordre |
| --- | --- | --- |
| `EihwazUsersMigration` | `eihwaz_users` (id, username, email, password, is_active, is_staff, is_superuser, created_at, updated_at) | 1er |
| `AdminTableMigration` | `eihwaz_groupes`, `eihwaz_groupes_droits`, `eihwaz_users_groupes` | 2e |

> Vous n'avez pas besoin de déclarer `eihwaz_users` dans vos entités.

---

## Étendre une table framework — `extend!{}`

Pour ajouter des colonnes à une table framework, utilisez `extend!{}` dans un fichier d'entité :

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

`makemigrations` détecte ces blocs et génère les `ALTER TABLE ADD COLUMN` correspondants.
Le snapshot d'extension est stocké dans `migration/src/snapshots/runique/eihwaz_users.rs`.

**Tables extensibles :**

```text
eihwaz_users · eihwaz_groupes · eihwaz_droits
eihwaz_sessions · eihwaz_users_groupes · eihwaz_groupes_droits
```

> Étendre une table inconnue déclenche une erreur à la **compilation**.

---

## Utiliser une table user personnalisée

Si vous préférez gérer votre propre table utilisateur, déclarez-la dans `.env` :

```env
RUNIQUE_USER_TABLE=ma_table_users
```

`makemigrations` positionnera alors `AdminTableMigration` juste après la migration de `ma_table_users`.
La FK dans `eihwaz_users_groupes` ciblera automatiquement `ma_table_users`.

> Par défaut (`RUNIQUE_USER_TABLE` absent) : `eihwaz_users` est utilisée.

---

## Autres commandes de migration

```bash
sea-orm-cli migrate down --migration-dir migration/src   # Annuler la dernière migration
sea-orm-cli migrate status --migration-dir migration/src # Voir l'état des migrations
```

---

## Wrapper Runique — rollback atomique (avancé)

```bash
runique migration down --migrations migration/src <fichier>
runique migration down --migrations migration/src --batch <timestamp>
runique migration status --migrations migration/src
```

> Ces commandes utilisent le système de batch Runique avec rollback transactionnel.
> Préférer `sea-orm-cli` pour le workflow normal.

---

> `runique makemigrations` est le seul outil à utiliser pour **générer** les fichiers de migration.
> Ne pas utiliser `sea-orm-cli migrate generate` : Runique maintient ses propres snapshots et ordre chronologique.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Base de données](/docs/fr/installation/base-de-donnees) | SQLite, PostgreSQL, MariaDB |
| [CLI Runique](/docs/fr/installation/cli) | Commandes disponibles |
| [Modèles](/docs/fr/model) | DSL `model!{}` |

## Retour au sommaire

- [Installation](/docs/fr/installation)
