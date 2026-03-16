# Link with forms & technical considerations

## Link with forms via `#[form(...)]`

The `#[form(...)]` attribute macro expects:

- `schema = function_path` (required)
- `fields = [..]` (optional)
- `exclude = [..]` (optional)

It generates only:

- a struct containing `form: Forms`
- `impl ModelForm` (`schema()`, `fields()`, `exclude()`)

The developer then writes `impl RuniqueForm` with `impl_form_access!(model)`:

```rust
use runique::prelude::*;

#[form(schema = user_schema, fields = [username, email])]
pub struct UserForm;

impl RuniqueForm for UserForm {
    impl_form_access!(model);
}
```

### With business validation (`clean`)

Override `clean` directly in `impl RuniqueForm` — just like Django.
`#[async_trait]` is only required when overriding an async method:

```rust
#[form(schema = user_schema, fields = [username, email, password])]
pub struct RegisterForm;

#[async_trait]
impl RuniqueForm for RegisterForm {
    impl_form_access!(model);

    async fn clean(&mut self) -> Result<(), StrMap> {
        let mut errors = StrMap::new();
        if self.get_string("username").len() < 3 {
            errors.insert("username".to_string(), "Minimum 3 characters".to_string());
        }
        if !self.get_string("email").contains('@') {
            errors.insert("email".to_string(), "Invalid email".to_string());
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}
```

> `is_valid()` automatically calls `clean` after structural validation.
> Returned errors are attached to fields and displayed inline in the template.

---

## Technical considerations

### Advantages

- Single model/schema contract, centralized
- Coherent generation of migrations + forms
- Reduced duplication of field definitions
- `clean` is the official trait override — uniform between manual and model-based forms

### Points of attention

- Strict DSL: a syntax error causes a macro build error
- Misaligned `fields`/`exclude` with the schema can cause generation or runtime errors
- `#[async_trait]` required on `impl RuniqueForm` only when overriding `clean` or `clean_field`

---

## See also

| Section | Description |
| --- | --- |
| [DSL & AST](https://github.com/seb-alliot/runique/blob/main/docs/en/model/dsl/dsl.md) | `model!` syntax, types, options |
| [Generation & ModelSchema](https://github.com/seb-alliot/runique/blob/main/docs/en/model/generation/generation.md) | Generated code |

## Back to summary

- [Models](https://github.com/seb-alliot/runique/blob/main/docs/en/model/12-model.md)
