# Database errors

[← Field types](/docs/en/formulaire/fields)

---

`database_error()` analyzes a DB error and attaches it to the right field.

For an entity form, `ValidationForm::save` already calls it. The `Err` is the form, with the error set:

```rust
match validated.save(&request.engine.db).await {
    Ok(_form) => { /* success */ }
    Err(form) => { /* unique constraint on the field, otherwise a global error */ }
}
```

After a hand-written save that returns a `DbErr`, call it yourself:

```rust
if let Err(err) = insert_user(&request.engine.db).await {
    form.database_error(&err);
}
```

An error that still carries SQLx is classed, then replaced by a translated sentence. The engine's text is logged.

- **Unique**: on the field when the message or the constraint names the column ("This email is already taken"), otherwise a global sentence.
- **Foreign key, required value, check**: one global sentence each. A Postgres exclusion uses the check sentence.
- **Anything else**: one generic sentence (`forms.db_error`), without the engine's text.

A `DbErr::Custom` is a sentence already chosen (a gate refusal, a linked record, an unreadable field). It is shown as it is.

---

← [**Field types**](/docs/en/formulaire/fields) | [**Template rendering**](/docs/en/formulaire/templates) →
