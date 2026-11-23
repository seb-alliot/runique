//! Keys of `atomic`.
pub mod error {
    pub const CHANGED: &str = "atomic.error.changed";
    pub const DUPLICATE: &str = "atomic.error.duplicate";
    pub const GONE: &str = "atomic.error.gone";
    pub const IN_THE_WAY: &str = "atomic.error.in_the_way";
    pub const JOURNAL_FORGED: &str = "atomic.error.journal_forged";
    pub const LEAVES_JOURNAL: &str = "atomic.error.leaves_journal";
    pub const MOVED_OUT: &str = "atomic.error.moved_out";
    pub const NO_JOURNAL: &str = "atomic.error.no_journal";
    pub const NOT_A_FILE: &str = "atomic.error.not_a_file";
    pub const NOT_CREATED: &str = "atomic.error.not_created";
    pub const OUTSIDE_JOURNAL: &str = "atomic.error.outside_journal";
    pub const RESERVED_NAME: &str = "atomic.error.reserved_name";
    pub const STRANGER: &str = "atomic.error.stranger";
    pub const STRAY_BACKUP: &str = "atomic.error.stray_backup";
    pub const UNFINISHED: &str = "atomic.error.unfinished";
    pub const WRONG_STEP: &str = "atomic.error.wrong_step";
}
