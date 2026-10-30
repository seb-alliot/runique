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

**Supported error formats:**

- **PostgreSQL**: `UNIQUE constraint`, `Key (field)=(value)`
- **SQLite**: `UNIQUE constraint failed: table.field`
- **MySQL**: `Duplicate entry ... for key 'table.field'`

If the field is identified, the error appears on that field (e.g. "This email is already used"). Otherwise, it is added to global errors.

Any other database error is shown with the engine's own text (`forms.db_error`). Runique does not replace it with a generic sentence: that text can name a table, a column or a constraint. Template escaping still applies.

---

← [**Field types**](/docs/en/formulaire/fields) | [**Template rendering**](/docs/en/formulaire/templates) →
