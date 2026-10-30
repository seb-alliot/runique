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

Une erreur qui contient encore SQLx est classée, puis remplacée par une phrase traduite. Le texte du moteur est journalisé.

- **Unicité** : sur le champ quand le message ou la contrainte nomme la colonne (« Ce email est déjà utilisé »), sinon une phrase globale.
- **Clé étrangère, valeur obligatoire, contrainte** : une phrase globale chacune. L'exclusion Postgres prend celle de la contrainte.
- **Le reste** : une phrase générique (`forms.db_error`), sans le texte du moteur.

Une `DbErr::Custom` est une phrase déjà choisie (refus du gate, enregistrement lié, champ illisible). Elle est affichée telle quelle.

---

← [**Types de champs**](/docs/fr/formulaire/champs) | [**Rendu dans les templates**](/docs/fr/formulaire/templates) →
