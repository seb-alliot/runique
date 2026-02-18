pub mod admin;
pub mod bdd;
pub mod context;
pub mod forms;
pub mod migration;
pub mod routeur;
pub mod template;

pub use routeur::register_url::{
    add_urls, register_name_url, register_pending, reverse, reverse_with_parameters,
};
