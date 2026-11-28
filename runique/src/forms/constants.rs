//! The constants of the forms: the regex that read a database constraint
//! error, to name the field it is about, and the name of the form field.
use fancy_regex::Regex;
use std::sync::LazyLock;

pub static CONSTRAINT_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?:constraint|contrainte\s+unique)\s+(?:["«])?([a-zA-Z0-9_]+)(?:["»])?"#).unwrap()
});

pub static KEY_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"Key\s+\(([^)]+)\)").unwrap());

pub static FAILED_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"failed:\s+[a-zA-Z0-9_]+\.([a-zA-Z0-9_]+)").unwrap());

pub static FOR_KEY_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"for\s+key\s+'[^.]+\.([^']+)'").unwrap());

/// Name of the hidden form field carrying a form's name back on submit, so
/// that `ValidationForm::try_form` only binds the form that was sent.
pub const FORM_NAME_KEY: &str = "runique_form";
