use crate::backend::FieldGroup;
use crate::entities::{code_example, demo_page, form_field, page_doc_link};
use crate::formulaire::ImageForm;
use runique::prelude::*;

pub async fn fetch_upload_data(
    db: &sea_orm::DatabaseConnection,
) -> (Vec<code_example::Model>, Vec<page_doc_link::Model>) {
    crate::backend::fetch_page_examples("upload_image", db).await
}

pub async fn validate_upload(form: &mut ImageForm) -> Result<(), String> {
    if form.is_valid().await {
        Ok(())
    } else {
        let errors = form.get_form().errors();
        let msg = if errors.is_empty() {
            "Erreur de validation".to_string()
        } else {
            errors.values().cloned().collect::<Vec<_>>().join(" | ")
        };
        Err(msg)
    }
}

pub struct HelpersData {
    pub path_id: Option<String>,
    pub search_value: Option<String>,
    pub cleaned_search: Option<String>,
}

pub fn extract_helpers_data(request: &Request, cleaned_search: Option<String>) -> HelpersData {
    HelpersData {
        path_id: request.path_param("id").map(|s| s.to_string()),
        search_value: request.from_url("search").map(|s| s.to_string()),
        cleaned_search,
    }
}

pub async fn handle_upload_image(
    request: &mut Request,
    form: &mut ImageForm,
) -> AppResult<Response> {
    crate::backend::inject_auth(request).await;
    let template = "forms/upload_image.html";
    let db = request.engine.db.clone();
    let (code_examples, doc_links) = fetch_upload_data(&db).await;
    if request.is_get() {
        context_update!(request => {
            "title"         => "Uploader un fichier",
            "image_form"    => &*form,
            "code_examples" => &code_examples,
            "doc_links"     => &doc_links,
        });
        return request.render(template);
    }
    if request.is_post() {
        match validate_upload(form).await {
            Ok(_) => {
                success!(request.notices => "Fichier uploadé avec succès !");
            }
            Err(msg) => {
                error!(request.notices => &msg);
            }
        }
        return Ok(Redirect::to("/upload-image").into_response());
    }
    request.render(template)
}

pub async fn get_field_groups(db: &sea_orm::DatabaseConnection) -> Vec<FieldGroup> {
    let page = demo_page::Entity::find()
        .filter(demo_page::Column::Slug.eq("formulaires_champs"))
        .one(db)
        .await
        .unwrap_or(None);

    let form_fields = if let Some(ref p) = page {
        form_field::Entity::find()
            .filter(form_field::Column::PageId.eq(p.id))
            .order_by_asc(form_field::Column::SortOrder)
            .all(db)
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
