//! Form field types — text, number, boolean, choice, datetime, file, binary, hidden, special.
pub mod binary;
pub mod boolean;
pub mod choice;
pub mod datetime;
pub mod file;
pub mod hidden;
pub mod number;
pub mod special;
pub mod text;

pub use binary::*;
pub use boolean::*;
pub use choice::*;
pub use datetime::*;
pub use file::*;
pub use hidden::*;
pub use number::*;
pub use special::*;
pub use text::*;
