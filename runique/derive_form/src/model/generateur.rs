use crate::model::ast::{self, *};
use crate::model::utils::*;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

pub fn generate(model: &ModelInput) -> TokenStream2 {
    let schema = generate_schema(model);
    let sea_model = generate_sea_model(model);
    let relation_enum = generate_relation_enum();
    let active_model = generate_active_model();
    let from_str_map = generate_from_str_map(model);
    let admin_form = generate_admin_form(model);

    quote! {
        #schema
        #sea_model
        #relation_enum
        #active_model
        #from_str_map
        #admin_form
    }
}

/// Génère `pub fn admin_from_form(data: &HashMap<String, String>, id: Option<PkType>) -> ActiveModel`
/// Cette fonction est utilisée par la vue admin pour créer/mettre à jour une entrée en DB
/// à partir des données de formulaire (HashMap<String, String>).
pub fn generate_from_str_map(model: &ModelInput) -> TokenStream2 {
    let pk_name = &model.pk.name;

    // La valeur du PK selon son type
    let pk_set = match model.pk.ty {
        PkType::I32 => quote! {
            #pk_name: match __id {
                ::std::option::Option::Some(pk) => ::sea_orm::ActiveValue::Unchanged(pk),
                ::std::option::Option::None    => ::sea_orm::ActiveValue::NotSet,
            },
        },
        PkType::I64 => quote! {
            #pk_name: match __id {
                ::std::option::Option::Some(pk) => ::sea_orm::ActiveValue::Unchanged(pk),
                ::std::option::Option::None    => ::sea_orm::ActiveValue::NotSet,
            },
        },
        PkType::Uuid => quote! {
            #pk_name: match __id {
                ::std::option::Option::Some(pk) => ::sea_orm::ActiveValue::Unchanged(pk),
                ::std::option::Option::None    => ::sea_orm::ActiveValue::Set(::sea_orm::prelude::Uuid::new_v4()),
            },
        },
    };

    // Un assignment par champ (les champs auto_now/auto_now_update sont exclus du Model → ignorés)
    let field_assignments: Vec<TokenStream2> = model.fields.iter().filter_map(|field| {
        let fname = &field.name;
        let fname_str = fname.to_string();

        let is_auto_now = field.options.iter().any(|o| matches!(o, FieldOption::AutoNow));
        let is_auto_now_update = field.options.iter().any(|o| matches!(o, FieldOption::AutoNowUpdate));
        let is_nullable = field.options.iter().any(|o| matches!(o, FieldOption::Nullable));

        // Ces champs n'existent pas dans l'ActiveModel (filtrés par generate_sea_model) → on les saute
        if is_auto_now || is_auto_now_update {
            return None;
        }

        let ts = match &field.ty {
            FieldType::Bool => {
                if is_nullable {
                    quote! {
                        #fname: ::sea_orm::ActiveValue::Set(
                            __data.get(#fname_str).map(|v| {
                                let s = v.as_str();
                                s == "true" || s == "1" || s == "on"
                            })
                        ),
                    }
                } else {
                    quote! {
                        #fname: ::sea_orm::ActiveValue::Set(
                            __data.get(#fname_str)
                                .map(|v| { let s = v.as_str(); s == "true" || s == "1" || s == "on" })
                                .unwrap_or(false)
                        ),
                    }
                }
            }
            FieldType::I8 | FieldType::I16 | FieldType::I32 | FieldType::U32 => {
                if is_nullable {
                    quote! {
                        #fname: ::sea_orm::ActiveValue::Set(
                            __data.get(#fname_str).and_then(|v| v.parse().ok())
                        ),
                    }
                } else {
                    quote! {
                        #fname: ::sea_orm::ActiveValue::Set(
                            __data.get(#fname_str).and_then(|v| v.parse().ok()).unwrap_or_default()
                        ),
                    }
                }
            }
            FieldType::I64 | FieldType::U64 => {
                if is_nullable {
                    quote! {
                        #fname: ::sea_orm::ActiveValue::Set(
                            __data.get(#fname_str).and_then(|v| v.parse().ok())
                        ),
                    }
                } else {
                    quote! {
                        #fname: ::sea_orm::ActiveValue::Set(
                            __data.get(#fname_str).and_then(|v| v.parse().ok()).unwrap_or_default()
                        ),
                    }
                }
            }
            FieldType::F32 | FieldType::F64 => {
                if is_nullable {
                    quote! {
                        #fname: ::sea_orm::ActiveValue::Set(
                            __data.get(#fname_str).and_then(|v| v.parse().ok())
                        ),
                    }
                } else {
                    quote! {
                        #fname: ::sea_orm::ActiveValue::Set(
                            __data.get(#fname_str).and_then(|v| v.parse().ok()).unwrap_or_default()
                        ),
                    }
                }
            }
            FieldType::Datetime | FieldType::Timestamp | FieldType::TimestampTz
            | FieldType::Date | FieldType::Time => {
                // Champs date/time sans auto_now : laissé à Default (NotSet)
                return None;
            }
            FieldType::Uuid => {
                if is_nullable {
                    quote! {
                        #fname: ::sea_orm::ActiveValue::Set(
                            __data.get(#fname_str).and_then(|v| ::sea_orm::prelude::Uuid::parse_str(v).ok())
                        ),
                    }
                } else {
                    quote! {
                        #fname: ::sea_orm::ActiveValue::Set(
                            __data.get(#fname_str)
                                .and_then(|v| ::sea_orm::prelude::Uuid::parse_str(v).ok())
                                .unwrap_or_else(::sea_orm::prelude::Uuid::new_v4)
                        ),
                    }
                }
            }
            FieldType::Json | FieldType::JsonBinary => {
                if is_nullable {
                    quote! {
                        #fname: ::sea_orm::ActiveValue::Set(
                            __data.get(#fname_str)
                                .filter(|v| !v.is_empty())
                                .and_then(|v| ::runique::serde_json::from_str(v).ok())
                        ),
                    }
                } else {
                    quote! {
                        #fname: ::sea_orm::ActiveValue::Set(
                            __data.get(#fname_str)
                                .and_then(|v| ::runique::serde_json::from_str(v).ok())
                                .unwrap_or(::runique::serde_json::Value::Null)
                        ),
                    }
                }
            }
            // String, Text, Char, Varchar, Blob, Inet, Cidr, MacAddress, Interval, Enum, Binary, VarBinary
            _ => {
                let is_password = fname_str.contains("password");
                if is_password {
                    // Champs mot de passe : hachage automatique via la config globale du dev.
                    // Si vide → NotSet (ne pas écraser lors d'un edit sans nouveau mot de passe).
                    if is_nullable {
                        quote! {
                            #fname: match __data.get(#fname_str).map(|v| v.trim().to_string()).filter(|v| !v.is_empty()) {
                                Some(v) => ::sea_orm::ActiveValue::Set(
                                    Some(::runique::utils::password::hash(&v).unwrap_or_else(|_| v.clone()))
                                ),
                                None => ::sea_orm::ActiveValue::Set(None),
                            },
                        }
                    } else {
                        quote! {
                            #fname: match __data.get(#fname_str).map(|v| v.trim().to_string()).filter(|v| !v.is_empty()) {
                                Some(v) => ::sea_orm::ActiveValue::Set(
                                    ::runique::utils::password::hash(&v).unwrap_or_else(|_| v.clone())
                                ),
                                None => ::sea_orm::ActiveValue::NotSet,
                            },
                        }
                    }
                } else if is_nullable {
                    quote! {
                        #fname: ::sea_orm::ActiveValue::Set(
                            __data.get(#fname_str).map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
                        ),
                    }
                } else {
                    quote! {
                        #fname: ::sea_orm::ActiveValue::Set(
                            __data.get(#fname_str).map(|v| v.trim().to_string()).unwrap_or_default()
                        ),
                    }
                }
            }
        };
        Some(ts)
    }).collect();

    // Type du PK pour la signature
    let pk_type = match model.pk.ty {
        PkType::I32 => quote! { i32 },
        PkType::I64 => quote! { i64 },
        PkType::Uuid => quote! { ::sea_orm::prelude::Uuid },
    };

    quote! {
        /// Construit un `ActiveModel` à partir d'une map de données de formulaire (vue admin).
        /// - `id = Some(pk)` → mise à jour (Unchanged sur la PK)
        /// - `id = None`     → création (NotSet ou Uuid::new_v4() selon le type de PK)
        #[allow(clippy::needless_update)]
        pub fn admin_from_form(
            __data: &::std::collections::HashMap<::std::string::String, ::std::string::String>,
            __id: ::std::option::Option<#pk_type>,
        ) -> ActiveModel {
            ActiveModel {
                #pk_set
                #(#field_assignments)*
                ..::std::default::Default::default()
            }
        }
    }
}

pub fn generate_pk(pk: &PkDef) -> TokenStream2 {
    let name = pk.name.to_string();
    match pk.ty {
        PkType::I32 => quote! {
            .primary_key(::runique::migration::PrimaryKeyDef::new(#name).i32().auto_increment())
        },
        PkType::I64 => quote! {
            .primary_key(::runique::migration::PrimaryKeyDef::new(#name).i64().auto_increment())
        },
        PkType::Uuid => quote! {
            .primary_key(::runique::migration::PrimaryKeyDef::new(#name).uuid())
        },
    }
}

pub fn generate_column(field: &FieldDef) -> TokenStream2 {
    let name = field.name.to_string();
    let ty = generate_field_type(&field.ty);
    let options: Vec<TokenStream2> = field.options.iter().map(generate_option).collect();

    quote! {
        .column(::runique::migration::ColumnDef::new(#name) #ty #(#options)*)
    }
}

fn generate_field_type(ty: &FieldType) -> TokenStream2 {
    match ty {
        FieldType::String => quote! { .string() },
        FieldType::Text => quote! { .text() },
        FieldType::Char => quote! { .char() },
        FieldType::Varchar(n) => quote! { .varchar(#n) },
        FieldType::I8 => quote! { .tiny_integer() },
        FieldType::I16 => quote! { .small_integer() },
        FieldType::I32 => quote! { .integer() },
        FieldType::I64 => quote! { .big_integer() },
        FieldType::U32 => quote! { .unsigned() },
        FieldType::U64 => quote! { .big_unsigned() },
        FieldType::F32 => quote! { .float() },
        FieldType::F64 => quote! { .double() },
        FieldType::Decimal(None) => quote! { .decimal() },
        FieldType::Decimal(Some((p, s))) => quote! { .decimal_len(#p, #s) },
        FieldType::Bool => quote! { .boolean() },
        FieldType::Date => quote! { .date() },
        FieldType::Time => quote! { .time() },
        FieldType::Datetime => quote! { .datetime() },
        FieldType::Timestamp => quote! { .timestamp() },
        FieldType::TimestampTz => quote! { .timestamp_tz() },
        FieldType::Uuid => quote! { .uuid() },
        FieldType::Json => quote! { .json() },
        FieldType::JsonBinary => quote! { .json_binary() },
        FieldType::Binary(None) => quote! { .binary() },
        FieldType::Binary(Some(n)) => quote! { .binary_len(#n) },
        FieldType::VarBinary(n) => quote! { .var_binary(#n) },
        FieldType::Blob => quote! { .blob() },
        FieldType::Inet => quote! { .inet() },
        FieldType::Cidr => quote! { .cidr() },
        FieldType::MacAddress => quote! { .mac_address() },
        FieldType::Interval => quote! { .interval() },
        FieldType::Enum(variants) => {
            let variant_strs: Vec<String> = variants.iter().map(|v| v.to_string()).collect();
            quote! { .enum_type("enum", vec![#(#variant_strs.to_string()),*]) }
        }
    }
}

/// Génère `{ModelName}AdminForm` — formulaire auto-généré à partir du modèle.
/// Couvre tous les champs non-auto_now avec le type de field approprié.
pub fn generate_admin_form(model: &ModelInput) -> TokenStream2 {
    let model_name = &model.name;
    let form_name = quote::format_ident!("{}AdminForm", model_name);

    let field_registrations: Vec<TokenStream2> = model.fields.iter().filter_map(|field| {
        let fname = &field.name;
        let fname_str = fname.to_string();

        let is_auto_now = field.options.iter().any(|o| matches!(o, FieldOption::AutoNow));
        let is_auto_now_update = field.options.iter().any(|o| matches!(o, FieldOption::AutoNowUpdate));
        let is_required = field.options.iter().any(|o| matches!(o, FieldOption::Required));
        let is_nullable = field.options.iter().any(|o| matches!(o, FieldOption::Nullable));

        if is_auto_now || is_auto_now_update {
            return None;
        }

        // Label auto-généré : snake_case → "First word capitalized"
        let label = {
            let s = fname_str.replace('_', " ");
            let mut chars = s.chars();
            match chars.next() {
                None => s,
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        };

        let required_suffix = if is_required && !is_nullable {
            quote! { .required() }
        } else {
            quote! {}
        };

        // Champ fichier — priorité sur l'inférence par type
        if let Some(file_opt) = field.options.iter().find_map(|o| {
            if let FieldOption::File { kind, upload_to } = o { Some((kind, upload_to)) } else { None }
        }) {
            let (kind, upload_to) = file_opt;
            let file_constructor = match kind {
                FileKind::Image    => quote! { ::runique::forms::fields::FileField::image(#fname_str) },
                FileKind::Document => quote! { ::runique::forms::fields::FileField::document(#fname_str) },
                FileKind::Any      => quote! { ::runique::forms::fields::FileField::any(#fname_str) },
            };
            let upload_suffix = match upload_to {
                Some(path) => quote! { .upload_to(#path) },
                None       => quote! {},
            };
            return Some(quote! {
                form.field(&#file_constructor.label(#label) #upload_suffix #required_suffix);
            });
        }

        let registration = match &field.ty {
            FieldType::Bool => quote! {
                form.field(&::runique::forms::fields::BooleanField::new(#fname_str).label(#label) #required_suffix);
            },
            FieldType::I8 | FieldType::I16 | FieldType::I32 | FieldType::U32
            | FieldType::I64 | FieldType::U64 => quote! {
                form.field(&::runique::forms::fields::NumericField::integer(#fname_str).label(#label));
            },
            FieldType::F32 | FieldType::F64 => quote! {
                form.field(&::runique::forms::fields::NumericField::float(#fname_str).label(#label));
            },
            FieldType::Decimal(_) => quote! {
                form.field(&::runique::forms::fields::NumericField::decimal(#fname_str).label(#label));
            },
            FieldType::Date => quote! {
                form.field(&::runique::forms::fields::DateField::new(#fname_str).label(#label) #required_suffix);
            },
            FieldType::Time => quote! {
                form.field(&::runique::forms::fields::TimeField::new(#fname_str).label(#label) #required_suffix);
            },
            FieldType::Datetime | FieldType::Timestamp | FieldType::TimestampTz => quote! {
                form.field(&::runique::forms::fields::DateTimeField::new(#fname_str).label(#label) #required_suffix);
            },
            FieldType::Json | FieldType::JsonBinary => quote! {
                form.field(&::runique::forms::fields::TextField::textarea(#fname_str).label(#label) #required_suffix);
            },
            // String, Text, Char, Varchar, Uuid, Blob, Inet, Cidr, MacAddress, Interval, Enum, Binary, VarBinary
            _ => {
                let is_password = fname_str.contains("password");
                if is_password {
                    quote! {
                        form.field(&::runique::forms::fields::TextField::password(#fname_str).label(#label) #required_suffix);
                    }
                } else {
                    quote! {
                        form.field(&::runique::forms::fields::TextField::text(#fname_str).label(#label) #required_suffix);
                    }
                }
            }
        };

        Some(registration)
    }).collect();

    quote! {
        /// Alias stable utilisé par le daemon `runique start` pour référencer ce formulaire.
        /// Toujours disponible via `{module}::AdminForm`.
        pub type AdminForm = #form_name;

        /// Formulaire admin auto-généré à partir du modèle.
        /// Couvre tous les champs du modèle sauf les champs auto_now/auto_now_update.
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

fn generate_option(opt: &FieldOption) -> TokenStream2 {
    match opt {
        FieldOption::Required => quote! { .required() },
        FieldOption::Nullable => quote! { .nullable() },
        FieldOption::Unique => quote! { .unique() },
        FieldOption::Index => quote! {},
        FieldOption::AutoNow => quote! { .auto_now() },
        FieldOption::AutoNowUpdate => quote! { .auto_now_update() },
        FieldOption::Readonly => quote! { .ignore() },
        FieldOption::MaxLen(n) => quote! { .max_len(#n) },
        FieldOption::MinLen(n) => quote! { .min_len(#n) },
        FieldOption::Max(n) => quote! { .max_i64(#n) },
        FieldOption::Min(n) => quote! { .min_i64(#n) },
        FieldOption::MaxF(n) => quote! { .max_f64(#n) },
        FieldOption::MinF(n) => quote! { .min_f64(#n) },
        FieldOption::SelectAs(s) => quote! { .select_as(#s) },
        FieldOption::Default(lit) => quote! { .default(sea_query::Value::from(#lit)) },
        FieldOption::Label(_) | FieldOption::Help(_) => quote! {},
        FieldOption::File { .. } => quote! {},
        FieldOption::Fk(fk) => {
            let table = fk.table.to_string();
            let column = fk.column.to_string();
            let action = match fk.action {
                FkAction::Cascade => quote! { ::sea_orm::sea_query::ForeignKeyAction::Cascade },
                FkAction::SetNull => quote! { ::sea_orm::sea_query::ForeignKeyAction::SetNull },
                FkAction::Restrict => quote! { ::sea_orm::sea_query::ForeignKeyAction::Restrict },
                FkAction::SetDefault => {
                    quote! { ::sea_orm::sea_query::ForeignKeyAction::SetDefault }
                }
            };

            // note: les FK seront générées séparément
            let _ = (table, column, action);
            quote! {}
        }
    }
}
