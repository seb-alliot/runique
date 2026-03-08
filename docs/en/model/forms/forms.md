# Link with forms & technical considerations

## Link with forms via `#[form(...)]`

The `#[form(...)]` attribute macro expects:

- `schema = function_path` (required)
- `fields = [..]` (optional)
- `exclude = [..]` (optional)

Concrete example:

```rust
use runique::prelude::*;

#[form(schema = user_schema, fields = ["username", "email"], exclude = ["is_active"])]
pub struct UserForm;
```

This macro generates:

- a struct containing `form: Forms`,
- `impl ModelForm` (`schema()`, `fields()`, `exclude()`),
- `impl RuniqueForm` which delegates to `ModelForm::model_register_fields(...)`.

---

## Technical considerations

### Advantages

- Single model/schema contract, centralized
- Coherent generation of migrations + forms
- Reduced duplication of field definitions

### Points of attention

- Strict DSL: a syntax error causes a macro build error
- Misaligned `fields`/`exclude` with the schema can cause generation or runtime errors
- Pedagogical order matters: understand `model/schema` before model-based form generation

---

## See also

| Section | Description |
| --- | --- |
| [DSL & AST](https://github.com/seb-alliot/runique/blob/main/docs/en/model/dsl/dsl.md) | `model!` syntax, types, options |
| [Generation & ModelSchema](https://github.com/seb-alliot/runique/blob/main/docs/en/model/generation/generation.md) | Generated code |

## Back to summary

- [Models](https://github.com/seb-alliot/runique/blob/main/docs/en/model/12-model.md)
