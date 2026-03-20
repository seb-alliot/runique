use crate::entities::roadmap_entry::schema as roadmap_entry;
use runique::prelude::*;

#[form(schema = roadmap_entry, fields = [status, title, description, link_url, link_label, sort_order])]
pub struct RoadmapEntryForm;

#[async_trait]
impl RuniqueForm for RoadmapEntryForm {
    impl_form_access!(model);
}
