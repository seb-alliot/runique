//! A documentation page and its counterpart in the other language, for the
//! `hreflang` links that tell search engines the two pages are one.
//!
//! RULE for future documentation: a page keeps the same folder path in
//! `docs/fr/` and `docs/en/`. A folder (or file) whose name is translated
//! must be declared in `TRANSLATED_SLUGS`, or the
//! `every_doc_page_has_its_counterpart` test fails.

/// Page slugs (`{section}-{folder}-…`, see `seeds::doc::collect_section_pages`)
/// that differ between the French and the English documentation, as
/// `(french, english)`. Matched on a whole leading part of the slug: the pair
/// `formulaire-champs` covers `formulaire-champs` and `formulaire-champs-…`.
/// Every other page has the same slug in both languages.
pub const TRANSLATED_SLUGS: &[(&str, &str)] = &[
    ("admin-liste", "admin-list"),
    (
        "admin-template-surcharge-exemples",
        "admin-template-surcharge-examples",
    ),
    ("auth-exemple", "auth-example"),
    ("auth-modele", "auth-model"),
    ("env-securite", "env-security"),
    ("exemple-autres", "exemple-others"),
    ("exemple-formulaires", "exemple-forms"),
    ("formulaire-champs", "formulaire-fields"),
    ("formulaire-erreurs", "formulaire-errors"),
    ("formulaire-exemple", "formulaire-example"),
    ("installation-base-de-donnees", "installation-database"),
    ("installation-prerequis", "installation-prerequisites"),
    ("middleware-csp-profils", "middleware-csp-profiles"),
    ("model-formulaires", "model-forms"),
    ("orm-avance", "orm-advanced"),
    ("orm-requetes", "orm-queries"),
    ("routing-extracteurs", "routing-extractors"),
    ("routing-reponses", "routing-responses"),
    ("template-filtres", "template-filters"),
    ("template-formulaires", "template-forms"),
    ("template-syntaxe", "template-syntax"),
];

/// The other documentation language.
pub fn other_lang(lang: &str) -> &'static str {
    if lang == "fr" { "en" } else { "fr" }
}

/// The slug of the page `slug` of `lang` in the other language: the longest
/// declared pair that starts it is swapped, otherwise the slug is the same.
pub fn counterpart_slug(lang: &str, slug: &str) -> String {
    let from_french = lang == "fr";
    TRANSLATED_SLUGS
        .iter()
        .map(|&(fr, en)| if from_french { (fr, en) } else { (en, fr) })
        .filter(|(from, _)| {
            slug == *from
                || slug
                    .strip_prefix(from)
                    .is_some_and(|rest| rest.starts_with('-'))
        })
        .max_by_key(|(from, _)| from.len())
        .map_or_else(
            || slug.to_string(),
            |(from, to)| format!("{to}{}", &slug[from.len()..]),
        )
}

/// The URL path a page slug is served at: `{section}-index` is the section's
/// own page, `{section}-{page}` lives under it (same rule as the routing).
pub fn doc_page_path(lang: &str, section: &str, slug: &str) -> String {
    match slug.strip_prefix(&format!("{section}-")) {
        Some("index") | None => format!("/docs/{lang}/{section}"),
        Some(page) => format!("/docs/{lang}/{section}/{page}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_translated_folder_is_swapped_both_ways_and_keeps_its_suffix() {
        assert_eq!(
            counterpart_slug("fr", "formulaire-champs"),
            "formulaire-fields"
        );
        assert_eq!(
            counterpart_slug("en", "formulaire-fields"),
            "formulaire-champs"
        );
        assert_eq!(
            counterpart_slug("fr", "middleware-csp-profils"),
            "middleware-csp-profiles"
        );
    }

    #[test]
    fn a_shared_slug_or_a_mere_prefix_is_left_alone() {
        assert_eq!(
            counterpart_slug("fr", "admin-template-surcharge"),
            "admin-template-surcharge"
        );
        // `formulaire-champs` must not swap the start of `formulaire-champsx`.
        assert_eq!(
            counterpart_slug("fr", "formulaire-champsx"),
            "formulaire-champsx"
        );
    }

    #[test]
    fn the_longest_pair_wins() {
        assert_eq!(
            counterpart_slug("fr", "admin-template-surcharge-exemples"),
            "admin-template-surcharge-examples"
        );
    }

    #[test]
    fn index_and_pages_get_their_routed_path() {
        assert_eq!(doc_page_path("en", "orm", "orm-index"), "/docs/en/orm");
        assert_eq!(
            doc_page_path("en", "orm", "orm-queries"),
            "/docs/en/orm/queries"
        );
    }
}
