//! Reading validated form values into an entity's Rust types: the runtime half
//! of the `admin_from_form` / `admin_partial_update` code that `model!{}` and
//! `extend!{}` generate.
//!
//! A value that can't be read is an error the caller shows on the form — never
//! a default (`0`, `false`) saved in its place.
use crate::utils::aliases::StrMap;
use crate::utils::trad::tf;

/// Why a value from the form data can't be saved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormDataError {
    /// Present, but not a value of the column's type.
    Invalid(String),
    /// Missing or empty, for a column that can't be left empty.
    Required(String),
    /// The password couldn't be hashed — the submitted text is never saved instead.
    Password(String),
}

impl std::fmt::Display for FormDataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::Invalid(field) => tf("forms.data_invalid", &[field]),
            Self::Required(field) => tf("forms.data_required", &[field]),
            Self::Password(field) => tf("forms.data_password", &[field]),
        };
        f.write_str(&message)
    }
}

impl std::error::Error for FormDataError {}

/// So the generated code can use `?` inside create/update functions, which
/// return a `DbErr`: the admin shows a `Custom` error on the form.
impl From<FormDataError> for sea_orm::DbErr {
    fn from(e: FormDataError) -> Self {
        sea_orm::DbErr::Custom(e.to_string())
    }
}

/// The value `name` holds in `data`, read with `parse`: `Ok(None)` when it's
/// absent or empty, an error when it's there but `parse` can't read it.
pub fn read<T>(
    data: &StrMap,
    name: &str,
    parse: impl FnOnce(&str) -> Option<T>,
) -> Result<Option<T>, FormDataError> {
    match data.get(name).map(|v| v.trim()).filter(|v| !v.is_empty()) {
        None => Ok(None),
        Some(v) => parse(v)
            .map(Some)
            .ok_or_else(|| FormDataError::Invalid(name.to_string())),
    }
}

/// A value the column can't do without: missing or empty is an error.
pub fn required<T>(value: Option<T>, name: &str) -> Result<T, FormDataError> {
    value.ok_or_else(|| FormDataError::Required(name.to_string()))
}

/// What to store for a password: kept as is when it's already a hash (the
/// form's `finalize` hashed it), hashed otherwise. A failed hash is an error.
pub fn password(value: &str, name: &str) -> Result<String, FormDataError> {
    let service =
        crate::utils::password::PasswordService::new(crate::utils::password::password_get());
    if service.is_already_hashed(value) {
        return Ok(value.to_string());
    }
    crate::utils::password::hash(value).map_err(|_| FormDataError::Password(name.to_string()))
}
