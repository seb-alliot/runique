# Contributing to Runique

Thank you for your interest in contributing!

## Documentation

All contribution guidelines, conventions, and setup instructions are available in the documentation:

- **English** → [docs/en/](docs/en/)
- **Français** → [docs/fr/](docs/fr/)

## Quick Start

```bash
git clone https://github.com/seb-alliot/runique
cd runique
cargo test --tests
```

Without Docker, the database tests run on SQLite and the Postgres / MariaDB ones are skipped. To run those too, and to run demo-app locally, see [TESTING.md](TESTING.md).
