# UML — derive_form (proc-macro : `model!{}` / `#[form]` / `extend!{}`)

Crate séparée [`derive_form/`](../../../runique/derive_form/). Flux :
`DSL source → parser (syn) → AST → generateur → TokenStream Rust`.

## AST (model)

[`derive_form/src/model/ast.rs`](../../../runique/derive_form/src/model/ast.rs)

```mermaid
classDiagram
    class ModelInput {
        +Ident name
        +String table
        +Vec~FieldDef~ fields
        +Vec~EnumDef~ enums
        +PkDef pk
        +Vec~RelationDef~ relations
        +MetaDef meta
    }
    class FieldDef {
        +Ident name
        +FieldType ty
        +Vec~FieldOption~ options
    }
    class FieldType {
        <<enum>> String/Text/Varchar/I32/I64/F64/Decimal/Bool/Datetime/Uuid/Json/Enum…
    }
    class FieldOption {
        <<enum>> Required/Unique/Default/MaxLen/MaxSize/File{kind,upload_to}/Fk…
    }
    class FileKind { <<enum>> Image/Document/Any }
    class EnumDef { +Ident name +Vec~Variant~ variants +EnumBackingType }
    class PkDef
    class RelationDef { <<enum>> BelongsTo/HasMany/HasOne }
    class FkDef { +Ident table +Ident column +FkAction action }
    ModelInput "1" *-- "*" FieldDef
    ModelInput "1" *-- "*" EnumDef
    ModelInput "1" *-- "*" RelationDef
    ModelInput "1" *-- "1" PkDef
    FieldDef "1" *-- "1" FieldType
    FieldDef "1" *-- "*" FieldOption
    FieldOption ..> FileKind
    FieldOption ..> FkDef
```

## Pipeline d'expansion

```mermaid
flowchart LR
    SRC[DSL model!/extend!/#form] --> PAR[parser.rs syn]
    PAR -->|syn::Error spanné| CE[compile_error! inline]
    PAR --> AST[AST: ModelInput…]
    AST --> GEN[generateur.rs]
    GEN --> ENT[Entity SeaORM]
    GEN --> COL[ColumnDef migration .file/.max_size_bytes]
    GEN --> FORM[AdminForm + FileField]
    GEN --> SCH[schema() → ModelSchema]
    REG[registry.rs phantom builtins] --> GEN
```

`#[form(schema=Path)]` délègue à `Schema::schema()` au **runtime** (ne lit pas le `max_size`
du modèle à l'expansion — cf. discussion uploads). Le registre fantôme ne couvre que les
tables builtin `eihwaz_*` (name/type/widget, **pas** `max_size`).

## Anomalies / flux suspects

### 🟡 DF1 — Validation des bornes d'override impossible à l'expansion cross-macro
`#[form]` n'a que le `Path` du schéma → un override DSL de `max_size` ne peut pas être
comparé au plafond modèle à la compilation (faute de littéral). Compile-error possible
uniquement via émission d'une `const` par `model!{}` + `const assert!` côté override
(non implémenté). Borne runtime (`set_max_size_bounded`) en place.

### 🟢 DF2 — Erreurs DSL spannées (audit clean)
Le parser émet `syn::Error::new(span, msg)` → `compile_error!` pointé sur le token fautif,
visible inline dans rust-analyzer. Bonne ergonomie, rien à corriger.

### 🟢 DF3 — Générateur : `let _ = write!(buf, …)` = bénins
Les ~308 `let _ =` du générateur/parsers écrivent dans une `String` (infaillible). Pas des
erreurs avalées.
