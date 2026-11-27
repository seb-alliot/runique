use crate::backend::lang::visitor_lang;
use runique::axum::http::HeaderMap;
use runique::prelude::*;
use serde::Serialize;

#[derive(Serialize)]
pub struct ImageForm {
    #[serde(flatten)]
    pub form: Forms,
}

impl FormAccess for ImageForm {
    impl_form_access!();
    fn register_fields(form: &mut Forms) {
        form.field(
            &MediaField::image("image")
                .label("Choose an image to upload")
                .upload_to("uploads/images")
                .required()
                .max_size(FileSize::mb(5))
                .max_files(3)
                .max_dimensions(1920, 1080)
                .allowed_extensions(vec!["png", "jpg", "jpeg", "gif"]),
        );
    }
}

impl RuniqueForm for ImageForm {}

impl ImageForm {
    pub fn in_language(mut self, headers: &HeaderMap) -> Self {
        if visitor_lang(headers) == "fr" {
            self.forms_mut()
                .field_label("image", "Choisissez une image à envoyer");
        }
        self
    }
}
