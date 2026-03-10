# Migrations (SeaORM)

## Workflow en deux étapes

### 1. Générer les fichiers de migration

`runique makemigrations` lit vos entités déclarées dans `src/entities` et génère les fichiers de migration correspondants :

```bash
runique makemigrations --entities src/entities --migrations migration/src
```

### 2. Appliquer les migrations

Via le CLI SeaORM (recommandé) :

```bash
sea-orm-cli migrate up --migration-dir migration/src
```

---

## Autres commandes de migration

```bash
sea-orm-cli migrate down --migration-dir migration/src   # Annuler la dernière migration
sea-orm-cli migrate status --migration-dir migration/src # Voir l’état des migrations
```

---

## Wrapper Runique (avancé)

Les commandes suivantes existent dans le CLI Runique mais **contournent le suivi chronologique de SeaORM** :

```bash
runique migration up --migrations migration/src
runique migration down --migrations migration/src
runique migration status --migrations migration/src
```

> ⚠️ Ces commandes ne mettent pas à jour la table de suivi de SeaORM. Utiliser uniquement en connaissance de cause — préférer `sea-orm-cli` pour le workflow normal.

---

> ⚠️ `runique makemigrations` est le seul outil à utiliser pour **générer** les fichiers de migration. Ne pas utiliser `sea-orm-cli migrate generate` : le système Runique maintient un ordre chronologique et des snapshots que la CLI SeaORM ne connaît pas.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Base de données](https://github.com/seb-alliot/runique/blob/main/docs/fr/installation/base-de-donnees/base-de-donnees.md) | SQLite, PostgreSQL |
| [CLI Runique](https://github.com/seb-alliot/runique/blob/main/docs/fr/installation/cli/cli.md) | Commandes disponibles |

## Retour au sommaire

- [Installation](https://github.com/seb-alliot/runique/blob/main/docs/fr/installation/01-installation.md)
