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
    default_value: Option<String>,
    enum_name: Option<String>,
    renamed_from: Option<String>,
}

/// Concatenates tokens in `buf` up to the next comma (or end) — used for attr values.
fn capture_value_until_comma(buf: &syn::parse::ParseBuffer) -> String {
    let mut s = String::new();
    while !buf.is_empty() && !buf.peek(Token![,]) {
        match buf.parse::<TokenTree>() {
            Ok(tt) => s.push_str(&tt.to_string()),
            Err(_) => break,
        }
    }
    s
}

struct DslExtend {
    table: String,
    /// (enum_name, backing_type, string_values) — same shape as parser_builder.
    enum_types: Vec<(String, String, Vec<String>)>,
    fields: Vec<DslExtendField>,
}

impl Parse for DslExtendField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty: Ident = input.parse()?;

        let mut options = Vec::new();
        let mut default_value: Option<String> = None;
        let mut enum_name: Option<String> = None;
        let mut renamed_from: Option<String> = None;
        if input.peek(syn::token::Bracket) {
            let opts;
            bracketed!(opts in input);
            while !opts.is_empty() {
                // enum(Name) in attribute position (choice/radio) — capture the name.
                if opts.peek(Token![enum]) {
                    opts.parse::<Token![enum]>()?;
                    let inner;
                    syn::parenthesized!(inner in opts);
                    if let Ok(name) = inner.parse::<Ident>() {
                        enum_name = Some(name.to_string());
                    }
                    while !inner.is_empty() {
                        inner.parse::<TokenTree>().ok();
                    }
                    let _ = opts.parse::<Token![,]>();
                    continue;
                }

                let opt: Ident = opts.parse()?;
                let opt_str = opt.to_string();
                options.push(opt_str.clone());

                // `key: value` syntax
                if opts.peek(Token![:]) {
                    opts.parse::<Token![:]>()?;
                    let captured = capture_value_until_comma(&opts);
                    if opt_str == "default" {
                        default_value = Some(captured);
                    } else if opt_str == "renamed_from" {
                        renamed_from = Some(captured.trim_matches('"').to_string());
                    }
                }
                // `key(value)` syntax — e.g. default(0)
                else if opts.peek(syn::token::Paren) {
                    let inner;
                    syn::parenthesized!(inner in opts);
                    let mut captured = String::new();
                    while !inner.is_empty() {
                        match inner.parse::<TokenTree>() {
                            Ok(tt) => captured.push_str(&tt.to_string()),
                            Err(_) => break,
                        }
                    }
                    if opt_str == "default" {
                        default_value = Some(captured);
                    } else if opt_str == "renamed_from" {
                        renamed_from = Some(captured.trim_matches('"').to_string());
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
            default_value,
            enum_name,
            renamed_from,
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

        // enums: { ... } optional — same shape and resolution as parser_builder.
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
                                        } else if variants.peek(syn::token::Paren) {
                                            let tuple;
                                            syn::parenthesized!(tuple in variants);
                                            if let Ok(syn::Lit::Str(s)) = tuple.parse::<syn::Lit>()
                                            {
                                                string_values.push(s.value());
                                            } else {
                                                string_values.push(vname.to_string());
                                            }
                                            while !tuple.is_empty() {
                                                tuple.parse::<TokenTree>().ok();
                                            }
                                        } else {
                                            string_values.push(vname.to_string());
                                        }
                                    } else {
                                        string_values.push(vname.to_string());
                                    }
                                } else {
                                    variants.parse::<TokenTree>().ok();
                                }
                                let _ = variants.parse::<Token![,]>();
                            }
                        }
                        enum_types.push((enum_name.to_string(), backing, string_values));
                        let _ = enum_block.parse::<Token![,]>();
                    } else {
                        enum_block.parse::<TokenTree>().ok();
                    }
                }
                let _ = input.parse::<Token![,]>();
            }
        }

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
            enum_types,
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

fn extend_field_to_col(
    f: DslExtendField,
    enum_types: &[(String, String, Vec<String>)],
) -> ParsedColumn {
    let has_required = f.options.contains(&"required".to_string());
    let unique = f.options.contains(&"unique".to_string());

    // Resolution of the associated enum (choice/radio + `[enum(Name)]`) — mirrors parser_builder.
    let is_enum_field = f.ty == "choice" || f.ty == "radio";
    let enum_entry = if is_enum_field {
        f.enum_name
            .as_deref()
            .and_then(|n| enum_types.iter().find(|(name, _, _)| name == n))
    } else {
        None
    };

    let col_type = if is_enum_field {
        match enum_entry.map(|(_, bt, _)| bt.as_str()).unwrap_or("Auto") {
            "i32" => "Integer".to_string(),
            "i64" => "BigInteger".to_string(),
            _ => "String".to_string(),
        }
    } else {
        extend_field_type_to_col_type(&f.ty)
    };

    // Only string-backed enums carry values (the generator emits CREATE TYPE for them on PG).
    let (enum_name, enum_string_values) = match enum_entry {
        Some((name, backing, values)) if backing != "i32" && backing != "i64" => {
            (Some(name.clone()), values.clone())
        }
        _ => (None, Vec::new()),
    };

    ParsedColumn {
        name: f.name,
        col_type,
        nullable: !has_required,
        unique,
        ignored: false,
        created_at: false,
        updated_at: false,
        has_default_now: false,
        default_value: f.default_value,
        enum_name,
        enum_string_values,
        enum_is_pg: false,
        renamed_from: f.renamed_from,
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
            let enum_types = ext.enum_types;
            let columns = ext
                .fields
                .into_iter()
                .map(|f| extend_field_to_col(f, &enum_types))
                .collect();
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
