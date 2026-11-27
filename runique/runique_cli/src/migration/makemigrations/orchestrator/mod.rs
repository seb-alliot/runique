//! The orchestrator of `makemigrations`: the steps of a run, one method
//! each, called one after the other by `run`. It reads what the project
//! declares and what it has migrated, compares the two, checks what changed,
//! puts the changes in order, has the files created (`BuilderPlan`, one per
//! file) and builds the final plan, written in one batch.
mod build;
mod check;
mod compare;
mod create_files;
mod nothing_to_migrate;
mod order;
mod read;

use super::*;

pub(crate) struct BuilderOrchestrator<'a> {
    entities_path: &'a str,
    migrations_path: &'a str,
    engine: DatabaseEngine,
    force: bool,

    // ── what `read` finds ──
    schemas: Vec<ParsedSchema>,
    paused: Vec<String>,
    /// (former table, table), as the models declare them.
    declared_renames: Vec<(String, String)>,
    extend_blocks: Vec<ParsedSchema>,
    /// The snapshots of the models' tables, then of the extended ones.
    migrated: Vec<ParsedSchema>,
    extended: Vec<ParsedSchema>,
    enum_snapshots: Vec<ParsedEnum>,
    /// The migrations no `migration up` applied yet: module and content.
    waiting: Vec<(String, String)>,

    // ── what `compare` finds ──
    main_changes: Vec<Changes>,
    extend_planned: Vec<(ParsedSchema, Changes)>,
    enum_changes: Vec<EnumChanges>,
    dropped: Vec<ParsedSchema>,
    /// Snapshots written in a former format: they go in the same batch.
    upgrades: Vec<(String, String)>,

    /// Every change of the run as `check` saw it: tables, extensions, drops.
    checked: Vec<Changes>,
    timestamp: String,
    stages: Stages,
}

impl<'a> BuilderOrchestrator<'a> {
    /// `engine` is the one the project's database runs on.
    pub(crate) fn new(
        entities_path: &'a str,
        migrations_path: &'a str,
        engine: DatabaseEngine,
    ) -> Self {
        Self {
            entities_path,
            migrations_path,
            engine,
            force: false,
            schemas: Vec::new(),
            paused: Vec::new(),
            declared_renames: Vec::new(),
            extend_blocks: Vec::new(),
            migrated: Vec::new(),
            extended: Vec::new(),
            enum_snapshots: Vec::new(),
            waiting: Vec::new(),
            main_changes: Vec::new(),
            extend_planned: Vec::new(),
            enum_changes: Vec::new(),
            dropped: Vec::new(),
            upgrades: Vec::new(),
            checked: Vec::new(),
            timestamp: String::new(),
            stages: Stages::default(),
        }
    }

    /// Lets the destructive changes through.
    pub(crate) fn force(mut self, force: bool) -> Self {
        self.force = force;
        self
    }
}
