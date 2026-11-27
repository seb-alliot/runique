use super::*;

impl BuilderOrchestrator<'_> {
    /// Reads what the project declares (models, `extend!{}` blocks) and what
    /// it has migrated (snapshots of tables, of extensions, of enums), then
    /// the migrations still waiting. Parallel runs are refused here, before
    /// anything is compared.
    pub(crate) fn read(mut self) -> Result<Self> {
        let ScannedEntities {
            schemas,
            paused,
            renames,
        } = scan_models(self.entities_path)?;
        if !paused.is_empty() {
            println!(
                "{}",
                tf(
                    runique_i18n::keys::makemigrations::info::PAUSED_TABLES,
                    &[paused.join(", ")]
                )
            );
        }
        self.schemas = schemas;
        self.paused = paused;
        self.declared_renames = renames;

        init_migrate::ensure(self.migrations_path, self.engine.clone().into())?;
        // Before anything reads the snapshots: a run the power cut short is
        // taken back, so the changes are computed on migrations that are whole.
        init_migrate::repair_beside(
            std::path::Path::new(self.migrations_path),
            std::path::Path::new(self.entities_path),
        )?;

        self.migrated = snapshot::read_all(self.migrations_path)?;
        check_parallel(self.migrations_path)?;
        self.waiting = waiting_migrations(self.migrations_path)?;
        fs::create_dir_all(snapshot_dir(self.migrations_path))?;

        self.extended = snapshot::read_extend(self.migrations_path)?;
        self.enum_snapshots = read_enum_snapshots(self.migrations_path)?;
        self.extend_blocks = merge_extend_schemas(scan_extend_blocks(self.entities_path)?);
        Ok(self)
    }
}
