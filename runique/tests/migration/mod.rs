//! Tests — Système de migration (parsing, diff, génération, paths).
//!
//! | Fichier                 | Ce qui est testé                             |
//! | ----------------------- | -------------------------------------------- |
//! | `test_parser`           | Parser DSL (model!)                          |
//! | `test_parser_builder`   | parse_schema_from_source                     |
//! | `test_parser_seaorm`    | Parsing depuis source SeaORM                 |
//! | `test_diff`             | diff_schemas, db_columns, Changes            |
//! | `test_convertisseur`    | Conversion de noms (snake_case, PascalCase)  |
//! | `test_helpers`          | col_type_to_method et utilitaires internes   |
//! | `test_paths`            | snapshot_dir, migration_dir, chemins         |
//! | `test_relation_kind`    | Enum RelationKind                            |
//! | `test_types`            | ParsedSchema et types de colonnes            |

pub mod test_convertisseur;
pub mod test_diff;
pub mod test_helpers;
pub mod test_parser;
pub mod test_parser_builder;
pub mod test_parser_seaorm;
pub mod test_paths;
pub mod test_relation_kind;
pub mod test_types;
