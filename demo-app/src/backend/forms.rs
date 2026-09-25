use crate::backend::FieldGroup;
use crate::entities::{code_example, demo_page, form_field, page_doc_link};
use crate::formulaire::ImageForm;
use runique::prelude::*;

pub async fn fetch_upload_data(db: &ADb) -> (Vec<code_example::Model>, Vec<page_doc_link::Model>) {
    crate::backend::fetch_page_examples("upload_image", db).await
}

pub struct HelpersData {
    pub path_id: Option<String>,
    pub search_value: Option<String>,
    pub cleaned_search: Option<String>,
}

pub fn extract_helpers_data(request: &Request, cleaned_search: Option<String>) -> HelpersData {
    HelpersData {
        path_id: request.get_path("id").map(|s| s.to_string()),
        search_value: request.get_query("search").map(|s| s.to_string()),
        cleaned_search,
    }
}

pub async fn handle_upload_image(request: &mut Request, form: ImageForm) -> AppResult<Response> {
    crate::backend::inject_globals(request).await;
    let template = "forms/upload_image.html";
    let db = request.engine.db.clone();
    let (code_examples, doc_links) = fetch_upload_data(&db).await;

    match ValidationForm::try_new(form, request).await {
        Ok(_) => {
            success!(request.notices => "File uploaded successfully!");
            Ok(Redirect::to("/upload-image").into_response())
        }
        Err(form) => {
            // GET (nothing submitted yet): show the blank form. POST that
            // failed validation: this route redirects with a flash instead
            // of re-rendering inline (unlike the other form handlers).
            if request.method.is_safe() {
                context_update!(request => {
                    "title"         => "Upload a file",
                    "image_form"    => &form,
                    "code_examples" => &code_examples,
                    "doc_links"     => &doc_links,
                });
                request.render(template)
            } else {
                let errors = form.get_form().errors();
                let msg = if errors.is_empty() {
                    "Validation error".to_string()
                } else {
                    errors.values().cloned().collect::<Vec<_>>().join(" | ")
                };
                error!(request.notices => &msg);
                Ok(Redirect::to("/upload-image").into_response())
            }
        }
    }
}

pub async fn get_field_groups(db: &ADb) -> Vec<FieldGroup> {
    let page = search!(demo_page::Entity => Slug eq "formulaires_champs")
        .first(db)
        .await
        .unwrap_or(None);

    let form_fields = if let Some(ref p) = page {
        form_field::Entity::find()
            .filter(form_field::Column::PageId.eq(p.id))
            .order_by_asc(form_field::Column::SortOrder)
            .all(db.as_ref())
            .await
            .unwrap_or_default()
    } else {
        vec![]
    };

    let mut groups: Vec<FieldGroup> = vec![];
    for field in form_fields {
        if groups.last().map(|g: &FieldGroup| g.type_name.as_str()) != Some(&field.field_type) {
            groups.push(FieldGroup {
                type_name: field.field_type.clone(),
                fields: vec![],
            });
        }
        groups.last_mut().unwrap().fields.push(field);
    }
    groups
}
