//! `new`: the command's entry point. Its logic (layout, bundled files,
//! `SECRET_KEY`, the `migration` crate) lives in `new_project`.
use anyhow::Result;

/// Scaffolds a new Runique project named `name`.
pub fn create_new_project(name: &str) -> Result<()> {
    crate::new_project::create_new_project(name)
}
