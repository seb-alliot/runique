🌍 **Languages**: [English](ROADMAP.md) | [Français](ROADMAP.fr.md)

# Runique — Roadmap

Few things at a time, each one shipped finished. The map will grow once these are done. What
has already shipped is in the [CHANGELOG](CHANGELOG.md).

> This roadmap isn't fixed. Things may have been overlooked, and fixes will be needed: a bug or
> a vulnerability that turns up comes before the planned work, and may lead to a patch release
> at any time.

## What "finished" means

A feature only goes into a published version when:

- it is tested on both sides: the accepted case and the refused one, limit included;
- a `cargo mutants --in-diff` pass on the changed code has run and every survivor is handled;
- security checks happen **before** the effect (write, redirect, query);
- it has been checked for real on demo-app and on a project created by `runique new`;
- the documentation (fr/en) and the CHANGELOG are up to date.

---

## Current state — 4.0.0 (unreleased)

Workspace is `4.0.0`. Breaking code is in the tree; the crates.io release has not shipped yet.

### Shipped on this branch

- [x] **Admin rework: typed `ModelAdmin<Entity>` builder** — replaces `admin!{}` and
  daemon-generated code. Typed columns, allow-lists, startup invariants. Declare in
  `src/admin.rs`, wire with `.with_admin(admin::config)`, `cargo run` is enough (no
  `runique start`, no `src/admins/`).
- [x] **Documentation aligned to 4.0** — fr/en guides, README, migration, SECURITY, demo-app
  (`seed.sql`, hub/override templates), INDEX and PROJECT_STATUS: the current API is
  `ModelAdmin`, not the old generator. Historical 3.0.x CHANGELOG entries still say
  "regenerate the admin" where that was true at the time.
- [x] **Security fixes already in the 4.0 CHANGELOG** — forgot-password timing, generic
  superuser refusal message, same-origin-only `csrf.js`, `password`/`is_active` out of
  `extend!` forms, debug page confined to `templates/`, form global errors escaped.

### Still open before or right after the 4.0 release

Remaining breaking items (ship with 4.0 if ready; otherwise defer to 5.0 with a guide):

- [ ] **Mandatory delete action** — a `belongs_to` without `[cascade]`, `[set_null]` or
  `[restrict]` no longer compiles, like Django's `on_delete`.
- [ ] **Typed uploads** — files kept apart (`StagedFile`), never among text field values; no
  server path in form data.

The release cycle is detailed in [CONTRIBUTING](CONTRIBUTING.md#versions-and-releases).

---

## Maintenance and security — ongoing

This work never ends: it comes before the items below.

- **Regular security checks** on what already exists: reviewing sensitive flows (from input to
  effect), hostile inputs, `cargo mutants` on the code involved.
- **Dependency audit**: `cargo audit` on every push, blocking in CI.
- **Bug fixes**, each one with a test that fails without it.
- Fixes are grouped in a **patch** (`4.0.x` while the 4.0 line is supported). A serious
  vulnerability is released right away.

### Identified fixes (next 4.0.x patch)

**Required work order** — these items ship **before** any 4.x minor (M2M, makemigrations,
tracing, languages). A serious vulnerability may jump the queue; otherwise follow the numbering.
Each item needs its regression test before it is checked off.

1. [x] **Password reset without the email in the link** — the email was encrypted with a key
   derived from the token, which sits in the same link: it protects nothing. Compare the typed
   email with the account's, and drop the email from the link. The custom
   `encrypt_email` / `decrypt_email` helpers were removed. Regression coverage verifies token-only
   links and that a mismatched email does not consume a reset token.
2. [x] **Uploads of a refused form** — deleted before the page is sent back when the form never
   reaches `finalize` (token, another field, `clean()`). The request then removes the `.staging-*`
   directory it created, including when the handler never validates. The one-hour sweep remains
   only if the process dies first. Tests cover a failing sibling field, a forced-invalid token, a
   form that was not submitted, and an accepted upload the end of the request does not remove.
3. [ ] **In-memory counters capped** — `RateLimiter` and `LoginGuard` bound their number of keys
   between two sweeps.
4. [ ] **A superuser's sessions** — protected like their account: a staff member can't delete them.
5. [ ] **Open redirect tests** — run the payloads of PortSwigger's *URL validation bypass* cheat
   sheet.
6. [x] **Deletion refused by a foreign key** — the admin shows "Cannot delete: other records still
   use it." The sentence is the same whatever the read right: no table name, no count. The engine's
   text stays in the log. By default a `belongs_to` without an action refuses (`NoAction`): never
   an implicit cascade.

Typical outcome: one or more **patch** releases `4.0.x` (e.g. `4.0.1`) while the 4.0 line is
supported. No minor feature mixed into these patches.

---

## After 4.0 — non-breaking minors (4.x)

**Prerequisite**: the security / maintenance `4.0.x` queue above is handled (or an urgent
vulnerability already forced a patch). Only then: one feature at a time.

A project already on 4.0 upgrades without touching its code. One feature at a time, in this
order: each ships in a minor release (4.**x**.0), then gets a targeted security check, then the
next one starts.

- [ ] **Many-to-many relations** — the junction table is generated from the DSL (`through`
  keeps a hand-written junction table, with its extra columns), migration included, and the
  model gets its access methods. *(Admin already handles M2M via a declared junction table;
  DSL generation + migration are what remain.)*
- [ ] **makemigrations finished** — no longer blocked on the admin rework (shipped). Gaps found
  on 2026-10-10 while moving a real project (Campanile) to 3.0:
  - a column **renamed and changed** at once (`renamed_from` + `NOT NULL` / default) only gets
    the rename; the change is lost, and the snapshot records it as done — a new `makemigrations`
    will never see it;
  - constraint names are **assumed** (`{table}_{column}_{target}_fkey`) instead of **read from
    the database**: a database created with another convention makes the migration fail;
  - a column becoming `NOT NULL` with a default must get its existing `NULL`s filled before the
    constraint (today: refused, to be done by hand);
  - the identifier length limit must depend on the engine (63 for Postgres, 64 for
    MariaDB/MySQL) — today 63 everywhere, with a message announcing 64;
  - a real `ALTER` when a column's type changes; the many-to-many junction table.
- [ ] **Tracing** — finish the file output (rotation) and the request id; complete the "no
  swallowed error" sweep. Translation fallbacks become visible: a key missing from the current
  language (falling back to English) and a key missing everywhere are traced.
- [ ] **Languages added by a project** — a trait to provide a language Runique doesn't ship,
  registered through the builder, without changing the `Lang` enum.

### Built on the builder: parity with the Django admin

Each item will become a typed method of the builder (4.x minors, additive API preferred).

- **List** — date filters (today, this week…), filters across a foreign key,
  `date_hierarchy`, `list_display_links`, `list_editable`, default ordering per resource,
  `empty_value_display`
- **Forms** — `fieldsets`, `readonly_fields`, `prepopulated_fields`, `autocomplete_fields`,
  "Save as new", link to the public view, configurable redirect after saving
- **Actions** — bulk actions with Rust logic, extra form for an action
- **Filters and search** — custom filters, search across related tables, indexes generated
  for filtered or searched columns
- **Customization** — base query per resource, per-request configuration, CSS/JS assets per
  resource
- **Inlines** — editing related objects inside the parent's form (FK and many-to-many)
- **Misc** — CSV/JSON export, "Save and continue", confirmation through `<dialog>`, several
  admin sites
- **Admin visuals** — mobile-first / CSS consolidation (started; polish remaining)

---

## Later — no commitment

These will move up when a piece of work is finished.

- **DSL and migrations** — numeric steps and bounds (`step`, `min`, `max`), declared indexes,
  circular foreign keys
- **ORM** — hooks/signals (`before_save`, `after_save`…), aggregates in `search!`,
  `.first()` returning `Option<T>`, several values for one field in `search!`
- **Framework** — typed redirects (`redirect("route_name")`, `redirect_external(url)` limited to allowed hosts, `?next=` checked against declared routes), `crud!{}` for public views, `#[form]` field overrides, several connections
  of different types (TypeMap), language detection middleware, `path_params` /
  `query_params` through getters
- **Storage** — a `StorageBackend` extension point (local disk by default, S3/R2 or another
  service as an option) shared by media, uploads in transit and future exports. Until then,
  external storage mounted as a folder (`s3fs`, `rclone mount`, NFS) works as `MEDIA_ROOT`
- **Authentication** — two-level brute-force protection, without breaking change: lock per username + IP (the real user, elsewhere, can still sign in), and a slowdown per username across all addresses (against attacks spread over many IPs); one shared function, called by `LoginGuard` and by the admin sign-in. Then OAuth/OIDC, JWT and API keys, sign-in log, CSP violation reports
- **Tooling** — built-in test client, fixtures, custom commands, sitemap and RSS, image
  resizing, full documentation of the public API
