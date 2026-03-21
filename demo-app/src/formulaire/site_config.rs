use crate::entities::site_config::schema as site_config;
use runique::prelude::*;

#[form(schema = site_config, fields = [key, value, description])]
pub struct SiteConfigForm;

#[async_trait]
impl RuniqueForm for SiteConfigForm {
    impl_form_access!(model);
}
