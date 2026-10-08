// Tests d'intégration pour utils/forms/parse_html.rs — parse_multipart + sanitize_filename
//
// Stratégie : router oneshot avec handler qui extrait Multipart et appelle parse_multipart.

use axum::{
    Router,
    body::Body,
    extract::Multipart,
    http::{Request, StatusCode, header},
    response::IntoResponse,
    routing::post,
};
use runique::utils::parse_html::parse_multipart;
use tower::ServiceExt;

// ── Handler de test ───────────────────────────────────────────────────────────

async fn multipart_handler(multipart: Multipart) -> impl IntoResponse {
    let upload_dir = std::env::temp_dir().join("runique_test_upload");
    match parse_multipart(multipart, &upload_dir, 100, 1024).await {
        Ok(data) => {
            // Retourne le nombre de champs parsés dans le body
            let count = data.len().to_string();
            (StatusCode::OK, count)
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "error".to_string()),
    }
}

fn multipart_app() -> Router {
    Router::new().route("/upload", post(multipart_handler))
}

// ── Helper : construction d'une requête multipart texte ──────────────────────

fn multipart_text_request(fields: &[(&str, &str)]) -> Request<Body> {
    let boundary = "----boundary123456";
    let mut body = String::new();
    for (name, value) in fields {
        body.push_str(&format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"{name}\"\r\n\r\n{value}\r\n"
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

// ── Tests — champs texte ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_parse_multipart_champ_texte_simple() {
    let req = multipart_text_request(&[("nom", "alice")]);
    let resp = multipart_app().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_parse_multipart_plusieurs_champs() {
    let req = multipart_text_request(&[("nom", "alice"), ("age", "30"), ("ville", "Paris")]);
    let resp = multipart_app().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_parse_multipart_body_vide() {
    let boundary = "----boundary_empty";
    let body = format!("--{boundary}--\r\n");
    let req = Request::builder()
        .method("POST")
        .uri("/upload")
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(body))
        .unwrap();
    let resp = multipart_app().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

// ── Tests — upload de fichier ─────────────────────────────────────────────────

fn multipart_file_request(field_name: &str, filename: &str, content: &str) -> Request<Body> {
    let boundary = "----fileboundary789";
    let body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"{field_name}\"; filename=\"{filename}\"\r\nContent-Type: text/plain\r\n\r\n{content}\r\n--{boundary}--\r\n"
    );
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

#[tokio::test]
async fn test_parse_multipart_upload_fichier() {
    let req = multipart_file_request("fichier", "test.txt", "contenu du fichier");
    let resp = multipart_app().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_parse_multipart_upload_sans_extension() {
    let req = multipart_file_request("fichier", "noext", "données sans extension");
    let resp = multipart_app().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_parse_multipart_mix_texte_et_fichier() {
    let boundary = "----mixboundary";
    let body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"nom\"\r\n\r\nbob\r\n\
         --{boundary}\r\nContent-Disposition: form-data; name=\"doc\"; filename=\"doc.pdf\"\r\nContent-Type: application/pdf\r\n\r\n%PDF-1.4\r\n\
         --{boundary}--\r\n"
    );
    let req = Request::builder()
        .method("POST")
        .uri("/upload")
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(body))
        .unwrap();
    let resp = multipart_app().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

// ── Written from cargo-mutants survivors (2026-10-08) ─────────────────────────

/// Parses a multipart request into a fresh `dir`, with 1 MB per file and 1 KB
/// per text field; returns the status.
async fn parse_into(dir: std::path::PathBuf, req: Request<Body>) -> StatusCode {
    let app = Router::new().route(
        "/upload",
        post(move |multipart: Multipart| {
            let dir = dir.clone();
            async move {
                match parse_multipart(multipart, &dir, 1, 1).await {
                    Ok(_) => StatusCode::OK,
                    Err(resp) => resp.status(),
                }
            }
        }),
    );
    app.oneshot(req).await.unwrap().status()
}

fn temp_upload_dir() -> std::path::PathBuf {
    std::env::temp_dir().join(format!("rq_parse_{}", uuid::Uuid::new_v4()))
}

/// Exactly at a limit is accepted; one byte over is refused (parse_html.rs:108, 154).
#[tokio::test]
async fn upload_limits_take_the_limit_itself() {
    let dir = temp_upload_dir();
    let text = |n: usize| multipart_text_request(&[("bio", &"a".repeat(n))]);
    assert_eq!(parse_into(dir.clone(), text(1024)).await, StatusCode::OK);
    assert_eq!(
        parse_into(dir.clone(), text(1025)).await,
        StatusCode::PAYLOAD_TOO_LARGE
    );

    let file = |n: usize| multipart_file_request("doc", "a.txt", &"b".repeat(n));
    assert_eq!(
        parse_into(dir.clone(), file(1024 * 1024)).await,
        StatusCode::OK
    );
    assert_eq!(
        parse_into(dir.clone(), file(1024 * 1024 + 1)).await,
        StatusCode::PAYLOAD_TOO_LARGE
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// A parse sweeps staging folders older than an hour, and only those: a fresh
/// one stays, and so does any other old folder (parse_html.rs:193, 209, 221).
#[tokio::test]
async fn stale_staging_folders_are_swept_and_nothing_else() {
    let dir = temp_upload_dir();
    let two_hours_ago = std::time::SystemTime::now() - std::time::Duration::from_secs(7200);
    for name in [".staging-old", ".staging-new", "kept-old"] {
        std::fs::create_dir_all(dir.join(name)).unwrap();
    }
    for name in [".staging-old", "kept-old"] {
        std::fs::File::open(dir.join(name))
            .unwrap()
            .set_modified(two_hours_ago)
            .unwrap();
    }

    let status = parse_into(dir.clone(), multipart_text_request(&[("nom", "x")])).await;
    let (old, new, other) = (
        dir.join(".staging-old").exists(),
        dir.join(".staging-new").exists(),
        dir.join("kept-old").exists(),
    );
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(status, StatusCode::OK);
    assert!(!old, "an hour-old staging folder is removed");
    assert!(new, "a fresh one stays");
    assert!(other, "anything that isn't staging is never touched");
}
