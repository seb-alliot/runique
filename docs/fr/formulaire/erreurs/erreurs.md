# Erreurs de base de données

[← Types de champs](/docs/fr/formulaire/champs)

---

La méthode `database_error()` analyse automatiquement les erreurs DB pour remonter l'erreur au bon champ :

```rust
match form.save(&request.engine.db).await {
    Ok(_) => { /* succès */ }
    Err(err) => {
        form.database_error(&err);
        // L'erreur est positionnée sur le champ concerné
    }
}
```

**Formats d'erreur supportés :**

- **PostgreSQL** : `UNIQUE constraint`, `Key (field)=(value)`
- **SQLite** : `UNIQUE constraint failed: table.field`
- **MySQL** : `Duplicate entry ... for key 'table.field'`

Si le champ est identifié, l'erreur apparaît sur ce champ (ex: « Ce email est déjà utilisé »). Sinon, elle est ajoutée aux erreurs globales.

---

← [**Types de champs**](/docs/fr/formulaire/champs) | [**Rendu dans les templates**](/docs/fr/formulaire/templates) →
