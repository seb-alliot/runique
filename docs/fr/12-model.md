# Models et AST (`model!`)

> Fichiers concernés : `src/entities/*`

## 1) Macros réellement exposées

Dans la crate `derive_form`, les macros disponibles sont :

- `model!(...)` (proc macro)
- `#[form(...)]` (proc macro attribut)
- `#[derive(DeriveModelForm)]`

Côté API Runique (`prelude`), `model` et `form` sont ré-exportées.

## 2) DSL `model!(...)` : structure attendue

Le parseur attend une structure stricte :

1. nom du modèle,
2. `table: "..."`,
3. `pk: id => i32|i64|uuid`,
4. `fields: { ... }`,
5. `relations: { ... }` optionnel,
6. `meta: { ... }` optionnel.

Exemple concret :

```rust
use runique::prelude::*;

model!(
    User,
    table: "users",
    pk: id => i32,
    fields: {
        username: String [required, max_len(150), unique],
        email: String [required, unique],
        password: String [required],
        is_active: bool [default(true)],
        created_at: datetime [auto_now],
    },
    relations: {
        has_many(Post),
    },
    meta: {
        ordering: [-created_at],
    }
);
```

## 3) AST interne (ce qui est parsé)

La DSL est convertie en AST `Model` avec notamment :

- `name`, `table`, `pk`
- `fields: Vec<FieldDef>`
- `relations: Vec<RelationDef>`
- `meta: Option<MetaDef>`

Types pris en charge (extraits) :

- texte : `String`, `text`, `char`, `varchar(n)`
- numériques : `i8/i16/i32/i64/u32/u64/f32/f64`, `decimal(p,s)`
- date/temps : `date`, `time`, `datetime`, `timestamp`, `timestamp_tz`
- autres : `bool`, `uuid`, `json`, `json_binary`, `binary`, `blob`, `enum(...)`, `inet`, `cidr`

Options de champ (extraits) :

- `required`, `nullable`, `unique`, `index`
- `default(...)`, `max_len(...)`, `min_len(...)`, `max(...)`, `min(...)`
- `auto_now`, `auto_now_update`, `readonly`
- `fk(table.column, cascade|set_null|restrict|set_default)`

## 4) Génération produite par `model!(...)`

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

## 5) Rôle de `ModelSchema`

`ModelSchema` est la source de vérité structurelle (table, PK, colonnes, FK, relations, index).

Méthodes importantes côté runtime :

- `to_migration()` : génère le statement de migration
- `fill_form(form, fields, exclude)` : remplit un formulaire à partir du schéma

Comportement de `fill_form` :

- la PK est toujours exclue,
- si `fields` est fourni : whitelist prioritaire (ordre conservé),
- sinon `exclude` sert de blacklist.

## 6) Lien avec les formulaires via `#[form(...)]`

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

## 7) Enjeux techniques

### Avantages

- Contrat unique modèle/schéma centralisé
- Génération cohérente migration + formulaire
- Réduction de duplication de définition de champs

### Points d’attention

- DSL stricte : erreur de syntaxe = erreur de macro au build
- `fields`/`exclude` mal alignés avec le schéma => erreurs de génération/exécution
- Ordre pédagogique important : comprendre `model/schema` avant la méthode formulaire basée modèle

## 8) Ordre recommandé de lecture

1. Ce document (`model.md`)
2. ORM (`07-orm.md`) pour l’usage DB
3. Formulaires (`05-forms.md`) pour l’intégration Prisme et rendu