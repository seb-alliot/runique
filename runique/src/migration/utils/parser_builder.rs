use syn::{
    Ident, LitStr, Token, braced, bracketed,
    parse::{Parse, ParseStream},
    visit::Visit,
};

use crate::migration::utils::types::{ParsedColumn, ParsedFk, ParsedIndex, ParsedSchema};

// ── Structures de parsing légères ────────────────────────────────────────────

struct DslModel {
    table: String,
    pk: DslPk,
    fields: Vec<DslField>,
    relations: Vec<DslRelation>,
}

struct DslPk {
    name: String,
    ty: String, // "i32", "i64", "uuid"
}

struct DslField {
    name: String,
    ty: String,
    options: Vec<String>,
}

enum DslRelationKind {
    BelongsTo {
        from_column: String,
        on_delete: String,
        on_update: String,
    },
    HasMany,
    HasOne,
    ManyToMany {
        #[allow(dead_code)]
        via: String,
    },
}

struct DslRelation {
    kind: DslRelationKind,
    target: String,
}

// ── Parse impls ───────────────────────────────────────────────────────────────

impl Parse for DslModel {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // NomModele,
        let _name: Ident = input.parse()?;
        input.parse::<Token![,]>()?;

        // table: "...",
        let kw: Ident = input.parse()?;
        if kw != "table" {
            return Err(syn::Error::new(kw.span(), "attendu 'table'"));
        }
        input.parse::<Token![:]>()?;
        let table: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;

        // pk: name => type,
        let kw: Ident = input.parse()?;
        if kw != "pk" {
            return Err(syn::Error::new(kw.span(), "attendu 'pk'"));
        }
        input.parse::<Token![:]>()?;
        let pk_name: Ident = input.parse()?;
        input.parse::<Token![=>]>()?;
        let pk_ty: Ident = input.parse()?;
        input.parse::<Token![,]>()?;

        // fields: { ... }
        let kw: Ident = input.parse()?;
        if kw != "fields" {
            return Err(syn::Error::new(kw.span(), "attendu 'fields'"));
        }
        input.parse::<Token![:]>()?;
        let fields_content;
        braced!(fields_content in input);

        let mut fields = Vec::new();
        while !fields_content.is_empty() {
            fields.push(DslField::parse(&fields_content)?);
        }

        // blocs optionnels : relations, indexes, meta (extensible)
        let mut relations = Vec::new();
        while !input.is_empty() {
            // consume virgule séparatrice
            let _ = input.parse::<Token![,]>();
            if input.is_empty() {
                break;
            }

            // peek next keyword
            if !input.peek(Ident) {
                input.parse::<proc_macro2::TokenTree>().ok();
                continue;
            }
            let kw: Ident = input.parse()?;
            input.parse::<Token![:]>()?;

            let block_content;
            braced!(block_content in input);

            match kw.to_string().as_str() {
                "relations" => {
                    while !block_content.is_empty() {
                        if let Ok(rel) = DslRelation::parse(&block_content) {
                            relations.push(rel);
                        } else {
                            // ignorer une entrée invalide
                            while !block_content.is_empty() && !block_content.peek(Token![,]) {
                                block_content.parse::<proc_macro2::TokenTree>().ok();
                            }
                            let _ = block_content.parse::<Token![,]>();
                        }
                    }
                }
                _ => {
                    // indexes, meta, etc. — ignorer pour l'instant
                    while !block_content.is_empty() {
                        block_content.parse::<proc_macro2::TokenTree>().ok();
                    }
                }
            }
        }

        Ok(DslModel {
            table: table.value(),
            pk: DslPk {
                name: pk_name.to_string(),
                ty: pk_ty.to_string(),
            },
            fields,
            relations,
        })
    }
}

impl Parse for DslField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // name: type [opt1, opt2],
        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty: Ident = input.parse()?;

        let mut options = Vec::new();
        if input.peek(syn::token::Bracket) {
            let opts;
            bracketed!(opts in input);
            while !opts.is_empty() {
                let opt: Ident = opts.parse()?;
                options.push(opt.to_string());
                // consommer les arguments éventuels: max_len(150)
                if opts.peek(syn::token::Paren) {
                    let inner;
                    syn::parenthesized!(inner in opts);
                    while !inner.is_empty() {
                        inner.parse::<proc_macro2::TokenTree>().ok();
                    }
                }
                let _ = opts.parse::<Token![,]>();
            }
        }

        let _ = input.parse::<Token![,]>();
        Ok(DslField {
            name: name.to_string(),
            ty: ty.to_string(),
            options,
        })
    }
}

impl Parse for DslRelation {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // belongs_to: User via user_id [cascade],
        // has_many: Comment,
        // has_one: Profile,
        // many_to_many: Tag via post_tags,
        let kind_kw: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let target: Ident = input.parse()?;

        let rel = match kind_kw.to_string().as_str() {
            "belongs_to" => {
                // via from_column [on_delete, on_update]
                let via_kw: Ident = input.parse()?;
                if via_kw != "via" {
                    return Err(syn::Error::new(
                        via_kw.span(),
                        "attendu 'via' après le modèle cible",
                    ));
                }
                let from_col: Ident = input.parse()?;

                let mut on_delete = "NoAction".to_string();
                let mut on_update = "NoAction".to_string();
                if input.peek(syn::token::Bracket) {
                    let opts;
                    bracketed!(opts in input);
                    let mut i = 0;
                    while !opts.is_empty() {
                        let opt: Ident = opts.parse()?;
                        let val = normalize_fk_action(&opt.to_string());
                        if i == 0 {
                            on_delete = val;
                        } else {
                            on_update = val;
                        }
                        i += 1;
                        let _ = opts.parse::<Token![,]>();
                    }
                }
                let _ = input.parse::<Token![,]>();
                DslRelation {
                    kind: DslRelationKind::BelongsTo {
                        from_column: from_col.to_string(),
                        on_delete,
                        on_update,
                    },
                    target: target.to_string(),
                }
            }
            "has_many" => {
                let _ = input.parse::<Token![,]>();
                DslRelation {
                    kind: DslRelationKind::HasMany,
                    target: target.to_string(),
                }
            }
            "has_one" => {
                let _ = input.parse::<Token![,]>();
                DslRelation {
                    kind: DslRelationKind::HasOne,
                    target: target.to_string(),
                }
            }
            "many_to_many" => {
                let via_kw: Ident = input.parse()?;
                if via_kw != "via" {
                    return Err(syn::Error::new(
                        via_kw.span(),
                        "attendu 'via' après le modèle cible",
                    ));
                }
                let via: Ident = input.parse()?;
                let _ = input.parse::<Token![,]>();
                DslRelation {
                    kind: DslRelationKind::ManyToMany {
                        via: via.to_string(),
                    },
                    target: target.to_string(),
                }
            }
            _ => {
                // entrée inconnue — consommer et ignorer
                while !input.is_empty() && !input.peek(Token![,]) {
                    input.parse::<proc_macro2::TokenTree>().ok();
                }
                let _ = input.parse::<Token![,]>();
                DslRelation {
                    kind: DslRelationKind::HasMany,
                    target: target.to_string(),
                }
            }
        };
        Ok(rel)
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn normalize_fk_action(s: &str) -> String {
    match s.to_lowercase().as_str() {
        "cascade" => "Cascade".to_string(),
        "restrict" => "Restrict".to_string(),
        "set_null" | "setnull" => "SetNull".to_string(),
        "set_default" | "setdefault" => "SetDefault".to_string(),
        _ => "NoAction".to_string(),
    }
}

/// PascalCase → snake_case pour dériver le nom de la table cible
fn pascal_to_snake(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(ch.to_lowercase().next().unwrap());
    }
    result
}

// ── Mapping de types ──────────────────────────────────────────────────────────

fn dsl_field_type_to_col_type(ty: &str) -> String {
    match ty {
        "String" | "text" | "char" | "varchar" => "String".to_string(),
        "i8" => "TinyInteger".to_string(),
        "i16" => "SmallInteger".to_string(),
        "i32" | "integer" => "Integer".to_string(),
        "i64" | "big_integer" => "BigInteger".to_string(),
        "u32" => "Unsigned".to_string(),
        "u64" => "BigUnsigned".to_string(),
        "f32" => "Float".to_string(),
        "f64" => "Double".to_string(),
        "decimal" => "Decimal".to_string(),
        "bool" => "Boolean".to_string(),
        "date" => "Date".to_string(),
        "time" => "Time".to_string(),
        "datetime" | "timestamp" => "DateTime".to_string(),
        "timestamp_tz" => "TimestampWithTimeZone".to_string(),
        "uuid" => "Uuid".to_string(),
        "json" | "json_binary" => "Json".to_string(),
        "binary" | "blob" => "Binary".to_string(),
        "inet" | "cidr" | "mac_address" | "interval" => "String".to_string(),
        _ => "String".to_string(),
    }
}

fn dsl_pk_to_col_type(ty: &str) -> String {
    match ty {
        "i32" => "Integer".to_string(),
        "i64" => "BigInteger".to_string(),
        "uuid" => "Uuid".to_string(),
        _ => "Integer".to_string(),
    }
}

// ── Conversion vers ParsedSchema ──────────────────────────────────────────────

fn dsl_to_parsed_schema(model: DslModel) -> ParsedSchema {
    let primary_key = Some(ParsedColumn {
        name: model.pk.name,
        col_type: dsl_pk_to_col_type(&model.pk.ty),
        nullable: false,
        unique: false,
        ignored: false,
        created_at: false,
        updated_at: false,
        has_default_now: false,
    });

    let columns = model
        .fields
        .into_iter()
        .map(|f| {
            let has_auto_now = f.options.contains(&"auto_now".to_string());
            let has_auto_now_update = f.options.contains(&"auto_now_update".to_string());
            let is_created_at = f.name == "created_at";
            let is_updated_at = f.name == "updated_at";

            let nullable = f.options.contains(&"nullable".to_string())
                && !has_auto_now
                && !has_auto_now_update;

            let unique = f.options.contains(&"unique".to_string());

            let col_type = if has_auto_now || has_auto_now_update {
                "DateTime".to_string()
            } else {
                dsl_field_type_to_col_type(&f.ty)
            };

            let ignored = f.options.contains(&"readonly".to_string()) || f.name == "cache_key";

            ParsedColumn {
                name: f.name,
                col_type,
                nullable,
                unique,
                ignored,
                created_at: has_auto_now || is_created_at,
                updated_at: has_auto_now_update || is_updated_at,
                has_default_now: has_auto_now || has_auto_now_update || is_created_at || is_updated_at,
            }
        })
        .collect();

    // Convertir belongs_to → ParsedFk (DB level)
    let foreign_keys = model
        .relations
        .iter()
        .filter_map(|rel| {
            if let DslRelationKind::BelongsTo {
                from_column,
                on_delete,
                on_update,
            } = &rel.kind
            {
                Some(ParsedFk {
                    from_column: from_column.clone(),
                    to_table: pascal_to_snake(&rel.target),
                    to_column: "id".to_string(),
                    on_delete: on_delete.clone(),
                    on_update: on_update.clone(),
                })
            } else {
                None
            }
        })
        .collect();

    ParsedSchema {
        table_name: model.table,
        primary_key,
        columns,
        foreign_keys,
        indexes: Vec::<ParsedIndex>::new(),
    }
}

// ── Visiteur ──────────────────────────────────────────────────────────────────

struct DslVisitor {
    pub schema: Option<ParsedSchema>,
}

impl DslVisitor {
    fn new() -> Self {
        Self { schema: None }
    }
}

impl<'ast> Visit<'ast> for DslVisitor {
    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        if self.schema.is_some() {
            return;
        }
        let is_model = mac
            .path
            .segments
            .last()
            .map(|s| s.ident == "model")
            .unwrap_or(false);

        if is_model {
            if let Ok(model) = syn::parse2::<DslModel>(mac.tokens.clone()) {
                self.schema = Some(dsl_to_parsed_schema(model));
            }
        }
        syn::visit::visit_macro(self, mac);
    }
}

// ── Point d'entrée public ─────────────────────────────────────────────────────

pub fn parse_schema_from_source(source: &str) -> Option<ParsedSchema> {
    let file = syn::parse_str::<syn::File>(source).ok()?;
    let mut visitor = DslVisitor::new();
    visitor.visit_file(&file);
    visitor.schema
}
