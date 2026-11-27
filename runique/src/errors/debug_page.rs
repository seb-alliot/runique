//! What the debug error page reads from the project itself: the source of a
//! template, and the name and version of the application.

/// Reads a template's raw source from the `templates/` directory, returning `None`
/// if the file doesn't exist or can't be read.
pub fn read_template_source(template_name: &str) -> Option<String> {
    // The name can come from a request (a template picked by a slug): only a
    // plain relative path under `templates/` is read, never `../` nor an absolute
    // one, which the debug page would otherwise show (`../../.env`).
    let relative = std::path::Path::new(template_name);
    if !relative
        .components()
        .all(|part| matches!(part, std::path::Component::Normal(_)))
    {
        return None;
    }
    std::fs::read_to_string(std::path::Path::new("templates").join(relative)).ok()
}

/// The developer's package name and version, read from the `Cargo.toml` in
/// `base_dir`. Runique can't know them at compile time (its own `env!` gives
/// Runique's version), so the debug page reads the project on disk. Only the
/// debug page calls it: production never reads the manifest.
#[must_use]
pub fn app_package(base_dir: &str) -> (Option<String>, Option<String>) {
    let dir = std::path::Path::new(base_dir);
    let Some(manifest) = read_manifest(&dir.join("Cargo.toml")) else {
        return (None, None);
    };
    let Some(package) = manifest.get("package") else {
        return (None, None);
    };
    let name = package
        .get("name")
        .and_then(toml::Value::as_str)
        .map(str::to_string);
    let version = match package.get("version") {
        Some(toml::Value::String(v)) => Some(v.clone()),
        // `version.workspace = true`: the value sits in the workspace manifest.
        Some(v) if v.get("workspace").and_then(toml::Value::as_bool) == Some(true) => {
            workspace_version(dir)
        }
        _ => None,
    };
    (name, version)
}

fn workspace_version(dir: &std::path::Path) -> Option<String> {
    let dir = dir.canonicalize().ok()?;
    dir.ancestors().find_map(|d| {
        read_manifest(&d.join("Cargo.toml"))?
            .get("workspace")?
            .get("package")?
            .get("version")?
            .as_str()
            .map(str::to_string)
    })
}

fn read_manifest(path: &std::path::Path) -> Option<toml::Table> {
    std::fs::read_to_string(path).ok()?.parse().ok()
}
