# Link with forms & technical considerations

## Link with forms via `#[form(...)]`

The `#[form(...)]` attribute macro expects:

- `schema = model_module` (required);
- **and exactly one** of these three declarations, saying which fields the form takes:

| Declaration | Fields of the form |
| --- | --- |
| `fields = [a, b]` | only those, in that order |
| `exclude = [a, b]` | all of them, but those |
| `fields = All` | all of them |

Without one of the three, or with `fields` and `exclude` together, the code **doesn't compile**: the declaration is incomplete. A name that isn't a column of the model (`fields = [username, roles]`) doesn't compile either, and the error points at the name.

`fields = All` and `exclude` also take the columns added to the model later. For accounts (`runique_users`), "all" includes the ranks (`is_admin`, `is_dirigeant`, `is_superuser`) and `is_active`: keep it for a form whose use you control. On a public form (sign-up, profile), list the fields.

It generates:

- a struct containing `form: Forms`
- `impl ModelForm` (`schema()`, `fields()`, `exclude()`)
- `impl FormAccess` (fields + accessors — no `impl_form_access!(model)` in user code)
- `impl FormEntity` + `pub const objects` (automatic link to `<schema>::Entity`)

The developer then writes only `impl RuniqueForm` (empty, or with hooks):

```rust
use runique::prelude::*;

#[form(schema = users, fields = [username, email])]
pub struct UserForm;

impl RuniqueForm for UserForm {}
```

### ORM access from the form

`#[form(schema = …)]` ties the form to its model module's entity automatically (`schema = blog` → `blog::Entity`), which gives `Form::objects` and `search!(@Form => …)`: the form becomes a direct entry point for ORM queries, without going through the entity type.

```rust
#[form(schema = users, fields = All)]
pub struct UserForm;

#[form(schema = blog, fields = All)]
pub struct BlogForm;

#[form(schema = document, fields = All)]
pub struct DocumentForm;
```

This automatically generates:

```rust
impl FormEntity for UserForm {
    type Entity = users::Entity;
}
impl UserForm {
    pub const objects: Objects<users::Entity> = Objects::new();
}
```

**Direct ORM access via the form:**

```rust
// All records
let users = UserForm::objects.all().all(&db).await?;

// With filter
let user = UserForm::objects
    .filter(users::Column::Email.eq("alice@example.com"))
    .first(&db)
    .await?;

// Via the search! macro
let results = search!(@UserForm => Username eq "alice").all(&db).await?;
let adults  = search!(@UserForm => Age gte 18).all(&db).await?;

// All search! operators are supported
let results = search!(@BlogForm =>
    or(Status eq "published", Status eq "featured"),
    AuthorId eq author_id,
)
.order_by_desc(blog::Column::CreatedAt)
.limit(10)
.all(&db)
.await?;
```

> `fields` and `exclude` combine freely with the automatic link to the entity.

```rust
#[form(schema = users, fields = [username, email])]
pub struct UserForm;
```

### With business validation (`clean`)

Override `clean` directly in `impl RuniqueForm` — just like Django.
`#[async_trait]` is only required when overriding an async method:

```rust
#[form(schema = users, fields = [username, email, password])]
pub struct RegisterForm;

#[async_trait]
impl RuniqueForm for RegisterForm {
    async fn clean(&mut self) -> Result<(), StrMap> {
        let mut errors = StrMap::new();
        if self.cleaned::<String>("username").unwrap_or_default().len() < 3 {
            errors.insert("username".to_string(), "Minimum 3 characters".to_string());
        }
        if !self.cleaned::<String>("email").unwrap_or_default().contains('@') {
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

### Customizing one field — `customize()`

A `#[form]` form is customized field by field without rewriting `register_fields`: the `customize()` hook runs right after the model's fields are registered, every time the form is built. Declaring a field again under the same name replaces it.

```rust
#[form(schema = article, fields = All)]
pub struct ArticleForm;

impl RuniqueForm for ArticleForm {
    fn customize(form: &mut Forms) {
        // Replaces the `image` field only; the others stay the model's
        form.field(
            &MediaField::image("image")
                .label("Illustration")
                .upload_to("articles")
                .max_size(FileSize::mb(5)),
        );
    }
}
```

The limits declared in the model (length, file size…) are applied again after `customize()`: a form may tighten them, never exceed them.

## See also

| Section | Description |
| --- | --- |
| [DSL & AST](/docs/en/model/dsl) | `model!` syntax, types, options |
| [Generation & ModelSchema](/docs/en/model/generation) | Generated code |
| [CRUD Queries](/docs/en/orm/queries) | Full `search!` reference |

## Back to summary

- [Models](/docs/en/model)
