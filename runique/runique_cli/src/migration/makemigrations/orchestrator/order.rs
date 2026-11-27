use super::*;

impl BuilderOrchestrator<'_> {
    /// Puts the tables in the order their keys ask for: a referenced table
    /// is created before the one that references it. The order of the files
    /// among themselves is the one of their stage (`Stages`).
    pub(in crate::migration::makemigrations) fn order(mut self) -> Result<Self> {
        self.timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
        self.main_changes = topological_sort_changes(std::mem::take(&mut self.main_changes));
        settle_renamed_keys(&mut self.main_changes)?;
        Ok(self)
    }
}
