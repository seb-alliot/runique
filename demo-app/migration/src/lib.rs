pub use sea_orm_migration::prelude::*;
mod m20260405_185430_create_blog_table;
mod m20260405_185430_create_changelog_entry_table;
mod m20260405_185430_create_chapitre_table;
mod m20260405_185430_create_code_example_table;
mod m20260405_185430_create_contrainte_ia_table;
mod m20260405_185430_create_contributions_table;
mod m20260405_185430_create_cour_block_table;
mod m20260405_185430_create_cour_ia_table;
mod m20260405_185430_create_cour_table;
mod m20260405_185430_create_demo_category_table;
mod m20260405_185430_create_demo_page_table;
mod m20260405_185430_create_demo_section_table;
mod m20260405_185430_create_doc_block_table;
mod m20260405_185430_create_doc_page_table;
mod m20260405_185430_create_doc_section_table;
mod m20260405_185430_create_eihwaz_droits_table;
mod m20260405_185430_create_eihwaz_groupes_table;
mod m20260405_185430_create_eihwaz_sessions_table;
mod m20260405_185430_create_eihwaz_users_table;
mod m20260405_185430_create_form_field_table;
mod m20260405_185430_create_known_issue_table;
mod m20260405_185430_create_page_doc_link_table;
mod m20260405_185430_create_roadmap_entry_table;
mod m20260405_185430_create_runique_release_table;
mod m20260405_185430_create_site_config_table;
mod m20260405_185430_create_test_all_fields_table;
mod m20260405_185430_create_users_booster_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        let migrations: Vec<Box<dyn MigrationTrait>> = vec![
            Box::new(m20260405_185430_create_test_all_fields_table::Migration),
            Box::new(m20260405_185430_create_known_issue_table::Migration),
            Box::new(m20260405_185430_create_form_field_table::Migration),
            Box::new(m20260405_185430_create_demo_category_table::Migration),
            Box::new(m20260405_185430_create_site_config_table::Migration),
            Box::new(m20260405_185430_create_eihwaz_droits_table::Migration),
            Box::new(m20260405_185430_create_runique_release_table::Migration),
            Box::new(m20260405_185430_create_doc_section_table::Migration),
            Box::new(m20260405_185430_create_blog_table::Migration),
            Box::new(m20260405_185430_create_users_booster_table::Migration),
            Box::new(m20260405_185430_create_contrainte_ia_table::Migration),
            Box::new(m20260405_185430_create_roadmap_entry_table::Migration),
            Box::new(m20260405_185430_create_eihwaz_users_table::Migration),
            Box::new(m20260405_185430_create_changelog_entry_table::Migration),
            Box::new(m20260405_185430_create_eihwaz_groupes_table::Migration),
            Box::new(m20260405_185430_create_cour_ia_table::Migration),
            Box::new(m20260405_185430_create_cour_table::Migration),
            Box::new(m20260405_185430_create_demo_page_table::Migration),
            Box::new(m20260405_185430_create_doc_page_table::Migration),
            Box::new(m20260405_185430_create_contributions_table::Migration),
            Box::new(m20260405_185430_create_eihwaz_sessions_table::Migration),
            Box::new(m20260405_185430_create_chapitre_table::Migration),
            Box::new(m20260405_185430_create_demo_section_table::Migration),
            Box::new(m20260405_185430_create_code_example_table::Migration),
            Box::new(m20260405_185430_create_page_doc_link_table::Migration),
            Box::new(m20260405_185430_create_doc_block_table::Migration),
            Box::new(m20260405_185430_create_cour_block_table::Migration),
        ];
        migrations
    }
}
