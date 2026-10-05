//! Converts a `FormFieldDecl` (anonymous block v2) into an equivalent SQL
//! `FieldDef`. SQL types are inferred from semantic types.
use crate::ast::{FieldDef, FieldOption, FormFieldAttr, FormFieldDecl};

pub fn form_field_to_field_def(ff: &FormFieldDecl) -> FieldDef {
    use crate::ast::{FileKind, FormFieldAttr::*, FormFieldKind::*};

    let is_required = ff.attrs.iter().any(|a| matches!(a, Required));
    let is_nullable = ff.attrs.iter().any(|a| matches!(a, Nullable));

    let default = ff.attrs.iter().find_map(|a| {
        if let Default(lit) = a {
            Some(lit.clone())
        } else {
            None
        }
    });
    let upload_to = ff.attrs.iter().find_map(|a| {
        if let UploadTo(p) = a {
            Some(p.clone())
        } else {
            None
        }
    });
    let enum_ref = ff.attrs.iter().find_map(|a| {
        if let EnumRef(id) = a {
            Some(id.clone())
        } else {
            None
        }
    });

    let is_auto_now = ff.attrs.iter().any(|a| matches!(a, AutoNow));
    let is_auto_now_update = ff.attrs.iter().any(|a| matches!(a, AutoNowUpdate));

    // NOT NULL unless `[nullable]`; `required` only makes the form field mandatory.
    let mut options: Vec<FieldOption> = Vec::new();
    if is_auto_now {
        options.push(FieldOption::AutoNow);
    } else if is_auto_now_update {
        options.push(FieldOption::AutoNowUpdate);
    }
    if is_required {
        options.push(FieldOption::Required);
    }
    if is_nullable {
        options.push(FieldOption::Nullable);
    }
    if ff.attrs.iter().any(|a| matches!(a, FormFieldAttr::Unique)) {
        options.push(FieldOption::Unique);
    }
    if let Some(lit) = default {
        options.push(FieldOption::Default(lit));
    }
    if let Some(path) = upload_to {
        let file_kind = match &ff.kind {
            Image => FileKind::Image,
            Document => FileKind::Document,
            _ => FileKind::Any,
        };
        options.push(FieldOption::File {
            kind: file_kind,
            upload_to: Some(path),
        });
    }
    if let Some(FormFieldAttr::MaxLength(n)) = ff
        .attrs
        .iter()
        .find(|a| matches!(a, FormFieldAttr::MaxLength(_)))
    {
        options.push(FieldOption::MaxLen(*n));
    }
    if let Some(FormFieldAttr::MinLength(n)) = ff
        .attrs
        .iter()
        .find(|a| matches!(a, FormFieldAttr::MinLength(_)))
    {
        options.push(FieldOption::MinLen(*n));
    }
    if let Some(FormFieldAttr::Min(n)) =
        ff.attrs.iter().find(|a| matches!(a, FormFieldAttr::Min(_)))
    {
        options.push(FieldOption::Min(*n));
    }
    if let Some(FormFieldAttr::Max(n)) =
        ff.attrs.iter().find(|a| matches!(a, FormFieldAttr::Max(_)))
    {
        options.push(FieldOption::Max(*n));
    }
    if let Some(FormFieldAttr::MinF(n)) = ff
        .attrs
        .iter()
        .find(|a| matches!(a, FormFieldAttr::MinF(_)))
    {
        options.push(FieldOption::MinF(*n));
    }
    if let Some(FormFieldAttr::MaxF(n)) = ff
        .attrs
        .iter()
        .find(|a| matches!(a, FormFieldAttr::MaxF(_)))
    {
        options.push(FieldOption::MaxF(*n));
    }
    if let Some(FormFieldAttr::MaxSize(n)) = ff
        .attrs
        .iter()
        .find(|a| matches!(a, FormFieldAttr::MaxSize(_)))
    {
        options.push(FieldOption::MaxSize(*n));
    }
    if ff
        .attrs
        .iter()
        .any(|a| matches!(a, FormFieldAttr::Readonly))
    {
        options.push(FieldOption::Readonly);
    }
    if let Some(FormFieldAttr::Label(s)) = ff
        .attrs
        .iter()
        .find(|a| matches!(a, FormFieldAttr::Label(_)))
    {
        options.push(FieldOption::Label(s.clone()));
    }

    FieldDef {
        name: ff.name.clone(),
        kind: ff.kind,
        enum_ref,
        options,
    }
}

#[cfg(test)]
mod tests {
    use super::form_field_to_field_def;
    use crate::ast::{FieldType, FormFieldDecl, FormFieldKind};

    fn field(src: &str) -> crate::ast::FieldDef {
        form_field_to_field_def(&syn::parse_str::<FormFieldDecl>(src).expect("parses"))
    }

    // The generated code hashes on the declared type, never on the field's name.
    #[test]
    fn password_keeps_its_own_type_whatever_the_name() {
        let secret = field("secret: password");
        assert_eq!(secret.kind, FormFieldKind::Password);
        assert!(matches!(secret.column_type(), FieldType::String));
        assert_eq!(field("password_hint: text").kind, FormFieldKind::Text);
    }

    #[test]
    fn column_type_follows_the_declared_lengths_and_enum() {
        assert!(matches!(
            field("t: text [max_length: 80]").column_type(),
            FieldType::Varchar(80)
        ));
        assert!(matches!(
            field("e: email").column_type(),
            FieldType::Varchar(254)
        ));
        assert!(matches!(
            field("p: phone").column_type(),
            FieldType::Varchar(20)
        ));
        assert!(matches!(
            field("b: var_binary").column_type(),
            FieldType::VarBinary(255)
        ));
        assert!(matches!(
            field("s: choice [enum(Status), required]").column_type(),
            FieldType::Enum(id) if id == "Status"
        ));
    }

    fn has(def: &crate::ast::FieldDef, wanted: fn(&crate::ast::FieldOption) -> bool) -> bool {
        def.options.iter().any(wanted)
    }

    #[test]
    fn required_nullable_and_auto_now_options() {
        use crate::ast::FieldOption::*;
        let plain = field("a: text");
        assert!(
            !has(&plain, |o| matches!(o, Nullable | Required)),
            "NOT NULL by default"
        );
        let null = field("a: int [nullable]");
        assert!(has(&null, |o| matches!(o, Nullable)) && !has(&null, |o| matches!(o, Required)));
        let req = field("a: int [required]");
        assert!(has(&req, |o| matches!(o, Required)) && !has(&req, |o| matches!(o, Nullable)));
        let created = field("a: datetime [auto_now]");
        assert!(has(&created, |o| matches!(o, AutoNow)));
        assert!(!has(&created, |o| matches!(o, Nullable | Required)));
        let updated = field("a: datetime [auto_now_update]");
        assert!(has(&updated, |o| matches!(o, AutoNowUpdate)));
    }

    #[test]
    fn upload_kind_follows_the_declared_type() {
        use crate::ast::{FieldOption, FileKind};
        let kind = |src: &str| {
            field(src).options.iter().find_map(|o| match o {
                FieldOption::File { kind, .. } => Some(*kind),
                _ => None,
            })
        };
        assert_eq!(kind(r#"a: image [upload_to: "x/"]"#), Some(FileKind::Image));
        assert_eq!(
            kind(r#"a: document [upload_to: "x/"]"#),
            Some(FileKind::Document)
        );
        assert_eq!(kind(r#"a: file [upload_to: "x/"]"#), Some(FileKind::Any));
    }
}
