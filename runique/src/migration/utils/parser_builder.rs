use syn::{
    braced, bracketed,
    parse::{Parse, ParseStream},
    visit::Visit,
    Ident, LitStr, Token,
};

use crate::migration::utils::types::{ParsedColumn, ParsedFk, ParsedIndex, ParsedSchema};

// ── Structures de parsing légères ────────────────────────────────────────────

struct DslModel {
    table: String,
    pk: DslPk,
    fields: Vec<DslField>,
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

        // ignorer relations/meta éventuels
        while !input.is_empty() {
            input.parse::<proc_macro2::TokenTree>().ok();
        }

        Ok(DslModel {
            table: table.value(),
            pk: DslPk {
                name: pk_name.to_string(),
                ty: pk_ty.to_string(),
            },
            fields,
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

// ── Conversion vers ParsedSchema ──────────────────────────────────────────────

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

fn dsl_to_parsed_schema(model: DslModel) -> ParsedSchema {
    let primary_key = Some(ParsedColumn {
        name: model.pk.name,
        col_type: dsl_pk_to_col_type(&model.pk.ty),
        nullable: false,
        unique: false,
        ignored: false,
    });

    let columns = model
        .fields
        .into_iter()
        .map(|f| {
            let nullable = f
                .options
                .iter()
                .any(|o| matches!(o.as_str(), "nullable" | "auto_now" | "auto_now_update"));
            let unique = f.options.contains(&"unique".to_string());
            let ignored = f.options.contains(&"readonly".to_string());
            let col_type = if f.options.contains(&"auto_now".to_string())
                || f.options.contains(&"auto_now_update".to_string())
            {
                "DateTime".to_string()
            } else {
                dsl_field_type_to_col_type(&f.ty)
            };

            ParsedColumn {
                name: f.name,
                col_type,
                nullable,
                unique,
                ignored,
            }
        })
        .collect();

    ParsedSchema {
        table_name: model.table,
        primary_key,
        columns,
        foreign_keys: Vec::<ParsedFk>::new(),
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
