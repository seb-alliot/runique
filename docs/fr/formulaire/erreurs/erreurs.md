# Erreurs de base de données

[← Types de champs](/docs/fr/formulaire/champs)

---

`database_error()` analyse une erreur DB pour la remonter au bon champ.

Pour un formulaire d'entité, `ValidationForm::save` l'appelle déjà. Le `Err` est le formulaire, avec l'erreur posée :

```rust
match validated.save(&request.engine.db).await {
    Ok(_form) => { /* succès */ }
    Err(form) => { /* contrainte unique sur le champ, sinon erreur globale */ }
}
```

Après une écriture maison qui renvoie un `DbErr`, on l'appelle soi-même :

```rust
if let Err(err) = insert_user(&request.engine.db).await {
    form.database_error(&err);
}
```

**Formats d'erreur supportés :**

- **PostgreSQL** : `UNIQUE constraint`, `Key (field)=(value)`
- **SQLite** : `UNIQUE constraint failed: table.field`
- **MySQL** : `Duplicate entry ... for key 'table.field'`

Si le champ est identifié, l'erreur apparaît sur ce champ (ex: « Ce email est déjà utilisé »). Sinon, elle est ajoutée aux erreurs globales.

Toute autre erreur de base est affichée avec le texte du moteur (`forms.db_error`). Runique ne le remplace pas par une phrase générique : ce texte peut nommer une table, une colonne ou une contrainte. L'échappement du template reste appliqué.

---

← [**Types de champs**](/docs/fr/formulaire/champs) | [**Rendu dans les templates**](/docs/fr/formulaire/templates) →
