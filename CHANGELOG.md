🌍 **Languages**:[English](https://github.com/seb-alliot/runique/blob/main/CHANGELOG.md) | [Français](https://github.com/seb-alliot/runique/blob/main/CHANGELOG.fr.md)

# Changelog

All notable changes to this project will be documented in this file.

---

## [2.1.2] - 2026-05-02

### Fixed — `derive_form` 2.0.2

* **`fk()` in v2 blocks silently ignored:** `FormFieldAttr::Fk(FkDef)` added to AST, parser, and propagation to `FieldOption::Fk`.
* **`skip` attribute unknown:** `FormFieldAttr::Skip` added to AST, parser, and generator (field excluded from form rendering).
* **`many_to_many(target).through(via)` syntax broken:** corrected to `many_to_many(target, via)` in `foreignkey.rs`.
* **`sea_query::ForeignKeyAction` not found:** re-exported as `runique::migration::ForeignKeyAction`; generator paths updated.
* **`.references_column()` method missing:** replaced with `.to_column()` in FK builder.
* **PascalCase model names in relation paths:** `to_snake_case()` used consistently instead of `.to_lowercase()` across `relation_enum.rs` and `foreignkey.rs` (e.g. `super::menuimage` → `super::menu_image`).
* **`rust_decimal::Decimal` not found:** type mapped to `::runique::sea_orm::prelude::Decimal` in `sea_model.rs`.
* **`via_self` FK column → wrong relation variant:** `_id` suffix stripped and PascalCase applied to derive the correct variant name in `ManyToMany` `Related` impl.
* **`Decimal` missing from `generate_partial_update`:** `FieldType::Decimal(_)` added to the numeric arm.
* **`Decimal` missing from `generate_from_str_map`:** `FieldType::Decimal(_)` added to the float/decimal arm.
* **`unique_together` / `indexes` never generated in SQL:** `parser_builder.rs` was silently ignoring the `meta` block. Now parsed and converted to `ParsedIndex` entries (`{table}_{cols}_uniq` for unique constraints, `idx_{table}_{cols}` for plain indexes).

### Added — `runique` 2.1.1-alpha.3

* **`OrderDir` enum** added to `migration::schema` (`Asc` / `Desc`).
* **`ModelSchema` builder methods:** `order_by()`, `unique_together()`, `verbose_name()`, `verbose_name_plural()`.
* **`ForeignKeyAction` re-exported** from `runique::migration`.
* **`RelationDef::as_name()`** no-op method added for DSL compatibility.

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

