use crate::model::ast::{FormFieldAttr, FormFieldDecl, FormFieldKind};
use crate::registry::{FormWidget, PhantomColumn, PkKind, phantom_columns};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Ident, LitStr, Token, braced, parse::ParseStream};

// ── DSL ──────────────────────────────────────────────────────────────────────

pub(crate) struct ExtendDsl {
    pub table: String,
    pub fields: Vec<FormFieldDecl>,
}

impl syn::parse::Parse for ExtendDsl {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let kw: Ident = input.parse()?;
        if kw != "table" {
            return Err(syn::Error::new(kw.span(), "extend!{}: expected 'table'"));
        }
        input.parse::<Token![:]>()?;
        let table: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;

        let kw: Ident = input.parse()?;
        if kw != "fields" {
            return Err(syn::Error::new(kw.span(), "extend!{}: expected 'fields'"));
        }
        input.parse::<Token![:]>()?;
        let fields_content;
        braced!(fields_content in input);

        let mut fields = Vec::new();
        while !fields_content.is_empty() {
            fields.push(FormFieldDecl::parse(&fields_content)?);
        }
        let _ = input.parse::<Token![,]>();

        Ok(ExtendDsl {
            table: table.value(),
            fields,
        })
    }
}

// ── Code generation ──────────────────────────────────────────────────────────

fn field_to_coldef(ff: &FormFieldDecl) -> TokenStream2 {
    let name = ff.name.to_string();
    let is_required = ff
        .attrs
        .iter()
        .any(|a| matches!(a, FormFieldAttr::Required));
    let nullable = if is_required {
        quote! {}
    } else {
        quote! { .nullable() }
    };
    let max_length = ff.attrs.iter().find_map(|a| {
        if let FormFieldAttr::MaxLength(n) = a {
            Some(*n)
        } else {
            None
        }
    });

    let type_call = match &ff.kind {
        FormFieldKind::Textarea | FormFieldKind::Richtext => quote! { .text() },
        FormFieldKind::Int => quote! { .integer() },
        FormFieldKind::Bigint => quote! { .big_integer() },
        FormFieldKind::Float | FormFieldKind::Percent => quote! { .double() },
        FormFieldKind::Decimal => quote! { .decimal() },
        FormFieldKind::Bool => quote! { .boolean() },
        FormFieldKind::Date => quote! { .date() },
        FormFieldKind::Time => quote! { .time() },
        FormFieldKind::Datetime => quote! { .datetime() },
        FormFieldKind::Uuid => quote! { .uuid() },
        FormFieldKind::Json => quote! { .json() },
        // Text variants + files + choice → varchar(n) or string()
        _ => {
            if let Some(n) = max_length {
                quote! { .varchar(#n) }
            } else {
                quote! { .string() }
            }
        }
    };

    quote! {
        .column(
            ::runique::migration::column::ColumnDef::new(#name)
                #type_call
                #nullable
        )
    }
}

fn form_field_to_rust_type(kind: &FormFieldKind, nullable: bool) -> TokenStream2 {
    let base = match kind {
        FormFieldKind::Text
        | FormFieldKind::Email
        | FormFieldKind::Password
        | FormFieldKind::Textarea
        | FormFieldKind::Richtext
        | FormFieldKind::Url
        | FormFieldKind::Color
        | FormFieldKind::Slug
        | FormFieldKind::Phone
        | FormFieldKind::Ip
        | FormFieldKind::Image
        | FormFieldKind::Document
        | FormFieldKind::File
        | FormFieldKind::Choice
        | FormFieldKind::Radio
        | FormFieldKind::Checkbox => quote! { String },
        FormFieldKind::Int => quote! { i32 },
        FormFieldKind::Bigint => quote! { i64 },
        FormFieldKind::Float | FormFieldKind::Percent => quote! { f64 },
        FormFieldKind::Decimal => quote! { ::sea_orm::prelude::Decimal },
        FormFieldKind::Bool => quote! { bool },
        FormFieldKind::Date => quote! { ::chrono::NaiveDate },
        FormFieldKind::Time => quote! { ::chrono::NaiveTime },
        FormFieldKind::Datetime => quote! { ::chrono::NaiveDateTime },
        FormFieldKind::Uuid => quote! { ::sea_orm::prelude::Uuid },
        FormFieldKind::Json => quote! { ::runique::serde_json::Value },
    };
    if nullable {
        quote! { Option<#base> }
    } else {
        base
    }
}

fn table_to_form_ident(table: &str) -> proc_macro2::Ident {
    let pascal: String = table
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect();
    format_ident!("{}AdminForm", pascal)
}

/// Generates the `ActiveValue::Set(...)` expression for a single extended field.
/// `partial = true` → wraps in `if __data.contains_key(...) { Set } else { NotSet }`.
fn extend_active_model_field(ff: &FormFieldDecl, partial: bool) -> TokenStream2 {
    let name = &ff.name;
    let name_str = name.to_string();
    let required = ff
        .attrs
        .iter()
        .any(|a| matches!(a, FormFieldAttr::Required));

    let set_expr: TokenStream2 = match &ff.kind {
        FormFieldKind::Bool => {
            if required {
                quote! {
                    ::sea_orm::ActiveValue::Set(
                        __data.get(#name_str)
                            .map(|v| { let s = v.as_str(); s == "true" || s == "1" || s == "on" })
                            .unwrap_or(false)
                    )
                }
            } else {
                quote! {
                    ::sea_orm::ActiveValue::Set(
                        __data.get(#name_str).map(|v| { let s = v.as_str(); s == "true" || s == "1" || s == "on" })
                    )
                }
            }
        }
        FormFieldKind::Int => {
            if required {
                quote! { ::sea_orm::ActiveValue::Set(__data.get(#name_str).and_then(|v| v.parse::<i32>().ok()).unwrap_or_default()) }
            } else {
                quote! { ::sea_orm::ActiveValue::Set(__data.get(#name_str).and_then(|v| v.parse::<i32>().ok())) }
            }
        }
        FormFieldKind::Bigint => {
            if required {
                quote! { ::sea_orm::ActiveValue::Set(__data.get(#name_str).and_then(|v| v.parse::<i64>().ok()).unwrap_or_default()) }
            } else {
                quote! { ::sea_orm::ActiveValue::Set(__data.get(#name_str).and_then(|v| v.parse::<i64>().ok())) }
            }
        }
        FormFieldKind::Float | FormFieldKind::Percent => {
            if required {
                quote! { ::sea_orm::ActiveValue::Set(__data.get(#name_str).and_then(|v| v.parse::<f64>().ok()).unwrap_or_default()) }
            } else {
                quote! { ::sea_orm::ActiveValue::Set(__data.get(#name_str).and_then(|v| v.parse::<f64>().ok())) }
            }
        }
        FormFieldKind::Decimal => {
            if required {
                quote! { ::sea_orm::ActiveValue::Set(__data.get(#name_str).and_then(|v| v.parse::<::sea_orm::prelude::Decimal>().ok()).unwrap_or_default()) }
            } else {
                quote! { ::sea_orm::ActiveValue::Set(__data.get(#name_str).and_then(|v| v.parse::<::sea_orm::prelude::Decimal>().ok())) }
            }
        }
        FormFieldKind::Date => {
            quote! {
                ::sea_orm::ActiveValue::Set(
                    __data.get(#name_str).and_then(|v| {
                        if v.is_empty() { return None; }
                        ::chrono::NaiveDate::parse_from_str(v, "%Y-%m-%d").ok()
                    })
                )
            }
        }
        FormFieldKind::Time => {
            quote! {
                ::sea_orm::ActiveValue::Set(
                    __data.get(#name_str).and_then(|v| {
                        if v.is_empty() { return None; }
                        ::chrono::NaiveTime::parse_from_str(v, "%H:%M:%S")
                            .or_else(|_| ::chrono::NaiveTime::parse_from_str(v, "%H:%M"))
                            .ok()
                    })
                )
            }
        }
        FormFieldKind::Datetime => {
            quote! {
                ::sea_orm::ActiveValue::Set(
                    __data.get(#name_str).and_then(|v| {
                        if v.is_empty() { return None; }
                        ::chrono::NaiveDateTime::parse_from_str(v, "%Y-%m-%dT%H:%M:%S")
                            .or_else(|_| ::chrono::NaiveDateTime::parse_from_str(v, "%Y-%m-%dT%H:%M"))
                            .ok()
                    })
                )
            }
        }
        FormFieldKind::Uuid => {
            if required {
                quote! {
                    ::sea_orm::ActiveValue::Set(
                        __data.get(#name_str)
                            .and_then(|v| ::sea_orm::prelude::Uuid::parse_str(v).ok())
                            .unwrap_or_else(::sea_orm::prelude::Uuid::new_v4)
                    )
                }
            } else {
                quote! {
                    ::sea_orm::ActiveValue::Set(
                        __data.get(#name_str).and_then(|v| ::sea_orm::prelude::Uuid::parse_str(v).ok())
                    )
                }
            }
        }
        FormFieldKind::Json => {
            if required {
                quote! {
                    ::sea_orm::ActiveValue::Set(
                        __data.get(#name_str)
                            .and_then(|v| ::runique::serde_json::from_str(v).ok())
                            .unwrap_or(::runique::serde_json::Value::Null)
                    )
                }
            } else {
                quote! {
                    ::sea_orm::ActiveValue::Set(
                        __data.get(#name_str)
                            .filter(|v| !v.is_empty())
                            .and_then(|v| ::runique::serde_json::from_str(v).ok())
                    )
                }
            }
        }
        // All string-like fields
        _ => {
            if required {
                quote! {
                    ::sea_orm::ActiveValue::Set(
                        __data.get(#name_str).map(|v| v.trim().to_string()).unwrap_or_default()
                    )
                }
            } else {
                quote! {
                    ::sea_orm::ActiveValue::Set(
                        __data.get(#name_str).map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
                    )
                }
            }
        }
    };

    if partial {
        quote! {
            #name: match __data.contains_key(#name_str) {
                true  => #set_expr,
                false => ::sea_orm::ActiveValue::NotSet,
            },
        }
    } else {
        quote! { #name: #set_expr, }
    }
}

fn phantom_label(name: &str) -> String {
    let s = name.replace('_', " ");
    let mut chars = s.chars();
    match chars.next() {
        None => s,
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

/// Generates `form.field(...)` for a phantom column based on its FormWidget.
/// Returns None for Skip / AutoDateTime columns.
fn phantom_form_registration(col: &PhantomColumn) -> Option<TokenStream2> {
    let name = col.name;
    let label = phantom_label(name);
    match &col.widget {
        FormWidget::Text => Some(quote! {
            form.field(&::runique::forms::fields::TextField::text(#name).label(#label).required());
        }),
        FormWidget::Email => Some(quote! {
            form.field(&::runique::forms::fields::TextField::email(#name).label(#label).required());
        }),
        FormWidget::Password => Some(quote! {
            form.field(&::runique::forms::fields::TextField::password(#name).label(#label));
        }),
        FormWidget::Bool => Some(quote! {
            form.field(&::runique::forms::fields::BooleanField::new(#name).label(#label));
        }),
        FormWidget::AutoDateTime | FormWidget::Skip => None,
    }
}

/// Generates the ActiveModel assignment for a phantom column.
/// Returns None for Skip / AutoDateTime (left to `..Default::default()`).
fn phantom_active_model_field(col: &PhantomColumn) -> Option<TokenStream2> {
    let name = format_ident!("{}", col.name);
    let name_str = col.name;
    match &col.widget {
        FormWidget::Skip | FormWidget::AutoDateTime => None,
        FormWidget::Password => Some(quote! {
            #name: match __data.get(#name_str).map(|v| v.trim().to_string()).filter(|v| !v.is_empty()) {
                Some(v) => ::sea_orm::ActiveValue::Set(
                    ::runique::utils::password::hash(&v).unwrap_or_else(|_| v.clone())
                ),
                None => ::sea_orm::ActiveValue::NotSet,
            },
        }),
        FormWidget::Bool => Some(quote! {
            #name: ::sea_orm::ActiveValue::Set(
                __data.get(#name_str)
                    .map(|v| { let s = v.as_str(); s == "true" || s == "1" || s == "on" })
                    .unwrap_or(false)
            ),
        }),
        FormWidget::Text | FormWidget::Email => Some(quote! {
            #name: ::sea_orm::ActiveValue::Set(
                __data.get(#name_str).map(|v| v.trim().to_string()).unwrap_or_default()
            ),
        }),
    }
}

fn extend_file_attrs(attrs: &[FormFieldAttr]) -> TokenStream2 {
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

fn extend_form_field_registration(ff: &FormFieldDecl) -> TokenStream2 {
    if ff.attrs.iter().any(|a| matches!(a, FormFieldAttr::Skip)) {
        return quote! {};
    }

    let name_str = ff.name.to_string();
    let label = {
        let s = name_str.replace('_', " ");
        let mut chars = s.chars();
        match chars.next() {
            None => s,
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        }
    };

    let required = if ff
        .attrs
        .iter()
        .any(|a| matches!(a, FormFieldAttr::Required))
    {
        quote! { .required() }
    } else {
        quote! {}
    };

    let field_expr: TokenStream2 = match &ff.kind {
        FormFieldKind::Text => {
            quote! { ::runique::forms::fields::TextField::text(#name_str).label(#label) #required }
        }
        FormFieldKind::Email => {
            quote! { ::runique::forms::fields::TextField::email(#name_str).label(#label) #required }
        }
        FormFieldKind::Password => {
            quote! { ::runique::forms::fields::TextField::password(#name_str).label(#label) #required }
        }
        FormFieldKind::Richtext => {
            quote! { ::runique::forms::fields::TextField::richtext(#name_str).label(#label) #required }
        }
        FormFieldKind::Textarea => {
            quote! { ::runique::forms::fields::TextField::textarea(#name_str).label(#label) #required }
        }
        FormFieldKind::Url => {
            quote! { ::runique::forms::fields::TextField::url(#name_str).label(#label) #required }
        }
        FormFieldKind::Phone => {
            quote! { ::runique::forms::fields::TextField::phone(#name_str).label(#label) #required }
        }
        FormFieldKind::Slug => {
            quote! { ::runique::forms::fields::SlugField::new(#name_str).label(#label) }
        }
        FormFieldKind::Color => {
            quote! { ::runique::forms::fields::ColorField::new(#name_str).label(#label) #required }
        }
        FormFieldKind::Ip => {
            quote! { ::runique::forms::fields::IPAddressField::new(#name_str).label(#label) #required }
        }
        FormFieldKind::Uuid => {
            quote! { ::runique::forms::fields::UUIDField::new(#name_str).label(#label) #required }
        }
        FormFieldKind::Json => {
            quote! { ::runique::forms::fields::JSONField::new(#name_str).label(#label) #required }
        }
        FormFieldKind::Int | FormFieldKind::Bigint => {
            quote! { ::runique::forms::fields::NumericField::integer(#name_str).label(#label) }
        }
        FormFieldKind::Float | FormFieldKind::Percent => {
            quote! { ::runique::forms::fields::NumericField::float(#name_str).label(#label) }
        }
        FormFieldKind::Decimal => {
            quote! { ::runique::forms::fields::NumericField::decimal(#name_str).label(#label) }
        }
        FormFieldKind::Bool => {
            quote! { ::runique::forms::fields::BooleanField::new(#name_str).label(#label) #required }
        }
        FormFieldKind::Date => {
            quote! { ::runique::forms::fields::DateField::new(#name_str).label(#label) #required }
        }
        FormFieldKind::Time => {
            quote! { ::runique::forms::fields::TimeField::new(#name_str).label(#label) #required }
        }
        FormFieldKind::Datetime => {
            quote! { ::runique::forms::fields::DateTimeField::new(#name_str).label(#label) #required }
        }
        FormFieldKind::Image => {
            let extras = extend_file_attrs(&ff.attrs);
            quote! { ::runique::forms::fields::FileField::image(#name_str).label(#label) #extras #required }
        }
        FormFieldKind::Document => {
            let extras = extend_file_attrs(&ff.attrs);
            quote! { ::runique::forms::fields::FileField::document(#name_str).label(#label) #extras #required }
        }
        FormFieldKind::File => {
            let extras = extend_file_attrs(&ff.attrs);
            quote! { ::runique::forms::fields::FileField::any(#name_str).label(#label) #extras #required }
        }
        FormFieldKind::Choice | FormFieldKind::Radio | FormFieldKind::Checkbox => {
            quote! { ::runique::forms::fields::ChoiceField::new(#name_str).label(#label) #required }
        }
    };

    quote! { form.field(&#field_expr); }
}

/// Generates a complete SeaORM entity (Model + Relation + ActiveModelBehavior)
/// plus an AdminForm, from the phantom base columns + user-declared extended columns.
pub(crate) fn generate_entity(dsl: &ExtendDsl) -> TokenStream2 {
    let table = &dsl.table;
    let base_cols = phantom_columns(table);

    let phantom_fields: Vec<TokenStream2> = base_cols
        .iter()
        .map(|col| {
            let name = format_ident!("{}", col.name);
            let base_ty = col.ty.to_tokens();
            let ty = if col.nullable {
                quote! { Option<#base_ty> }
            } else {
                base_ty
            };
            match col.pk {
                PkKind::Auto => quote! {
                    #[sea_orm(primary_key)]
                    pub #name: #ty,
                },
                PkKind::Composite => quote! {
                    #[sea_orm(primary_key, auto_increment = false)]
                    pub #name: #ty,
                },
                PkKind::NotPk => quote! {
                    pub #name: #ty,
                },
            }
        })
        .collect();

    let extended_fields: Vec<TokenStream2> = dsl
        .fields
        .iter()
        .map(|ff| {
            let name = &ff.name;
            let nullable = !ff
                .attrs
                .iter()
                .any(|a| matches!(a, FormFieldAttr::Required));
            let ty = form_field_to_rust_type(&ff.kind, nullable);
            quote! { pub #name: #ty, }
        })
        .collect();

    let form_name = table_to_form_ident(table);

    // Form: phantom columns first, then extended
    let phantom_registrations: Vec<TokenStream2> = base_cols
        .iter()
        .filter_map(phantom_form_registration)
        .collect();
    let extended_registrations: Vec<TokenStream2> = dsl
        .fields
        .iter()
        .map(extend_form_field_registration)
        .collect();

    // ActiveModel: phantom columns + extended columns
    let phantom_am_fields: Vec<TokenStream2> = base_cols
        .iter()
        .filter_map(phantom_active_model_field)
        .collect();
    let full_assignments: Vec<TokenStream2> = dsl
        .fields
        .iter()
        .map(|ff| extend_active_model_field(ff, false))
        .collect();
    let partial_assignments: Vec<TokenStream2> = dsl
        .fields
        .iter()
        .map(|ff| extend_active_model_field(ff, true))
        .collect();

    quote! {
        #[derive(
            Clone, Debug, PartialEq,
            ::sea_orm::DeriveEntityModel,
            ::serde::Serialize,
            ::serde::Deserialize,
        )]
        #[sea_orm(table_name = #table)]
        pub struct Model {
            #(#phantom_fields)*
            #(#extended_fields)*
        }

        #[derive(Copy, Clone, Debug, ::sea_orm::EnumIter, ::sea_orm::DeriveRelation)]
        pub enum Relation {}

        impl ::sea_orm::ActiveModelBehavior for ActiveModel {}

        pub fn admin_from_form(
            __data: &::std::collections::HashMap<::std::string::String, ::std::string::String>,
            __id: ::std::option::Option<::runique::utils::config::Pk>,
        ) -> ActiveModel {
            ActiveModel {
                id: match __id {
                    ::std::option::Option::Some(pk) => ::sea_orm::ActiveValue::Unchanged(pk),
                    ::std::option::Option::None     => ::sea_orm::ActiveValue::NotSet,
                },
                #(#phantom_am_fields)*
                #(#full_assignments)*
                ..::std::default::Default::default()
            }
        }

        pub fn admin_partial_update(
            __data: &::std::collections::HashMap<::std::string::String, ::std::string::String>,
            __id: ::runique::utils::config::Pk,
        ) -> ActiveModel {
            ActiveModel {
                id: ::sea_orm::ActiveValue::Unchanged(__id),
                #(#phantom_am_fields)*
                #(#partial_assignments)*
                ..::std::default::Default::default()
            }
        }

        pub type AdminForm = #form_name;

        #[derive(::runique::serde::Serialize, Debug, Clone)]
        pub struct #form_name {
            pub form: ::runique::forms::Forms,
        }

        impl ::runique::forms::field::RuniqueForm for #form_name {
            fn register_fields(form: &mut ::runique::forms::Forms) {
                #(#phantom_registrations)*
                #(#extended_registrations)*
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

/// Generates `pub fn schema() -> ModelSchema { ... }` from the parsed DSL.
pub(crate) fn generate_schema_fn(dsl: &ExtendDsl) -> TokenStream2 {
    let table = &dsl.table;
    let col_defs: Vec<TokenStream2> = dsl.fields.iter().map(field_to_coldef).collect();

    quote! {
        pub fn schema() -> ::runique::migration::schema::ModelSchema {
            ::runique::migration::schema::ModelSchema::new(#table)
                .table_name(#table)
                #(#col_defs)*
        }
    }
}
