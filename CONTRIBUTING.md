🌍 **Languages**: [English](CONTRIBUTING.md) | [Français](CONTRIBUTING.fr.md)

# Contributing to Runique

Thank you for your interest in contributing! This page lists what a change needs before it is
merged. To run the tests and the demo site locally, see [TESTING.md](TESTING.md).

**Found a security issue?** Don't open a public issue: follow [SECURITY.md](SECURITY.md).

---

## Repository layout

| Folder | Content |
| --- | --- |
| `runique/` | The framework (crate `runique`) |
| `runique/derive_form/` | The `model!{}`, `extend!{}` and `#[form]` macros |
| `runique/runique_dsl/` | The model DSL parser, shared by the macros and the CLI |
| `demo-app/` | The demo site (runique.io), Postgres only |
| `docs/fr/`, `docs/en/` | The documentation served by the site |

---

## Before opening a pull request

The CI runs these checks; run them locally first:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --features orm,sqlite,postgres,mysql,mariadb,all-databases,acme -- -D warnings

cd runique
DB_ENGINE=postgres cargo clippy --all-targets --features big-pk -- -D warnings
DB_ENGINE=postgres cargo clippy --all-targets --features pk-uuid -- -D warnings
cargo test --features orm,sqlite,postgres,mysql,mariadb,all-databases,acme
cargo test --features big-pk,all-databases
cargo test --features pk-uuid,all-databases
```

Warnings are errors (`-D warnings`). The three test runs cover the three primary key types
(`i32`, `i64`, `Uuid`): a test that writes ids by hand must use the `pk` helpers of
`runique/tests/helpers/pk.rs`, not integer literals.

---

## Code

- Rust edition 2024. Chain conditions with let-chains (`if let Some(x) = a && cond { … }`)
  rather than nested `if`s.
- Comments explain **why**, when it isn't obvious (a hidden constraint, an invariant). No
  comment that repeats the code.
- No refactoring without a concrete gain (behavior, performance, security).
- A public API removed or changed is a breaking change: it goes in the CHANGELOG and in the
  migration guide (`MIGRATION-x.y.md`).

---

## Tests

- Every bug fix comes with a test that fails without the fix.
- Test both sides of a condition: the case that passes and the one that's refused, limit
  included.
- Security checks are tested by an attempt that must fail (wrong token, other user's id,
  oversized body…), not only by the case that works.

---

## Generated code

`src/admins/` (in demo-app or a project) is generated from `admin!{}` by `runique start`: never
edit it by hand. Change the generator (`runique/src/admin/daemon/generator.rs`), then refresh
its reference file and review the diff:

```bash
cd runique
RUNIQUE_UPDATE_GOLDEN=1 cargo test --lib admin::daemon::generator
```

---

## Documentation

- The documentation exists in French and English: update `docs/fr/` and `docs/en/` together,
  and both `CHANGELOG.md` / `CHANGELOG.fr.md`.
- Internal links are checked:

  ```bash
  cargo test -p demo-app internal_doc_links_resolve
  ```

---

## Security-sensitive code

For any change touching authentication, permissions, sessions, tokens, redirects or user input,
check explicitly:

- timing attacks (compare secrets with `ct_eq`, never `==`);
- injection (SQL, HTML, templates);
- access control bypass (another user's object, a hidden button whose route still answers);
- CSRF;
- open redirect;
- exposure of sensitive data (logs, error pages, admin lists).
