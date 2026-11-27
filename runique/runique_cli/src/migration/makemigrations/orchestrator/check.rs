use super::*;

impl BuilderOrchestrator<'_> {
    /// One guard over the whole run, tables, extensions and drops together:
    /// a destructive change is refused unless forced, a name too long for
    /// the database always is.
    pub(in crate::migration::makemigrations) fn check(mut self) -> Result<Self> {
        let mut checked: Vec<Changes> = self.main_changes.clone();
        checked.extend(self.extend_planned.iter().map(|(_, c)| c.clone()));
        checked.extend(dropped_as_changes(&self.dropped));
        check_destructive(&checked, self.force)?;
        check_identifier_lengths(&checked)?;
        self.checked = checked;
        Ok(self)
    }
}
