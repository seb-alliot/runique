use crate::backend::lang::msg;
use crate::formulaire::MediaForm;
use runique::prelude::*;

/// The audio and video upload page. On success the files are shown in the
/// page itself, to check they play under the site's CSP.
pub async fn handle_upload_media(request: &mut Request, form: MediaForm) -> AppResult<Response> {
    crate::backend::inject_globals(request).await;
    let template = "forms/upload_media.html";

    match ValidationForm::try_form(form, request).await {
        Ok(validated) => {
            let mut form = validated.into_form();
            if let Err(error) = form.forms_mut().publish_uploads().await {
                error!(request.notices => &error);
                return Ok(Redirect::to("/upload-media").into_response());
            }
            form.forms_mut().complete_uploads().await;
            // The path each file now has under the media root (`uploads/videos/…`).
            let video = form
                .cleaned::<String>("video")
                .filter(|path| !path.is_empty());
            let audio = form
                .cleaned::<String>("audio")
                .filter(|path| !path.is_empty());
            context_update!(request => {
                "title"      => "Upload audio or video",
                "media_form" => &form,
                "video_path" => &video,
                "audio_path" => &audio,
            });
            request.render(template)
        }
        Err(mut form) => {
            crate::backend::lang::localize_errors(form.forms_mut(), &request.headers);
            if !form.is_submitted() {
                context_update!(request => {
                    "title"      => "Upload audio or video",
                    "media_form" => &form,
                });
                request.render(template)
            } else {
                let errors = form.forms().errors();
                let msg = if errors.is_empty() {
                    msg(&request.headers, "Validation error", "Erreur de validation").to_string()
                } else {
                    errors.values().cloned().collect::<Vec<_>>().join(" | ")
                };
                error!(request.notices => &msg);
                Ok(Redirect::to("/upload-media").into_response())
            }
        }
    }
}
