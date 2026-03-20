use crate::entities::changelog_entry::schema as changelog_entry;
use runique::prelude::*;

#[form(schema = changelog_entry, fields = [version, release_date, category, title, description, sort_order])]
pub struct ChangelogEntryForm;

#[async_trait]
impl RuniqueForm for ChangelogEntryForm {
    impl_form_access!(model);
}
