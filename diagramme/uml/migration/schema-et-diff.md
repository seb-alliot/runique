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

### 🟠 M2 — `to_form_field` : fallback `TextField` silencieux — ✅ CORRIGÉ
**Corrigé (2.1.21).** Le fallback `_ => TextField` émet désormais un log `debug`
(« type de colonne non géré → TextField par défaut »). Brancher les types manquants
(binary/inet/interval…) reste un nice-to-have non bloquant.

### 🟡 M3 — `max_size`/`is_file` côté schéma vs AdminForm généré (rappel F2) — ✅ VÉRIFIÉ clean
**Vérifié (2.1.21).** Voir F2 : le plafond modèle (`model_max_size`) borne tout override via
`set_max_size_bounded` (rejet si dépassement). Pas de divergence entre les chemins.
