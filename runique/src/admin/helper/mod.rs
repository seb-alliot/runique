pub mod dyn_form;
pub mod fk_resolve;
pub mod m2m;
pub mod resource_entry;
pub mod template;

pub use dyn_form::DynForm;
pub use fk_resolve::{
    fetch_fk_label_map, fetch_fk_matching_ids, fk_key, resolve_fk_labels, resolve_fk_labels_in_rows,
};
pub use resource_entry::{
    CountFn, CreateFn, DeleteFn, EnumLabelFn, FilterFn, FormBuilder, GetFn, GroupAction, ListFn,
    ListParams, ResourceEntry, SortDir, UpdateFn,
};
// Moved to `db`: the search helpers aren't the admin's (`search_cond!` uses
// them). Re-exported here for the current admin and its generated code.
pub use crate::db::sql_dialect;
pub use crate::db::sql_dialect::{ilike, text_cast_type, text_cast_type_of, text_eq, text_eq_ci};
pub(crate) use template::AdminTemplate;
