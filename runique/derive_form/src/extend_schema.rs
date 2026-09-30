use crate::model::ast::{EnumDef, FormFieldAttr, FormFieldDecl, FormFieldKind};
use crate::model::generateur::{
    field_from_form, field_from_partial, generate_column, generate_enum_defs,
    generate_form_field_decl,
};
use crate::model::utils::generate_model_field;
use crate::registry::{FormWidget, PhantomColumn, PhantomType, PkKind, phantom_columns};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use runique_dsl::ast::FieldDef;
pub(crate) use runique_dsl::extend::ExtendDsl;
use runique_dsl::form_field_to_field_def;

// ── Code generation ──────────────────────────────────────────────────────────

/// Resolves the `EnumDef` a choice/radio field refers to via its `[enum(Name)]` attr.
fn field_enum_def<'a>(ff: &FormFieldDecl, enums: &'a [EnumDef]) -> Option<&'a EnumDef> {
    if !matches!(
        ff.kind,
        FormFieldKind::Choice | FormFieldKind::Radio | FormFieldKind::Checkbox
    ) {
        return None;
    }
    ff.attrs.iter().find_map(|a| {
        if let FormFieldAttr::EnumRef(id) = a {
            enums.iter().find(|e| e.name == *id)
        } else {
            None
        }
    })
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
/// `partial = true` → use `NotSet` when the key is absent (for `admin_partial_update`).
fn phantom_active_model_field(col: &PhantomColumn, partial: bool) -> Option<TokenStream2> {
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
        FormWidget::Bool => {
            if partial {
                Some(quote! {
                    #name: match __data.get(#name_str) {
                        Some(v) => ::sea_orm::ActiveValue::Set({
                            let s = v.as_str(); s == "true" || s == "1" || s == "on"
                        }),
                        None => ::sea_orm::ActiveValue::NotSet,
                    },
                })
            } else {
                Some(quote! {
                    #name: ::sea_orm::ActiveValue::Set(
                        __data.get(#name_str)
                            .map(|v| { let s = v.as_str(); s == "true" || s == "1" || s == "on" })
                            .unwrap_or(false)
                    ),
                })
            }
        }
        FormWidget::Text | FormWidget::Email => {
            if partial {
                Some(quote! {
                    #name: match __data.get(#name_str) {
                        Some(v) => ::sea_orm::ActiveValue::Set(v.trim().to_string()),
                        None => ::sea_orm::ActiveValue::NotSet,
                    },
                })
            } else {
                Some(quote! {
                    #name: ::sea_orm::ActiveValue::Set(
                        __data.get(#name_str).map(|v| v.trim().to_string()).unwrap_or_default()
                    ),
                })
            }
        }
    }
}

/// Generates a complete SeaORM entity (Model + Relation + ActiveModelBehavior)
/// plus an AdminForm, from the phantom base columns + user-declared extended columns.
pub(crate) fn generate_entity(dsl: &ExtendDsl) -> TokenStream2 {
    let table = &dsl.table;
    let base_cols = phantom_columns(table);

    // `admin_from_form`/`admin_partial_update` need the REAL id type of this
    // specific table, not a hardcoded one: `eihwaz_sessions.id` is a literal
    // `i32` (always auto-increment, whatever feature is active), while
    // `eihwaz_users.id`/`eihwaz_groupes.id` are the `Pk` alias (i32/i64/Uuid
    // depending on `big-pk`/`pk-uuid`). Composite-PK tables (`eihwaz_users_groupes`,
    // `eihwaz_groupes_droits`) have no single `id` column at all — `admin_from_form`
    // for those is a separate, currently-unimplemented gap; not generated here.
    let auto_pk_col = base_cols.iter().find(|c| matches!(c.pk, PkKind::Auto));
    let id_ty = auto_pk_col
        .map(|c| c.ty.to_tokens())
        .unwrap_or_else(|| quote! { ::runique::utils::config::Pk });
    let id_is_pk_alias = matches!(auto_pk_col.map(|c| &c.ty), Some(PhantomType::Pk));
    // Only `Pk`-aliased tables need a feature-gated branch: under `pk-uuid` there is
    // no DB auto-increment to fall back on, so a fresh id must be generated here;
    // literal-typed tables (sessions) always rely on the DB, in every feature combo.
    let id_none_arm = if id_is_pk_alias {
        if cfg!(feature = "pk-uuid") {
            quote! {
                ::std::option::Option::None => ::sea_orm::ActiveValue::Set(::sea_orm::prelude::Uuid::now_v7()),
            }
        } else {
            quote! {
                ::std::option::Option::None => ::sea_orm::ActiveValue::NotSet,
            }
        }
    } else {
        quote! {
            ::std::option::Option::None => ::sea_orm::ActiveValue::NotSet,
        }
    };

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
                // `PhantomType::Pk` emits the `Pk` alias (see `to_tokens` above), and
                // SeaORM's derive macro infers `auto_increment` from the literal type
                // name — it can't see through an alias, so it silently defaults to
                // `false`. Explicit per-feature attribute required here; columns typed
                // with a literal integer (e.g. `PhantomType::I32`) don't need it, SeaORM
                // recognizes those directly.
                PkKind::Auto if matches!(col.ty, PhantomType::Pk) => {
                    let auto_increment = !cfg!(feature = "pk-uuid");
                    quote! {
                        #[sea_orm(primary_key, auto_increment = #auto_increment)]
                        pub #name: #ty,
                    }
                }
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

    // Extended columns go through the same generators as `model!{}` fields:
    // same Rust type, same conversion, same form field.
    let extended_defs: Vec<FieldDef> = dsl.fields.iter().map(form_field_to_field_def).collect();
    let extended_fields: Vec<TokenStream2> =
        extended_defs.iter().map(generate_model_field).collect();

    let form_name = table_to_form_ident(table);

    // Form: phantom columns first, then extended
    let phantom_registrations: Vec<TokenStream2> = base_cols
        .iter()
        .filter_map(phantom_form_registration)
        .collect();
    let extended_registrations: Vec<TokenStream2> = dsl
        .fields
        .iter()
        .map(|ff| generate_form_field_decl(ff, &dsl.enums))
        .collect();

    // ActiveModel: phantom columns + extended columns
    // Two versions: full (admin_from_form) vs partial (admin_partial_update)
    let phantom_am_fields: Vec<TokenStream2> = base_cols
        .iter()
        .filter_map(|col| phantom_active_model_field(col, false))
        .collect();
    let phantom_am_fields_partial: Vec<TokenStream2> = base_cols
        .iter()
        .filter_map(|col| phantom_active_model_field(col, true))
        .collect();
    let full_assignments: Vec<TokenStream2> =
        extended_defs.iter().filter_map(field_from_form).collect();
    let partial_assignments: Vec<TokenStream2> = extended_defs
        .iter()
        .filter_map(field_from_partial)
        .collect();

    let enum_defs = generate_enum_defs(&dsl.enums);

    // Same enum-label resolver as `model!` (admin display layer). No-op if no enum.
    let enum_label_resolvers: Vec<TokenStream2> = dsl
        .fields
        .iter()
        .filter_map(|ff| {
            let def = field_enum_def(ff, &dsl.enums)?;
            let col = ff.name.to_string();
            let ename = &def.name;
            Some(quote! {
                {
                    let current = row
                        .get(#col)
                        .and_then(|v| v.as_str())
                        .map(::std::string::ToString::to_string);
                    if let Some(s) = current
                        && let Ok(e) = <#ename as ::std::str::FromStr>::from_str(&s)
                    {
                        row[#col] = ::runique::serde_json::Value::String(
                            ::std::string::ToString::to_string(&e),
                        );
                    }
                }
            })
        })
        .collect();

    quote! {
        #enum_defs

        pub fn apply_enum_labels(row: &mut ::runique::serde_json::Value) {
            let _ = &row;
            #(#enum_label_resolvers)*
        }

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

        #[allow(clippy::needless_update)]
        pub fn admin_from_form(
            __data: &::std::collections::HashMap<::std::string::String, ::std::string::String>,
            __id: ::std::option::Option<#id_ty>,
        ) -> ActiveModel {
            ActiveModel {
                id: match __id {
                    ::std::option::Option::Some(pk) => ::sea_orm::ActiveValue::Unchanged(pk),
                    #id_none_arm
                },
                #(#phantom_am_fields)*
                #(#full_assignments)*
                ..::std::default::Default::default()
            }
        }

        #[allow(clippy::needless_update)]
        pub fn admin_partial_update(
            __data: &::std::collections::HashMap<::std::string::String, ::std::string::String>,
            __id: #id_ty,
        ) -> ActiveModel {
            ActiveModel {
                id: ::sea_orm::ActiveValue::Unchanged(__id),
                #(#phantom_am_fields_partial)*
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

        pub const UNIQUE_FIELDS: &[&str] = &[];
    }
}

/// Generates `pub fn schema() -> ModelSchema { ... }` from the parsed DSL.
pub(crate) fn generate_schema_fn(dsl: &ExtendDsl) -> TokenStream2 {
    let table = &dsl.table;
    let col_defs: Vec<TokenStream2> = dsl
        .fields
        .iter()
        .map(|ff| generate_column(&form_field_to_field_def(ff), &dsl.enums))
        .collect();

    quote! {
        pub fn schema() -> ::runique::migration::schema::ModelSchema {
            ::runique::migration::schema::ModelSchema::new(#table)
                .table_name(#table)
                #(#col_defs)*
        }
    }
}
