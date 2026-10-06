//! `{Model}AdminForm`: the admin form built from the declared fields.
use crate::model::ast::*;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

/// Generates `{ModelName}AdminForm` — the form registering each declared field.
pub fn generate_admin_form(model: &ModelInput) -> TokenStream2 {
    let model_name = &model.name;
    let form_name = quote::format_ident!("{}AdminForm", model_name);

    let field_registrations: Vec<TokenStream2> = model
        .form_fields
        .iter()
        .chain(&model.lists)
        .map(|ff| generate_form_field_decl(ff, &model.enums))
        .collect();

    quote! {
        /// Stable alias used by the `runique start` daemon to reference this form.
        /// Always available via `{module}::AdminForm`.
        pub type AdminForm = #form_name;

        /// Admin form auto-generated from model.
        /// Covers all model fields except auto_now/auto_now_update fields.
        #[derive(::runique::serde::Serialize, Debug, Clone)]
        pub struct #form_name {
            pub form: ::runique::forms::Forms,
        }

        impl ::runique::forms::field::RuniqueForm for #form_name {
            fn register_fields(form: &mut ::runique::forms::Forms) {
                #(#field_registrations)*
            }

            fn from_form(form: ::runique::forms::Forms) -> Self {
                Self { form }
            }

            fn get_form(&self) -> &::runique::forms::Forms {
                &self.form
            }

            fn get_form_mut(&mut self) -> &mut ::runique::forms::Forms {
                &mut self.form
            }
        }
    }
}

/// Generates `form.field(&...)` from a `FormFieldDecl` declaration (form_fields: block).
/// `model` is passed to allow resolution of enum variants for `Choice`/`Radio`.
pub(crate) fn generate_form_field_decl(ff: &FormFieldDecl, enums: &[EnumDef]) -> TokenStream2 {
    // auto_now / auto_now_update / skip fields: excluded from generated forms.
    if ff.attrs.iter().any(|a| {
        matches!(
            a,
            FormFieldAttr::AutoNow | FormFieldAttr::AutoNowUpdate | FormFieldAttr::Skip
        )
    }) {
        return quote! {};
    }

    let name = &ff.name;
    let name_str = name.to_string();

    // `[label: "…"]` if declared, otherwise derived from the snake_case name.
    let declared_label = ff.attrs.iter().find_map(|a| match a {
        FormFieldAttr::Label(s) => Some(s.clone()),
        _ => None,
    });
    let label = if let Some(declared) = declared_label {
        declared
    } else {
        let s = name_str.replace('_', " ");
        let mut chars = s.chars();
        match chars.next() {
            None => s,
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        }
    };

    // Common attribute suffixes (order: constructor → label → specific attrs → required)
    let required_suffix = if ff
        .attrs
        .iter()
        .any(|a| matches!(a, FormFieldAttr::Required))
    {
        quote! { .required() }
    } else {
        quote! {}
    };

    let field_expr: TokenStream2 = match &ff.kind {
        // ── Text fields ──────────────────────────────────────────────
        FormFieldKind::Text => {
            let extras = text_attrs_tokens(&ff.attrs, false);
            quote! { ::runique::forms::fields::TextField::text(#name_str).label(#label) #extras #required_suffix }
        }
        FormFieldKind::Email => {
            let extras = text_attrs_tokens(&ff.attrs, false);
            quote! { ::runique::forms::fields::TextField::email(#name_str).label(#label) #extras #required_suffix }
        }
        FormFieldKind::Password => {
            let no_hash = if ff.attrs.iter().any(|a| matches!(a, FormFieldAttr::NoHash)) {
                quote! { .no_hash() }
            } else {
                quote! {}
            };
            let extras = text_attrs_tokens(&ff.attrs, false);
            quote! { ::runique::forms::fields::TextField::password(#name_str).label(#label) #no_hash #extras #required_suffix }
        }
        FormFieldKind::Richtext => {
            let extras = text_attrs_tokens(&ff.attrs, true);
            quote! { ::runique::forms::fields::TextField::richtext(#name_str).label(#label) #extras #required_suffix }
        }
        FormFieldKind::Textarea => {
            let extras = text_attrs_tokens(&ff.attrs, true);
            quote! { ::runique::forms::fields::TextField::textarea(#name_str).label(#label) #extras #required_suffix }
        }
        FormFieldKind::Url => {
            let extras = text_attrs_tokens(&ff.attrs, false);
            quote! { ::runique::forms::fields::TextField::url(#name_str).label(#label) #extras #required_suffix }
        }

        // ── Numeric fields ─────────────────────────────────────────
        FormFieldKind::Int
        | FormFieldKind::Bigint
        | FormFieldKind::I8
        | FormFieldKind::I16
        | FormFieldKind::U32
        | FormFieldKind::U64 => {
            let runique_dsl::types::Widget::Integer { min, max } = ff.kind.widget() else {
                unreachable!("integer kinds map to Widget::Integer");
            };
            let extras = numeric_attrs_tokens(&ff.attrs);
            quote! { ::runique::forms::fields::NumericField::integer_in(#name_str, #min, #max).label(#label) #extras #required_suffix }
        }
        FormFieldKind::Float | FormFieldKind::F32 => {
            let extras = numeric_attrs_tokens(&ff.attrs);
            quote! { ::runique::forms::fields::NumericField::float(#name_str).label(#label) #extras #required_suffix }
        }
        FormFieldKind::Decimal => {
            let extras = numeric_attrs_tokens(&ff.attrs);
            quote! { ::runique::forms::fields::NumericField::decimal(#name_str).label(#label) #extras #required_suffix }
        }
        FormFieldKind::Percent => {
            quote! { ::runique::forms::fields::NumericField::percent(#name_str).label(#label) #required_suffix }
        }

        // ── Bool ──────────────────────────────────────────────────────
        FormFieldKind::Bool => {
            let default_suffix = ff
                .attrs
                .iter()
                .find_map(|a| {
                    if let FormFieldAttr::Default(syn::Lit::Bool(b)) = a {
                        if b.value {
                            Some(quote! { .checked() })
                        } else {
                            Some(quote! { .unchecked() })
                        }
                    } else {
                        None
                    }
                })
                .unwrap_or_default();
            quote! { ::runique::forms::fields::BooleanField::new(#name_str).label(#label) #default_suffix #required_suffix }
        }

        // ── Date/Time ────────────────────────────────────────────────
        FormFieldKind::Date => {
            quote! { ::runique::forms::fields::DateField::new(#name_str).label(#label) #required_suffix }
        }
        FormFieldKind::Time => {
            quote! { ::runique::forms::fields::TimeField::new(#name_str).label(#label) #required_suffix }
        }
        FormFieldKind::Datetime | FormFieldKind::Timestamp | FormFieldKind::TimestampTz => {
            quote! { ::runique::forms::fields::DateTimeField::new(#name_str).label(#label) #required_suffix }
        }
        FormFieldKind::Char => {
            let extras = text_attrs_tokens(&ff.attrs, false);
            quote! { ::runique::forms::fields::TextField::text(#name_str).label(#label) #extras #required_suffix }
        }

        // ── Files ──────────────────────────────────────────────────
        FormFieldKind::Image => {
            let file_extras = file_attrs_tokens(&ff.attrs);
            quote! { ::runique::forms::fields::FileField::image(#name_str).label(#label) #file_extras #required_suffix }
        }
        FormFieldKind::Document => {
            let file_extras = file_attrs_tokens(&ff.attrs);
            quote! { ::runique::forms::fields::FileField::document(#name_str).label(#label) #file_extras #required_suffix }
        }
        FormFieldKind::File => {
            let file_extras = file_attrs_tokens(&ff.attrs);
            quote! { ::runique::forms::fields::FileField::any(#name_str).label(#label) #file_extras #required_suffix }
        }

        // ── Choice / Radio / Checkbox — resolution via EnumRef attr or fields: ─────
        FormFieldKind::Choice
        | FormFieldKind::Radio
        | FormFieldKind::Checkbox
        | FormFieldKind::Multichoice => {
            // Priority: Explicit EnumRef in attrs, otherwise lookup via fields:
            let enum_ident = ff.attrs.iter().find_map(|a| {
                if let FormFieldAttr::EnumRef(id) = a {
                    Some(id)
                } else {
                    None
                }
            });
            let enum_def = enum_ident.and_then(|id| enums.iter().find(|e| e.name == *id));

            let choices: Vec<TokenStream2> = enum_def
                .map(|e| {
                    e.variants
                        .iter()
                        .map(|v| {
                            let db_val = v.db_str();
                            let display = v.display_str();
                            quote! { .add_choice(#db_val, #display) }
                        })
                        .collect()
                })
                .unwrap_or_default();

            if matches!(ff.kind, FormFieldKind::Radio) {
                quote! { ::runique::forms::fields::RadioField::new(#name_str).label(#label) #(#choices)* #required_suffix }
            } else if matches!(ff.kind, FormFieldKind::Checkbox) {
                quote! { ::runique::forms::fields::CheckboxField::new(#name_str).label(#label) #(#choices)* #required_suffix }
            } else if matches!(ff.kind, FormFieldKind::Multichoice) {
                quote! { ::runique::forms::fields::ChoiceField::new(#name_str).label(#label).multiple() #(#choices)* #required_suffix }
            } else {
                quote! { ::runique::forms::fields::ChoiceField::new(#name_str).label(#label) #(#choices)* #required_suffix }
            }
        }

        // ── Special fields ───────────────────────────────────────────
        FormFieldKind::Color => {
            quote! { ::runique::forms::fields::ColorField::new(#name_str).label(#label) #required_suffix }
        }
        FormFieldKind::Slug => {
            quote! { ::runique::forms::fields::SlugField::new(#name_str).label(#label) #required_suffix }
        }
        FormFieldKind::Uuid => {
            quote! { ::runique::forms::fields::UUIDField::new(#name_str).label(#label) #required_suffix }
        }
        FormFieldKind::Json | FormFieldKind::JsonBinary => {
            let rows_suffix = rows_token(&ff.attrs);
            quote! { ::runique::forms::fields::JSONField::new(#name_str).label(#label) #rows_suffix #required_suffix }
        }
        FormFieldKind::Ip => {
            quote! { ::runique::forms::fields::IPAddressField::new(#name_str).label(#label) #required_suffix }
        }
        // No dedicated widget for raw bytes / network / duration types — a plain text
        // input (no format validation opinion) keeps the form usable rather than
        // refusing to generate one, or worse, applying the wrong validator (e.g. a MAC
        // address would fail IPAddressField's IPv4/IPv6 checks).
        FormFieldKind::Binary | FormFieldKind::VarBinary | FormFieldKind::Blob => {
            let max_length = ff.attrs.iter().find_map(|a| match a {
                FormFieldAttr::MaxLength(n) => Some(*n),
                _ => None,
            });
            let limit = ff
                .kind
                .byte_limit(max_length)
                .map(|n| {
                    let n = u64::from(n);
                    quote! { .max_size(#n) }
                })
                .unwrap_or_default();
            quote! { ::runique::forms::fields::BinaryField::new(#name_str).label(#label) #limit #required_suffix }
        }
        FormFieldKind::Cidr | FormFieldKind::MacAddress | FormFieldKind::Interval => {
            let extras = text_attrs_tokens(&ff.attrs, false);
            quote! { ::runique::forms::fields::TextField::text(#name_str).label(#label) #extras #required_suffix }
        }

        FormFieldKind::Phone => {
            let extras = text_attrs_tokens(&ff.attrs, false);
            quote! { ::runique::forms::fields::TextField::phone(#name_str).label(#label) #extras #required_suffix }
        }
    };

    quote! { form.field(&#field_expr); }
}

/// Generates builder suffixes for text fields (max_length, min_length, rows).
fn text_attrs_tokens(attrs: &[FormFieldAttr], with_rows: bool) -> TokenStream2 {
    let mut ts = quote! {};
    for attr in attrs {
        match attr {
            FormFieldAttr::MaxLength(n) => ts.extend(quote! { .max_length(#n, "") }),
            FormFieldAttr::MinLength(n) => ts.extend(quote! { .min_length(#n, "") }),
            FormFieldAttr::Rows(n) if with_rows => {
                let n_usize = *n as usize;
                ts.extend(quote! { .rows(#n_usize) });
            }
            _ => {}
        }
    }
    ts
}

/// Generates builder suffixes for numeric fields (min, max, step).
fn numeric_attrs_tokens(attrs: &[FormFieldAttr]) -> TokenStream2 {
    let mut ts = quote! {};
    for attr in attrs {
        match attr {
            FormFieldAttr::Min(n) => {
                let v = *n as f64;
                ts.extend(quote! { .min(#v, "") });
            }
            FormFieldAttr::Max(n) => {
                let v = *n as f64;
                ts.extend(quote! { .max(#v, "") });
            }
            FormFieldAttr::MinF(n) => ts.extend(quote! { .min(#n, "") }),
            FormFieldAttr::MaxF(n) => ts.extend(quote! { .max(#n, "") }),
            FormFieldAttr::Step(n) => ts.extend(quote! { .step(#n) }),
            _ => {}
        }
    }
    ts
}

/// Generates builder suffixes for file fields (upload_to, max_size).
fn file_attrs_tokens(attrs: &[FormFieldAttr]) -> TokenStream2 {
    let mut ts = quote! {};
    for attr in attrs {
        match attr {
            FormFieldAttr::UploadTo(path) => ts.extend(quote! { .upload_to(#path) }),
            FormFieldAttr::MaxSize(n) => {
                ts.extend(quote! { .max_size(::runique::forms::fields::FileSize::bytes(#n)) })
            }
            _ => {}
        }
    }
    ts
}

/// Generates the `.rows(n)` suffix if present.
fn rows_token(attrs: &[FormFieldAttr]) -> TokenStream2 {
    attrs
        .iter()
        .find_map(|a| {
            if let FormFieldAttr::Rows(n) = a {
                let n_usize = *n as usize;
                Some(quote! { .rows(#n_usize) })
            } else {
                None
            }
        })
        .unwrap_or_default()
}
