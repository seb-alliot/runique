# Forms

## Table of Contents

- [Overview](#overview)
- [Prisme extractor](https://github.com/seb-alliot/runique/blob/main/docs/en/formulaire/prisme/prisme.md)
- [RuniqueForm trait](https://github.com/seb-alliot/runique/blob/main/docs/en/formulaire/trait/trait.md)
  - Base structure
  - Trait methods
  - `is_valid()` pipeline
- [Typed conversion helpers](https://github.com/seb-alliot/runique/blob/main/docs/en/formulaire/helpers/helpers.md)
- [Field types](https://github.com/seb-alliot/runique/blob/main/docs/en/formulaire/fields/fields.md)
  - TextField, NumericField, BooleanField, ChoiceField, RadioField…
  - Summary table
- [Database errors](https://github.com/seb-alliot/runique/blob/main/docs/en/formulaire/errors/errors.md)
- [Template rendering](https://github.com/seb-alliot/runique/blob/main/docs/en/formulaire/templates/templates.md)
- [Full example & common pitfalls](https://github.com/seb-alliot/runique/blob/main/docs/en/formulaire/example/example.md)

---

## Overview

Runique provides a powerful form system inspired by Django. There are **two approaches**:

1. **Manual** — Define fields via the `RuniqueForm` trait.
2. **Automatic** — Derive a form from a `model!` schema with `#[form(...)]`.

Forms are automatically extracted from requests via the **Prisme** extractor, handle validation (including via the `validator` crate for emails/URLs), CSRF, Argon2 password hashing, and can be saved directly to the database.

---

## Next steps

← [**Routing**](https://github.com/seb-alliot/runique/blob/main/docs/en/routing/04-routing.md) | [**Templates**](https://github.com/seb-alliot/runique/blob/main/docs/en/template/06-templates.md) →
