//! Keys of `makemigrations`.
pub mod error {
    pub const APPLIED_UNKNOWN: &str = "makemigrations.error.applied_unknown";
    pub const DATABASE_UNREACHABLE: &str = "makemigrations.error.database_unreachable";
    pub const DESTRUCTIVE_DETECTED: &str = "makemigrations.error.destructive_detected";
    pub const DESTRUCTIVE_REQUIRE_FORCE: &str = "makemigrations.error.destructive_require_force";
    pub const DUPLICATE_TABLE: &str = "makemigrations.error.duplicate_table";
    pub const LONG_IDENTIFIER_DETECTED: &str = "makemigrations.error.long_identifier_detected";
    pub const LONG_IDENTIFIER_RENAME_REQUIRED: &str =
        "makemigrations.error.long_identifier_rename_required";
    pub const PARALLEL_CHANGES: &str = "makemigrations.error.parallel_changes";
    pub const PROJECT_PK_CARGO: &str = "makemigrations.error.project_pk_cargo";
    pub const PROJECT_PK_NO_RUNIQUE: &str = "makemigrations.error.project_pk_no_runique";
    pub const RENAME_WITH_NEW_KEY: &str = "makemigrations.error.rename_with_new_key";
    pub const RENAMED_TWICE: &str = "makemigrations.error.renamed_twice";
    pub const SNAPSHOT_MISNAMED: &str = "makemigrations.error.snapshot_misnamed";
    pub const SNAPSHOT_UNREADABLE: &str = "makemigrations.error.snapshot_unreadable";
    pub const TYPE_CHANGE_NOT_MIGRATED: &str = "makemigrations.error.type_change_not_migrated";
    pub const TYPE_CHANGE_WRITE_BY_HAND: &str = "makemigrations.error.type_change_write_by_hand";
}
pub mod hint {
    pub const APPLY: &str = "makemigrations.hint.apply";
}
pub mod info {
    pub const FILES_READY: &str = "makemigrations.info.files_ready";
    pub const NO_CHANGES: &str = "makemigrations.info.no_changes";
    pub const PAUSED_TABLES: &str = "makemigrations.info.paused_tables";
    pub const SNAPSHOTS_UPGRADED: &str = "makemigrations.info.snapshots_upgraded";
    pub const WAITING_REMOVED: &str = "makemigrations.info.waiting_removed";
    pub const WAITING_UNCHANGED: &str = "makemigrations.info.waiting_unchanged";
}
