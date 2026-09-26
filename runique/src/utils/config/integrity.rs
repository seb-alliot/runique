use crate::utils::aliases::StrMap;
use base64::{Engine, engine::general_purpose::STANDARD};
use sha2::{Digest, Sha384};
use std::{fs, path::Path};

/// Recursively walks `static_dir` and returns a map from each file's path
/// (relative to `static_dir`, `/`-separated) to its Subresource Integrity hash
/// (`sha384-<base64>`), for use in `integrity` attributes on `<script>`/`<link>` tags.
pub fn build_integrity_map(static_dir: &Path) -> StrMap {
    let mut map = StrMap::new();
    scan_dir(static_dir, static_dir, &mut map);
    map
}

fn scan_dir(root: &Path, dir: &Path, map: &mut StrMap) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_dir(root, &path, map);
        } else {
            let Ok(bytes) = fs::read(&path) else { continue };
            let hash = Sha384::digest(&bytes);
            let b64 = STANDARD.encode(hash);
            if let Ok(relative) = path.strip_prefix(root) {
                let key = relative.to_string_lossy().replace('\\', "/");
                map.insert(key, format!("sha384-{}", b64));
            }
        }
    }
}
