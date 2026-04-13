use runique::prelude::*;
use serde::Serialize;

#[derive(Serialize)]
pub struct ImageForm {
    #[serde(flatten)]
    pub form: Forms,
}

impl RuniqueForm for ImageForm {
    impl_form_access!();
    fn register_fields(form: &mut Forms) {
        form.field(
            &FileField::image("image")
                .label("Choisissez une image à uploader")
                .upload_to("media/uploads/images")
                .required()
                .max_size(5)
                .max_files(3)
                .max_dimensions(1920, 1080)
                .allowed_extensions(vec!["png", "jpg", "jpeg", "gif"]),
        );
        form.add_js(&["js/test_csrf.js"]);
    }
}
