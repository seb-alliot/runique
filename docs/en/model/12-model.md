# Models and AST (`model!`)

> Related files: `src/entities/*`

The `model!` macro generates SeaORM entities, migration schemas, and associated forms from a declarative DSL.

---

## Table of Contents

| Section | Content |
| --- | --- |
| [DSL & AST](/docs/en/model/dsl) | Exposed macros, `model!` syntax, internal AST, field types and options |
| [Generation & ModelSchema](/docs/en/model/generation) | Generated code, `ModelSchema`, `to_migration()`, `fill_form()` |
| [Forms & Challenges](/docs/en/model/formulaires) | `#[form(...)]`, technical considerations, reading order |

---

## Recommended Reading Order

1. This document (`12-model.md`)
2. [ORM](/docs/en/orm) for database usage
3. [Forms](/docs/en/formulaire) for form extraction and rendering
