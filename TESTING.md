🌍 **Languages**: [English](TESTING.md) | [Français](TESTING.fr.md)

# Testing Runique and running demo-app locally

Two separate things live in this repository:

| What | Database | Docker |
| --- | --- | --- |
| The test suite (`runique/`) | SQLite in memory, plus Postgres and MariaDB when available | Optional |
| The demo site (`demo-app/`, runique.io) | **Postgres only** | Recommended |

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
docker compose -f docker-compose.yml up -d
```

> Pass `-f docker-compose.yml`: the root also holds a `compose.yml` (the deployment of the
> site), and a bare `docker compose up` picks that one — it starts no database.

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
docker compose -f docker-compose.yml up -d postgres
docker compose -f docker-compose.yml exec postgres createdb -U runique runique_demo
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
sea-orm-cli migrate up
```

They create the framework's tables (accounts, sessions, groups) and the demo's own.

**4. Run**

```bash
cargo run -p demo-app
```

The site answers on `http://127.0.0.1:3000`. The content (docs, courses, examples) is loaded
from `seed.sql` at every start; changing it needs no migration.

**5. Admin access**

The admin account is created with the `runique` CLI. Install it from the workspace, with the
`postgres` feature so it can talk to the demo's database:

```bash
cargo install --path runique --features postgres
cd demo-app
runique create-superuser
```

The admin is at `http://127.0.0.1:3000/prefix-test/admin-runique/`.

`runique start` regenerates `src/admins/` from `src/admin.rs` before running the site: needed
only after changing the `admin!{}` declarations. Reinstall the CLI after each update of the
workspace, so the generated code follows the framework's version.
