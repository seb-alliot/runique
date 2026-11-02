# Troubleshooting

## `no such table: eihwaz_sessions` (or `relation "eihwaz_sessions" does not exist`)

The migrations haven't been applied to this database.

```bash
runique migration up
```

The error may look intermittent: a page shows with no session cookie (an anonymous session, kept in memory), then fails as soon as the browser sends a cookie the server no longer knows, after a restart for instance, and looks it up in the database. In debug mode the error page may then be empty: the cause is in the terminal logs (`ERROR tower_sessions_core::session`).

---

## "driver not enabled" at startup

The engine `.env` asks for (`DB_ENGINE` / `DATABASE_URL`) isn't the one Runique was compiled for. The message names the compiled engine: align `.env` on it, or change the feature in `Cargo.toml` **and** `migration/Cargo.toml`. See [Database](/docs/en/installation/database).

For the CLI (`create-superuser`), it's the installed binary that needs the feature: `cargo install runique --features <engine> --locked --force`.

---

## `can't find crate for serde` on every `model!` / `#[form]`

`serde`, `sea-orm`… aren't under `[dependencies]`. The usual cause: a `[dev-dependencies]` section inserted **in the middle** of `Cargo.toml`. In TOML, a line belongs to the last section opened above it: everything after `[dev-dependencies]` becomes a test-only dependency.

```toml
[dependencies]
runique = { version = "4.0.0", features = ["sqlite"] }
serde = { version = "1", features = ["derive"] }
chrono = "0.4"
sea-orm = "=2.0.4"
sea-orm-migration = "=2.0.4"

# Always last
[dev-dependencies]
runique = { version = "4.0.0", features = ["test-utils"] }
```

The "`Column` / `Entity` not found" errors that follow are only a consequence: the `model!` failed, so it generated nothing.

---

## Signed out when switching between local projects

Two Runique projects run on the same host (`127.0.0.1:3000` and `127.0.0.1:3001`) share the same session cookie: **cookies ignore the port**. The second project doesn't know the session, creates one and replaces the cookie: the first one sees you signed out. Open one on `localhost` and the other on `127.0.0.1`, two distinct host names for the browser.

It's not a flaw: a session id is only valid for the database that created it. Two projects sharing one database share their sessions too.

---

## `makemigrations` writes nothing while the tables don't exist

The snapshots in `migration/src/snapshots/` describe a schema "already migrated". Copied from another project without their migration files, they make `makemigrations` see no change. Delete `migration/src/snapshots/` and `migration/src/lib.rs`, then run `runique makemigrations` again.

---

## `Connection refused` (PostgreSQL)

```bash
sudo systemctl status postgresql   # Linux
brew services list                 # macOS
```

---

## `permission denied` on the database (PostgreSQL)

```sql
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO runique_user;
GRANT ALL PRIVILEGES ON SCHEMA public TO runique_user;
```

---

## Port 3000 already in use

Change `PORT` in `.env`.

---

## See also

| Section | Description |
| --- | --- |
| [Prerequisites](/docs/en/installation/prerequisites) | Initial setup |
| [Database](/docs/en/installation/database) | Switching engines |
| [Migrations](/docs/en/installation/migrations) | Workflow and limits |

## Back to summary

- [Installation](/docs/en/installation)
