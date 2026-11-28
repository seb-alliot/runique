//! The templates at startup: the admin's replaced blocks written into its
//! pages, Tera loaded, then the replaced pages checked against what they get.
#[cfg(feature = "admin")]
use crate::app::staging::AdminStaging;
use crate::{
    config::RuniqueConfig,
    errors::build::BuildError,
    templates::loader::{Integrity, TemplateLoader},
    utils::aliases::ARlockmap,
};
use tera::Tera;

/// Tera with every template of the project and of Runique, and the
/// integrity hashes of the static files.
pub(super) fn load_templates(
    config: &RuniqueConfig,
    url_registry: &ARlockmap,
) -> Result<(Tera, Integrity), BuildError> {
    TemplateLoader::init_with_integrity(config, url_registry.clone())
        .map_err(|e| BuildError::template(e.to_string()))
}

/// The admin's replaced blocks, written into its pages before Tera loads
/// them; a page replaced both ways refused first.
#[cfg(feature = "admin")]
pub(super) fn write_admin_blocks(
    admin: &AdminStaging,
    config: &mut RuniqueConfig,
) -> Result<(), BuildError> {
    use crate::admin::template::{Page, assemble};
    if !admin.enabled {
        return Ok(());
    }
    let models = admin.site.model_templates();
    let replaced = admin
        .declared_routes
        .iter()
        .filter(|route| route.overrides == crate::admin::AdminView::Detail)
        .map(|route| (route.model_key.clone(), Page::Detail));
    assemble::check_replaced_views(&models, replaced).map_err(BuildError::template)?;
    config.block_dispatch =
        assemble::dispatch(&admin.config.templates, &models).map_err(BuildError::template)?;
    Ok(())
}

/// A replaced admin page is only looked up when rendered: a missing one
/// refuses the boot instead of failing on its first request. And every
/// variable a replaced page or block reads must be one its page provides: a
/// typo is refused here, whatever the environment, instead of failing on render.
#[cfg(feature = "admin")]
pub(super) fn check_admin_pages(
    admin: &AdminStaging,
    config: &RuniqueConfig,
    url_registry: &ARlockmap,
    tera: &Tera,
) -> Result<(), BuildError> {
    use crate::admin::template::assemble;
    if !admin.enabled {
        return Ok(());
    }
    let loaded: std::collections::HashSet<&str> = tera.get_template_names().collect();
    assemble::missing_pages(
        &admin.config.templates,
        &admin.site.model_templates(),
        |path| loaded.contains(path),
    )
    .map_err(BuildError::template)?;

    let pristine = TemplateLoader::load_without_replacements(config, url_registry.clone())
        .map_err(|e| BuildError::template(e.to_string()))?;
    assemble::unknown_variables(
        &pristine,
        tera,
        &admin.config.templates,
        &admin.site.model_templates(),
    )
    .map_err(BuildError::template)
}
