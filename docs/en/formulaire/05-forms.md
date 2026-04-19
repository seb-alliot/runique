# Forms

## Table of Contents

- [Overview](#overview)
- [Form extraction — `request.form()`](/docs/en/formulaire/prisme)
- [RuniqueForm trait](/docs/en/formulaire/trait)
  - Base structure
  - Trait methods
  - `is_valid()` pipeline
- [Typed conversion helpers](/docs/en/formulaire/helpers)
- [Field types](/docs/en/formulaire/fields)
  - TextField, NumericField, BooleanField, ChoiceField, RadioField…
  - Summary table
- [Database errors](/docs/en/formulaire/errors)
- [Template rendering](/docs/en/formulaire/templates)
- [Full example & common pitfalls](/docs/en/formulaire/example)

---

## Overview

Runique provides a powerful form system inspired by Django. There are **two approaches**:

1. **Manual** — Define fields via the `RuniqueForm` trait.
2. **Automatic** — Derive a form from a `model!` schema with `#[form(...)]`.

Forms are automatically extracted from requests via `request.form()`, handle validation (including via the `validator` crate for emails/URLs), CSRF, Argon2 password hashing, and can be saved directly to the database.

---

## Next steps

← [**Routing**](/docs/en/routing) | [**Templates**](/docs/en/template) →
