//! Maps a parsed `FormFieldAttr` back to its DSL keyword, for error messages.
use crate::model::ast::FormFieldAttr;

pub(super) fn attr_name_str(attr: &FormFieldAttr) -> &'static str {
    match attr {
        FormFieldAttr::Required => "required",
        FormFieldAttr::Nullable => "nullable",
        FormFieldAttr::NoHash => "no_hash",
        FormFieldAttr::EnumRef(_) => "enum",
        FormFieldAttr::MaxLength(_) => "max_length",
        FormFieldAttr::MinLength(_) => "min_length",
        FormFieldAttr::Min(_) => "min",
        FormFieldAttr::Max(_) => "max",
        FormFieldAttr::MinF(_) => "min",
        FormFieldAttr::MaxF(_) => "max",
        FormFieldAttr::Default(_) => "default",
        FormFieldAttr::UploadTo(_) => "upload_to",
        FormFieldAttr::MaxSize(_) => "max_size",
        FormFieldAttr::Rows(_) => "rows",
        FormFieldAttr::Step(_) => "step",
        FormFieldAttr::AutoNow => "auto_now",
        FormFieldAttr::AutoNowUpdate => "auto_now_update",
        FormFieldAttr::Unique => "unique",
        FormFieldAttr::Readonly => "readonly",
        FormFieldAttr::Label(_) => "label",
        FormFieldAttr::Fk(_) => "fk",
        FormFieldAttr::Skip => "skip",
    }
}
