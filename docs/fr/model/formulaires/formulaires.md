# Lien avec les formulaires & enjeux techniques

## Lien avec les formulaires via `#[form(...)]`

La macro attribut `#[form(...)]` attend :

- `schema = chemin_fonction` (obligatoire)
- `fields = [..]` (optionnel)
- `exclude = [..]` (optionnel)

Elle génère uniquement :

- la struct avec `form: Forms`
- `impl ModelForm` (`schema()`, `fields()`, `exclude()`)

Le dev écrit ensuite `impl RuniqueForm` avec `impl_form_access!(model)` :

```rust
use runique::prelude::*;

#[form(schema = user_schema, fields = [username, email])]
pub struct UserForm;

impl RuniqueForm for UserForm {
    impl_form_access!(model);
}
```

### Avec validation métier (`clean`)

Overrider `clean` directement dans `impl RuniqueForm` — comme Django.
`#[async_trait]` est requis uniquement quand on override une méthode async :

```rust
#[form(schema = user_schema, fields = [username, email, password])]
pub struct RegisterForm;

#[async_trait]
impl RuniqueForm for RegisterForm {
    impl_form_access!(model);

    async fn clean(&mut self) -> Result<(), StrMap> {
        let mut errors = StrMap::new();
        if self.get_string("username").len() < 3 {
            errors.insert("username".to_string(), "Minimum 3 caractères".to_string());
        }
        if !self.get_string("email").contains('@') {
            errors.insert("email".to_string(), "Email invalide".to_string());
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}
```

> `is_valid()` appelle automatiquement `clean` après la validation structurelle.
> Les erreurs retournées sont attachées aux champs et affichées inline dans le template.

---

## Enjeux techniques

### Avantages

- Contrat unique modèle/schéma centralisé
- Génération cohérente migration + formulaire
- Réduction de duplication de définition de champs
- `clean` est l'override officiel du trait — uniforme entre formulaires manuels et basés modèle

### Points d'attention

- DSL stricte : erreur de syntaxe = erreur de macro au build
- `fields`/`exclude` mal alignés avec le schéma => erreurs de génération/exécution
- `#[async_trait]` requis sur `impl RuniqueForm` uniquement quand on override `clean` ou `clean_field`

---

## Voir aussi

| Section | Description |
| --- | --- |
| [DSL & AST](https://github.com/seb-alliot/runique/blob/main/docs/fr/model/dsl/dsl.md) | Syntaxe `model!`, types, options |
| [Génération & ModelSchema](https://github.com/seb-alliot/runique/blob/main/docs/fr/model/generation/generation.md) | Code généré |

## Retour au sommaire

- [Models](https://github.com/seb-alliot/runique/blob/main/docs/fr/model/12-model.md)
