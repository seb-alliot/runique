# Génération & ModelSchema

## Génération produite par `model!(...)`

Après parsing AST, la génération construit entre autres :

- une fonction `schema() -> ModelSchema`
- le modèle SeaORM (code généré)
- les relations associées

La fonction `schema()` générée suit ce pattern :

```rust
pub fn schema() -> runique::migration::schema::ModelSchema {
    runique::migration::ModelSchema::new("User")
        .table_name("users")
        // pk, colonnes, FK, relations, meta...
        .build()
        .unwrap()
}
```

---

## Rôle de `ModelSchema`

`ModelSchema` est la source de vérité structurelle (table, PK, colonnes, FK, relations, index).

### Méthodes importantes côté runtime

- `to_migration()` : génère le statement de migration
- `fill_form(form, fields, exclude)` : remplit un formulaire à partir du schéma

### Comportement de `fill_form`

- la PK est toujours exclue,
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
