use crate::model::ast::{
    EnumBackingType, EnumDef, EnumVariant, FieldDef, FieldOption, FieldType, FkAction, FkDef,
    MetaDef, ModelInput, PkDef, PkType, RelationDef,
};
use proc_macro2;
use syn::token;
use syn::{
    parse::{Parse, ParseStream},
    Ident, LitFloat, LitInt, LitStr, Result, Token,
};

impl Parse for EnumDef {
    fn parse(input: ParseStream) -> Result<Self> {
        // Status: [Active, Inactive] ou Status: String [Fix="fix"] ou Priority: i32 [Low=1]
        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;

        // Type optionnel : String | i32 | i64 (sinon String implicite)
        let backing_type = if input.peek(Ident) {
            let ty: Ident = input.fork().parse()?;
            match ty.to_string().as_str() {
                "String" => {
                    input.parse::<Ident>()?;
                    EnumBackingType::String
                }
                "i32" => {
                    input.parse::<Ident>()?;
                    EnumBackingType::I32
                }
                "i64" => {
                    input.parse::<Ident>()?;
                    EnumBackingType::I64
                }
                "pg" => {
                    input.parse::<Ident>()?;
                    EnumBackingType::Pg
                }
                _ => EnumBackingType::String,
            }
        } else {
            EnumBackingType::String
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

        // fields: { ... },
        let fields_kw: Ident = input.parse()?;
        assert_eq!(fields_kw.to_string(), "fields");
        input.parse::<Token![:]>()?;
        let fields_content;
        syn::braced!(fields_content in input);
        let mut fields = Vec::new();
        while !fields_content.is_empty() {
            fields.push(FieldDef::parse(&fields_content)?);
        }
        // virgule optionnelle après fields { }
        let _ = input.parse::<Token![,]>();

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

        Ok(ModelInput {
            name,
            table: table.value(),
            pk,
            enums,
            fields,
            relations,
            meta,
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
                ))
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
                        ))
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
                ))
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

                RelationDef::ManyToMany { model, through, via_self }
            }
            other => return Err(syn::Error::new(
                kind.span(),
                format!("Relation inconnue : '{}'. Attendu : belongs_to, has_many, has_one, many_to_many", other),
            )),
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
                    ))
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
