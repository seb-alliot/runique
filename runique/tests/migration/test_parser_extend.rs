//! Tests — migration/utils/parser_extend
//! Couvre : parse_extend_blocks_from_source

use runique::migration::parse_extend_blocks_from_source;

#[test]
fn test_empty_source_returns_empty() {
    let result = parse_extend_blocks_from_source("");
    assert!(result.is_empty());
}

#[test]
fn test_invalid_syntax_returns_empty() {
    let result = parse_extend_blocks_from_source("fn foo( {{{");
    assert!(result.is_empty());
}

#[test]
fn test_no_extend_block_returns_empty() {
    let source = r#"
        fn main() {
            println!("hello");
        }
    "#;
    let result = parse_extend_blocks_from_source(source);
    assert!(result.is_empty());
}

#[test]
fn test_single_extend_block() {
    let source = r#"
        extend! {
            table: "eihwaz_users",
            fields: {
                bio: textarea,
            }
        }
    "#;
    let result = parse_extend_blocks_from_source(source);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].table_name, "eihwaz_users");
    assert_eq!(result[0].columns.len(), 1);
    assert_eq!(result[0].columns[0].name, "bio");
}

#[test]
fn test_primary_key_is_none() {
    let source = r#"
        extend! {
            table: "my_table",
            fields: {
                title: text,
            }
        }
    "#;
    let result = parse_extend_blocks_from_source(source);
    assert_eq!(result.len(), 1);
    assert!(result[0].primary_key.is_none());
}

#[test]
fn test_multiple_fields() {
    let source = r#"
        extend! {
            table: "profiles",
            fields: {
                avatar: image,
                website: url,
                age: integer,
            }
        }
    "#;
    let result = parse_extend_blocks_from_source(source);
    assert_eq!(result.len(), 1);
    let cols = &result[0].columns;
    assert_eq!(cols.len(), 3);
    assert_eq!(cols[0].name, "avatar");
    assert_eq!(cols[1].name, "website");
    assert_eq!(cols[2].name, "age");
}

#[test]
fn test_required_field_not_nullable() {
    let source = r#"
        extend! {
            table: "profiles",
            fields: {
                username: text [required],
            }
        }
    "#;
    let result = parse_extend_blocks_from_source(source);
    assert_eq!(result.len(), 1);
    assert!(!result[0].columns[0].nullable);
}

#[test]
fn test_optional_field_is_nullable() {
    let source = r#"
        extend! {
            table: "profiles",
            fields: {
                bio: textarea,
            }
        }
    "#;
    let result = parse_extend_blocks_from_source(source);
    assert_eq!(result.len(), 1);
    assert!(result[0].columns[0].nullable);
}

#[test]
fn test_unique_flag() {
    let source = r#"
        extend! {
            table: "profiles",
            fields: {
                handle: text [unique],
            }
        }
    "#;
    let result = parse_extend_blocks_from_source(source);
    assert!(result[0].columns[0].unique);
}

#[test]
fn test_multiple_extend_blocks() {
    let source = r#"
        extend! {
            table: "eihwaz_users",
            fields: {
                bio: textarea,
            }
        }
        extend! {
            table: "posts",
            fields: {
                slug: text,
            }
        }
    "#;
    let result = parse_extend_blocks_from_source(source);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].table_name, "eihwaz_users");
    assert_eq!(result[1].table_name, "posts");
}

#[test]
fn test_col_type_text_maps_to_string() {
    let source = r#"
        extend! {
            table: "t",
            fields: {
                name: text,
            }
        }
    "#;
    let result = parse_extend_blocks_from_source(source);
    assert_eq!(result[0].columns[0].col_type, "String");
}

#[test]
fn test_col_type_integer_maps_to_integer() {
    let source = r#"
        extend! {
            table: "t",
            fields: {
                count: integer,
            }
        }
    "#;
    let result = parse_extend_blocks_from_source(source);
    assert_eq!(result[0].columns[0].col_type, "Integer");
}

#[test]
fn test_col_type_bool_maps_to_boolean() {
    let source = r#"
        extend! {
            table: "t",
            fields: {
                active: bool,
            }
        }
    "#;
    let result = parse_extend_blocks_from_source(source);
    assert_eq!(result[0].columns[0].col_type, "Boolean");
}
