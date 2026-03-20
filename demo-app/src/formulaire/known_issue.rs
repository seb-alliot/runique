use crate::entities::known_issue::schema as known_issue;
use runique::prelude::*;

#[form(schema = known_issue, fields = [version, title, description, issue_type, sort_order])]
pub struct KnownIssueForm;

#[async_trait]
impl RuniqueForm for KnownIssueForm {
    impl_form_access!(model);
}
