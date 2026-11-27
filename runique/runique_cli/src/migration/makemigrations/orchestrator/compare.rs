use super::*;

impl BuilderOrchestrator<'_> {
    /// Compares what is declared with what is migrated: the tables, the
    /// extensions, the enums, and the tables no model produces any more.
    /// Nothing is written.
    pub(in crate::migration::makemigrations) fn compare(mut self) -> Result<Self> {
        let renames = effective_renames(
            &self.declared_renames,
            &self.schemas,
            &self.paused,
            self.migrations_path,
        )?;
        self.main_changes = compute_main_changes(&self.schemas, &renames, self.migrations_path)?;
        self.extend_planned = plan_extend_changes(self.entities_path, self.migrations_path)?;

        // The enums, each compared once whatever the number of columns that
        // use it: the snapshots give what the database has, the models and
        // the `extend!{}` blocks what is declared.
        let tables_before: Vec<ParsedSchema> = self
            .migrated
            .iter()
            .chain(&self.extended)
            .cloned()
            .collect();
        let enums_before = previous_enums(&tables_before, &self.enum_snapshots);
        let declared: Vec<ParsedSchema> = self
            .schemas
            .iter()
            .chain(&self.extend_blocks)
            .cloned()
            .collect();
        self.enum_changes = diff_enums(&enums_before, &enums_of_schemas(&declared));

        // A table dropped in this run brings its enums back on the way down
        // with the values they have now, not those its snapshot was written
        // with.
        let mut migrated = std::mem::take(&mut self.migrated);
        for schema in &mut migrated {
            with_enum_values(schema, &enums_before);
        }
        // A table Runique migrated that no model produces any more. A paused
        // table stays, and so does the former name of a renamed one.
        let kept: Vec<String> = self
            .paused
            .iter()
            .chain(renames.iter().map(|(old, _)| old))
            .cloned()
            .collect();
        self.dropped = dropped_tables(migrated, &self.schemas, &kept);

        self.upgrades = snapshot_upgrades(
            self.entities_path,
            &self.schemas,
            &self.main_changes,
            &self.extend_planned,
            self.migrations_path,
        )?;
        self.upgrades.extend(enum_snapshot_upgrades(
            &enums_before,
            &self.enum_snapshots,
            self.migrations_path,
        ));
        Ok(self)
    }
}
