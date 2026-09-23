//! `DslField` — one `name: type [opt1, opt2, ...]` entry inside a `model!{}`
//! anonymous fields block.
use syn::{
    Ident, Token, bracketed,
    parse::{Parse, ParseStream},
};

pub(super) struct DslField {
    pub name: String,
    pub ty: String,
    pub enum_name: Option<String>, // from type (v1: `enum(X)`) or attrs (v2: `choice [enum(X)]`)
    pub options: Vec<String>,
    pub default_value: Option<String>, // literal `[default: X]` value (rendered for `.default(...)`)
    pub renamed_from: Option<String>, // `[renamed_from: "old"]` → RENAME COLUMN instead of DROP+ADD
}

/// Concatenates every remaining token in `buf` (used for `default(...)` paren values).
fn capture_tokens_string(buf: &syn::parse::ParseBuffer) -> String {
    let mut s = String::new();
    while !buf.is_empty() {
        match buf.parse::<proc_macro2::TokenTree>() {
            Ok(tt) => s.push_str(&tt.to_string()),
            Err(_) => break,
        }
    }
    s
}

/// Concatenates tokens in `buf` up to the next comma (used for `key: value` attr values).
fn capture_value_until_comma(buf: &syn::parse::ParseBuffer) -> String {
    let mut s = String::new();
    while !buf.is_empty() && !buf.peek(Token![,]) {
        match buf.parse::<proc_macro2::TokenTree>() {
            Ok(tt) => s.push_str(&tt.to_string()),
            Err(_) => break,
        }
    }
    s
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
            // Consume optional (…) qualifier — e.g. `decimal(10, 2)` or `varchar(255)`
            if input.peek(syn::token::Paren) {
                let inner;
                syn::parenthesized!(inner in input);
                while !inner.is_empty() {
                    inner.parse::<proc_macro2::TokenTree>().ok();
                }
            }
            (ty.to_string(), None)
        };

        let mut options = Vec::new();
        let mut default_value: Option<String> = None;
        let mut renamed_from: Option<String> = None;
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
                let opt_str = opt.to_string();
                options.push(opt_str.clone());

                // paren syntax (v1): max_len(150), default(0)
                if opts.peek(syn::token::Paren) {
                    let inner;
                    syn::parenthesized!(inner in opts);
                    let captured = capture_tokens_string(&inner);
                    if opt_str == "default" {
                        default_value = Some(captured);
                    } else if opt_str == "renamed_from" {
                        renamed_from = Some(captured.trim_matches('"').to_string());
                    }
                }
                // colon syntax (v2): max_length: 150, rows: 6, upload_to: "path", default: 0
                else if opts.peek(Token![:]) {
                    opts.parse::<Token![:]>()?;
                    let captured = capture_value_until_comma(&opts);
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
        Ok(DslField {
            name: name.to_string(),
            ty,
            enum_name,
            options,
            default_value,
            renamed_from,
        })
    }
}
