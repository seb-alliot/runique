# Runique CLI

The CLI creates projects, writes and applies migrations, creates the first superuser and runs the business tests. Run it from the **project root** (where `.env` is).

## Every command

```bash
runique new <name>                # Create a new project
runique makemigrations            # Write the migrations of the models that changed
runique makemigrations --force    # Same, allowing destructive changes
runique migration up              # Apply every pending migration
runique migration down -n 1       # Roll back the last migration (its down() runs)
runique migration status          # List the migrations and their state
runique create-superuser          # Create a superuser
runique test [file] [test]        # Run src/runique_test/ (see Tests)
```

No `runique start`: the app runs with `cargo run`. The admin is declared in Rust (`ModelAdmin` + `.with_admin(...)`), with no daemon and no code generation.

---

## `runique new`

Creates the whole project: the app, the `migration/` crate, templates, CSS, and a `.env` with a random `SECRET_KEY`. See [Prerequisites & first project](/docs/en/installation/prerequisites).

Refused names: empty, characters other than letters / digits / `_` / `-`, starting with a digit or `-`, Rust keywords, built-in crates (`std`, `core`, `alloc`, `proc_macro`, `test`), cargo's folders (`build`, `deps`, `examples`, `incremental`), Windows device names (`con`, `nul`, `com1`…), and `migration` / `runique`. The comparison ignores case and reads `-` as `_`.

---

## `runique makemigrations`

```bash
runique makemigrations [--entities src/entities] [--migrations migration/src] [--force]
```

Reads the `model!{}` and `extend!{}` blocks of `src/entities/`, compares them to their snapshots, and writes the migrations. When the `migration/` crate doesn't exist yet (a project not created by `runique new`), it is created, with the engine of `DB_ENGINE` (SQLite when unset). Details: [Migrations](/docs/en/installation/migrations).

---

## `runique migration up | down | status`

```bash
runique migration up [--migrations migration/src]
runique migration down [-n 1] [--migrations migration/src]
runique migration status [--migrations migration/src]
```

Runs the project's `migration` crate (`cargo run` on `migration/Cargo.toml`), which reads `DATABASE_URL` from `.env`. `sea-orm-cli` isn't needed: the SeaORM version that applies the migrations is the one `migration/Cargo.toml` pins, the same as runique's.

---

## `runique create-superuser`

Creates an active superuser account, activated at once. The command reaches the database as the app does: `DATABASE_URL`, or `DB_ENGINE` + `DB_*`, from `.env`. Apply the migrations first.

The wizard goes through five steps, with keyboard menus (arrows + Enter):

1. **Hashing algorithm**: Argon2 (recommended), Bcrypt, Scrypt, or a custom provider
2. **Username**: refused when empty or already taken
3. **Email**: refused when invalid or already taken
4. **Password**, then its confirmation
5. **Review**: confirm and create, change the algorithm, or change the password

The password needs **at least 12 characters**, with a lowercase letter, an uppercase letter, a digit and a special character. `Ctrl+C` cancels at any time.

> The CLI runs without the app: it can't see the `PasswordConfig` of `main.rs`, so the algorithm is picked on each run. Pick the app's; otherwise the hash is moved to the configured algorithm on the first successful sign-in (`Auto` mode).
>
> **Custom provider**: give the path of an executable (not a symbolic link) that reads the password on **stdin** and writes the hash on **stdout**. It runs without a shell and must answer within 10 seconds.

---

## `runique test`

Runs the tests of `src/runique_test/`, each inside a transaction rolled back at the end. The command refuses to run when the `test-utils` feature is missing, or declared under `[dependencies]` instead of `[dev-dependencies]` (it would then ship in the production binary). See [Tests](/docs/en/architecture/testing).

---

## See also

| Section | Description |
| --- | --- |
| [Migrations](/docs/en/installation/migrations) | Migration workflow |
| [Troubleshooting](/docs/en/installation/troubleshooting) | Fixing common problems |

## Back to summary

- [Installation](/docs/en/installation)
