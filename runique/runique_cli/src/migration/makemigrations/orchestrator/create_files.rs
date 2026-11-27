use super::*;

impl BuilderOrchestrator<'_> {
    /// Has one file created per change, each put in its stage. Nothing is
    /// written yet, and no file knows yet which migration it starts from:
    /// `build` says it, once the stages are in order.
    pub(in crate::migration::makemigrations) fn create_files(mut self) -> Self {
        let (migrations_path, timestamp, engine) =
            (self.migrations_path, self.timestamp.as_str(), &self.engine);
        let stages = &mut self.stages;
        let enum_changes = &self.enum_changes;
        let enums =
            move |change: EnumChange| enum_changes.iter().filter(move |c| c.change == change);

        // An enum that is new, or that takes another name, comes before the
        // tables that use it; one whose values change comes after them, once
        // every place has its new name.
        build_enum_plan(
            stages.files_of(Stage::CreateEnums),
            enums(EnumChange::Created),
            migrations_path,
            timestamp,
            engine,
        );
        build_enum_plan(
            stages.files_of(Stage::RenameEnums),
            enums(EnumChange::Renamed),
            migrations_path,
            timestamp,
            engine,
        );
        build_enum_plan(
            stages.files_of(Stage::EnumValues),
            enums(EnumChange::Altered),
            migrations_path,
            timestamp,
            engine,
        );
        let tables = stages.files_of(Stage::Tables);
        build_main_plan(
            tables,
            &self.main_changes,
            &self.schemas,
            migrations_path,
            timestamp,
        );
        build_extend_plan(tables, &self.extend_planned, migrations_path, timestamp);
        build_drop_plan(
            stages.files_of(Stage::DropTables),
            &self.dropped,
            migrations_path,
            timestamp,
        );
        self
    }
}
