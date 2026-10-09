🌍 **Languages**: [English](TESTING.md) | [Français](TESTING.fr.md)

# Testing Runique and running demo-app locally

Two separate things live in this repository:

| What | Database | Docker |
| --- | --- | --- |
| The test suite (`runique/`) | SQLite in memory, plus Postgres and MariaDB when available | Optional |
| The demo site (`demo-app/`, runique.io) | **Postgres only** | Recommended |

---

## Prerequisites

| Tool | For | Install |
| --- | --- | --- |
| Rust 1.94+ | Everything | [rustup](https://rustup.rs) |
| Docker | Postgres and MariaDB (optional for the tests, recommended for demo-app) | [docs.docker.com](https://docs.docker.com/get-docker/) |
| `sea-orm-cli` | demo-app: `runique migration up` delegates to it | `cargo install sea-orm-cli` |
| `runique` CLI | demo-app: migrations and the admin account | `cargo install --path runique --features postgres` |

The `runique` CLI is installed from the workspace, to follow the framework's version: run the
command at the root of the repository, where `--path runique` points to the framework's folder. The
`postgres` feature is required: built on its own, the CLI enables no database driver. Reinstall
it after each update of the workspace.

---

## 1. Run the test suite

### SQLite only — nothing to install

Every test that needs a database gets a fresh in-memory SQLite one. The Postgres and MariaDB
tests are skipped, not failed, when no URL is configured.

```bash
cd runique
cargo test --features orm,sqlite,postgres,mysql,mariadb,all-databases,acme
```

This is the exact feature set the CI uses.

### With Postgres and MariaDB

The root `docker-compose.yml` starts both engines for the tests:

```bash
docker compose up -d
```

Then create `runique/.env.test`:

```env
DATABASE_URL_PG=postgres://runique:runique_test@localhost:5433/runique_test
DATABASE_URL_MARIADB=mysql://runique:runique_test@localhost:3307/runique_test
```

Each clone of the repository gets its own database, created on first use (named after the
clone's path), so two checkouts don't overwrite each other's tables. Don't run two `cargo test`
at once from the **same** checkout: they would share those databases.

### Primary key variants

The CI runs the suite three times, once per primary key type:

```bash
cd runique
cargo test --features big-pk,all-databases
cargo test --features pk-uuid,all-databases
```

`cargo clippy --features big-pk` (or `pk-uuid`) alone enables no database engine: set
`DB_ENGINE=postgres` for it, as the CI does, or the model macros can't pick an engine.

---

## 2. Run demo-app locally

demo-app only runs on Postgres: its `Cargo.toml` enables the `postgres` feature, and its
`seed.sql`, replayed at every start, uses Postgres types (`CREATE TYPE … AS ENUM`, sequences).

**1. Start Postgres and create the demo database**

```bash
docker compose up -d postgres
docker compose exec postgres createdb -U runique runique_demo
```

A Postgres installed on the machine works too: point `DATABASE_URL` at it.

**2. Create `demo-app/.env`**

```env
DEBUG=true
DB_ENGINE=postgres
DATABASE_URL=postgres://runique:runique_test@localhost:5433/runique_demo
SECRET_KEY=change-me-with-32-random-characters-at-least
```

Optional keys:

| Key | For |
| --- | --- |
| `SMTP_HOST`, `SMTP_PORT`, `SMTP_USER`, `SMTP_PASS`, `SMTP_FROM`, `SMTP_STARTTLS` | Sending emails (account activation, password reset). Without them, the admin shows the reset link on screen instead |
| `GROQ_API_KEY` | The AI feedback on course exercises (`/cours/…/exercice`) |
| `RUNIQUE_MAX_UPLOAD_MB` | Uploads above 2 MB |

**3. Apply the migrations**

```bash
cd demo-app
runique migration up
```

They create the framework's tables (accounts, sessions, groups) and the demo's own.

**4. Run**

```bash
cargo run -p demo-app
```

The site answers on `http://127.0.0.1:3000`. The content (docs, courses, examples) is loaded
from `seed.sql` at every start; changing it needs no migration.

**5. Admin access**

The site holds the first terminal: open a second one, at the root of the repository. The CLI
runs on your machine, not in Docker: it reads `DATABASE_URL` from `demo-app/.env` and reaches the
Postgres container through port 5433, so the container must be up (`docker compose ps`).

```bash
cd demo-app
runique create-superuser
```

The admin is at `http://127.0.0.1:3000/prefix-test/admin-runique/`.

`runique start` regenerates `src/admins/` from `src/admin.rs` before running the site: needed
only after changing the `admin!{}` declarations.
