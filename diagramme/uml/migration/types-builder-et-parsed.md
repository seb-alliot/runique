# UML — Migration : defs builder + types parsés/diff

Complément de [schema-et-diff.md](schema-et-diff.md) (ColumnDef/ModelSchema/SchemaDiff).

## Defs de schéma (builder)

[`migration/{primary_key,foreign_key,index,relation,hooks}`](../../../runique/src/migration/)

```mermaid
classDiagram
    class PrimaryKeyDef {
        +String name
        +ColumnType col_type
        +bool auto_increment
    }
    class ForeignKeyDef {
        +String from_column
        +String to_table / to_column
        +ForeignKeyAction on_delete / on_update
        +references() / to_column() / on_delete()
    }
    class IndexDef {
        +Vec~String~ columns
        +bool unique
        +Option~String~ name
    }
    class RelationDef {
        +RelationKind kind
        +String target
        +has_one/has_many/belongs_to/many_to_many()
    }
    class RelationKind {
        <<enum>> HasOne / HasMany / BelongsTo{from,to} / ManyToMany{via}
    }
    class HooksDef {
        +Vec~Hook~ hooks
        +Option~String~ file_path
    }
    class Hook {
        +HookType hook_type
        +u8 slot
        +String handler_path
    }
    class HookType {
        <<enum>> BeforeSave / AfterSave / BeforeDelete / AfterDelete
    }
    RelationDef *-- RelationKind
    HooksDef *-- "*" Hook
    Hook *-- HookType
```

`ModelSchema` agrège : `Vec<ColumnDef>`, `Option<PrimaryKeyDef>`, `Vec<ForeignKeyDef>`,
`Vec<IndexDef>`, `Vec<RelationDef>` (cf. schema-et-diff.md).

## Types parsés + diff (`migration/utils/types.rs`)

```mermaid
classDiagram
    class ParsedSchema {
        +String table_name
        +Option~ParsedColumn~ primary_key
        +Vec~ParsedColumn~ columns
        +Vec~ParsedFk~ foreign_keys
        +Vec~ParsedIndex~ indexes
    }
    class ParsedColumn {
        +String name / col_type
        +bool nullable / unique / ignored
        +bool created_at / updated_at / has_default_now
        +Option~String~ default_value / enum_name / renamed_from
        +Vec~String~ enum_string_values
        +bool enum_is_pg
    }
    class ParsedFk { +from_column +to_table +to_column +on_delete +on_update }
    class ParsedIndex { +name +Vec~String~ columns +unique }
    class Changes {
        +String table_name
        +Vec~ParsedColumn~ added_columns / dropped_columns
        +Vec~(ParsedColumn,ParsedColumn)~ modified_columns
        +Vec~(String,String)~ renamed_columns
        +Vec~ParsedFk~ added_fks / dropped_fks
        +Vec~ParsedIndex~ added_indexes / dropped_indexes
        +bool is_new_table
        +enum_renames / enum_value_adds / enum_value_drops
    }
    class DbKind { <<enum>> Postgres / Mysql / Other }
    ParsedSchema *-- "*" ParsedColumn
    ParsedSchema *-- "*" ParsedFk
    ParsedSchema *-- "*" ParsedIndex
    Changes ..> ParsedColumn
    Changes ..> ParsedFk
    Changes ..> ParsedIndex
```

## Anomalies / flux suspects

### ✅ Confirmation — `Changes` est le vrai diff (AM1/M1 = faux positifs)
`diff_schemas` produit un `Changes` complet : `modified_columns`, `renamed_columns`
(RENAME COLUMN sans perte), `added/dropped_fks`, `added/dropped_indexes`, `enum_renames`,
`enum_value_adds/drops`. La détection de modification existe bien — le `ModelSchema::diff`
limité (add/drop) n'est qu'un diff secondaire non utilisé par la CLI.

### 🟢 Note — `ParsedColumn.renamed_from` transient (design sain)
`renamed_from` vit uniquement dans le modèle source, jamais écrit en snapshot → consommé par
le diff pour émettre `RENAME COLUMN` au lieu de DROP+ADD (préserve les données). Bonne
conception, pas d'anomalie.
