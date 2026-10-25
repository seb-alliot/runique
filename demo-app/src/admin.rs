//! Admin declared with the typed `ModelAdmin` builder, served under
//! `{admin}/{resource}/…`. Every `model!{}` of the project is registered:
//! the rights of a model left out would be pruned at boot.
//!
//! One function per model that declares its columns: its `use …::Column::*`
//! then names them alone, and two models' columns of the same name
//! (`Email`, `Title`…) never meet.
use crate::entities::{
    blog, changelog_entry, chapitre, code_example, contrainte_ia, contribution, demo_category,
    demo_page, demo_section, doc_block, doc_page, doc_section, form_field, known_issue,
    page_doc_link, roadmap_entry, runique_release, site_config, user_profile,
};
use runique::admin::{AdminSite, ModelAdmin};
use runique::app::staging::AdminStaging;
use runique::auth::LoginGuard;
use runique::middleware::rate_limit::RateLimiter;

/// The whole admin, handed to `with_admin` by `main`: the boot test below
/// checks this very configuration, not a copy.
pub fn config(admin: AdminStaging) -> AdminStaging {
    admin
        .site_title("Administration")
        .sitemap("https://runique.io/sitemap.xml")
        .path("/admin-runique/")
        .prefix("/prefix-test")
        .templates(|t| t.with_dashboard("admin/test_dashboard.html"))
        .with_rate_limiter(RateLimiter::new().max_requests(20).retry_after(3600))
        .with_login_guard(LoginGuard::new().max_attempts(20).lockout_secs(300))
        .page_size(15)
        .site(site)
}

/// The accounts, with the profile columns `user_profile` adds to them.
fn accounts() -> ModelAdmin<user_profile::Entity> {
    use user_profile::Column::*;
    ModelAdmin::new()
        .list(|l| {
            l.display([Username, Email, IsActive, Website])
                .sortable([Username])
                .search([Username, Email])
        })
        .form(|f| f.fields([Username, Email, Bio, Website, Phone, BirthDate]))
}

fn blog() -> ModelAdmin<blog::Entity> {
    use blog::Column::*;
    ModelAdmin::new().list(|l| {
        l.display([Title, Email, Status])
            .sortable([Title])
            .search([Title, Summary])
            .filter([Status])
    })
}

pub fn site(site: AdminSite) -> AdminSite {
    site.accounts_users(accounts())
        .register(blog())
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

    /// The admin's configuration and every registered model pass the checks
    /// the app runs at boot.
    #[test]
    fn every_registered_model_passes_the_boot_checks() {
        let staging = super::config(AdminStaging::new().enable());
        if let Err(e) = staging.validate() {
            panic!("{e}");
        }
    }
}
