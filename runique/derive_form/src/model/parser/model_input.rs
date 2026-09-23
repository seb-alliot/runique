//! `model! { Name, table: "...", pk: id => i32, enums: {...}, { fields }, relations: {...}, meta: {...} }`
//! — the top-level DSL invocation. `Parse` is split into one function per
//! concern (header, `enums:` block, fields block, `relations:`/`meta:`
//! trailing blocks) so a parsing failure in one area points at a small,
//! independently readable/testable function instead of a single ~170-line
//! `parse`.
use crate::model::ast::{
    EnumDef, FieldDef, FormFieldDecl, MetaDef, ModelInput, PkDef, RelationDef,
};
use std::collections::HashSet;
use syn::{
    Ident, LitStr, Result, Token,
    parse::{Parse, ParseStream},
};

use super::form_field::form_field_to_field_def;
use super::sql_identifier::validate_sql_identifier;

/// `(fields, form_fields, seen_field_names)` — the SQL-level `FieldDef`s
/// derived from each declaration, the raw declarations themselves (needed
/// downstream to generate SeaORM form-field registration code), and the set
/// of names seen so far (fed into the `meta:` field-reference check).
type FieldsBlock = (Vec<FieldDef>, Vec<FormFieldDecl>, HashSet<String>);

/// `ModelName, table: "...", pk: name => type,` — the three always-present,
/// always-in-order header fields.
struct ModelHeader {
    name: Ident,
    table: LitStr,
    pk: PkDef,
}

fn parse_header(input: ParseStream) -> Result<ModelHeader> {
    // EihwazUsers,
    let name: Ident = input.parse()?;
    input.parse::<Token![,]>()?;

    // table: "eihwaz_users",
    let table_kw: Ident = input.parse()?;
    if table_kw != "table" {
        return Err(syn::Error::new(table_kw.span(), "Expected `table:`"));
    }
    input.parse::<Token![:]>()?;
    let table: LitStr = input.parse()?;
    validate_sql_identifier(&table)?;
    input.parse::<Token![,]>()?;

    // pk: id => i32,
    let pk_kw: Ident = input.parse()?;
    if pk_kw != "pk" {
        return Err(syn::Error::new(pk_kw.span(), "Expected `pk:`"));
    }
    input.parse::<Token![:]>()?;
    let pk = PkDef::parse(input)?;
    input.parse::<Token![,]>()?;

    Ok(ModelHeader { name, table, pk })
}

/// Optional `enums: { Name [Variant, ...], ... }` block, right after the header.
fn parse_optional_enums_block(input: ParseStream) -> Result<Vec<EnumDef>> {
    let mut enums = Vec::new();
    if !input.peek(Ident) {
        return Ok(enums);
    }
    let peek: Ident = input.fork().parse()?;
    if peek != "enums" {
        return Ok(enums);
    }
    input.parse::<Ident>()?;
    input.parse::<Token![:]>()?;
    let enum_content;
    syn::braced!(enum_content in input);
    let mut seen_enum_names: HashSet<String> = HashSet::new();
    while !enum_content.is_empty() {
        let def = EnumDef::parse(&enum_content)?;
        if !seen_enum_names.insert(def.name.to_string()) {
            return Err(syn::Error::new(
                def.name.span(),
                format!("Enum '{}' is declared more than once", def.name),
            ));
        }
        enums.push(def);
    }
    let _ = input.parse::<Token![,]>();
    Ok(enums)
}

/// The required anonymous `{ name: type [opts], ... }` fields block.
/// `pk_name` is pre-seeded into the returned name set so a field re-declaring
/// it (`pk: id => i32, { id: text }`) is caught by the caller instead of
/// silently shadowing the primary key downstream.
fn parse_fields_block(input: ParseStream, pk_name: &str) -> Result<FieldsBlock> {
    let mut fields = Vec::new();
    let mut form_fields = Vec::new();
    let mut seen_field_names: HashSet<String> = HashSet::from([pk_name.to_string()]);

    let ff_content;
    syn::braced!(ff_content in input);
    while !ff_content.is_empty() {
        let ff = FormFieldDecl::parse(&ff_content)?;
        if !seen_field_names.insert(ff.name.to_string()) {
            return Err(syn::Error::new(
                ff.name.span(),
                format!("Field '{}' is declared more than once", ff.name),
            ));
        }
        fields.push(form_field_to_field_def(&ff));
        form_fields.push(ff);
    }
    let _ = input.parse::<Token![,]>();

    Ok((fields, form_fields, seen_field_names))
}

/// Optional `relations: { belongs_to: ..., has_many: ..., ... }` block.
fn parse_optional_relations_block(input: ParseStream) -> Result<Vec<RelationDef>> {
    let mut relations = Vec::new();
    if !input.peek(Ident) {
        return Ok(relations);
    }
    let peek: Ident = input.fork().parse()?;
    if peek != "relations" {
        return Ok(relations);
    }
    input.parse::<Ident>()?;
    input.parse::<Token![:]>()?;
    let rel_content;
    syn::braced!(rel_content in input);
    let mut seen_relations: HashSet<(&'static str, String, String)> = HashSet::new();
    while !rel_content.is_empty() {
        let rel = RelationDef::parse(&rel_content)?;
        let (kind, model_ident, disambiguator) = match &rel {
            RelationDef::BelongsTo { model, via } => ("belongs_to", model, via.to_string()),
            RelationDef::HasMany { model, as_name } => (
                "has_many",
                model,
                as_name.as_ref().map(|a| a.to_string()).unwrap_or_default(),
            ),
            RelationDef::HasOne { model, as_name } => (
                "has_one",
                model,
                as_name.as_ref().map(|a| a.to_string()).unwrap_or_default(),
            ),
            RelationDef::ManyToMany { model, through, .. } => {
                ("many_to_many", model, through.to_string())
            }
        };
        let key = (kind, model_ident.to_string(), disambiguator);
        if !seen_relations.insert(key.clone()) {
            return Err(syn::Error::new(
                model_ident.span(),
                format!(
                    "Relation '{}' toward '{}' is declared more than once",
                    key.0, key.1
                ),
            ));
        }
        relations.push(rel);
    }
    let _ = input.parse::<Token![,]>();
    Ok(relations)
}

/// Optional `meta: { ordering:, unique_together:, ... }` block.
fn parse_optional_meta_block(input: ParseStream) -> Result<Option<MetaDef>> {
    if !input.peek(Ident) {
        return Ok(None);
    }
    let peek: Ident = input.fork().parse()?;
    if peek != "meta" {
        return Ok(None);
    }
    input.parse::<Ident>()?;
    input.parse::<Token![:]>()?;
    let meta_content;
    syn::braced!(meta_content in input);
    Ok(Some(MetaDef::parse(&meta_content)?))
}

/// `meta: { ordering:, unique_together:, indexes: }` reference field names as
/// free-standing idents with no link back to `fields` at parse time — a typo'd
/// or renamed column silently produced dead/broken generated code instead of
/// failing here, where the mistake actually is.
fn validate_meta_field_refs(
    meta: &Option<MetaDef>,
    seen_field_names: &HashSet<String>,
) -> Result<()> {
    let Some(m) = meta else {
        return Ok(());
    };
    let check = |ident: &Ident| -> Result<()> {
        if !seen_field_names.contains(&ident.to_string()) {
            return Err(syn::Error::new(
                ident.span(),
                format!("'{}' in `meta` is not a declared field", ident),
            ));
        }
        Ok(())
    };
    for (_, ident) in &m.ordering {
        check(ident)?;
    }
    for group in m.unique_together.iter().chain(m.indexes.iter()) {
        for ident in group {
            check(ident)?;
        }
    }
    Ok(())
}

impl Parse for ModelInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let header = parse_header(input)?;
        let enums = parse_optional_enums_block(input)?;
        let (fields, form_fields, seen_field_names) =
            parse_fields_block(input, &header.pk.name.to_string())?;
        let relations = parse_optional_relations_block(input)?;
        let meta = parse_optional_meta_block(input)?;
        validate_meta_field_refs(&meta, &seen_field_names)?;

        Ok(ModelInput {
            name: header.name,
            table: header.table.value(),
            pk: header.pk,
            enums,
            fields,
            relations,
            meta,
            form_fields,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parses a full `model! { ... }` body from a DSL string.
    fn parse_model(src: &str) -> syn::Result<ModelInput> {
        syn::parse_str::<ModelInput>(src)
    }

    fn model_ok(src: &str) {
        assert!(
            parse_model(src).is_ok(),
            "expected OK but error for: `{src}`"
        );
    }

    fn model_err(src: &str) {
        assert!(
            parse_model(src).is_err(),
            "expected ERROR but OK for: `{src}`"
        );
    }

    const BASE_FIELDS: &str = r#"{ name: text, }"#;

    #[test]
    fn table_name_plain_identifier_accepted() {
        model_ok(&format!(
            r#"Test, table: "tests", pk: id => i32, {BASE_FIELDS},"#
        ));
    }

    #[test]
    fn table_name_with_quote_rejected() {
        model_err(&format!(
            r#"Test, table: "tests\"); //", pk: id => i32, {BASE_FIELDS},"#
        ));
    }

    #[test]
    fn table_name_starting_with_digit_rejected() {
        model_err(&format!(
            r#"Test, table: "1tests", pk: id => i32, {BASE_FIELDS},"#
        ));
    }

    #[test]
    fn table_name_empty_rejected() {
        model_err(&format!(
            r#"Test, table: "", pk: id => i32, {BASE_FIELDS},"#
        ));
    }

    #[test]
    fn table_keyword_typo_gives_clean_error() {
        model_err(&format!(
            r#"Test, tabel: "tests", pk: id => i32, {BASE_FIELDS},"#
        ));
    }

    #[test]
    fn pk_keyword_typo_gives_clean_error() {
        model_err(r#"Test, table: "tests", primary: id => i32, { name: text, },"#);
    }

    #[test]
    fn duplicate_field_names_rejected() {
        model_err(r#"Test, table: "tests", pk: id => i32, { name: text, name: text, },"#);
    }

    #[test]
    fn field_named_like_pk_rejected() {
        model_err(r#"Test, table: "tests", pk: id => i32, { id: text, },"#);
    }

    #[test]
    fn duplicate_enum_names_rejected() {
        model_err(&format!(
            r#"Test, table: "tests", pk: id => i32, enums: {{ Status: [Active], Status: [Inactive], }}, {BASE_FIELDS},"#
        ));
    }

    #[test]
    fn empty_enum_rejected() {
        model_err(&format!(
            r#"Test, table: "tests", pk: id => i32, enums: {{ Status: [], }}, {BASE_FIELDS},"#
        ));
    }

    #[test]
    fn duplicate_enum_variant_rejected() {
        model_err(&format!(
            r#"Test, table: "tests", pk: id => i32, enums: {{ Status: [Active, Active], }}, {BASE_FIELDS},"#
        ));
    }

    #[test]
    fn has_many_with_as_alias_now_parses() {
        model_ok(&format!(
            r#"Test, table: "tests", pk: id => i32, {BASE_FIELDS}, relations: {{ has_many: comment as user_comments, }},"#
        ));
    }

    #[test]
    fn has_one_with_as_alias_now_parses() {
        model_ok(&format!(
            r#"Test, table: "tests", pk: id => i32, {BASE_FIELDS}, relations: {{ has_one: profile as user_profile, }},"#
        ));
    }

    #[test]
    fn has_many_without_alias_still_parses() {
        model_ok(&format!(
            r#"Test, table: "tests", pk: id => i32, {BASE_FIELDS}, relations: {{ has_many: comment, }},"#
        ));
    }

    #[test]
    fn duplicate_relation_rejected() {
        model_err(&format!(
            r#"Test, table: "tests", pk: id => i32, {BASE_FIELDS}, relations: {{ has_many: comment, has_many: comment, }},"#
        ));
    }

    #[test]
    fn distinct_belongs_to_same_model_different_via_accepted() {
        // two FKs toward the same target model, through different columns —
        // must NOT be flagged as a duplicate relation.
        model_ok(
            r#"Test, table: "tests", pk: id => i32, { created_by: int [fk(users.id, cascade)], updated_by: int [fk(users.id, cascade)], }, relations: { belongs_to: users via created_by, belongs_to: users via updated_by, },"#,
        );
    }

    #[test]
    fn meta_ordering_unknown_field_rejected() {
        model_err(&format!(
            r#"Test, table: "tests", pk: id => i32, {BASE_FIELDS}, meta: {{ ordering: [does_not_exist], }}"#
        ));
    }

    #[test]
    fn meta_ordering_known_field_accepted() {
        model_ok(&format!(
            r#"Test, table: "tests", pk: id => i32, {BASE_FIELDS}, meta: {{ ordering: [name], }}"#
        ));
    }

    #[test]
    fn meta_ordering_on_pk_accepted() {
        model_ok(&format!(
            r#"Test, table: "tests", pk: id => i32, {BASE_FIELDS}, meta: {{ ordering: [id], }}"#
        ));
    }

    #[test]
    fn meta_unique_together_unknown_field_rejected() {
        model_err(
            r#"Test, table: "tests", pk: id => i32, { name: text, }, meta: { unique_together: [(name, ghost)], }"#,
        );
    }
}
