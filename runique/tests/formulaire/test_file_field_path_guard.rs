//! A file field only ever acts on a file this request uploaded.
//!
//! Its value is a path: the one `parse_multipart` staged for the upload. But
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

// ── Through the real upload chain: parse_multipart → fill → validate → finalize

mod through_upload {
    use crate::utils::env::{del_env, set_env};
    use axum::{
        Router,
        body::Body,
        extract::Multipart,
        http::{Request, header},
        routing::post,
    };
    use runique::forms::Forms;
    use runique::forms::fields::FileField;
    use runique::utils::parse_html::parse_multipart;
    use serial_test::serial;
    use tower::ServiceExt;

    /// Stages the request's files the way Prisme does, then runs a form with
    /// one `doc` file field; answers with the value the field ends up with.
    async fn handler(multipart: Multipart) -> String {
        let media = std::env::var("MEDIA_ROOT").expect("MEDIA_ROOT set by the test");
        let parsed = parse_multipart(multipart, std::path::Path::new(&media), 10, 64)
            .await
            .expect("multipart parsed");
        let data = parsed.into_iter().map(|(k, v)| (k, v.join(","))).collect();
        let mut form = Forms::new("csrf");
        form.field(&FileField::document("doc"));
        form.fill(&data, axum::http::Method::POST);
        if !form.is_valid().await.unwrap_or(false) {
            return "invalid".into();
        }
        form.finalize().await.expect("finalize");
        form.fields["doc"].value().to_string()
    }

    fn request(parts: &[(&str, Option<&str>, &str)]) -> Request<Body> {
        let boundary = "----rqboundary";
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

    fn media_root() -> std::path::PathBuf {
        let media = std::env::temp_dir().join(format!("rq_media_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&media).unwrap();
        set_env("MEDIA_ROOT", media.to_str().unwrap());
        media
    }

    #[tokio::test]
    #[serial]
    async fn a_real_upload_is_committed_to_media_root() {
        let media = media_root();
        let value = send(request(&[("doc", Some("report.pdf"), "%PDF-1.4")])).await;
        // Staging renames the upload (random name, extension kept).
        assert!(value.ends_with(".pdf") && !value.contains('/'), "{value}");
        assert!(media.join(&value).exists(), "committed under MEDIA_ROOT");
        del_env("MEDIA_ROOT");
        let _ = std::fs::remove_dir_all(&media);
    }

    #[tokio::test]
    #[serial]
    async fn a_path_sent_as_text_is_ignored() {
        let media = media_root();
        let outside = std::env::temp_dir().join(format!("rq_out_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&outside).unwrap();
        let victim = outside.join("secret.pdf");
        std::fs::write(&victim, b"server file").unwrap();
        let path = victim.display().to_string();

        // Alone, then next to a real upload under the same name.
        let alone = send(request(&[("doc", None, &path)])).await;
        let mixed = send(request(&[
            ("doc", Some("report.pdf"), "%PDF-1.4"),
            ("doc", None, &path),
        ]))
        .await;

        assert_eq!(alone, "", "no upload, nothing kept");
        assert_eq!(mixed, "", "one path that isn't an upload rejects the value");
        assert!(victim.exists(), "the server file is untouched");
        assert!(
            !media.join("secret.pdf").exists(),
            "and never moved into media"
        );
        del_env("MEDIA_ROOT");
        let _ = std::fs::remove_dir_all(&media);
        let _ = std::fs::remove_dir_all(&outside);
    }
}
