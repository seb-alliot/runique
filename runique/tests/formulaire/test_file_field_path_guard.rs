//! A file field only ever acts on a file this request uploaded.
//!
//! Its value is a path: the one the multipart parser staged for the upload. But
//! the same name can also arrive as a plain text value (urlencoded body, or a
//! multipart part without a filename), and the field used to treat whatever
//! path it got as its upload: deleted on a failed validation, moved into
//! MEDIA_ROOT on success.
use axum::http::Method;
use runique::forms::Forms;
use runique::forms::fields::FileField;
use runique::utils::aliases::StrMap;

fn victim(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("rq_victim_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    std::fs::write(&path, b"server file, not an upload").unwrap();
    path
}

#[tokio::test]
async fn a_text_value_cannot_delete_a_server_file() {
    let target = victim("app.db");
    let mut form = Forms::new("csrf");
    form.field(&FileField::image("avatar"));
    let mut data = StrMap::new();
    data.insert("avatar".into(), target.display().to_string());
    form.fill(&data, Method::POST);

    let _ = form.is_valid().await;

    assert!(target.exists(), "the server file must still be there");
    let _ = std::fs::remove_dir_all(target.parent().unwrap());
}
