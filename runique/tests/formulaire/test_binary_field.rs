//! `binary`, `var_binary` and `blob` fields: an upload stored as bytes.
//! The whole chain is exercised: `parse_multipart` stages the upload, the
//! admin form takes it, `finalize` encodes it, the generated conversion
//! decodes it into the entity's `Vec<u8>`.
use crate::utils::env::{del_env, set_env};
use axum::{
    Router,
    body::Body,
    extract::Multipart,
    http::{Method, Request, header},
    routing::post,
};
use runique::forms::Forms;
use runique::forms::field::RuniqueForm;
use runique::utils::parse_html::parse_multipart;
use serial_test::serial;
use tower::ServiceExt;

mod docs {
    use runique::prelude::*;

    model! {
        Doc,
        table: "docs",
        pk: id => i32,
        {
            thumb: var_binary [max_length: 8],
            body: blob,
        }
    }
}

mod users_ext {
    use runique::prelude::*;

    extend! {
        table: "eihwaz_users",
        fields: {
            signature: blob,
        }
    }
}

/// Stages the request's uploads like Prisme, runs the `Doc` admin form, and
/// returns the entity's bytes (`thumb`, `body`), or `invalid`.
async fn handler(multipart: Multipart) -> String {
    let media = std::env::var("MEDIA_ROOT").expect("MEDIA_ROOT");
    let parsed = parse_multipart(multipart, std::path::Path::new(&media), 10, 64)
        .await
        .expect("multipart");
    let data = parsed.into_iter().map(|(k, v)| (k, v.join(","))).collect();
    let mut form = Forms::new("csrf");
    docs::DocAdminForm::register_fields(&mut form);
    form.fill(&data, Method::POST);
    if !form.is_valid().await.unwrap_or(false) {
        return "invalid".into();
    }
    form.finalize().await.expect("finalize");
    let values = form
        .fields
        .iter()
        .map(|(k, f)| (k.clone(), f.value().to_string()))
        .collect();
    let model = docs::admin_from_form(&values, None);
    let text = |value: runique::sea_orm::ActiveValue<Option<Vec<u8>>>| match value {
        runique::sea_orm::ActiveValue::Set(bytes) => {
            String::from_utf8_lossy(&bytes.unwrap_or_default()).into_owned()
        }
        _ => "unset".to_string(),
    };
    format!("{}|{}", text(model.thumb.clone()), text(model.body.clone()))
}

fn request(parts: &[(&str, Option<&str>, &str)]) -> Request<Body> {
    let boundary = "----rqbin";
    let mut body = String::new();
    for (name, filename, content) in parts {
        let disposition = match filename {
            Some(f) => format!("form-data; name=\"{name}\"; filename=\"{f}\""),
            None => format!("form-data; name=\"{name}\""),
        };
        body.push_str(&format!(
            "--{boundary}\r\nContent-Disposition: {disposition}\r\n\r\n{content}\r\n"
        ));
    }
    body.push_str(&format!("--{boundary}--\r\n"));
    Request::builder()
        .method("POST")
        .uri("/upload")
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(body))
        .unwrap()
}

async fn send(req: Request<Body>) -> String {
    let app = Router::new().route("/upload", post(handler));
    let resp = app.oneshot(req).await.expect("response");
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .expect("body");
    String::from_utf8(bytes.to_vec()).expect("utf-8")
}

fn with_media<T>(run: impl FnOnce(&std::path::Path) -> T) -> T {
    let media = std::env::temp_dir().join(format!("rq_bin_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&media).unwrap();
    set_env("MEDIA_ROOT", media.to_str().unwrap());
    let out = run(&media);
    del_env("MEDIA_ROOT");
    out
}

#[tokio::test]
#[serial]
async fn uploaded_bytes_reach_the_entity() {
    let media = with_media(|m| m.to_path_buf());
    set_env("MEDIA_ROOT", media.to_str().unwrap());
    let out = send(request(&[
        ("thumb", Some("t.bin"), "ABC"),
        ("body", Some("b.bin"), "hello blob"),
    ]))
    .await;
    del_env("MEDIA_ROOT");
    assert_eq!(out, "ABC|hello blob");
    let leftovers: Vec<_> = walk(&media);
    assert!(
        leftovers.is_empty(),
        "staged uploads removed: {leftovers:?}"
    );
    let _ = std::fs::remove_dir_all(&media);
}

#[tokio::test]
#[serial]
async fn an_upload_longer_than_the_column_is_refused() {
    let media = with_media(|m| m.to_path_buf());
    set_env("MEDIA_ROOT", media.to_str().unwrap());
    let out = send(request(&[("thumb", Some("t.bin"), "123456789")])).await;
    del_env("MEDIA_ROOT");
    assert_eq!(out, "invalid", "var_binary [max_length: 8] holds 8 bytes");
    let _ = std::fs::remove_dir_all(&media);
}

#[tokio::test]
#[serial]
async fn a_path_sent_as_text_is_never_read() {
    let media = with_media(|m| m.to_path_buf());
    set_env("MEDIA_ROOT", media.to_str().unwrap());
    let secret = std::env::temp_dir().join(format!("rq_secret_{}", uuid::Uuid::new_v4()));
    std::fs::write(&secret, b"TOP SECRET").unwrap();
    let out = send(request(&[("body", None, secret.to_str().unwrap())])).await;
    del_env("MEDIA_ROOT");
    assert_eq!(out, "unset|unset", "nothing read, nothing written");
    assert!(secret.exists());
    let _ = std::fs::remove_file(&secret);
    let _ = std::fs::remove_dir_all(&media);
}

#[test]
fn schema_and_extend_forms_get_a_binary_field() {
    let mut form = Forms::new("csrf");
    docs::schema().fill_form(&mut form, None, None);
    assert_eq!(form.fields["body"].field_type(), "binary");

    let mut form = Forms::new("csrf");
    users_ext::EihwazUsersAdminForm::register_fields(&mut form);
    assert_eq!(form.fields["signature"].field_type(), "binary");
}

fn walk(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.is_dir() {
            files.extend(walk(&path));
        } else {
            files.push(path);
        }
    }
    files
}
