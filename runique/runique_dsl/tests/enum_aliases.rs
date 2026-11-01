use runique_dsl::ast::EnumDef;

#[test]
fn rejects_alias_collisions_between_display_labels_and_variant_names() {
    let result = syn::parse_str::<EnumDef>(r#"Status: [Admin: "Draft", Draft]"#);

    assert!(
        result.is_err(),
        "FromStr must not map the form value for Draft to the earlier Admin variant"
    );
}

#[test]
fn rejects_duplicate_database_values() {
    let result = syn::parse_str::<EnumDef>(
        r#"Status: [Admin=("draft", "Administrator"), Draft=("draft", "Draft")]"#,
    );

    assert!(
        result.is_err(),
        "stored values must unambiguously identify variants"
    );
}

#[test]
fn accepts_enum_with_unambiguous_aliases() {
    let result = syn::parse_str::<EnumDef>(r#"Status: [Draft: "Brouillon", Published]"#);

    assert!(result.is_ok());
}
