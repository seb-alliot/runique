# UML — Migration (ColumnDef, ModelSchema, diff)

[`migration/column/mod.rs`](../../../runique/src/migration/column/mod.rs),
[`migration/schema/mod.rs`](../../../runique/src/migration/schema/mod.rs)

```mermaid
classDiagram
    class ModelSchema {
        +String model_name
        +String table_name
        +Option~PrimaryKeyDef~ primary_key
        +Vec~ColumnDef~ columns
        +Vec~ForeignKeyDef~ foreign_keys
        +Vec~RelationDef~ relations
        +Vec~IndexDef~ indexes
        +build() / diff(other) SchemaDiff
        +fill_form(form, fields, exclude)
        +to_migration() / to_model()
    }
    class ColumnDef {
        +String name
        +ColumnType col_type
        +bool nullable / unique / ignored
        +Option~Value~ default
        +Vec~String~ enum_variants
        +Option~u32~ max_length / min_length
        +Option~i64~ max_value / min_value
        +Option~f64~ max_float / min_float
        +bool is_file
        +Option~FileKind~ file_kind
        +Option~u64~ max_size
        +to_sea_column() sea_query::ColumnDef
        +to_form_field() Option~GenericField~
    }
    class SchemaDiff {
        +String table_name
        +Vec~ColumnDef~ added_columns
        +Vec~String~ dropped_columns
    }
    ModelSchema "1" *-- "*" ColumnDef
    ModelSchema ..> SchemaDiff : diff()
    ColumnDef ..> GenericField : to_form_field()
    ColumnDef ..> FileKind
```

Double rôle de `ColumnDef` : génération SQL (`to_sea_column`) **et** génération de champ de
formulaire (`to_form_field`). Les métadonnées `is_file`/`file_kind`/`max_size` sont des
side-fields « form only » ignorés par `to_sea_column` (cf. chantier uploads).

## Anomalies / flux suspects

### 🔴 M1 — `SchemaDiff` ne détecte PAS les colonnes modifiées
[`schema/mod.rs:388`](../../../runique/src/migration/schema/mod.rs#L388)
`SchemaDiff` n'a que `added_columns` et `dropped_columns`. Le `diff()` compare les **ensembles
de noms** de colonnes (`difference`). Conséquence : un changement de **type**, de **nullabilité**,
d'**unicité**, de **default** ou de **longueur** sur une colonne existante **n'est jamais
détecté** → `makemigrations` ne génère **aucun `ALTER COLUMN`**. Le dev croit sa migration
générée alors que le schéma réel diverge du modèle. C'est un faux négatif silencieux, le pire
genre. À confirmer dans le flux makemigrations (03), mais la structure le prouve déjà.

### 🟠 M2 — `to_form_field` : aucune branche pour beaucoup de `ColumnType`
Le `match col_type` retombe sur `_ => TextField::text` pour tout type non explicitement géré
(binary, blob, inet, cidr, interval, json géré, etc.). Un champ `interval`/`binary` devient un
input texte silencieusement. Dégradation muette → au minimum un log/warn serait utile.

### 🟡 M3 — `max_size`/`is_file` côté schéma vs AdminForm généré (rappel F2)
Deux chemins produisent le `FileField` (schéma `to_form_field` et `generate_admin_form`).
Tant que les deux ne partagent pas une source unique, risque de divergence du `max_size`.
