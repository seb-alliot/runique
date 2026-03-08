# Lien avec les formulaires & enjeux techniques

## Lien avec les formulaires via `#[form(...)]`

La macro attribut `#[form(...)]` attend :

- `schema = chemin_fonction` (obligatoire)
- `fields = [..]` (optionnel)
- `exclude = [..]` (optionnel)

Exemple concret :

```rust
use runique::prelude::*;

#[form(schema = user_schema, fields = ["username", "email"], exclude = ["is_active"])]
pub struct UserForm;
```

Cette macro génère :

- une struct avec `form: Forms`,
- `impl ModelForm` (`schema()`, `fields()`, `exclude()`),
- `impl RuniqueForm` qui délègue à `ModelForm::model_register_fields(...)`.

---

## Enjeux techniques

### Avantages

- Contrat unique modèle/schéma centralisé
- Génération cohérente migration + formulaire
- Réduction de duplication de définition de champs

### Points d'attention

- DSL stricte : erreur de syntaxe = erreur de macro au build
- `fields`/`exclude` mal alignés avec le schéma => erreurs de génération/exécution
- Ordre pédagogique important : comprendre `model/schema` avant la méthode formulaire basée modèle

---

## Voir aussi

| Section | Description |
| --- | --- |
| [DSL & AST](https://github.com/seb-alliot/runique/blob/main/docs/fr/model/dsl/dsl.md) | Syntaxe `model!`, types, options |
| [Génération & ModelSchema](https://github.com/seb-alliot/runique/blob/main/docs/fr/model/generation/generation.md) | Code généré |

## Retour au sommaire

- [Models](https://github.com/seb-alliot/runique/blob/main/docs/fr/model/12-model.md)
