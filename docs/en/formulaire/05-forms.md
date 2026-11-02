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

Forms are built from the request by `request.form()` (filling and CSRF check). `ValidationForm::try_form` names the form and validates it when it was submitted (formats through the `validator` crate, password hashing per the `PasswordConfig`); a validated entity form is then written with `save` / `delete`.

---

## Next steps

← [**Routing**](/docs/en/routing) | [**Templates**](/docs/en/template) →
