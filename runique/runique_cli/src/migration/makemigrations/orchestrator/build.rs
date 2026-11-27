use super::*;

impl BuilderOrchestrator<'_> {
    /// The final plan: the files of the run in the order of their stages,
    /// taking the place of the migrations that were waiting, written with
    /// `lib.rs` in one batch. All of it, or the project as it was.
    pub(in crate::migration::makemigrations) fn build(self) -> Result<()> {
        if self.main_changes.is_empty()
            && self.extend_planned.is_empty()
            && self.dropped.is_empty()
            && self.enum_changes.is_empty()
        {
            return self.nothing_to_migrate();
        }
        let settled = settled_migrations(self.migrations_path, &self.waiting)?;
        let mut plan = self.stages.in_order(&settled);
        // While the plan holds the migrations and nothing else.
        let rewritten = replace_waiting(&mut plan, &self.waiting);
        plan.files.extend(self.upgrades);

        let module_count = plan.lib_modules.len();
        commit_plan(&plan, self.migrations_path)?;

        if rewritten {
            println!(
                "{}",
                tf(
                    runique_i18n::keys::makemigrations::info::FILES_READY,
                    &[module_count]
                )
            );
        } else {
            println!(
                "{}",
                tf(
                    runique_i18n::keys::makemigrations::info::WAITING_UNCHANGED,
                    &[self.waiting.len()]
                )
            );
        }
        println!("{}", t(runique_i18n::keys::makemigrations::hint::APPLY));
        warn_unmigrated_type_changes(&self.checked);
        Ok(())
    }
}
