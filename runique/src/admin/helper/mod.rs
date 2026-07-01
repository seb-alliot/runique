pub mod dyn_form;
pub mod fk_resolve;
pub mod resource_entry;
pub mod roles;
pub mod template;

pub use dyn_form::DynForm;
pub use fk_resolve::{fetch_fk_label_map, fk_key, resolve_fk_labels, resolve_fk_labels_in_rows};
pub use resource_entry::{
    CountFn, CreateFn, DeleteFn, EnumLabelFn, FilterFn, FormBuilder, GetFn, GroupAction, ListFn,
    ListParams, ResourceEntry, SortDir, UpdateFn,
};
pub use roles::{get_roles, register_roles};
pub(crate) use template::AdminTemplate;
