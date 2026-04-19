# Troubleshooting

## ❌ "Connection refused" PostgreSQL

```bash
# Vérifier que PostgreSQL est running
sudo systemctl status postgresql

# Ou macOS :
brew services list
```

---

## ❌ "Permission denied" sur la base de données

```bash
# Vérifier les permissions
psql -U postgres -d runique -c "\dp"

# Réappliquer les permissions
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO runique_user;
```

---

## ❌ Feature SQLite non activée

Vérifier que la feature est activée dans `Cargo.toml` :

```toml
runique = { version = "2.1.0", features = ["orm", "postgres"] }
```

---

## ❌ Erreur de compilation "sea_orm"

```bash
# Nettoyer et reconstruire
cargo clean
cargo build
```

---

## Pre-commit hooks (optionnel)

```bash
# Installer pre-commit
pip install pre-commit

# Setup hooks
pre-commit install

# Test hooks
pre-commit run --all-files
```

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Prérequis](/docs/fr/installation/prerequis) | Setup initial |
| [Base de données](/docs/fr/installation/base-de-donnees) | SQLite, PostgreSQL |

## Retour au sommaire

- [Installation](/docs/fr/installation)
