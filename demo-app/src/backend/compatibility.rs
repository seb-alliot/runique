//! The compatibility page (`/releases/compatibility`). Its versions are read
//! from the workspace manifest and its test databases from
//! `docker-compose.yml`, both at compile time: when a dependency moves, the
//! page follows without anyone editing it. Only the defaults and practices
//! below them are written by hand (`docs/{fr,en}/COMPATIBILITY*.md`).
use crate::backend::{inject_globals, lang::visitor_lang, split_markdown_sections};
use runique::prelude::*;

const MANIFEST: &str = include_str!("../../../Cargo.toml");
const COMPOSE: &str = include_str!("../../../docker-compose.yml");
const TEXT_FR: &str = include_str!("../../../docs/fr/COMPATIBILITY.md");
const TEXT_EN: &str = include_str!("../../../docs/en/COMPATIBILITY.en.md");

/// The dependencies a project meets directly, in the order shown.
const SHOWN: [&str; 14] = [
    "axum",
    "tower",
    "tower-http",
    "tower-sessions",
    "tokio",
    "sea-orm",
    "sea-orm-migration",
    "sea-query",
    "tera",
    "serde",
    "chrono",
    "uuid",
    "argon2",
    "lettre",
];

/// The lines of the `[name]` table of a TOML file, up to the next table.
fn table<'a>(toml: &'a str, name: &str) -> impl Iterator<Item = &'a str> {
    let header = format!("[{name}]");
    toml.lines()
        .skip_while(move |line| line.trim() != header)
        .skip(1)
        .take_while(|line| !line.trim_start().starts_with('['))
}

/// `name = "1.2"` or `name = { version = "1.2", … }`: the name and its version.
fn requirement(line: &str) -> Option<(&str, &str)> {
    let (name, rest) = line.split_once('=')?;
    let rest = rest.trim();
    let quoted = if rest.starts_with('{') {
        rest.split_once("version")?.1.split_once('"')?.1
    } else {
        rest.strip_prefix('"')?
    };
    Some((name.trim(), quoted.split_once('"')?.0))
}

fn versions_markdown(manifest: &str, lang: &str) -> String {
    let package: Vec<_> = table(manifest, "workspace.package")
        .filter_map(requirement)
        .collect();
    let deps: Vec<_> = table(manifest, "workspace.dependencies")
        .filter_map(requirement)
        .collect();
    fn get<'a>(list: &[(&str, &'a str)], key: &str) -> Option<&'a str> {
        list.iter().find(|(name, _)| *name == key).map(|(_, v)| *v)
    }
    let fr = lang == "fr";
    let mut out = if fr {
        "## Versions\n\nLues dans le `Cargo.toml` de Runique à chaque version publiée.\n\n| Élément | Version |\n|---|---|\n"
    } else {
        "## Versions\n\nRead from Runique's `Cargo.toml` at each release.\n\n| Item | Version |\n|---|---|\n"
    }
    .to_string();
    if let Some(version) = get(&package, "rust-version") {
        out.push_str(&format!("| Rust (minimum) | {version} |\n"));
    }
    if let Some(edition) = get(&package, "edition") {
        out.push_str(&format!(
            "| {} | {edition} |\n",
            if fr { "Édition Rust" } else { "Rust edition" }
        ));
    }
    for name in SHOWN {
        if let Some(version) = get(&deps, name) {
            let shown = match version.strip_prefix('=') {
                Some(exact) => format!(
                    "**{exact}** ({})",
                    if fr {
                        "épinglée : utiliser exactement celle-ci"
                    } else {
                        "pinned: use exactly this one"
                    }
                ),
                None => version.to_string(),
            };
            out.push_str(&format!("| `{name}` | {shown} |\n"));
        }
    }
    out.push('\n');
    out
}

fn engines_markdown(compose: &str, lang: &str) -> String {
    let fr = lang == "fr";
    let mut out = if fr {
        "## Moteurs testés\n\nLa suite de tests passe sur ces moteurs (images de `docker-compose.yml`).\n\n| Moteur | Image de test |\n|---|---|\n"
    } else {
        "## Tested engines\n\nThe test suite runs on these engines (images from `docker-compose.yml`).\n\n| Engine | Test image |\n|---|---|\n"
    }
    .to_string();
    for image in compose
        .lines()
        .filter_map(|line| line.trim().strip_prefix("image:"))
        .map(str::trim)
    {
        let engine = match image.split(':').next() {
            Some("postgres") => "PostgreSQL",
            Some("mariadb") => "MariaDB",
            Some("mysql") => "MySQL",
            Some(other) => other,
            None => continue,
        };
        out.push_str(&format!("| {engine} | `{image}` |\n"));
    }
    out.push_str(if fr {
        "| SQLite | en mémoire, embarqué |\n\n"
    } else {
        "| SQLite | in memory, bundled |\n\n"
    });
    out
}

pub async fn compatibility_page(request: &mut Request) -> AppResult<Response> {
    inject_globals(request).await;
    let lang = visitor_lang(&request.headers);
    let text = if lang == "fr" { TEXT_FR } else { TEXT_EN };
    let markdown = format!(
        "{}{}{text}",
        versions_markdown(MANIFEST, lang),
        engines_markdown(COMPOSE, lang)
    );
    let sections = split_markdown_sections(&markdown);
    context_update!(request => {
        "title"       => if lang == "fr" { "Compatibilité" } else { "Compatibility" },
        "description" => if lang == "fr" {
            "Versions de Rust et des crates utilisées par Runique, moteurs testés, réglages par défaut et bonnes pratiques."
        } else {
            "Rust and crate versions used by Runique, tested engines, defaults and good practices."
        },
        "sections"    => &sections,
        "release_tab" => "compatibility",
    });
    request.render("info/release_document.html")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_requirement_is_read_in_both_forms() {
        assert_eq!(requirement(r#"tera = "2.1""#), Some(("tera", "2.1")));
        assert_eq!(
            requirement(r#"axum = { version = "0.8.9", features = ["macros"] }"#),
            Some(("axum", "0.8.9"))
        );
        assert_eq!(requirement(r#"sea-orm = "=2.0.4""#), Some(("sea-orm", "=2.0.4")));
        for not_one in ["", "[dependencies]", r#"    "set-header","#, "] }", "name = 3"] {
            assert_eq!(requirement(not_one), None, "{not_one:?}");
        }
    }

    #[test]
    fn a_table_stops_at_the_next_one() {
        let toml = "[a]\nx = \"1\"\n[b]\ny = \"2\"\n";
        assert_eq!(table(toml, "a").collect::<Vec<_>>(), ["x = \"1\""]);
        assert_eq!(table(toml, "b").collect::<Vec<_>>(), ["y = \"2\""]);
        assert_eq!(table(toml, "c").count(), 0);
    }

    #[test]
    fn the_real_manifest_gives_rust_and_the_pinned_orm() {
        let fr = versions_markdown(MANIFEST, "fr");
        assert!(fr.contains("| Rust (minimum) |"), "{fr}");
        assert!(fr.contains("| `sea-orm` | **"), "{fr}");
        assert!(fr.contains("épinglée"), "{fr}");
        let en = versions_markdown(MANIFEST, "en");
        assert!(en.contains("pinned: use exactly this one"), "{en}");
        assert!(!en.contains("épinglée"), "{en}");
    }

    #[test]
    fn the_real_compose_file_lists_its_engines_and_sqlite() {
        let md = engines_markdown(COMPOSE, "en");
        assert!(md.contains("| PostgreSQL |"), "{md}");
        assert!(md.contains("| MariaDB |"), "{md}");
        assert!(md.contains("| SQLite |"), "{md}");
    }
}
