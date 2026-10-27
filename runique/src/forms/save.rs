//! What `save` / `delete` and the gate's `save_as` share: the row read from
//! the URL, and where a refused write's reasons go.
use crate::auth::gate::PkOf;
use crate::auth::gate::change::FieldErrors;
use crate::auth::meta::ModelMeta;
use crate::forms::form::Forms;
use crate::utils::trad::t;
use sea_orm::{DbErr, IdenStatic, Iterable, PrimaryKeyToColumn};
use std::str::FromStr;

/// What the handler asked: `save` (create or update, by the key) or `delete`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SaveAction {
    Save,
    Delete,
}

/// The row a form's write points at: the URL's `{<name>_<pk>}`, else `{<pk>}`;
/// none means a create. A key that doesn't read as the entity's key type is an
/// error, never a create.
pub(crate) fn url_key<E: ModelMeta>(form: &Forms) -> Result<Option<PkOf<E>>, ()> {
    let Some(pk) = E::PrimaryKey::iter().next() else {
        return Ok(None);
    };
    let pk = pk.into_column();
    let pk = pk.as_str();
    let text = form
        .name
        .and_then(|name| form.path_params.get(&format!("{name}_{pk}")))
        .or_else(|| form.path_params.get(pk));
    match text {
        None => Ok(None),
        Some(text) => PkOf::<E>::from_str(text).map(Some).map_err(|_| ()),
    }
}

/// Puts each refused column's reason on its field, for the form shown again;
/// on the form itself for a column the form has no field for.
pub(crate) fn on_fields(form: &mut Forms, errors: FieldErrors) -> DbErr {
    for (name, message) in errors {
        match form.fields.get_mut(&name) {
            Some(field) => field.set_error(message),
            None => form.errors.push(format!("{name}: {message}")),
        }
    }
    DbErr::Custom("save: fields refused".to_string())
}

pub(crate) fn not_found() -> DbErr {
    DbErr::Custom(t("error.not_found").into_owned())
}
