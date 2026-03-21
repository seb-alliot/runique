# Troubleshooting

## "Connection refused" PostgreSQL

```bash
# Check that PostgreSQL is running
sudo systemctl status postgresql

# Or on macOS:
brew services list
```

---

## "Permission denied" on the Database

```bash
# Check permissions
psql -U postgres -d runique -c "\dp"

# Reapply permissions
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO runique_user;
```

---

## SQLite Feature Not Enabled

Make sure the feature is enabled in `Cargo.toml`:

```toml
runique = { version = "1.1.52", features = ["orm", "postgres"] }
```

---

## Compilation Error "sea_orm"

```bash
# Clean and rebuild
cargo clean
cargo build
```

---

## Pre-commit Hooks (optional)

```bash
# Install pre-commit
pip install pre-commit

# Setup hooks
pre-commit install

# Test hooks
pre-commit run --all-files
```

---

## See also

| Section | Description |
| --- | --- |
| [Prerequisites](https://github.com/seb-alliot/runique/blob/main/docs/en/installation/prerequisites/prerequisites.md) | Initial setup |
| [Database](https://github.com/seb-alliot/runique/blob/main/docs/en/installation/database/database.md) | SQLite, PostgreSQL |

## Back to summary

- [Installation](https://github.com/seb-alliot/runique/blob/main/docs/en/installation/01-installation.md)
