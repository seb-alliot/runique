# Génération & ModelSchema

## Ce que génère `model!(...)`

À partir du DSL lu par `runique_dsl`, la macro génère notamment :

- l'entité SeaORM (`Model`, `Entity`, `Column`, `ActiveModel`) et ses relations (`belongs_to` vise la vraie clé primaire de la cible, avec ses actions) ;
- l'`ActiveModelBehavior` qui remplit `auto_now` / `auto_now_update` ;
- un enum Rust par entrée du bloc `enums:` ;
- le formulaire du modèle (`{Modele}ModelForm`, alias `Form`) et les fonctions utilisées par l'admin ;
- pour chaque champ liste (`multichoice` / `checkbox`), l'entité de sa table et les méthodes `champ()`, `set_champ()`, `load_champ()` ;
- une fonction `schema() -> ModelSchema` ;
- une constante `PRIMARY_KEY`, le nom de la clé primaire (`#[form]` refuse ce nom dans `fields`).

```rust
pub fn schema() -> runique::migration::schema::ModelSchema {
    runique::migration::ModelSchema::new("User")
        .table_name("users")
        // pk, colonnes (types, bornes, enums), FK, index, meta...
        .build()
        .unwrap()
}
```

---

## Rôle de `ModelSchema`

`ModelSchema` décrit les **colonnes** du modèle (types, bornes déclarées, enums, clés étrangères, index) pour les formulaires. Il ne sert pas aux migrations : `makemigrations` lit directement le DSL. Les champs liste n'en font pas partie, puisqu'ils n'ont pas de colonne.

### Méthodes utilisées par les formulaires

- `fill_form(form, fields, exclude)` : remplit un formulaire `#[form(schema = ...)]` à partir du schéma ;
- `enforce_limits(form)` : appelée après `customize`, vérifie que le formulaire n'a pas desserré ce que le modèle déclare (`min_length`, `max_length`, `min`, `max`, chiffres d'un `decimal`, type entier, champ mot de passe) — sinon panic. Elle tourne à chaque construction du formulaire : l'erreur apparaît à la première requête qui l'utilise, pas au démarrage. Resserrer une limite reste permis.

### Comportement de `fill_form`

- la PK n'est jamais un champ : elle ne fait pas partie des colonnes du schéma. La citer dans `fields = [...]` compile, mais fait paniquer la construction du formulaire (« field does not exist in the schema »),
- si `fields` est fourni : whitelist prioritaire (ordre conservé),
- sinon `exclude` sert de blacklist.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [DSL & AST](/docs/fr/model/dsl) | Syntaxe `model!`, types, options |
| [Formulaires & enjeux](/docs/fr/model/formulaires) | `#[form(...)]` |

## Retour au sommaire

- [Models](/docs/fr/model)
