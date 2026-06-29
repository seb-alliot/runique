# UML — Formulaires (Forms, champs, ModelForm, Prisme)

## Forms + hiérarchie de champs

[`runique/src/forms/form.rs:30`](../../../runique/src/forms/form.rs#L30),
[`base.rs`](../../../runique/src/forms/base.rs), [`generic.rs`](../../../runique/src/forms/generic.rs)

```mermaid
classDiagram
    class Forms {
        +FieldsMap fields
        +Vec~String~ errors
        +String session_csrf_token
        -Option~FormRenderer~ renderer
        -bool submitted
        -bool validated
        -HashMap path_params
        -HashMap query_params
        -bool force_invalid
        -Option~String~ honeypot_field_name
        +field_label(name, v) &mut Self
        +field_placeholder(name, v) &mut Self
        +field_required/readonly/disabled(name, b) &mut Self
        +field_attr(name, k, v) &mut Self
        +field_max_size(name, size) Result
        +fill(raw, method)
        +finalize() / validate() / render()
    }
    class FormField {
        <<trait>>
        +name()/label()/placeholder()/value()
        +set_label()/set_placeholder()/set_value()
        +validate() bool
        +render()
        +model_max_size() Option~u64~
        +set_max_size_bounded(size) Result
    }
    class GenericField {
        +FieldKind kind
    }
    class FieldKind {
        <<enum>>
        Text(TextField)
        Numeric(NumericField)
        File(FileField)
        Boolean / Choice / Radio / Checkbox
        Date / Time / DateTime / Duration
        Color / Slug / UUID / JSON / IPAddress / Hidden
    }
    Forms "1" o-- "*" GenericField : FieldsMap
    GenericField "1" *-- "1" FieldKind
    GenericField ..|> FormField
    FieldKind ..> FileField
    FileField ..|> FormField
    TextField ..|> FormField
```

## Pipeline form & traits de définition

```mermaid
classDiagram
    class RuniqueForm {
        <<trait>>
        +register_fields(form)*
        +from_form(form) Self*
        +get_form() / get_form_mut()*
        +customize(form) [défaut no-op]
        +label/placeholder/required/readonly/disabled/attr(name,..)
        +cleaned_string/i32/i64/...(name)
    }
    class ModelForm {
        <<trait>>
        +schema() ModelSchema*
        +fields() Option~&[&str]~
        +exclude() Option~&[&str]~
        +model_register_fields(form)
    }
    class FileField {
        +FieldConfig base
        +FileFieldType field_type
        +AllowedExtensions allowed_extensions
        +FileUploadConfig upload_config
        +Option~u64~ model_max_size
        +max_size(FileSize) Self
        -apply_max_size_bounded(size) Result
    }
    RuniqueForm ..> Forms
    ModelForm ..> ModelSchema : schema().fill_form()
    ModelForm ..> RuniqueForm : via impl_form_access!(model)
```

Flux de construction d'un `#[form(schema=…)]` :
`build/build_with_data` → `register_fields` → `model_register_fields` →
`ModelSchema::fill_form` → `ColumnDef::to_form_field` (recrée chaque champ) →
**`Self::customize(form)`** (hook ajouté) → `fill(raw)` → `validate`.

## Anomalies / flux suspects

### 🟡 F1 — `customize` câblé seulement sur l'arm `(model)` — ✅ VÉRIFIÉ (voulu)
Le hook `customize` n'est appelé que dans le `register_fields` généré par
`impl_form_access!(model)`. **C'est voulu** : `customize` ne sert qu'aux forms générés depuis
un modèle ; un form écrit à la main (arms `()`/`($field)`) gère ses champs directement.
**Bug corrigé au passage (2.1.21)** : l'arm `(model)` était **dupliqué** dans la macro (2ᵉ copie
sans l'appel `customize`, inatteignable car `macro_rules!` prend le 1ᵉ match) → doublon mort supprimé.

### 🟡 F2 — `max_size` du modèle vs override de form — ✅ VÉRIFIÉ clean
**Vérifié (2.1.21).** Le modèle pose le plafond (`model_max_size`), l'override de form passe par
`Forms::field_max_size` → `set_max_size_bounded` → `apply_max_size_bounded` qui **rejette si
l'override dépasse le plafond modèle**. Les deux chemins sont réconciliés (pas en concurrence) :
override possible vers le bas, jamais au-dessus du plafond. Pas de divergence.

### 🟠 F3 — `force_invalid` / honeypot : ordre vs `fill`/`validate` — ✅ VÉRIFIÉ clean
**Vérifié (2.1.21).** `force_invalid` est posé à la construction du form (honeypot + CSRF)
**avant** `fill()` (qui n'y touche jamais) ; `is_valid()` court-circuite dessus avant toute
validation, et `is_save_allowed()` double-garde. Honeypot POST-only mais la garde CSRF couvre
toutes les méthodes mutantes. Pas de contournement possible.
