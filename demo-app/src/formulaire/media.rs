use crate::backend::lang::visitor_lang;
use runique::prelude::*;

#[derive(Serialize, Debug, Clone)]
pub struct MediaForm {
    #[serde(flatten)]
    pub form: Forms,
}

impl FormAccess for MediaForm {
    impl_form_access!();
    fn register_fields(form: &mut Forms) {
        form.field(
            &MediaField::video("video")
                .label("Vidéo")
                .upload_to("uploads/videos")
                .max_size(FileSize::mb(50)),
        );
        form.field(
            &MediaField::audio("audio")
                .label("Audio")
                .upload_to("uploads/audio")
                .max_size(FileSize::mb(20)),
        );
    }
}

impl RuniqueForm for MediaForm {}

impl MediaForm {
    pub fn in_language(mut self, headers: &HeaderMap) -> Self {
        if visitor_lang(headers) == "fr" {
            self.forms_mut()
                .field_label("image", "Choisissez une image à envoyer");
        }
        self
    }
}
