//! Tests — utils/config/integrity.rs : the SRI hashes put on `<link>` / `<script>`.
//! Written from cargo-mutants survivors (2026-10-07): the map was never tested,
//! and a stale map is exactly what blocked runique.io's CSS on 2026-10-07.

use base64::{Engine, engine::general_purpose::STANDARD};
use runique::utils::config::integrity::build_integrity_map;
use sha2::{Digest, Sha384};

#[test]
fn every_file_gets_its_sha384_under_its_relative_path() {
    let root = std::env::temp_dir().join(format!("rq_sri_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(root.join("css/admin")).unwrap();
    std::fs::write(root.join("app.js"), b"console.log(1)").unwrap();
    std::fs::write(root.join("css/admin/main.css"), b"body{}").unwrap();

    let map = build_integrity_map(&root);
    let expected = |bytes: &[u8]| format!("sha384-{}", STANDARD.encode(Sha384::digest(bytes)));
    assert_eq!(map.get("app.js"), Some(&expected(b"console.log(1)")));
    assert_eq!(
        map.get("css/admin/main.css"),
        Some(&expected(b"body{}")),
        "nested folders are scanned, keys use `/`"
    );
    assert_eq!(map.len(), 2);
}
