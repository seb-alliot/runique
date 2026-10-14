//! Admin declared with the typed `ModelAdmin` builder (rebuild in progress),
//! served under `{admin}/next/{resource}/list` beside the generated admin.
//! Kept out of `src/admins/`, which `runique start` regenerates.
use crate::entities::blog;
use runique::admin::refonte_admin::builder::{admin_builder::ModelAdmin, site::AdminSite};

pub fn site(site: AdminSite) -> AdminSite {
    site.register(ModelAdmin::<blog::Entity>::new().list(|l| {
        l.display([
            blog::Column::Title,
            blog::Column::Email,
            blog::Column::Status,
        ])
        .sortable([blog::Column::Title])
        .search([blog::Column::Title, blog::Column::Summary])
        .filter([blog::Column::Status])
    }))
}
