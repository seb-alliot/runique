# Database errors

[← Field types](/docs/en/formulaire/fields)

---

The `database_error()` method automatically analyzes DB errors to attach the error to the correct field:

```rust
match form.save(&request.engine.db).await {
    Ok(_) => { /* success */ }
    Err(err) => {
        form.database_error(&err);
        // The error is set on the relevant field
    }
}
```

**Supported error formats:**

- **PostgreSQL**: `UNIQUE constraint`, `Key (field)=(value)`
- **SQLite**: `UNIQUE constraint failed: table.field`
- **MySQL**: `Duplicate entry ... for key 'table.field'`

If the field is identified, the error appears on that field (e.g. "This email is already used"). Otherwise, it is added to global errors.

---

← [**Field types**](/docs/en/formulaire/fields) | [**Template rendering**](/docs/en/formulaire/templates) →
