//! AST Parser for the builder DSL (`ModelSchema`) — extracts the `ParsedSchema` from Rust source code.
//! Supports two syntaxes:
//!   v1: `fields: { name: String [required], ... }`
//!   v2: `{ name: text [required], ... }` (anonymous block, semantic types)
use syn::{
    Ident, LitStr, Token, braced, bracketed,
    parse::{Parse, ParseStream},
    visit::Visit,
};

use crate::migration::utils::types::{ParsedColumn, ParsedFk, ParsedIndex, ParsedSchema};

// ── Lightweight parsing structures ────────────────────────────────────────────

struct DslModel {
    table: String,
    pk: DslPk,
    enum_types: Vec<(String, String, Vec<String>)>, // (enum_name, backing_type, string_values)
    fields: Vec<DslField>,
    relations: Vec<DslRelation>,
}

struct DslPk {
    name: String,
    ty: String, // "i32", "i64", "uuid", "Pk"
}

struct DslField {
    name: String,
    ty: String,
    enum_name: Option<String>, // from type (v1: `enum(X)`) or attrs (v2: `choice [enum(X)]`)
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
        // ModelName,
        let _name: Ident = input.parse()?;
        input.parse::<Token![,]>()?;

        // table: "...",
        let kw: Ident = input.parse()?;
        if kw != "table" {
            return Err(syn::Error::new(kw.span(), "expected 'table'"));
        }
        input.parse::<Token![:]>()?;
        let table: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;

        // pk: name => type,
        let kw: Ident = input.parse()?;
        if kw != "pk" {
            return Err(syn::Error::new(kw.span(), "expected 'pk'"));
        }
        input.parse::<Token![:]>()?;
        let pk_name: Ident = input.parse()?;
        input.parse::<Token![=>]>()?;
        let pk_ty: Ident = input.parse()?;
        input.parse::<Token![,]>()?;

        // enums: { ... } optional
        let mut enum_types: Vec<(String, String, Vec<String>)> = Vec::new();
        if input.peek(Ident) {
            let peek: Ident = input.fork().parse()?;
            if peek == "enums" {
                input.parse::<Ident>()?;
                input.parse::<Token![:]>()?;
                let enum_block;
                braced!(enum_block in input);
                while !enum_block.is_empty() {
                    if let Ok(enum_name) = enum_block.parse::<Ident>() {
                        let _ = enum_block.parse::<Token![:]>();
                        // optional type: i32 | i64 (String and pg are obsolete but tolerated here)
                        let backing = if enum_block.peek(Ident) {
                            let ty: Ident = enum_block.fork().parse().unwrap();
                            match ty.to_string().as_str() {
                                "String" | "i32" | "i64" | "pg" => {
                                    enum_block.parse::<Ident>().ok();
                                    ty.to_string()
                                }
                                _ => "Auto".to_string(),
                            }
                        } else {
                            "Auto".to_string()
                        };
                        // [Name] or [Name="value", ...] variants
                        let mut string_values: Vec<String> = Vec::new();
                        if enum_block.peek(syn::token::Bracket) {
                            let variants;
                            bracketed!(variants in enum_block);
                            while !variants.is_empty() {
                                if let Ok(vname) = variants.parse::<Ident>() {
                                    if variants.peek(Token![=]) {
                                        let _ = variants.parse::<Token![=]>();
                                        if let Ok(lit) = variants.parse::<syn::Lit>() {
                                            match lit {
                                                syn::Lit::Str(s) => string_values.push(s.value()),
                                                syn::Lit::Int(n) => {
                                                    string_values.push(n.to_string())
                                                }
                                                _ => string_values.push(vname.to_string()),
                                            }
                                        } else {
                                            string_values.push(vname.to_string());
                                        }
                                    } else {
                                        string_values.push(vname.to_string());
                                    }
                                } else {
                                    variants.parse::<proc_macro2::TokenTree>().ok();
                                }
                                let _ = variants.parse::<Token![,]>();
                            }
                        }
                        enum_types.push((enum_name.to_string(), backing, string_values));
                        let _ = enum_block.parse::<Token![,]>();
                    } else {
                        enum_block.parse::<proc_macro2::TokenTree>().ok();
                    }
                }
                let _ = input.parse::<Token![,]>();
            }
        }

        // Style detection:
        //   v2 — anonymous `{ ... }` block (semantic types)
        //   v1 — `fields: { ... }` block (SQL types)
        let mut fields = Vec::new();
        if input.peek(syn::token::Brace) {
            // v2
            let fields_content;
            braced!(fields_content in input);
            while !fields_content.is_empty() {
                fields.push(DslField::parse(&fields_content)?);
            }
        } else {
            // v1
            let kw: Ident = input.parse()?;
            if kw != "fields" {
                return Err(syn::Error::new(
                    kw.span(),
                    "expected 'fields' or a `{ ... }` block",
                ));
            }
            input.parse::<Token![:]>()?;
            let fields_content;
            braced!(fields_content in input);
            while !fields_content.is_empty() {
                fields.push(DslField::parse(&fields_content)?);
            }
        }

        // optional blocks: relations, indexes, meta, form_fields (ignored), etc.
        let mut relations = Vec::new();
        while !input.is_empty() {
            let _ = input.parse::<Token![,]>();
            if input.is_empty() {
                break;
            }
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
                            while !block_content.is_empty() && !block_content.peek(Token![,]) {
                                block_content.parse::<proc_macro2::TokenTree>().ok();
                            }
                            let _ = block_content.parse::<Token![,]>();
                        }
                    }
                }
                _ => {
                    // form_fields, indexes, meta, etc. — ignored
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
            enum_types,
            fields,
            relations,
        })
    }
}

impl Parse for DslField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // name: type [opt1, opt2, ...],
        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;

        // `enum` is a Rust keyword — separate handling (v1: `enum(X)` in type position)
        let (ty, mut enum_name) = if input.peek(Token![enum]) {
            input.parse::<Token![enum]>()?;
            let inner;
            syn::parenthesized!(inner in input);
            let ename: Ident = inner.parse()?;
            ("enum".to_string(), Some(ename.to_string()))
        } else {
            let ty: Ident = input.parse()?;
            (ty.to_string(), None)
        };

        let mut options = Vec::new();
        if input.peek(syn::token::Bracket) {
            let opts;
            bracketed!(opts in input);
            while !opts.is_empty() {
                // v2: `enum(X)` in attr position (choice/radio)
                if opts.peek(Token![enum]) {
                    opts.parse::<Token![enum]>()?;
                    let inner;
                    syn::parenthesized!(inner in opts);
                    let ename: Ident = inner.parse()?;
                    enum_name = Some(ename.to_string());
                    let _ = opts.parse::<Token![,]>();
                    continue;
                }

                let opt: Ident = opts.parse()?;
                options.push(opt.to_string());

                // paren syntax (v1): max_len(150)
                if opts.peek(syn::token::Paren) {
                    let inner;
                    syn::parenthesized!(inner in opts);
                    while !inner.is_empty() {
                        inner.parse::<proc_macro2::TokenTree>().ok();
                    }
                }
                // colon syntax (v2): max_length: 150, rows: 6, upload_to: "path"
                else if opts.peek(Token![:]) {
                    opts.parse::<Token![:]>()?;
                    opts.parse::<proc_macro2::TokenTree>().ok();
                }

                let _ = opts.parse::<Token![,]>();
            }
        }

        let _ = input.parse::<Token![,]>();
        Ok(DslField {
            name: name.to_string(),
            ty,
            enum_name,
            options,
        })
    }
}

impl Parse for DslRelation {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let kind_kw: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let target: Ident = input.parse()?;

        let rel = match kind_kw.to_string().as_str() {
            "belongs_to" => {
                let via_kw: Ident = input.parse()?;
                if via_kw != "via" {
                    return Err(syn::Error::new(
                        via_kw.span(),
                        "expected 'via' after target model",
                    ));
                }
                let from_col: Ident = input.parse()?;

                let mut on_delete = "NoAction".to_string();
                let mut on_update = "NoAction".to_string();
                if input.peek(syn::token::Bracket) {
                    let opts;
                    bracketed!(opts in input);
                    let mut i: i32 = 0;
                    while !opts.is_empty() {
                        let opt: Ident = opts.parse()?;
                        let val = normalize_fk_action(&opt.to_string());
                        if i == 0 {
                            on_delete = val;
                        } else {
                            on_update = val;
                        }
                        i = i.saturating_add(1);
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
                        "expected 'via' after target model",
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

/// PascalCase → snake_case to derive target table name from model
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

// ── Type Mapping ──────────────────────────────────────────────────────────

/// Converts a DSL type (v1 SQL or v2 semantic) to a SeaORM type name.
fn dsl_field_type_to_col_type(ty: &str) -> String {
    match ty {
        // v1 SQL
        "String" | "char" | "varchar" => "String".to_string(),
        // "text" in v2 = short text field (VARCHAR) — same behavior as String in v1
        "text" => "String".to_string(),
        "i8" => "TinyInteger".to_string(),
        "i16" => "SmallInteger".to_string(),
        "i32" | "integer" => "Integer".to_string(),
        "i64" | "big_integer" | "bigint" => "BigInteger".to_string(),
        "u32" => "Unsigned".to_string(),
        "u64" => "BigUnsigned".to_string(),
        "f32" => "Float".to_string(),
        "f64" | "float" | "percent" => "Double".to_string(),
        "decimal" => "Decimal".to_string(),
        "bool" => "Boolean".to_string(),
        "date" => "Date".to_string(),
        "time" => "Time".to_string(),
        "datetime" | "timestamp" => "DateTime".to_string(),
        "timestamp_tz" => "TimestampWithTimeZone".to_string(),
        "uuid" => "Uuid".to_string(),
        "json" | "json_binary" => "Json".to_string(),
        "binary" | "blob" => "Binary".to_string(),
        "inet" | "cidr" | "mac_address" | "interval" | "ip" => "String".to_string(),
        // v2 semantic text → String (VARCHAR) or Text
        "email" | "url" | "password" | "slug" | "color" => "String".to_string(),
        "richtext" | "textarea" => "Text".to_string(),
        // v2 files → String (JSON path)
        "image" | "document" | "file" => "String".to_string(),
        // v2 choice/radio → resolved via enum_ref in dsl_to_parsed_schema, fallback String
        "choice" | "radio" => "String".to_string(),
        // explicit int
        "int" => "Integer".to_string(),
        _ => "String".to_string(),
    }
}

fn dsl_pk_to_col_type(ty: &str) -> String {
    match ty {
        "i32" | "Pk" => "Integer".to_string(),
        "i64" => "BigInteger".to_string(),
        "uuid" => "Uuid".to_string(),
        _ => "Integer".to_string(),
    }
}

// ── Conversion to ParsedSchema ──────────────────────────────────────────────

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
        enum_name: None,
        enum_string_values: Vec::new(),
        enum_is_pg: false,
    });

    let enum_types = model.enum_types;

    let columns = model
        .fields
        .into_iter()
        .map(|f| {
            let has_auto_now = f.options.contains(&"auto_now".to_string());
            let has_auto_now_update = f.options.contains(&"auto_now_update".to_string());
            let has_required = f.options.contains(&"required".to_string());
            let has_nullable = f.options.contains(&"nullable".to_string());
            let is_created_at = f.name == "created_at";
            let is_updated_at = f.name == "updated_at";

            // Semantic v2 types (lowercase): lack of `required` → nullable by default.
            // v1 types (SQL / Rust: String, i32...): only explicit `[nullable]` makes it nullable.
            // auto_now / auto_now_update → never nullable in both cases.
            const V2_TYPES: &[&str] = &[
                "text",
                "email",
                "password",
                "richtext",
                "textarea",
                "url",
                "int",
                "float",
                "decimal",
                "percent",
                "date",
                "time",
                "datetime",
                "timestamp",
                "timestamp_tz",
                "image",
                "document",
                "file",
                "color",
                "slug",
                "uuid",
                "json",
                "json_binary",
                "ip",
                "choice",
                "radio",
                "bigint",
                "binary",
                "blob",
                "inet",
                "cidr",
                "mac_address",
                "interval",
            ];
            let is_v2 = V2_TYPES.contains(&f.ty.as_str());
            let nullable = if has_auto_now || has_auto_now_update || has_required {
                false
            } else if has_nullable {
                true
            } else {
                is_v2 // v2 without required → nullable ; v1 without explicit nullable → not nullable
            };

            let unique = f.options.contains(&"unique".to_string());

            // Resolution of the associated enum (v1: ty=="enum", v2: ty=="choice"/"radio")
            let is_enum_field = f.ty == "enum" || f.ty == "choice" || f.ty == "radio";
            let enum_entry = if is_enum_field {
                f.enum_name
                    .as_deref()
                    .and_then(|n| enum_types.iter().find(|(name, _, _)| name == n))
            } else {
                None
            };

            let col_type = if has_auto_now || has_auto_now_update {
                "DateTime".to_string()
            } else if is_enum_field {
                match enum_entry.map(|(_, bt, _)| bt.as_str()).unwrap_or("Auto") {
                    "i32" => "Integer".to_string(),
                    "i64" => "BigInteger".to_string(),
                    _ => "String".to_string(), // Auto / String → VARCHAR
                }
            } else {
                dsl_field_type_to_col_type(&f.ty)
            };

            // Enum string values for diff (only string-backed enums)
            let (enum_name, enum_string_values, enum_is_pg) = if is_enum_field {
                match enum_entry {
                    Some((name, backing, values)) if backing != "i32" && backing != "i64" => {
                        // Auto → `enum_is_pg = false`, the generator decides via DbKind
                        (Some(name.clone()), values.clone(), false)
                    }
                    _ => (None, Vec::new(), false),
                }
            } else {
                (None, Vec::new(), false)
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
                has_default_now: has_auto_now
                    || has_auto_now_update
                    || is_created_at
                    || is_updated_at,
                enum_name,
                enum_string_values,
                enum_is_pg,
            }
        })
        .collect();

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

// ── Visitor ──────────────────────────────────────────────────────────────────

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

        if is_model && let Ok(model) = syn::parse2::<DslModel>(mac.tokens.clone()) {
            self.schema = Some(dsl_to_parsed_schema(model));
        }
        syn::visit::visit_macro(self, mac);
    }
}

// ── Public entry point ─────────────────────────────────────────────────────

pub fn parse_schema_from_source(source: &str) -> Option<ParsedSchema> {
    let file = syn::parse_str::<syn::File>(source).ok()?;
    let mut visitor = DslVisitor::new();
    visitor.visit_file(&file);
    visitor.schema
}
