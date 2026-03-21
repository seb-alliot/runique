pub use sea_orm_migration::prelude::*;
mod m20260321_193132_create_doc_page_table;
mod m20260321_193132_create_doc_block_table;
mod m20260321_193132_create_doc_section_table;
mod m20260321_193132_create_site_config_table;
mod m20260318_153933_create_blog_table;
mod m20260318_153933_create_contributions_table;
mod m20260318_153933_create_eihwaz_users_table;
mod m20260318_153933_create_test_all_fields_table;
mod m20260318_153933_create_users_booster_table;
mod m20260320_122444_create_changelog_entry_table;
mod m20260320_122444_create_known_issue_table;
mod m20260320_122444_create_roadmap_entry_table;
mod m20260320_130926_alter_roadmap_entry_table;
mod m20260320_143527_create_code_example_table;
mod m20260320_143527_create_demo_category_table;
mod m20260320_143527_create_demo_page_table;
mod m20260320_143527_create_demo_section_table;
mod m20260320_143527_create_form_field_table;
mod m20260320_143527_create_page_doc_link_table;
mod m20260320_151115_alter_demo_category_table;
mod m20260320_163000_alter_form_field_table;
mod m20260321_000000_create_doc_section_table;
mod m20260321_000001_create_doc_page_table;
mod m20260321_000002_create_doc_block_table;
mod m20260321_000003_create_site_config_table;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        // let mut migrations: Vec<Box<dyn MigrationTrait>> = user_runique()
        //     .into_iter()
        //     .collect();
        vec![
            Box::new(m20260318_153933_create_blog_table::Migration),
            Box::new(m20260318_153933_create_contributions_table::Migration),
            Box::new(m20260318_153933_create_users_booster_table::Migration),
            Box::new(m20260318_153933_create_test_all_fields_table::Migration),
            Box::new(m20260318_153933_create_eihwaz_users_table::Migration),
            Box::new(m20260320_122444_create_known_issue_table::Migration),
            Box::new(m20260320_122444_create_roadmap_entry_table::Migration),
            Box::new(m20260320_122444_create_changelog_entry_table::Migration),
            Box::new(m20260320_130926_alter_roadmap_entry_table::Migration),
            Box::new(m20260320_143527_create_demo_page_table::Migration),
            Box::new(m20260320_143527_create_demo_section_table::Migration),
            Box::new(m20260320_143527_create_demo_category_table::Migration),
            Box::new(m20260320_143527_create_code_example_table::Migration),
            Box::new(m20260320_143527_create_form_field_table::Migration),
            Box::new(m20260320_143527_create_page_doc_link_table::Migration),
            Box::new(m20260320_151115_alter_demo_category_table::Migration),
            Box::new(m20260320_163000_alter_form_field_table::Migration),
            Box::new(m20260321_000000_create_doc_section_table::Migration),
            Box::new(m20260321_000001_create_doc_page_table::Migration),
            Box::new(m20260321_000002_create_doc_block_table::Migration),
            Box::new(m20260321_000003_create_site_config_table::Migration),
            Box::new(m20260321_193132_create_site_config_table::Migration),
            Box::new(m20260321_193132_create_doc_section_table::Migration),
            Box::new(m20260321_193132_create_doc_block_table::Migration),
            Box::new(m20260321_193132_create_doc_page_table::Migration),
        ]
    }
}
