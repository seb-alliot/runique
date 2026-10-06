# Runique CLI

## Create a Superuser

Command-line interface to create superusers, start the server and manage migrations.

```bash
runique create-superuser
```

```
=== Create Superuser ===  [Ctrl+C to quit]

[1/5] Hash algorithm:
  1) Argon2  (recommended)
  2) Bcrypt
  3) Scrypt
  4) Custom provider
Choice [1-4] (default: 1):

[2/5] Username:
[3/5] Email:
[4/5] Password:
[5/5] Confirm password:

──────────────────────────────────
  Algorithm : Argon2
  Username  : admin
  Email     : admin@example.com
  Password  : ••••••••
──────────────────────────────────
[Enter] Confirm  [A] Change algorithm  [Ctrl+C] Cancel
```

**Navigation:** `ESC` only works on the algorithm selection step. The username/email/password steps (text input) and the final review screen have no way back — `Ctrl+C` cancels at any time.

> The CLI runs without the application runtime — it has no access to the `PasswordConfig` configured in `main.rs`. The algorithm is chosen explicitly at each run.
>
> For the `Custom` case, provide a binary or script that reads the password from **stdin** and returns the hash on **stdout**.

---

## All Commands

```bash
runique new <name>                                                    # Create a new project
runique start [--main src/main.rs] [--admin src/admin.rs]           # Start with admin daemon
runique makemigrations --entities src/entities --migrations migration/src  # Generate migrations
runique migration up --migrations migration/src                     # Apply migrations (sea-orm-cli migrate up)
runique create-superuser                                            # Create a superuser
runique test [file] [test]                                          # Run src/runique_test/ (see Testing)
```

Rolling back or listing migrations goes through SeaORM: `sea-orm-cli migrate down -n N`, `sea-orm-cli migrate status`. `runique test` is described in [Testing](/docs/en/architecture/testing).

---

## See also

| Section | Description |
| --- | --- |
| [Migrations](/docs/en/installation/migrations) | Migration workflow |
| [Troubleshooting](/docs/en/installation/troubleshooting) | Solving common issues |

## Back to summary

- [Installation](/docs/en/installation)
