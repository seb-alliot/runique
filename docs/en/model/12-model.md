```markdown
# Models and AST (`model!`)

> Related files: `src/entities/*`

The `model!` macro generates SeaORM entities, migration schemas, and associated forms from a declarative DSL.

---

## Table of Contents

| Section | Content |
| --- | --- |
| [DSL & AST](https://github.com/seb-alliot/runique/blob/main/docs/en/model/dsl/dsl.md) | Exposed macros, `model!` syntax, internal AST, field types and options |
| [Generation & ModelSchema](https://github.com/seb-alliot/runique/blob/main/docs/en/model/generation/generation.md) | Generated code, `ModelSchema`, `to_migration()`, `fill_form()` |
| [Forms & Challenges](https://github.com/seb-alliot/runique/blob/main/docs/en/model/formulaires/formulaires.md) | `#[form(...)]`, technical considerations, reading order |

---

## Recommended Reading Order

1. This document (`12-model.md`)
2. [ORM](https://github.com/seb-alliot/runique/blob/main/docs/en/orm/07-orm.md) for database usage
3. [Forms](https://github.com/seb-alliot/runique/blob/main/docs/en/formulaire/05-forms.md) for Prisme integration and rendering
```
