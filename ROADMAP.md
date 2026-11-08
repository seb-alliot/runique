🌍 **Languages**: [English](ROADMAP.md) | [Français](ROADMAP.fr.md)

# Runique — Roadmap

Few things at a time, each one shipped finished. What has already shipped is in the
[CHANGELOG](CHANGELOG.md).

This roadmap gives no dates and makes no commitment. A bug or a security flaw that turns up comes
before planned work, and may lead to a patch release at any time.

**What "finished" means**: a feature only goes into a published version when it is tested on both
sides (the accepted case and the refused one, limit included), when a `cargo mutants --in-diff`
pass has handled every survivor, when its security checks happen before the effect, when it has
been checked on demo-app and on a project created by `runique new`, and when the documentation
(fr/en) and the CHANGELOG are up to date.

## In progress

- **4.1, makemigrations**: generate every schema operation with no manual step: dropping tables,
  creating and dropping relation tables, honouring cascade, restrict and set_null, a real `ALTER`
  on a type change, detecting default value changes, and columns both renamed and modified.
  Constraint names are read from the database, and destructive changes state what they affect.
  The many-to-many pivot table is generated from the model DSL; a hand-written table with extra
  columns remains possible.

## Planned

- **Security fixes (4.0.x)**: cap the number of keys held in memory by `RateLimiter` and
  `LoginGuard`, and test open redirect protection against known URL validation bypass payloads.
- **4.2, database**: `get_or_create`, `update_or_create`, `exists()`, `last()`,
  loading relations in one query, many-to-many access methods
  and its management in the admin. `search!` also accepts AND groups inside OR, and the negation of
  a condition or a group, still through typed queries.
- **Tracing**: finish file output with rotation, add a request identifier, and leave no error
  silently ignored. Missing translation keys will be logged.
- **Languages added by a project**: let a project provide a language Runique does not ship,
  registered through the builder.
- **Admin**: date and foreign key filters, date hierarchy, `fieldsets`, read-only fields,
  autocomplete, inline editing, group actions with Rust logic, CSV/JSON export, and "save and
  continue".

## Later

No commitment: these will move up as work finishes.

- **Authentication**: two-level brute force protection, then OAuth/OIDC, JWT, API keys, a login
  journal and CSP violation reports.
- **ORM and DSL**: model hooks, aggregates in `search!`, declared indexes, and numeric bounds and
  steps.
- **Tools and storage**: public CRUD views, a built-in test client, fixtures, custom commands,
  sitemap and RSS, image resizing, and a storage extension point for S3-compatible services.
