🌍 **Languages**:
 [English](https://github.com/seb-alliot/runique/blob/main/CHANGELOG.md) | [Français](https://github.com/seb-alliot/runique/blob/main/CHANGELOG.fr.md)

# Changelog

All notable changes to this project will be documented in this file.

---

## [2.1.0] - 2026-04-20

### Breaking

* **`Prisme<T>` removed — form extraction via `req.form::<T>()`:**
  Handler parameters no longer accept `Prisme<MyForm>`. Use `let form = req.form::<MyForm>()` instead.
  `Request` must be the **last parameter** of every handler (body-consuming extractor).
  `AdminBody` extractor removed — admin POST handlers read form data from `req.prisme.data`.

### Added

* **`EihwazSessionsMigration` — persistent session table:**
  `migrations_table::EihwazSessionsMigration` creates the `eihwaz_sessions` table.
  To be added to the `Migrator` vec after `EihwazUsersMigration`.
  `eihwaz_sessions` is now listed in `FRAMEWORK_TABLES` and excluded from `makemigrations` scan.

### Fixed

* **`auth_login` — sessions now persisted in DB:**
  `auth_login()` now passes a `RuniqueSessionStore` to `login()`, ensuring a row is created
  in `eihwaz_sessions` at login. Sessions survive server restarts via the DB fallback.

---

