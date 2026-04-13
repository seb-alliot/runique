use crate::model::ast::{
    EnumBackingType, EnumDef, EnumVariant, FieldDef, FieldOption, FieldType, FkAction, FkDef,
    FormFieldAttr, FormFieldDecl, FormFieldKind, MetaDef, ModelInput, PkDef, PkType, RelationDef,
};
use proc_macro2;
use syn::token;
use syn::{
    Ident, LitFloat, LitInt, LitStr, Result, Token,
    parse::{Parse, ParseStream},
};

impl Parse for EnumDef {
    fn parse(input: ParseStream) -> Result<Self> {
        // Status: [Active, Inactive] ou Status: String [Fix="fix"] ou Priority: i32 [Low=1]
        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;

        // Type optionnel : String | i32 | i64 (sinon Auto — détecté depuis .env)
        let backing_type = if input.peek(Ident) {
            let ty: Ident = input.fork().parse()?;
            match ty.to_string().as_str() {
                "String" => {
                    let ident: Ident = input.parse()?;
                    return Err(syn::Error::new(
                        ident.span(),
                        "`String` est obsolète comme type d'enum. \
                        Retirez-le — le comportement correct est détecté automatiquement \
                        depuis DATABASE_URL dans `.env` (natif Postgres ou VARCHAR).",
                    ));
                }
                "i32" => {
                    input.parse::<Ident>()?;
                    EnumBackingType::I32
                }
                "i64" => {
                    input.parse::<Ident>()?;
                    EnumBackingType::I64
                }
                _ => EnumBackingType::Auto,
            }
        } else {
            EnumBackingType::Auto
        };

        let content;
        syn::bracketed!(content in input);
        let mut variants = Vec::new();
        while !content.is_empty() {
            let variant_name: Ident = content.parse()?;
            let (value, label) = if content.peek(Token![=]) {
                content.parse::<Token![=]>()?;
                if content.peek(syn::token::Paren) {
                    let inner;
                    syn::parenthesized!(inner in content);
                    let db_val: syn::Lit = inner.parse()?;
                    inner.parse::<Token![,]>()?;
                    let lbl: syn::Lit = inner.parse()?;
                    (Some(db_val), Some(lbl))
                } else {
                    (Some(content.parse::<syn::Lit>()?), None)
                }
            } else {
                (None, None)
            };
            variants.push(EnumVariant {
                name: variant_name,
                value,
                label,
            });
            let _ = content.parse::<Token![,]>();
        }
        let _ = input.parse::<Token![,]>();
        Ok(EnumDef {
            name,
            backing_type,
            variants,
        })
    }
}

// ── Parse ModelInput ──────────────────────────────────────────
impl Parse for ModelInput {
    fn parse(input: ParseStream) -> Result<Self> {
        // EihwazUsers,
        let name: Ident = input.parse()?;
        input.parse::<Token![,]>()?;

        // table: "eihwaz_users",
        let table_kw: Ident = input.parse()?;
        assert_eq!(table_kw.to_string(), "table");
        input.parse::<Token![:]>()?;
        let table: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;

        // pk: id => i32,
        let pk_kw: Ident = input.parse()?;
        assert_eq!(pk_kw.to_string(), "pk");
        input.parse::<Token![:]>()?;
        let pk = PkDef::parse(input)?;
        input.parse::<Token![,]>()?;

        // enums: { ... } optionnel
        let mut enums = Vec::new();
        if input.peek(Ident) {
            let peek: Ident = input.fork().parse()?;
            if peek == "enums" {
                input.parse::<Ident>()?;
                input.parse::<Token![:]>()?;
                let enum_content;
                syn::braced!(enum_content in input);
                while !enum_content.is_empty() {
                    enums.push(EnumDef::parse(&enum_content)?);
                }
                let _ = input.parse::<Token![,]>();
            }
        }

        // Détection du style :
        // Nouveau (v2) : bloc anonyme `{ ... }` — types sémantiques, dérivent le SQL
        // Ancien (v1)  : `fields: { ... }` — types SQL explicites
        let mut fields = Vec::new();
        let mut form_fields_early: Vec<FormFieldDecl> = Vec::new();

        if input.peek(syn::token::Brace) {
            // ── Nouveau style : bloc anonyme ──────────────────────
            let ff_content;
            syn::braced!(ff_content in input);
            while !ff_content.is_empty() {
                let ff = FormFieldDecl::parse(&ff_content)?;
                fields.push(form_field_to_field_def(&ff));
                form_fields_early.push(ff);
            }
            let _ = input.parse::<Token![,]>();
        } else {
            // ── Ancien style : fields: { ... } ────────────────────
            let fields_kw: Ident = input.parse()?;
            if fields_kw != "fields" {
                return Err(syn::Error::new(
                    fields_kw.span(),
                    "Attendu : `fields` ou un bloc `{ ... }`",
                ));
            }
            input.parse::<Token![:]>()?;
            let fields_content;
            syn::braced!(fields_content in input);
            while !fields_content.is_empty() {
                fields.push(FieldDef::parse(&fields_content)?);
            }
            let _ = input.parse::<Token![,]>();
        }

        // relations: { ... } optionnel
        let mut relations = Vec::new();
        if input.peek(Ident) {
            let peek: Ident = input.fork().parse()?;
            if peek == "relations" {
                input.parse::<Ident>()?;
                input.parse::<Token![:]>()?;
                let rel_content;
                syn::braced!(rel_content in input);
                while !rel_content.is_empty() {
                    relations.push(RelationDef::parse(&rel_content)?);
                }
                let _ = input.parse::<Token![,]>();
            }
        }

        // meta: { ... } optionnel
        let mut meta = None;
        if input.peek(Ident) {
            let peek: Ident = input.fork().parse()?;
            if peek == "meta" {
                input.parse::<Ident>()?;
                input.parse::<Token![:]>()?;
                let meta_content;
                syn::braced!(meta_content in input);
                meta = Some(MetaDef::parse(&meta_content)?);
            }
        }

        // form_fields: { ... } optionnel (ancien style uniquement — ignoré si bloc anonyme déjà parsé)
        let mut form_fields = if form_fields_early.is_empty() {
            let mut ff = Vec::new();
            if input.peek(Ident) {
                let peek: Ident = input.fork().parse()?;
                if peek == "form_fields" {
                    input.parse::<Ident>()?;
                    input.parse::<Token![:]>()?;
                    let ff_content;
                    syn::braced!(ff_content in input);
                    while !ff_content.is_empty() {
                        ff.push(FormFieldDecl::parse(&ff_content)?);
                    }
                    let _ = input.parse::<Token![,]>();
                }
            }
            ff
        } else {
            form_fields_early
        };
        let _ = &mut form_fields; // silence unused_mut

        Ok(ModelInput {
            name,
            table: table.value(),
            pk,
            enums,
            fields,
            relations,
            meta,
            form_fields,
        })
    }
}

impl Parse for PkDef {
    fn parse(input: ParseStream) -> Result<Self> {
        // id => i32
        let name: Ident = input.parse()?;
        input.parse::<Token![=>]>()?;
        let ty_ident: Ident = input.parse()?;
        let ty = match ty_ident.to_string().as_str() {
            "i32" => PkType::I32,
            "i64" => PkType::I64,
            "uuid" => PkType::Uuid,
            "Pk" => {
                #[cfg(feature = "big-pk")]
                {
                    PkType::I64
                }
                #[cfg(not(feature = "big-pk"))]
                {
                    PkType::I32
                }
            }
            other => {
                return Err(syn::Error::new(
                    ty_ident.span(),
                    format!(
                        "Type de PK inconnu : '{}'. Attendu : i32, i64, Pk, uuid",
                        other
                    ),
                ));
            }
        };
        Ok(PkDef { name, ty })
    }
}

impl Parse for FieldDef {
    fn parse(input: ParseStream) -> Result<Self> {
        // username: String [required, max_len(150)],
        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty = FieldType::parse(input)?;

        // options [ ... ] optionnelles
        let mut options = Vec::new();
        if input.peek(token::Bracket) {
            let options_content;
            syn::bracketed!(options_content in input);
            while !options_content.is_empty() {
                options.push(FieldOption::parse(&options_content)?);
                let _ = options_content.parse::<Token![,]>();
            }
        }

        // virgule de fin
        let _ = input.parse::<Token![,]>();

        Ok(FieldDef { name, ty, options })
    }
}

impl Parse for FieldType {
    fn parse(input: ParseStream) -> Result<Self> {
        // `enum` est un mot-clé Rust — traitement séparé
        if input.peek(Token![enum]) {
            input.parse::<Token![enum]>()?;
            let content;
            syn::parenthesized!(content in input);
            let name: Ident = content.parse()?;
            return Ok(FieldType::Enum(name));
        }

        let ty_ident: Ident = input.parse()?;
        match ty_ident.to_string().as_str() {
            "String" => Ok(FieldType::String),
            "text" => Ok(FieldType::Text),
            "char" => Ok(FieldType::Char),
            "varchar" => {
                let content;
                syn::parenthesized!(content in input);
                let n: LitInt = content.parse()?;
                Ok(FieldType::Varchar(n.base10_parse()?))
            }
            "i8" => Ok(FieldType::I8),
            "i16" => Ok(FieldType::I16),
            "i32" => Ok(FieldType::I32),
            "i64" => Ok(FieldType::I64),
            "u32" => Ok(FieldType::U32),
            "u64" => Ok(FieldType::U64),
            "f32" => Ok(FieldType::F32),
            "f64" => Ok(FieldType::F64),
            "decimal" => {
                if input.peek(token::Paren) {
                    let content;
                    syn::parenthesized!(content in input);
                    let precision: LitInt = content.parse()?;
                    content.parse::<Token![,]>()?;
                    let scale: LitInt = content.parse()?;
                    Ok(FieldType::Decimal(Some((
                        precision.base10_parse()?,
                        scale.base10_parse()?,
                    ))))
                } else {
                    Ok(FieldType::Decimal(None))
                }
            }
            "bool" => Ok(FieldType::Bool),
            "date" => Ok(FieldType::Date),
            "time" => Ok(FieldType::Time),
            "datetime" => Ok(FieldType::Datetime),
            "timestamp" => Ok(FieldType::Timestamp),
            "timestamp_tz" => Ok(FieldType::TimestampTz),
            "uuid" => Ok(FieldType::Uuid),
            "json" => Ok(FieldType::Json),
            "json_binary" => Ok(FieldType::JsonBinary),
            "binary" => {
                if input.peek(token::Paren) {
                    let content;
                    syn::parenthesized!(content in input);
                    let n: LitInt = content.parse()?;
                    Ok(FieldType::Binary(Some(n.base10_parse()?)))
                } else {
                    Ok(FieldType::Binary(None))
                }
            }
            "var_binary" => {
                let content;
                syn::parenthesized!(content in input);
                let n: LitInt = content.parse()?;
                Ok(FieldType::VarBinary(n.base10_parse()?))
            }
            "blob" => Ok(FieldType::Blob),
            "inet" => Ok(FieldType::Inet),
            "cidr" => Ok(FieldType::Cidr),
            "mac_address" => Ok(FieldType::MacAddress),
            "interval" => Ok(FieldType::Interval),
            other => Err(syn::Error::new(
                ty_ident.span(),
                format!("Type de champ inconnu : '{}'", other),
            )),
        }
    }
}

impl Parse for FieldOption {
    fn parse(input: ParseStream) -> Result<Self> {
        let option_ident: Ident = input.parse()?;
        match option_ident.to_string().as_str() {
            "required" => Ok(FieldOption::Required),
            "nullable" => Ok(FieldOption::Nullable),
            "unique" => Ok(FieldOption::Unique),
            "index" => Ok(FieldOption::Index),
            "auto_now" => Ok(FieldOption::AutoNow),
            "auto_now_update" => Ok(FieldOption::AutoNowUpdate),
            "readonly" => Ok(FieldOption::Readonly),
            "default" => {
                let content;
                syn::parenthesized!(content in input);
                let lit: syn::Lit = content.parse()?;
                Ok(FieldOption::Default(lit))
            }
            "max_len" => {
                let content;
                syn::parenthesized!(content in input);
                let n: LitInt = content.parse()?;
                Ok(FieldOption::MaxLen(n.base10_parse()?))
            }
            "min_len" => {
                let content;
                syn::parenthesized!(content in input);
                let n: LitInt = content.parse()?;
                Ok(FieldOption::MinLen(n.base10_parse()?))
            }
            "max" => {
                let content;
                syn::parenthesized!(content in input);
                let n: LitInt = content.parse()?;
                Ok(FieldOption::Max(n.base10_parse()?))
            }
            "min" => {
                let content;
                syn::parenthesized!(content in input);
                let n: LitInt = content.parse()?;
                Ok(FieldOption::Min(n.base10_parse()?))
            }
            "max_f" => {
                let content;
                syn::parenthesized!(content in input);
                let n: LitFloat = content.parse()?;
                Ok(FieldOption::MaxF(n.base10_parse()?))
            }
            "min_f" => {
                let content;
                syn::parenthesized!(content in input);
                let n: LitFloat = content.parse()?;
                Ok(FieldOption::MinF(n.base10_parse()?))
            }
            "select_as" => {
                let content;
                syn::parenthesized!(content in input);
                let s: LitStr = content.parse()?;
                Ok(FieldOption::SelectAs(s.value()))
            }
            "label" => {
                let content;
                syn::parenthesized!(content in input);
                let s: LitStr = content.parse()?;
                Ok(FieldOption::Label(s.value()))
            }
            "help" => {
                let content;
                syn::parenthesized!(content in input);
                let s: LitStr = content.parse()?;
                Ok(FieldOption::Help(s.value()))
            }
            "fk" => {
                let content;
                syn::parenthesized!(content in input);
                let fk = FkDef::parse(&content)?;
                Ok(FieldOption::Fk(fk))
            }
            "max_size" | "max_size_mb" => {
                let content;
                syn::parenthesized!(content in input);
                let n = parse_size(&content)?;
                Ok(FieldOption::MaxSize(n))
            }
            "file" => {
                let content;
                syn::parenthesized!(content in input);
                let kind_ident: Ident = content.parse()?;
                let kind = match kind_ident.to_string().as_str() {
                    "image" => crate::model::ast::FileKind::Image,
                    "document" => crate::model::ast::FileKind::Document,
                    "any" => crate::model::ast::FileKind::Any,
                    other => {
                        return Err(syn::Error::new(
                            kind_ident.span(),
                            format!(
                                "Type de fichier inconnu : '{}'. Attendu : image, document, any",
                                other
                            ),
                        ));
                    }
                };
                let upload_to = if content.peek(Token![,]) {
                    content.parse::<Token![,]>()?;
                    let s: LitStr = content.parse()?;
                    Some(s.value())
                } else {
                    None
                };
                Ok(FieldOption::File { kind, upload_to })
            }
            other => Err(syn::Error::new(
                option_ident.span(),
                format!("Option inconnue : '{}'", other),
            )),
        }
    }
}

impl Parse for FkDef {
    fn parse(input: ParseStream) -> Result<Self> {
        // users.id, cascade
        let table: Ident = input.parse()?;
        input.parse::<Token![.]>()?;
        let column: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let action_ident: Ident = input.parse()?;
        let action = match action_ident.to_string().as_str() {
            "cascade" => FkAction::Cascade,
            "set_null" => FkAction::SetNull,
            "restrict" => FkAction::Restrict,
            "set_default" => FkAction::SetDefault,
            other => {
                return Err(syn::Error::new(
                    action_ident.span(),
                    format!(
                        "Action FK inconnue : '{}'. Attendu : cascade, set_null, restrict, set_default",
                        other
                    ),
                ));
            }
        };
        Ok(FkDef {
            table,
            column,
            action,
        })
    }
}

impl Parse for RelationDef {
    fn parse(input: ParseStream) -> Result<Self> {
        // belongs_to: users via user_id,
        // has_many: comments as user_comments,
        // has_one: profile as user_profile,
        // many_to_many: roles through user_roles,
        let kind: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let model: Ident = input.parse()?;

        let relation = match kind.to_string().as_str() {
            "belongs_to" => {
                let via_kw: Ident = input.parse()?;
                if via_kw != "via" {
                    return Err(syn::Error::new(via_kw.span(), "Attendu : 'via'"));
                }
                let via: Ident = input.parse()?;
                // options FK optionnelles : [cascade], [cascade, restrict], etc.
                // consommées ici, gérées par le système de migration
                if input.peek(syn::token::Bracket) {
                    let opts;
                    syn::bracketed!(opts in input);
                    while !opts.is_empty() {
                        opts.parse::<proc_macro2::TokenTree>().ok();
                    }
                }
                RelationDef::BelongsTo { model, via }
            }
            "has_many" => {
                let as_name = if input.peek(Ident) {
                    let as_kw: Ident = input.fork().parse()?;
                    if as_kw == "as" {
                        input.parse::<Ident>()?;
                        Some(input.parse::<Ident>()?)
                    } else {
                        None
                    }
                } else {
                    None
                };
                RelationDef::HasMany { model, as_name }
            }
            "has_one" => {
                let as_name = if input.peek(Ident) {
                    let as_kw: Ident = input.fork().parse()?;
                    if as_kw == "as" {
                        input.parse::<Ident>()?;
                        Some(input.parse::<Ident>()?)
                    } else {
                        None
                    }
                } else {
                    None
                };
                RelationDef::HasOne { model, as_name }
            }
            "many_to_many" => {
                let through_kw: Ident = input.parse()?;
                if through_kw != "through" {
                    return Err(syn::Error::new(through_kw.span(), "Attendu : 'through'"));
                }
                let through: Ident = input.parse()?;

                // ← nouveau : via ViaIdent
                let via_kw: Ident = input.parse()?;
                if via_kw != "via" {
                    return Err(syn::Error::new(via_kw.span(), "Attendu : 'via'"));
                }
                let via_self: Ident = input.parse()?;

                RelationDef::ManyToMany {
                    model,
                    through,
                    via_self,
                }
            }
            other => {
                return Err(syn::Error::new(
                    kind.span(),
                    format!(
                        "Relation inconnue : '{}'. Attendu : belongs_to, has_many, has_one, many_to_many",
                        other
                    ),
                ));
            }
        };

        let _ = input.parse::<Token![,]>();
        Ok(relation)
    }
}

impl Parse for MetaDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut ordering = Vec::new();
        let mut unique_together = Vec::new();
        let mut verbose_name = None;
        let mut verbose_name_plural = None;
        let mut abstract_model = false;
        let mut indexes = Vec::new();

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![:]>()?;

            match key.to_string().as_str() {
                "ordering" => {
                    let content;
                    syn::bracketed!(content in input);
                    while !content.is_empty() {
                        // - = DESC
                        let desc = content.peek(Token![-]);
                        if desc {
                            content.parse::<Token![-]>()?;
                        }
                        let field: Ident = content.parse()?;
                        ordering.push((desc, field));
                        let _ = content.parse::<Token![,]>();
                    }
                }
                "unique_together" => {
                    let content;
                    syn::bracketed!(content in input);
                    while !content.is_empty() {
                        let tuple_content;
                        syn::parenthesized!(tuple_content in content);
                        let mut group = Vec::new();
                        while !tuple_content.is_empty() {
                            group.push(tuple_content.parse::<Ident>()?);
                            let _ = tuple_content.parse::<Token![,]>();
                        }
                        unique_together.push(group);
                        let _ = content.parse::<Token![,]>();
                    }
                }
                "verbose_name" => {
                    let s: LitStr = input.parse()?;
                    verbose_name = Some(s.value());
                }
                "verbose_name_plural" => {
                    let s: LitStr = input.parse()?;
                    verbose_name_plural = Some(s.value());
                }
                "abstract" => {
                    let b: syn::LitBool = input.parse()?;
                    abstract_model = b.value();
                }
                "indexes" => {
                    let content;
                    syn::bracketed!(content in input);
                    while !content.is_empty() {
                        let tuple_content;
                        syn::parenthesized!(tuple_content in content);
                        let mut group = Vec::new();
                        while !tuple_content.is_empty() {
                            group.push(tuple_content.parse::<Ident>()?);
                            let _ = tuple_content.parse::<Token![,]>();
                        }
                        indexes.push(group);
                        let _ = content.parse::<Token![,]>();
                    }
                }
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("Clé meta inconnue : '{}'", other),
                    ));
                }
            }

            let _ = input.parse::<Token![,]>();
        }

        Ok(MetaDef {
            ordering,
            unique_together,
            verbose_name,
            verbose_name_plural,
            abstract_model,
            indexes,
        })
    }
}

// ── Dérivation FieldDef depuis FormFieldDecl ──────────────────

/// Convertit un `FormFieldDecl` (bloc anonyme v2) en `FieldDef` SQL équivalent.
/// Les types SQL sont déduits des types sémantiques.
fn form_field_to_field_def(ff: &FormFieldDecl) -> FieldDef {
    use crate::model::ast::{FileKind, FormFieldAttr::*, FormFieldKind::*};

    let is_required = ff.attrs.iter().any(|a| matches!(a, Required));
    let is_nullable = ff.attrs.iter().any(|a| matches!(a, Nullable)) || !is_required; // sans required → nullable implicite

    let max_len = ff
        .attrs
        .iter()
        .find_map(|a| if let MaxLength(n) = a { Some(*n) } else { None });
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

    let ty = match &ff.kind {
        Text => {
            if let Some(n) = max_len {
                FieldType::Varchar(n)
            } else {
                FieldType::String
            }
        }
        Email => FieldType::Varchar(254),
        Password => FieldType::String,
        Richtext | Textarea | Json => FieldType::Text,
        Url => FieldType::String,
        Int => FieldType::I32,
        Float => FieldType::F64,
        Decimal => FieldType::Decimal(None),
        Percent => FieldType::F64,
        Bool => FieldType::Bool,
        Date => FieldType::Date,
        Time => FieldType::Time,
        Datetime => FieldType::Datetime,
        Uuid => FieldType::Uuid,
        Ip => FieldType::Inet,
        Color | Slug => FieldType::String,
        Image | Document | File => FieldType::String,
        Choice | Radio => {
            if let Some(ident) = enum_ref {
                FieldType::Enum(ident)
            } else {
                FieldType::String
            }
        }
        Bigint => FieldType::I64,
    };

    let is_auto_now = ff.attrs.iter().any(|a| matches!(a, AutoNow));
    let is_auto_now_update = ff.attrs.iter().any(|a| matches!(a, AutoNowUpdate));

    let mut options: Vec<FieldOption> = Vec::new();
    if is_auto_now {
        options.push(FieldOption::AutoNow);
    } else if is_auto_now_update {
        options.push(FieldOption::AutoNowUpdate);
    } else if is_required && !is_nullable {
        options.push(FieldOption::Required);
    } else if is_nullable && !is_required {
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

    FieldDef {
        name: ff.name.clone(),
        ty,
        options,
    }
}

// ── form_fields: parsing ──────────────────────────────────────

impl Parse for FormFieldDecl {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;

        let kind_ident: Ident = input.parse()?;
        let kind = match kind_ident.to_string().as_str() {
            "text" => FormFieldKind::Text,
            "email" => FormFieldKind::Email,
            "password" => FormFieldKind::Password,
            "richtext" => FormFieldKind::Richtext,
            "textarea" => FormFieldKind::Textarea,
            "url" => FormFieldKind::Url,
            "int" => FormFieldKind::Int,
            "float" => FormFieldKind::Float,
            "decimal" => FormFieldKind::Decimal,
            "percent" => FormFieldKind::Percent,
            "bool" => FormFieldKind::Bool,
            "date" => FormFieldKind::Date,
            "time" => FormFieldKind::Time,
            "datetime" => FormFieldKind::Datetime,
            "image" => FormFieldKind::Image,
            "document" => FormFieldKind::Document,
            "file" => FormFieldKind::File,
            "color" => FormFieldKind::Color,
            "slug" => FormFieldKind::Slug,
            "uuid" => FormFieldKind::Uuid,
            "json" => FormFieldKind::Json,
            "ip" => FormFieldKind::Ip,
            "choice" => FormFieldKind::Choice,
            "radio" => FormFieldKind::Radio,
            "bigint" => FormFieldKind::Bigint,
            other => {
                let suggestion = suggest_form_field_type(other);
                return Err(syn::Error::new(
                    kind_ident.span(),
                    format!(
                        "Type de champ inconnu : '{}' (champ: {}){}",
                        other, name, suggestion
                    ),
                ));
            }
        };

        // Attributs optionnels [ ... ]
        let mut attrs = Vec::new();
        if input.peek(token::Bracket) {
            let attrs_content;
            syn::bracketed!(attrs_content in input);
            while !attrs_content.is_empty() {
                // `enum` est un mot-clé Rust — traitement séparé avant le match sur Ident
                if attrs_content.peek(Token![enum]) {
                    attrs_content.parse::<Token![enum]>()?;
                    let content;
                    syn::parenthesized!(content in attrs_content);
                    let ident: Ident = content.parse()?;
                    attrs.push(FormFieldAttr::EnumRef(ident));
                    let _ = attrs_content.parse::<Token![,]>();
                    continue;
                }

                let attr_ident: Ident = attrs_content.parse()?;
                let attr = match attr_ident.to_string().as_str() {
                    "required" => FormFieldAttr::Required,
                    "nullable" => FormFieldAttr::Nullable,
                    "no_hash" => FormFieldAttr::NoHash,
                    "max_length" => {
                        attrs_content.parse::<Token![:]>()?;
                        let n: LitInt = attrs_content.parse()?;
                        FormFieldAttr::MaxLength(n.base10_parse()?)
                    }
                    "min_length" => {
                        attrs_content.parse::<Token![:]>()?;
                        let n: LitInt = attrs_content.parse()?;
                        FormFieldAttr::MinLength(n.base10_parse()?)
                    }
                    "min" => {
                        attrs_content.parse::<Token![:]>()?;
                        if attrs_content.peek(LitFloat) {
                            let n: LitFloat = attrs_content.parse()?;
                            FormFieldAttr::MinF(n.base10_parse()?)
                        } else {
                            let n: LitInt = attrs_content.parse()?;
                            FormFieldAttr::Min(n.base10_parse()?)
                        }
                    }
                    "max" => {
                        attrs_content.parse::<Token![:]>()?;
                        if attrs_content.peek(LitFloat) {
                            let n: LitFloat = attrs_content.parse()?;
                            FormFieldAttr::MaxF(n.base10_parse()?)
                        } else {
                            let n: LitInt = attrs_content.parse()?;
                            FormFieldAttr::Max(n.base10_parse()?)
                        }
                    }
                    "default" => {
                        attrs_content.parse::<Token![:]>()?;
                        let lit: syn::Lit = attrs_content.parse()?;
                        FormFieldAttr::Default(lit)
                    }
                    "upload_to" => {
                        attrs_content.parse::<Token![:]>()?;
                        let s: LitStr = attrs_content.parse()?;
                        FormFieldAttr::UploadTo(s.value())
                    }
                    "max_size" | "max_size_mb" => {
                        attrs_content.parse::<Token![:]>()?;
                        let n = parse_size(&attrs_content)?;
                        FormFieldAttr::MaxSize(n)
                    }
                    "rows" => {
                        attrs_content.parse::<Token![:]>()?;
                        let n: LitInt = attrs_content.parse()?;
                        FormFieldAttr::Rows(n.base10_parse()?)
                    }
                    "step" => {
                        attrs_content.parse::<Token![:]>()?;
                        let n: LitFloat = attrs_content.parse()?;
                        FormFieldAttr::Step(n.base10_parse()?)
                    }
                    "auto_now" => FormFieldAttr::AutoNow,
                    "auto_now_update" => FormFieldAttr::AutoNowUpdate,
                    "unique" => FormFieldAttr::Unique,
                    other => {
                        return Err(syn::Error::new(
                            attr_ident.span(),
                            format!("Attribut inconnu : '{}' (champ: {})", other, name),
                        ));
                    }
                };
                attrs.push(attr);
                let _ = attrs_content.parse::<Token![,]>();
            }
        }

        // Validation attributs vs type
        validate_form_field_attrs(&name, &kind_ident, &kind, &attrs)?;

        let _ = input.parse::<Token![,]>();
        Ok(FormFieldDecl { name, kind, attrs })
    }
}

fn suggest_form_field_type(input: &str) -> String {
    let known = [
        "text", "email", "password", "richtext", "textarea", "url", "int", "bigint", "float",
        "decimal", "percent", "bool", "date", "time", "datetime", "image", "document", "file",
        "color", "slug", "uuid", "json", "ip",
    ];
    // Suggestion par préfixe commun (≥ 3 caractères)
    let matches: Vec<&str> = known
        .iter()
        .filter(|&&k| {
            let min_len = k.len().min(input.len()).min(4);
            min_len >= 2 && k[..min_len] == input[..min_len.min(input.len())]
        })
        .copied()
        .collect();
    if matches.is_empty() {
        String::new()
    } else {
        format!(" — vouliez-vous dire `{}` ?", matches[0])
    }
}

fn validate_form_field_attrs(
    name: &Ident,
    kind_ident: &Ident,
    kind: &FormFieldKind,
    attrs: &[FormFieldAttr],
) -> Result<()> {
    use FormFieldAttr::*;
    use FormFieldKind::*;

    let kind_name = kind_ident.to_string();

    for attr in attrs {
        let valid = match (attr, kind) {
            // required / nullable / EnumRef — universels
            (Required, _) => true,
            (Nullable, _) => true,
            (EnumRef(_), FormFieldKind::Choice | FormFieldKind::Radio) => true,
            (EnumRef(_), _) => false,

            // no_hash — password uniquement
            (NoHash, Password) => true,
            (NoHash, _) => false,

            // max_length / min_length — types textuels
            (MaxLength(_), Text | Email | Password | Richtext | Textarea | Url) => true,
            (MaxLength(_), _) => false,
            (MinLength(_), Text | Textarea) => true,
            (MinLength(_), _) => false,

            // min / max entier — int uniquement
            (Min(_), Int) => true,
            (Min(_), _) => false,
            (Max(_), Int) => true,
            (Max(_), _) => false,

            // min_f / max_f flottant — float, decimal
            (MinF(_), Float | Decimal) => true,
            (MinF(_), _) => false,
            (MaxF(_), Float | Decimal) => true,
            (MaxF(_), _) => false,

            // default — tout sauf fichiers
            (Default(_), Image | Document | File) => false,
            (Default(_), _) => true,

            // upload_to / max_size_mb — fichiers uniquement
            (UploadTo(_), Image | Document | File) => true,
            (UploadTo(_), _) => false,
            (MaxSize(_), Image | Document | File) => true,
            (MaxSize(_), _) => false,

            // rows — types multilignes
            (Rows(_), Richtext | Textarea | Json) => true,
            (Rows(_), _) => false,

            // step — float, decimal
            (Step(_), Float | Decimal) => true,
            (Step(_), _) => false,

            // auto_now / auto_now_update — datetime uniquement
            (AutoNow, Datetime) => true,
            (AutoNow, _) => false,
            (AutoNowUpdate, Datetime) => true,
            (AutoNowUpdate, _) => false,

            // unique — tous les types sauf fichiers/bool
            (Unique, Image | Document | File | Bool) => false,
            (Unique, _) => true,
        };

        if !valid {
            let attr_name = attr_name_str(attr);
            return Err(syn::Error::new(
                kind_ident.span(),
                format!(
                    "`{}` n'est pas valide pour le type `{}` (champ: {})",
                    attr_name, kind_name, name
                ),
            ));
        }
    }

    // upload_to requis pour image / document / file
    if matches!(kind, Image | Document | File) && !attrs.iter().any(|a| matches!(a, UploadTo(_))) {
        return Err(syn::Error::new(
            name.span(),
            format!(
                "`upload_to` est requis pour les champs `{}` (champ: {})",
                kind_name, name
            ),
        ));
    }

    // min < max pour int
    let min_val = attrs
        .iter()
        .find_map(|a| if let Min(v) = a { Some(*v) } else { None });
    let max_val = attrs
        .iter()
        .find_map(|a| if let Max(v) = a { Some(*v) } else { None });
    if let (Some(mn), Some(mx)) = (min_val, max_val)
        && mn >= mx
    {
        return Err(syn::Error::new(
            name.span(),
            format!(
                "`min` doit être inférieur à `max` (champ: {}, min={}, max={})",
                name, mn, mx
            ),
        ));
    }

    // min_f < max_f pour float/decimal
    let min_f_val = attrs
        .iter()
        .find_map(|a| if let MinF(v) = a { Some(*v) } else { None });
    let max_f_val = attrs
        .iter()
        .find_map(|a| if let MaxF(v) = a { Some(*v) } else { None });
    if let (Some(mn), Some(mx)) = (min_f_val, max_f_val)
        && mn >= mx
    {
        return Err(syn::Error::new(
            name.span(),
            format!(
                "`min` doit être inférieur à `max` (champ: {}, min={}, max={})",
                name, mn, mx
            ),
        ));
    }

    Ok(())
}

fn attr_name_str(attr: &FormFieldAttr) -> &'static str {
    match attr {
        FormFieldAttr::Required => "required",
        FormFieldAttr::Nullable => "nullable",
        FormFieldAttr::NoHash => "no_hash",
        FormFieldAttr::EnumRef(_) => "enum",
        FormFieldAttr::MaxLength(_) => "max_length",
        FormFieldAttr::MinLength(_) => "min_length",
        FormFieldAttr::Min(_) => "min",
        FormFieldAttr::Max(_) => "max",
        FormFieldAttr::MinF(_) => "min",
        FormFieldAttr::MaxF(_) => "max",
        FormFieldAttr::Default(_) => "default",
        FormFieldAttr::UploadTo(_) => "upload_to",
        FormFieldAttr::MaxSize(_) => "max_size",
        FormFieldAttr::Rows(_) => "rows",
        FormFieldAttr::Step(_) => "step",
        FormFieldAttr::AutoNow => "auto_now",
        FormFieldAttr::AutoNowUpdate => "auto_now_update",
        FormFieldAttr::Unique => "unique",
    }
}

/// Parse une taille avec unité (KB, MB, GB). Retourne la valeur en Octets.
/// Sans unité, la valeur est traitée comme des MO (MB) par défaut pour la compatibilité.
fn parse_size(input: ParseStream) -> Result<u64> {
    let n: LitInt = input.parse()?;
    let value = n.base10_parse::<u64>()?;

    if input.peek(Ident) {
        let unit: Ident = input.parse()?;
        let unit_str = unit.to_string().to_uppercase();
        match unit_str.as_str() {
            "KB" | "K" | "KO" => Ok(value * 1024),
            "MB" | "M" | "MO" => Ok(value * 1024 * 1024),
            "GB" | "G" | "GO" => Ok(value * 1024 * 1024 * 1024),
            _ => Err(syn::Error::new(
                unit.span(),
                "Unité de taille inconnue. Attendu : KB, MB, GB (ou K, M, G, KO, MO, GO)",
            )),
        }
    } else {
        // Défaut : MB pour la compatibilité avec l'existant
        Ok(value * 1024 * 1024)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parse un `FormFieldDecl` depuis une chaîne DSL.
    /// Format : `nom: type [attr1, attr2, ...]`
    fn parse_field(src: &str) -> syn::Result<FormFieldDecl> {
        syn::parse_str::<FormFieldDecl>(src)
    }

    fn ok(src: &str) {
        assert!(
            parse_field(src).is_ok(),
            "attendu OK mais erreur pour : `{src}`"
        );
    }

    fn err(src: &str) {
        assert!(
            parse_field(src).is_err(),
            "attendu ERREUR mais OK pour : `{src}`"
        );
    }

    // ── max_length ────────────────────────────────────────────────

    #[test]
    fn max_length_valide_sur_text() {
        ok("nom: text [max_length: 100]");
    }

    #[test]
    fn max_length_valide_sur_email() {
        ok("email: email [max_length: 254]");
    }

    #[test]
    fn max_length_valide_sur_password() {
        ok("mdp: password [max_length: 128]");
    }

    #[test]
    fn max_length_valide_sur_url() {
        ok("site: url [max_length: 200]");
    }

    #[test]
    fn max_length_invalide_sur_int() {
        err("age: int [max_length: 50]");
    }

    #[test]
    fn max_length_invalide_sur_float() {
        err("prix: float [max_length: 10]");
    }

    #[test]
    fn max_length_invalide_sur_bool() {
        err("actif: bool [max_length: 5]");
    }

    #[test]
    fn max_length_invalide_sur_date() {
        err("naissance: date [max_length: 10]");
    }

    #[test]
    fn max_length_invalide_sur_uuid() {
        err("uid: uuid [max_length: 36]");
    }

    // ── min / max (entier) ────────────────────────────────────────

    #[test]
    fn min_max_valide_sur_int() {
        ok("age: int [min: 0, max: 150]");
    }

    #[test]
    fn min_invalide_sur_text() {
        err("nom: text [min: 0]");
    }

    #[test]
    fn max_invalide_sur_float() {
        // max entier (i64) n'est pas valide sur float — doit utiliser max flottant
        err("prix: float [max: 100]");
    }

    #[test]
    fn min_invalide_sur_bool() {
        err("flag: bool [min: 0]");
    }

    #[test]
    fn min_invalide_sur_date() {
        err("jour: date [min: 0]");
    }

    #[test]
    fn min_egal_max_rejete() {
        err("age: int [min: 5, max: 5]");
    }

    #[test]
    fn min_superieur_max_rejete() {
        err("age: int [min: 10, max: 5]");
    }

    #[test]
    fn min_inferieur_max_accepte() {
        ok("age: int [min: 0, max: 120]");
    }

    // ── min / max (flottant) ──────────────────────────────────────

    #[test]
    fn min_f_max_f_valide_sur_float() {
        ok("note: float [min: 0.0, max: 20.0]");
    }

    #[test]
    fn min_f_max_f_valide_sur_decimal() {
        ok("prix: decimal [min: 0.0, max: 9999.99]");
    }

    #[test]
    fn min_f_invalide_sur_int() {
        // float min sur un champ int → invalide
        err("age: int [min: 0.0]");
    }

    #[test]
    fn min_f_egal_max_f_rejete() {
        err("note: float [min: 5.0, max: 5.0]");
    }

    // ── upload_to ─────────────────────────────────────────────────

    #[test]
    fn upload_to_valide_sur_image() {
        ok(r#"avatar: image [upload_to: "avatars/"]"#);
    }

    #[test]
    fn upload_to_valide_sur_document() {
        ok(r#"cv: document [upload_to: "docs/"]"#);
    }

    #[test]
    fn upload_to_valide_sur_file() {
        ok(r#"piece: file [upload_to: "files/"]"#);
    }

    #[test]
    fn upload_to_requis_sur_image_sans_lui() {
        err("avatar: image []");
    }

    #[test]
    fn upload_to_requis_sur_document_sans_lui() {
        err("cv: document []");
    }

    #[test]
    fn upload_to_invalide_sur_text() {
        err(r#"nom: text [upload_to: "path/"]"#);
    }

    #[test]
    fn upload_to_invalide_sur_int() {
        err(r#"age: int [upload_to: "path/"]"#);
    }

    #[test]
    fn upload_to_invalide_sur_email() {
        err(r#"mail: email [upload_to: "path/"]"#);
    }

    // ── cas généraux ──────────────────────────────────────────────

    #[test]
    fn champ_sans_attrs() {
        ok("nom: text");
    }

    #[test]
    fn required_universel() {
        ok("age: int [required]");
        ok("nom: text [required]");
        ok("flag: bool [required]");
    }

    #[test]
    fn nullable_universel() {
        ok("age: int [nullable]");
        ok("nom: text [nullable]");
    }
}
