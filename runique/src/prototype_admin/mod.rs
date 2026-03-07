pub mod admin_main;
pub mod dyn_form;
pub mod registry;
pub mod resource_entry;

pub use admin_main::{admin_get, admin_get_id, admin_post, admin_post_id, PrototypeAdminState};
pub use dyn_form::DynForm;
pub use registry::AdminRegistry;
pub use resource_entry::{
    CountFn, CreateFn, DeleteFn, FormBuilder, GetFn, ListFn, ResourceEntry, UpdateFn,
};
