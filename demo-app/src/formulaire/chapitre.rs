use crate::entities::chapitre;
use runique::prelude::*;

#[form(schema = chapitre, fields = [cour_id, slug, title, lead, sort_order])]
pub struct ChapitreForm;

#[async_trait]
impl RuniqueForm for ChapitreForm {
    impl_form_access!(model);
}
