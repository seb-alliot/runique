//! The steps of the commands, each in its folder: `makemigrations` reads
//! (`parser`), compares (`diff`), checks and orders what changed (`changes`)
//! and writes (`generate`); `migration up` applies (`apply`).
pub mod apply;
pub mod changes;
pub mod diff;
pub mod generate;
pub mod parser;
