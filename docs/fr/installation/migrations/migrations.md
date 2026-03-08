# Migrations (SeaORM)

## Workflow en deux étapes

### 1. Générer les fichiers de migration

`runique makemigrations` lit vos entités déclarées dans `src/entities` et génère les fichiers de migration correspondants :

```bash
runique makemigrations --entities src/entities --migrations migration/src
```

### 2. Appliquer les migrations

Via le CLI SeaORM :

```bash
sea-orm-cli migrate up --migration-dir migration/src
```

Ou via le wrapper Runique :

```bash
runique migration up --migrations migration/src
```

---

## Autres commandes de migration

```bash
runique migration down --migrations migration/src    # Annuler la dernière migration
runique migration status --migrations migration/src  # Voir l'état des migrations
```

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Base de données](https://github.com/seb-alliot/runique/blob/main/docs/fr/installation/base-de-donnees/base-de-donnees.md) | SQLite, PostgreSQL |
| [CLI Runique](https://github.com/seb-alliot/runique/blob/main/docs/fr/installation/cli/cli.md) | Commandes disponibles |

## Retour au sommaire

- [Installation](https://github.com/seb-alliot/runique/blob/main/docs/fr/installation/01-installation.md)
