//! Keys of `cli`.
pub mod error {
    pub const FOLDER_EXISTS: &str = "cli.error.folder_exists";
    pub const INTERRUPTED_RUN: &str = "cli.error.interrupted_run";
    pub const NAME_DASH: &str = "cli.error.name_dash";
    pub const NAME_DIGIT: &str = "cli.error.name_digit";
    pub const NAME_EMPTY: &str = "cli.error.name_empty";
    pub const NAME_INVALID: &str = "cli.error.name_invalid";
    pub const NAME_RESERVED: &str = "cli.error.name_reserved";
}
pub mod hint {
    pub const CARGO_RUN: &str = "cli.hint.cargo_run";
    pub const CD: &str = "cli.hint.cd";
    pub const MIGRATION_UP: &str = "cli.hint.migration_up";
}
pub mod info {
    pub const CREATING_PROJECT: &str = "cli.info.creating_project";
    pub const GETTING_STARTED: &str = "cli.info.getting_started";
    pub const PROJECT_CREATED: &str = "cli.info.project_created";
    pub const RUN_REPAIRED: &str = "cli.info.run_repaired";
}
