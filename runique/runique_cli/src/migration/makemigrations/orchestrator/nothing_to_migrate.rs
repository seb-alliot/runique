use super::*;

impl BuilderOrchestrator<'_> {
    /// The run found no change. A batch is written all the same: the
    /// snapshots to upgrade, and the waiting migrations taken away, which no
    /// longer match the models (a change taken back before it was applied).
    ///
    /// A project without a model of its own still needs the framework's
    /// tables (accounts, sessions): wired into `lib.rs` even then, and the
    /// `sea-orm-cli migrate init` placeholder dropped.
    pub(super) fn nothing_to_migrate(self) -> Result<()> {
        let upgraded = self.upgrades.len();
        let plan = Plan {
            files: self.upgrades,
            dropped_modules: self
                .waiting
                .iter()
                .map(|(module, _)| module.clone())
                .collect(),
            ..Plan::default()
        };
        commit_plan(&plan, self.migrations_path)?;
        if upgraded != 0 {
            println!(
                "{}",
                tf(
                    runique_i18n::keys::makemigrations::info::SNAPSHOTS_UPGRADED,
                    &[upgraded]
                )
            );
        }
        if !self.waiting.is_empty() {
            println!(
                "{}",
                tf(
                    runique_i18n::keys::makemigrations::info::WAITING_REMOVED,
                    &[self.waiting.len()]
                )
            );
        }
        // Silent, a run looked like it did nothing at all; the framework's
        // tables may still be waiting to be applied.
        println!(
            "{}",
            t(runique_i18n::keys::makemigrations::info::NO_CHANGES)
        );
        println!("{}", t(runique_i18n::keys::makemigrations::hint::APPLY));
        Ok(())
    }
}
