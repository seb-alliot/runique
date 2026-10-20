//! Admin declared with the typed `ModelAdmin` builder, served under
//! `{admin}/next/{resource}/…`. Every `model!{}` of the project is registered:
//! the rights of a model left out would be pruned at boot.
use crate::entities::{
    blog, changelog_entry, chapitre, code_example, contrainte_ia, contribution, demo_category,
    demo_page, demo_section, doc_block, doc_page, doc_section, form_field, known_issue,
    page_doc_link, roadmap_entry, runique_release, site_config, user_profile,
};
use runique::admin::refonte_admin::builder::{admin_builder::ModelAdmin, site::AdminSite};

pub fn site(site: AdminSite) -> AdminSite {
    // The accounts, with the profile columns `user_profile` adds to them.
    site.accounts_users(
        ModelAdmin::<user_profile::Entity>::new()
            .list(|l| {
                l.display([
                    user_profile::Column::Username,
                    user_profile::Column::Email,
                    user_profile::Column::IsActive,
                    user_profile::Column::Website,
                ])
                .sortable([user_profile::Column::Username])
                .search([user_profile::Column::Username, user_profile::Column::Email])
            })
            .form(|f| {
                f.fields([
                    user_profile::Column::Username,
                    user_profile::Column::Email,
                    user_profile::Column::Bio,
                    user_profile::Column::Website,
                    user_profile::Column::Phone,
                    user_profile::Column::BirthDate,
                ])
            }),
    )
    .register(ModelAdmin::<blog::Entity>::new().list(|l| {
        l.display([
            blog::Column::Title,
            blog::Column::Email,
            blog::Column::Status,
        ])
        .sortable([blog::Column::Title])
        .search([blog::Column::Title, blog::Column::Summary])
        .filter([blog::Column::Status])
    }))
    .register(ModelAdmin::<changelog_entry::Entity>::new())
    .register(ModelAdmin::<chapitre::Entity>::new())
    .register(ModelAdmin::<code_example::Entity>::new())
    .register(ModelAdmin::<contrainte_ia::Entity>::new())
    .register(ModelAdmin::<contribution::Entity>::new())
    .register(ModelAdmin::<demo_category::Entity>::new())
    .register(ModelAdmin::<demo_page::Entity>::new())
    .register(ModelAdmin::<demo_section::Entity>::new())
    .register(ModelAdmin::<doc_block::Entity>::new())
    .register(ModelAdmin::<doc_page::Entity>::new())
    .register(ModelAdmin::<doc_section::Entity>::new())
    .register(ModelAdmin::<form_field::Entity>::new())
    .register(ModelAdmin::<known_issue::Entity>::new())
    .register(ModelAdmin::<page_doc_link::Entity>::new())
    .register(ModelAdmin::<roadmap_entry::Entity>::new())
    .register(ModelAdmin::<runique_release::Entity>::new())
    .register(ModelAdmin::<site_config::Entity>::new())
}

#[cfg(test)]
mod tests {
    use runique::app::staging::AdminStaging;

    /// Every registered model passes the checks the app runs at boot.
    #[test]
    fn every_registered_model_passes_the_boot_checks() {
        let staging = AdminStaging::new()
            .enable()
            .path("/admin-runique")
            .site(super::site);
        if let Err(e) = staging.validate() {
            panic!("{e}");
        }
    }
}
