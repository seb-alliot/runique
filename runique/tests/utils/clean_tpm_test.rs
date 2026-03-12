use std::fs;

pub async fn test_cleanup_final_supprime_tout() {
    let temp = std::env::temp_dir();

    if let Ok(entries) = fs::read_dir(&temp) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            if name.to_string_lossy().starts_with("runique_test_")
                || name.to_string_lossy().starts_with("runique_flow_")
            {
                let path = entry.path();
                if path.is_dir() {
                    fs::remove_dir_all(path).ok();
                } else {
                    fs::remove_file(path).ok();
                }
            }
        }
    }
}
