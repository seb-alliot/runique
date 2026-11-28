//! Reading source code into `ParsedSchema`: the `model!{}` DSL, `extend!{}`
//! blocks, and existing SeaORM migration files.
pub mod enums;
pub mod extend;
pub mod files;
pub mod model;
pub mod seaorm;
pub mod syn_extract;

pub use enums::*;
pub use extend::*;
pub use files::{scan::*, snapshot::parse_create_file};
pub use model::*;
pub use seaorm::*;
pub use syn_extract::*;
