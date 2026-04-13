//! Parser pour les blocs `extend!{}` — extrait les extensions de tables framework.
//!
//! Syntaxe attendue :
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

// ── DSL interne ───────────────────────────────────────────────────────────────

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
                // enum(...) en position attribut — on ignore le nom mais on consomme les tokens
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

                // syntaxe `key: value`
                if opts.peek(Token![:]) {
                    opts.parse::<Token![:]>()?;
                    opts.parse::<TokenTree>().ok();
                }
                // syntaxe `key(value)`
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
        // table: "nom_de_table",
        let kw: Ident = input.parse()?;
        if kw != "table" {
            return Err(syn::Error::new(kw.span(), "attendu 'table'"));
        }
        input.parse::<Token![:]>()?;
        let table: LitStr = input.parse()?;
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
            fields.push(DslExtendField::parse(&fields_content)?);
        }

        // virgule finale optionnelle
        let _ = input.parse::<Token![,]>();

        Ok(DslExtend {
            table: table.value(),
            fields,
        })
    }
}

// ── Conversion vers ParsedColumn ──────────────────────────────────────────────

fn extend_field_type_to_col_type(ty: &str) -> String {
    match ty {
        // Texte court (VARCHAR)
        "text" | "email" | "url" | "password" | "slug" | "color" | "String" | "char"
        | "varchar" => "String".to_string(),
        // Texte long
        "richtext" | "textarea" => "Text".to_string(),
        // Fichiers — stockés comme chemin String
        "image" | "document" | "file" => "String".to_string(),
        // Entiers
        "i8" => "TinyInteger".to_string(),
        "i16" => "SmallInteger".to_string(),
        "i32" | "integer" | "int" => "Integer".to_string(),
        "i64" | "bigint" | "big_integer" => "BigInteger".to_string(),
        // Flottants
        "f32" => "Float".to_string(),
        "f64" | "float" | "percent" | "double" => "Double".to_string(),
        "decimal" => "Decimal".to_string(),
        // Booléen
        "bool" => "Boolean".to_string(),
        // Dates / Temps
        "date" => "Date".to_string(),
        "time" => "Time".to_string(),
        "datetime" | "timestamp" => "DateTime".to_string(),
        "timestamp_tz" => "TimestampWithTimeZone".to_string(),
        // Divers
        "uuid" => "Uuid".to_string(),
        "json" | "json_binary" => "Json".to_string(),
        "binary" | "blob" => "Binary".to_string(),
        // Choix (enum) — stocké comme String
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

// ── Visiteur ──────────────────────────────────────────────────────────────────

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

        if is_extend {
            if let Ok(ext) = syn::parse2::<DslExtend>(mac.tokens.clone()) {
                let columns = ext.fields.into_iter().map(extend_field_to_col).collect();
                self.schemas.push(ParsedSchema {
                    table_name: ext.table,
                    primary_key: None,
                    columns,
                    foreign_keys: Vec::new(),
                    indexes: Vec::new(),
                });
            }
        }

        syn::visit::visit_macro(self, mac);
    }
}

// ── Point d'entrée public ─────────────────────────────────────────────────────

/// Parse tous les blocs `extend!{}` dans un fichier source Rust.
/// Retourne un vecteur de `ParsedSchema` (un par bloc `extend!{}`).
/// Les schémas ont `primary_key = None` — ce sont des extensions de tables existantes.
pub fn parse_extend_blocks_from_source(source: &str) -> Vec<ParsedSchema> {
    let file = match syn::parse_str::<syn::File>(source) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };
    let mut visitor = ExtendVisitor::new();
    visitor.visit_file(&file);
    visitor.schemas
}
