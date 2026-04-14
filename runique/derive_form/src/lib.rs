use proc_macro::TokenStream;

mod model;
mod schema_form;

/// #[form(...)] macro
#[proc_macro_attribute]
pub fn form(attr: TokenStream, item: TokenStream) -> TokenStream {
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

    // Translation JSONs embedded at derive_form compilation time
    const TRANSLATIONS: &[(&str, &str)] = &[(
        "en",
        r#"{"makemigrations": {"extend_invalid_syntax": "extend: incomplete feature", "extend_unknown_table": "extend: incomplete feature"}}"#,
    )];

    /// Extracts a nested key `"section.key"` from an embedded JSON.
    fn get_msg(lang_code: &str, key: &str) -> Option<String> {
        let json_str = TRANSLATIONS.iter().find(|(l, _)| *l == lang_code)?.1;
        let val: serde_json::Value = serde_json::from_str(json_str).ok()?;
        let mut parts = key.split('.');
        let section = parts.next()?;
        let subkey = parts.next()?;
        val.get(section)?
            .get(subkey)?
            .as_str()
            .map(|s| s.to_string())
    }

    /// Detects the language from LANG / LC_ALL / LC_MESSAGES in the environment (after dotenvy).
    fn detect_lang() -> String {
        dotenvy::dotenv().ok();
        let raw = std::env::var("LANG")
            .or_else(|_| std::env::var("LC_ALL"))
            .or_else(|_| std::env::var("LC_MESSAGES"))
            .unwrap_or_default();
        raw.chars().take(2).collect::<String>().to_lowercase()
    }

    // Extracts the table name from `table: "..."` to validate at compile time.
    let input2 = proc_macro2::TokenStream::from(input);
    let mut tokens = input2.into_iter();

    let table_name: Option<String> = (|| {
        loop {
            let tok = tokens.next()?;
            if let proc_macro2::TokenTree::Ident(ident) = &tok
                && ident == "table"
            {
                tokens.next()?; // consume ':'
                if let proc_macro2::TokenTree::Literal(lit) = tokens.next()? {
                    return Some(lit.to_string().trim_matches('"').to_string());
                }
            }
        }
    })();

    let lang = detect_lang();

    match table_name {
        None => {
            let msg = get_msg(&lang, "makemigrations.extend_invalid_syntax")
                .or_else(|| get_msg("en", "makemigrations.extend_invalid_syntax"))
                .unwrap_or_else(|| "extend!{} : invalid syntax.".to_string());
            return quote::quote! { compile_error!(#msg); }.into();
        }
        Some(name) if !FRAMEWORK_TABLES.contains(&name.as_str()) => {
            let tables_list = FRAMEWORK_TABLES.join(", ");
            let template = get_msg(&lang, "makemigrations.extend_unknown_table")
                .or_else(|| get_msg("en", "makemigrations.extend_unknown_table"))
                .unwrap_or_else(|| {
                    "extend!{{}} : \"{}\" is not a known framework table. Allowed tables: {}."
                        .to_string()
                });
            // Replaces {} with values (simple format: first {} = table, second {} = list)
            let msg = template
                .replacen("{}", &name, 1)
                .replacen("{}", &tables_list, 1);
            return quote::quote! { compile_error!(#msg); }.into();
        }
        _ => {}
    }

    TokenStream::new()
}
