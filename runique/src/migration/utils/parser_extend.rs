//! Parser for `extend!{}` blocks — extracts framework table extensions.
//!
//! Expected syntax:
//! ```rust,ignore
//! extend! {
//!     table: "eihwaz_users",
//!     fields: {
//!         avatar: image [upload_to: "avatars/"],
//!         bio: textarea,
//!         website: url [required],
//!     }
//! }
//! ```
use proc_macro2::TokenTree;
use syn::{
    Ident, LitStr, Token, braced, bracketed,
    parse::{Parse, ParseStream},
    visit::Visit,
};

use crate::migration::utils::types::{ParsedColumn, ParsedSchema};

// ── Internal DSL ───────────────────────────────────────────────────────────────

struct DslExtendField {
    name: String,
    ty: String,
    options: Vec<String>,
}

struct DslExtend {
    table: String,
    fields: Vec<DslExtendField>,
}

impl Parse for DslExtendField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty: Ident = input.parse()?;

        let mut options = Vec::new();
        if input.peek(syn::token::Bracket) {
            let opts;
            bracketed!(opts in input);
            while !opts.is_empty() {
                // enum(...) in attribute position — we ignore the name but consume tokens
                if opts.peek(Token![enum]) {
                    opts.parse::<Token![enum]>()?;
                    let inner;
                    syn::parenthesized!(inner in opts);
                    while !inner.is_empty() {
                        inner.parse::<TokenTree>().ok();
                    }
                    let _ = opts.parse::<Token![,]>();
                    continue;
                }

                let opt: Ident = opts.parse()?;
                options.push(opt.to_string());

                // `key: value` syntax
                if opts.peek(Token![:]) {
                    opts.parse::<Token![:]>()?;
                    opts.parse::<TokenTree>().ok();
                }
                // `key(value)` syntax
                else if opts.peek(syn::token::Paren) {
                    let inner;
                    syn::parenthesized!(inner in opts);
                    while !inner.is_empty() {
                        inner.parse::<TokenTree>().ok();
                    }
                }

                let _ = opts.parse::<Token![,]>();
            }
        }

        let _ = input.parse::<Token![,]>();
        Ok(DslExtendField {
            name: name.to_string(),
            ty: ty.to_string(),
            options,
        })
    }
}

impl Parse for DslExtend {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // table: "table_name",
        let kw: Ident = input.parse()?;
        if kw != "table" {
            return Err(syn::Error::new(kw.span(), "expected 'table'"));
        }
        input.parse::<Token![:]>()?;
        let table: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;

        // fields: { ... }
        let kw: Ident = input.parse()?;
        if kw != "fields" {
            return Err(syn::Error::new(kw.span(), "expected 'fields'"));
        }
        input.parse::<Token![:]>()?;
        let fields_content;
        braced!(fields_content in input);

        let mut fields = Vec::new();
        while !fields_content.is_empty() {
            fields.push(DslExtendField::parse(&fields_content)?);
        }

        // optional trailing comma
        let _ = input.parse::<Token![,]>();

        Ok(DslExtend {
            table: table.value(),
            fields,
        })
    }
}

// ── Conversion to ParsedColumn ──────────────────────────────────────────────

fn extend_field_type_to_col_type(ty: &str) -> String {
    match ty {
        // Short text (VARCHAR)
        "text" | "email" | "url" | "password" | "slug" | "color" | "String" | "char"
        | "varchar" => "String".to_string(),
        // Long text
        "richtext" | "textarea" => "Text".to_string(),
        // Files — stored as String path
        "image" | "document" | "file" => "String".to_string(),
        // Integers
        "i8" => "TinyInteger".to_string(),
        "i16" => "SmallInteger".to_string(),
        "i32" | "integer" | "int" => "Integer".to_string(),
        "i64" | "bigint" | "big_integer" => "BigInteger".to_string(),
        // Floats
        "f32" => "Float".to_string(),
        "f64" | "float" | "percent" | "double" => "Double".to_string(),
        "decimal" => "Decimal".to_string(),
        // Boolean
        "bool" => "Boolean".to_string(),
        // Date / Time
        "date" => "Date".to_string(),
        "time" => "Time".to_string(),
        "datetime" | "timestamp" => "DateTime".to_string(),
        "timestamp_tz" => "TimestampWithTimeZone".to_string(),
        // Miscellaneous
        "uuid" => "Uuid".to_string(),
        "json" | "json_binary" => "Json".to_string(),
        "binary" | "blob" => "Binary".to_string(),
        // Choice (enum) — stored as String
        "choice" | "radio" => "String".to_string(),
        _ => "String".to_string(),
    }
}

fn extend_field_to_col(f: DslExtendField) -> ParsedColumn {
    let has_required = f.options.contains(&"required".to_string());
    let unique = f.options.contains(&"unique".to_string());
    ParsedColumn {
        name: f.name,
        col_type: extend_field_type_to_col_type(&f.ty),
        nullable: !has_required,
        unique,
        ignored: false,
        created_at: false,
        updated_at: false,
        has_default_now: false,
        enum_name: None,
        enum_string_values: Vec::new(),
        enum_is_pg: false,
    }
}

// ── Visitor ──────────────────────────────────────────────────────────────────

struct ExtendVisitor {
    pub schemas: Vec<ParsedSchema>,
}

impl ExtendVisitor {
    fn new() -> Self {
        Self {
            schemas: Vec::new(),
        }
    }
}

impl<'ast> Visit<'ast> for ExtendVisitor {
    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        let is_extend = mac
            .path
            .segments
            .last()
            .map(|s| s.ident == "extend")
            .unwrap_or(false);

        if is_extend && let Ok(ext) = syn::parse2::<DslExtend>(mac.tokens.clone()) {
            let columns = ext.fields.into_iter().map(extend_field_to_col).collect();
            self.schemas.push(ParsedSchema {
                table_name: ext.table,
                primary_key: None,
                columns,
                foreign_keys: Vec::new(),
                indexes: Vec::new(),
            });
        }

        syn::visit::visit_macro(self, mac);
    }
}

// ── Public entry point ─────────────────────────────────────────────────────

/// Parses all `extend!{}` blocks in a Rust source file.
/// Returns a vector of `ParsedSchema` (one per `extend!{}` block).
/// Schemas have `primary_key = None` — these are extensions of existing tables.
pub fn parse_extend_blocks_from_source(source: &str) -> Vec<ParsedSchema> {
    let file = match syn::parse_str::<syn::File>(source) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };
    let mut visitor = ExtendVisitor::new();
    visitor.visit_file(&file);
    visitor.schemas
}
