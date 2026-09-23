//! `DslRelation` — a `belongs_to`/`has_many`/`has_one`/`many_to_many` entry
//! inside a model's `relations: { ... }` block.
use syn::{
    Ident, Token, bracketed,
    parse::{Parse, ParseStream},
};

pub(super) enum DslRelationKind {
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

pub(super) struct DslRelation {
    pub kind: DslRelationKind,
    pub target: String,
}

fn normalize_fk_action(s: &str) -> String {
    match s.to_lowercase().as_str() {
        "cascade" => "Cascade".to_string(),
        "restrict" => "Restrict".to_string(),
        "set_null" | "setnull" => "SetNull".to_string(),
        "set_default" | "setdefault" => "SetDefault".to_string(),
        _ => "NoAction".to_string(),
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
