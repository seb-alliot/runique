use crate::entities::runique_release::schema as runique_release;
use runique::prelude::*;

#[form(schema = runique_release, fields = [version, github_url, crates_url])]
pub struct RuniqueReleaseForm;

#[async_trait]
impl RuniqueForm for RuniqueReleaseForm {
    impl_form_access!(model);
}
