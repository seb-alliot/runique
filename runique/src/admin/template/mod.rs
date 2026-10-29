//! The admin's templates: the page each view renders (`page`) and the
//! blocks pages are made of (`block`), with the developer's override where
//! he declared one.
pub mod assemble;
pub mod block;
pub mod page;
pub mod stylesheet;
pub use block::{Block, BlockTemplate, GlobalBlock, ModelBlocks, Stylesheet};
pub use page::{AdminTemplate, ModelTemplates, PathAdminTemplate};
pub use stylesheet::{Page, PageSheets, ServedStylesheet};
