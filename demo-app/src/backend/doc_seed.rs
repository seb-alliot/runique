use crate::entities::{doc_block, doc_page, doc_section, site_config};
use runique::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};

// Supprime les anchors HTML GitHub (<a id="..."></a>) inutiles sur le web
fn strip_github_anchors(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut rest = content;
    while let Some(start) = rest.find("<a id=") {
        result.push_str(&rest[..start]);
        if let Some(end) = rest[start..].find("</a>") {
            rest = &rest[start + end + 4..];
        } else {
            rest = &rest[start..];
            break;
        }
    }
    result.push_str(rest);
    // Supprime les lignes vides consécutives laissées par la suppression
    result
}

// Cherche le dossier docs/ en remontant depuis le répertoire courant
fn find_docs_root() -> Option<PathBuf> {
    let candidates = ["docs", "../docs", "../../docs", "/app/docs"];
    for candidate in &candidates {
        let p = PathBuf::from(candidate);
        if p.is_dir() {
            return Some(p);
        }
    }
    None
}

// Extrait le sort_order depuis un nom de fichier type "01-installation.md"
fn extract_order(name: &str) -> i32 {
    name.split('-')
        .next()
        .and_then(|n| n.parse::<i32>().ok())
        .unwrap_or(99)
}

// Extrait le titre depuis le premier # heading du fichier
fn extract_title(content: &str) -> String {
    content
        .lines()
        .find(|l| l.starts_with("# "))
        .map(|l| l.trim_start_matches("# ").trim().to_string())
        .unwrap_or_else(|| "Sans titre".to_string())
}

// Extrait la première phrase après le titre comme lead
// Ignore les blocs Sommaire, les anchors HTML, les listes et les tableaux
// Extrait la première phrase d'intro (avant le premier ## heading)
// Si le fichier va directement aux sections sans intro, retourne None
fn extract_lead(content: &str) -> Option<String> {
    let mut after_title = false;
    let mut in_code_block = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if !in_code_block && trimmed.starts_with("# ") && !after_title {
            after_title = true;
            continue;
        }
        if !after_title {
            continue;
        }

        // Dès qu'on atteint une section ##, on arrête — pas d'intro
        if !in_code_block && trimmed.starts_with("## ") {
            return None;
        }

        // Gestion des blocs de code
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }

        // Ignore les éléments structurels
        if trimmed.is_empty()
            || trimmed.starts_with('#')
            || trimmed.starts_with("---")
            || trimmed.starts_with('-')
            || trimmed.starts_with('|')
            || trimmed.starts_with('[')
            || trimmed.starts_with('<')
            || trimmed.starts_with('>')
        {
            continue;
        }

        return Some(trimmed.to_string());
    }
    None
}

// Découpe le contenu en blocs sur les "## " headings
fn parse_blocks(content: &str) -> Vec<(Option<String>, String, String)> {
    let mut blocks: Vec<(Option<String>, String, String)> = Vec::new();

    // Retire le titre principal (# ...) du contenu avant de splitter
    let body = if let Some(nl) = content.find('\n') {
        let first_line = content[..nl].trim();
        if first_line.starts_with("# ") {
            content[nl..].trim_start()
        } else {
            content.trim_start()
        }
    } else {
        content.trim_start()
    };

    let parts: Vec<&str> = body.split("\n## ").collect();

    // Partie intro (avant le premier ##)
    // Si le body commence directement par "## Sommaire", on l'extrait comme bloc sommaire
    let intro = parts[0].trim();
    let intro_lower = intro.to_lowercase();
    let is_nav_block = intro_lower.starts_with("## sommaire")
        || intro_lower.starts_with("## table des matières")
        || intro_lower.starts_with("## table of contents")
        || intro_lower.starts_with("## contents");
    if !intro.is_empty() {
        if is_nav_block {
            // Retire le heading "## Sommaire" pour ne garder que la liste
            let list_content = intro
                .split_once('\n')
                .map(|x| x.1)
                .unwrap_or("")
                .trim()
                .to_string();
            if !list_content.is_empty() {
                blocks.push((None, list_content, "sommaire".to_string()));
            }
        } else {
            let has_code = intro.contains("```");
            let has_table = intro.contains('|');
            // Si l'intro est du texte simple (sans code ni tableau), c'est le lead — déjà affiché, on skip
            if has_code || has_table {
                let block_type = if has_code { "code" } else { "text" };
                blocks.push((None, intro.to_string(), block_type.to_string()));
            }
        }
    }

    // Sections suivantes
    for part in parts.iter().skip(1) {
        let nl = part.find('\n').unwrap_or(part.len());
        let heading = part[..nl].trim().to_string();

        let heading_lower = heading.to_lowercase();
        let body_part = part[nl..].trim().to_string();

        if heading_lower == "sommaire"
            || heading_lower == "table des matières"
            || heading_lower == "table of contents"
            || heading_lower == "contents"
        {
            // Bloc de navigation : stocké avec block_type "sommaire", sans heading
            if !body_part.is_empty() {
                blocks.push((None, body_part, "sommaire".to_string()));
            }
            continue;
        }

        // Ignore les sections de navigation récurrentes
        if heading_lower == "voir aussi"
            || heading_lower == "see also"
            || heading_lower == "retour au sommaire"
            || heading_lower == "back to summary"
            || heading_lower == "prochaines étapes"
            || heading_lower == "next steps"
        {
            continue;
        }

        if !body_part.is_empty() {
            let block_type = if body_part.contains("```") {
                "code"
            } else {
                "text"
            };
            blocks.push((Some(heading), body_part, block_type.to_string()));
        }
    }

    blocks
}


async fn insert_page_with_blocks(
    section_id: i32,
    slug: &str,
    lang: &str,
    content: &str,
    sort_order: i32,
    db: &DatabaseConnection,
) {
    let title = extract_title(content);
    let lead = extract_lead(content);

    let page = doc_page::ActiveModel {
        section_id: Set(section_id),
        slug: Set(slug.to_string()),
        lang: Set(lang.to_string()),
        title: Set(title),
        lead: Set(lead),
        sort_order: Set(sort_order),
        ..Default::default()
    };

    let page = match page.insert(db).await {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!("doc_seed: impossible d'insérer la page {slug}: {e}");
            return;
        }
    };

    let blocks = parse_blocks(content);
    for (i, (heading, block_content, block_type)) in blocks.into_iter().enumerate() {
        let block = doc_block::ActiveModel {
            page_id: Set(page.id),
            heading: Set(heading),
            content: Set(block_content),
            block_type: Set(block_type),
            sort_order: Set(i as i32),
            ..Default::default()
        };
        if let Err(e) = block.insert(db).await {
            tracing::warn!("doc_seed: impossible d'insérer un bloc pour {slug}: {e}");
        }
    }
}

async fn seed_language(lang: &str, lang_path: &Path, db: &DatabaseConnection) {
    let mut entries: Vec<_> = match fs::read_dir(lang_path) {
        Ok(e) => e.filter_map(|e| e.ok()).collect(),
        Err(_) => return,
    };
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let section_slug = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        if section_slug.is_empty() {
            continue;
        }

        // Les cours ont leur propre table — on les exclut du seed doc
        if section_slug == "cour" {
            continue;
        }

        // Trouve le fichier index (NN-nom.md) dans le dossier section
        let index_file = fs::read_dir(&path)
            .ok()
            .and_then(|mut d| {
                d.find(|e| {
                    e.as_ref()
                        .map(|e| {
                            let name = e.file_name();
                            let name = name.to_string_lossy();
                            name.ends_with(".md")
                                && name.chars().next().is_some_and(|c| c.is_ascii_digit())
                        })
                        .unwrap_or(false)
                })
            })
            .and_then(|e| e.ok());

        let sort_order = index_file
            .as_ref()
            .map(|f| extract_order(&f.file_name().to_string_lossy()))
            .unwrap_or(99);

        let title = index_file
            .as_ref()
            .and_then(|f| fs::read_to_string(f.path()).ok())
            .map(|c| extract_title(&c))
            .unwrap_or_else(|| {
                let mut s = section_slug.clone();
                if let Some(c) = s.get_mut(0..1) {
                    c.make_ascii_uppercase()
                }
                s
            });

        let section = doc_section::ActiveModel {
            slug: Set(section_slug.clone()),
            lang: Set(lang.to_string()),
            title: Set(title),
            sort_order: Set(sort_order),
            ..Default::default()
        };

        match section.insert(db).await {
            Ok(s) => {
                tracing::info!("doc_seed: section créée — {lang}/{section_slug}");
                seed_section_pages(&section_slug, lang, s.id, &path, db).await;
            }
            Err(e) => {
                tracing::warn!("doc_seed: erreur section {section_slug}: {e}");
            }
        }
    }
}

async fn seed_section_pages(
    section_slug: &str,
    lang: &str,
    section_id: i32,
    section_path: &Path,
    db: &DatabaseConnection,
) {
    let mut entries: Vec<_> = match fs::read_dir(section_path) {
        Ok(e) => e.filter_map(|e| e.ok()).collect(),
        Err(_) => return,
    };
    entries.sort_by_key(|e| e.file_name());

    let mut order = 0i32;

    for entry in entries {
        let path = entry.path();

        if path.is_file() && path.extension().is_some_and(|e| e == "md") {
            // Fichier index de la section (NN-nom.md)
            let content = match fs::read_to_string(&path) {
                Ok(c) => strip_github_anchors(&c),
                Err(_) => continue,
            };
            let slug = format!("{section_slug}-index");
            insert_page_with_blocks(section_id, &slug, lang, &content, order, db).await;
            order += 1;
        } else if path.is_dir() {
            // Sous-page : dossier contenant un .md du même nom
            let sub_slug = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            if sub_slug.is_empty() {
                continue;
            }

            // Cherche le .md dans le sous-dossier
            let md_file = fs::read_dir(&path)
                .ok()
                .and_then(|mut d| {
                    d.find(|e| {
                        e.as_ref()
                            .map(|e| e.path().extension().is_some_and(|x| x == "md"))
                            .unwrap_or(false)
                    })
                })
                .and_then(|e| e.ok());

            if let Some(md) = md_file {
                let content = match fs::read_to_string(md.path()) {
                    Ok(c) => strip_github_anchors(&c),
                    Err(_) => continue,
                };
                let page_slug = format!("{section_slug}-{sub_slug}");
                insert_page_with_blocks(section_id, &page_slug, lang, &content, order, db).await;
                order += 1;
            }
        }
    }
}

async fn seed_site_config(db: &DatabaseConnection) {
    let count = site_config::Entity::find().count(db).await.unwrap_or(0);

    if count > 0 {
        return;
    }

    let entries = [
        ("runique_version", "1.1.52", "Version actuelle de Runique"),
        ("release_date", "2026-03-21", "Date de la dernière release"),
        (
            "github_url",
            "https://github.com/seb-alliot/runique",
            "URL du dépôt GitHub",
        ),
        (
            "crates_url",
            "https://crates.io/crates/runique",
            "URL sur crates.io",
        ),
    ];

    for (key, value, description) in &entries {
        let row = site_config::ActiveModel {
            key: Set(std::string::ToString::to_string(key)),
            value: Set(std::string::ToString::to_string(value)),
            description: Set(Some(std::string::ToString::to_string(description))),
            ..Default::default()
        };
        if let Err(e) = row.insert(db).await {
            tracing::warn!("doc_seed: erreur site_config {key}: {e}");
        }
    }

    tracing::info!("doc_seed: site_config initialisé");
}

/// Point d'entrée principal. Vide et re-seede doc_section/page/block à chaque démarrage.
pub async fn seed_docs(db: &DatabaseConnection) {
    seed_site_config(db).await;

    // Nettoyage complet avant re-seed (ordre FK : block → page → section)
    let stmts = [
        "DELETE FROM doc_block",
        "DELETE FROM doc_page",
        "DELETE FROM doc_section",
    ];
    for sql in &stmts {
        if let Err(e) = db.execute_unprepared(sql).await
        {
            tracing::warn!("doc_seed: erreur nettoyage ({sql}): {e}");
            return;
        }
    }

    let docs_root = match find_docs_root() {
        Some(p) => p,
        None => {
            tracing::warn!("doc_seed: dossier docs/ introuvable, seed ignoré");
            return;
        }
    };

    tracing::info!("doc_seed: démarrage depuis {:?}", docs_root);

    for lang in &["fr", "en"] {
        let lang_path = docs_root.join(lang);
        if lang_path.is_dir() {
            seed_language(lang, &lang_path, db).await;
        }
    }

    tracing::info!("doc_seed: terminé");
}
