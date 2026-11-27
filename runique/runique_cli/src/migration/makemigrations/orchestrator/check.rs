use super::*;

impl BuilderOrchestrator<'_> {
    /// One guard over the whole run, tables, extensions and drops together:
    /// a destructive change is refused unless forced, a name too long for
    /// the database always is.
    pub(crate) fn check(mut self) -> Result<Self> {
        let mut checked: Vec<Changes> = self.main_changes.clone();
        checked.extend(self.extend_planned.iter().map(|(_, c)| c.clone()));
        // A dropped table is a change like the others, and a destructive one.
        checked.extend(self.dropped.iter().map(|schema| Changes {
            table_name: schema.table_name.clone(),
            table: TableChange::Deleted,
            ..Default::default()
        }));
        check_destructive(&checked, self.force)?;
        check_identifier_lengths(&checked)?;
        self.checked = checked;
        Ok(self)
    }
}
