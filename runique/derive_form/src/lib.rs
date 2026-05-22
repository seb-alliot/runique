use proc_macro::TokenStream;

mod extend_schema;
mod model;
mod registry;
mod schema_form;

/// Derives a validated HTML form bound to a SeaORM entity schema.
///
/// # Required attribute
///
/// `schema = path::to::entity` — the SeaORM entity module (e.g. `crate::entities::contact`).
/// The macro reads the entity's column list to generate field accessors and validation.
///
/// # Optional attributes
///
/// - `fields = [field1, field2, ...]` — include only these fields (allowlist)
/// - `exclude = [field1, field2, ...]` — exclude these fields (denylist)
/// - `model = MyModel` — override the generated model name (defaults to struct name)
///
/// # Example
///
/// ```rust,ignore
/// #[form(schema = crate::entities::contact, fields = [email, message])]
/// pub struct ContactForm;
/// impl RuniqueForm for ContactForm {
///     impl_form_access!(model);
/// }
/// ```
///
/// The struct must be a unit struct (`pub struct Foo;`). The macro generates
/// a `pub form: Forms` field and implements `ModelForm`.
///
/// # Field types available in templates
///
/// `TextField`, `DecimalField`, `IntField`, `BoolField`, `DateTimeField`,
/// `FileField`, `ChoiceField`, `FkField`, `TextAreaField`
#[proc_macro_attribute]
pub fn form(attr: TokenStream, item: TokenStream) -> TokenStream {
    if attr.is_empty() {
        return quote::quote! {
            compile_error!(
                "#[form] requires `schema = path::to::entity`.\n\
                Example: #[form(schema = crate::entities::my_entity, fields = [field1, field2])]"
            );
        }
        .into();
    }
    schema_form::model_schema(attr, item)
}

#[proc_macro]
pub fn model(input: TokenStream) -> TokenStream {
    model::model_impl(input)
}

/// Declares a framework table extension — used by `makemigrations` to generate
/// the corresponding `ALTER TABLE ADD COLUMN` statements. Has no effect at compile time.
///
/// # Example
///
/// ```rust,ignore
/// extend! {
///     table: "eihwaz_users",
///     fields: {
///         avatar: image [upload_to: "avatars/"],
///         bio: textarea,
///         website: url [required],
///     }
/// }
/// ```
#[proc_macro]
pub fn extend(input: TokenStream) -> TokenStream {
    const FRAMEWORK_TABLES: &[&str] = &[
        "eihwaz_users",
        "eihwaz_groupes",
        "eihwaz_droits",
        "eihwaz_sessions",
        "eihwaz_users_groupes",
        "eihwaz_groupes_droits",
    ];

    let dsl = match syn::parse::<extend_schema::ExtendDsl>(input) {
        Ok(d) => d,
        Err(e) => return e.to_compile_error().into(),
    };

    if !FRAMEWORK_TABLES.contains(&dsl.table.as_str()) {
        let tables_list = FRAMEWORK_TABLES.join(", ");
        let msg = format!(
            "extend!{{}}: \"{}\" is not a known framework table. Allowed: {}",
            dsl.table, tables_list
        );
        return quote::quote! { compile_error!(#msg); }.into();
    }

    let schema = extend_schema::generate_schema_fn(&dsl);
    let entity = extend_schema::generate_entity(&dsl);
    quote::quote! {
        #schema
        #entity
    }
    .into()
}
