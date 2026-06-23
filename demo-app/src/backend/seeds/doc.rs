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
    let documents = ["../docs", "docs", "../../docs", "/var/www/runique/docs"];
    for document in &documents {
        let p = PathBuf::from(document);
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

    // If body starts directly with ## (no intro text), prefix \n so the split captures the first heading
    let body_with_prefix;
    let body = if body.starts_with("## ") {
        body_with_prefix = format!("\n{body}");
        body_with_prefix.as_str()
    } else {
        body
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
            || heading_lower == "ia"
        {
            // Bloc de navigation : stocké avec block_type "sommaire", sans heading
            if !body_part.is_empty() {
                blocks.push((None, body_part, "Sommaire".to_string()));
            }
            continue;
        }

        // Ignore les sections de navigation récurrentes et IA
        if heading_lower == "voir aussi"
            || heading_lower == "see also"
            || heading_lower == "retour au sommaire"
            || heading_lower == "back to summary"
            || heading_lower == "prochaines étapes"
            || heading_lower == "next steps"
            || heading_lower == "ia"
        {
            continue;
        }

        if !body_part.is_empty() {
            let block_type = if body_part.contains("```") {
                "Code"
            } else {
                "Text"
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
            page_id: Set(page.id.try_into().unwrap()),
            heading: Set(heading),
            content: Set(block_content),
            block_type: Set(block_type
                .parse::<doc_block::BlockType>()
                .unwrap_or_default()),
            sort_order: Set(i as i32),
            ..Default::default()
        };
        if let Err(e) = block.insert(db).await {
            tracing::warn!("doc_seed: impossible d'insérer un bloc pour {slug}: {e}");
        }
    }
}
use sea_orm::ActiveValue::Set;

// Énumère les sections d'une langue (dossiers sous docs/{lang}/), triées,
// hors `cour` et `ia` qui ont leur propre pipeline.
// Source de vérité partagée par le seeder et le validateur de liens.
fn collect_sections(lang_path: &Path) -> Vec<(String, PathBuf)> {
    let mut entries: Vec<_> = match fs::read_dir(lang_path) {
        Ok(e) => e.filter_map(|e| e.ok()).collect(),
        Err(_) => return Vec::new(),
    };
    entries.sort_by_key(|e| e.file_name());

    let mut out = Vec::new();
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
        if section_slug.is_empty() || section_slug == "cour" || section_slug == "ia" {
            continue;
        }
        out.push((section_slug, path));
    }
    out
}

async fn seed_language(lang: &str, lang_path: &Path, db: &DatabaseConnection) {
    for (section_slug, path) in collect_sections(lang_path) {
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

        let theme_value = match section_slug.as_str() {
            "installation" | "architecture" | "configuration" | "env" => {
                doc_section::SectionTheme::Demarrage
            }
            "routing" | "formulaire" | "flash" | "template" => doc_section::SectionTheme::Web,
            "orm" | "model" => doc_section::SectionTheme::Database,
            "middleware" | "auth" | "session" => doc_section::SectionTheme::Security,
            "admin" => doc_section::SectionTheme::Admin,
            _ => doc_section::SectionTheme::Autres,
        };

        let section = doc_section::ActiveModel {
            slug: Set(section_slug.clone()),
            lang: Set(lang.to_string()),
            title: Set(title),
            sort_order: Set(sort_order),
            theme: Set(Some(theme_value)),
            ..Default::default()
        };

        match section.insert(db).await {
            Ok(s) => {
                seed_section_pages(&section_slug, lang, s.id.try_into().unwrap(), &path, db).await;
            }
            Err(e) => {
                tracing::warn!("doc_seed: erreur section {section_slug}: {e}");
            }
        }
    }
}

// Calcule, pour une section, la liste ordonnée de ses pages : (slug, sort_order, fichier).
//
// RÈGLE DE SLUG — source de vérité UNIQUE, identique au routage
// (`/docs/{lang}/{section}/{page}` ⇒ slug `{section}-{page}`, cf. backend/doc.rs).
// Le seeder l'utilise pour insérer, le validateur de liens pour dériver les URL valides :
// les deux ne peuvent plus diverger.
fn collect_section_pages(section_slug: &str, section_path: &Path) -> Vec<(String, i32, PathBuf)> {
    let mut entries: Vec<_> = match fs::read_dir(section_path) {
        Ok(e) => e.filter_map(|e| e.ok()).collect(),
        Err(_) => return Vec::new(),
    };
    entries.sort_by_key(|e| e.file_name());

    let mut pages = Vec::new();
    let mut order = 0i32;

    for entry in entries {
        let path = entry.path();

        if path.is_file() && path.extension().is_some_and(|e| e == "md") {
            // Fichier index de la section (NN-nom.md)
            pages.push((format!("{section_slug}-index"), order, path));
            order += 1;
        } else if path.is_dir() {
            let sub_slug = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            if sub_slug.is_empty() {
                continue;
            }

            let mut sub_entries: Vec<_> = match fs::read_dir(&path) {
                Ok(e) => e.filter_map(|e| e.ok()).collect(),
                Err(_) => continue,
            };
            sub_entries.sort_by_key(|e| e.file_name());

            for sub_entry in sub_entries {
                let sub_path = sub_entry.path();

                if sub_path.is_file() && sub_path.extension().is_some_and(|e| e == "md") {
                    // .md directement dans le sous-dossier
                    pages.push((format!("{section_slug}-{sub_slug}"), order, sub_path));
                    order += 1;
                } else if sub_path.is_dir() {
                    // Niveau supplémentaire : dossier dans le sous-dossier
                    let leaf_slug = sub_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();
                    if leaf_slug.is_empty() {
                        continue;
                    }
                    let mut md_files: Vec<PathBuf> = fs::read_dir(&sub_path)
                        .map(|d| {
                            d.filter_map(|e| e.ok())
                                .map(|e| e.path())
                                .filter(|p| p.extension().is_some_and(|x| x == "md"))
                                .collect()
                        })
                        .unwrap_or_default();
                    md_files.sort();

                    // Un seul .md (ou .md homonyme du dossier) → la page porte le nom
                    // du dossier feuille. Sinon chaque .md devient une page distincte
                    // suffixée par son nom, sans quoi le routage 2 segments
                    // (/docs/{lang}/{section}/{page}) les rendrait inaccessibles.
                    let single = md_files.len() == 1;
                    for md in md_files {
                        let stem = md.file_stem().and_then(|n| n.to_str()).unwrap_or("");
                        let page_slug = if single || stem == leaf_slug {
                            format!("{section_slug}-{sub_slug}-{leaf_slug}")
                        } else {
                            format!("{section_slug}-{sub_slug}-{leaf_slug}-{stem}")
                        };
                        pages.push((page_slug, order, md));
                        order += 1;
                    }
                }
            }
        }
    }

    pages
}

async fn seed_section_pages(
    section_slug: &str,
    lang: &str,
    section_id: i32,
    section_path: &Path,
    db: &DatabaseConnection,
) {
    for (slug, order, path) in collect_section_pages(section_slug, section_path) {
        let content = match fs::read_to_string(&path) {
            Ok(c) => strip_github_anchors(&c),
            Err(_) => continue,
        };
        insert_page_with_blocks(section_id, &slug, lang, &content, order, db).await;
    }
}

async fn seed_site_config(db: &DatabaseConnection) {
    let entries = [
        ("runique_version", "2.1.13", "Version actuelle de Runique"),
        ("release_date", "2026-06-03", "Date de la dernière release"),
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
        let _ = site_config::Entity::delete_many()
            .filter(site_config::Column::Key.eq(*key))
            .exec(db)
            .await;
        let row = site_config::ActiveModel {
            key: Set(ToString::to_string(key)),
            value: Set(ToString::to_string(value)),
            description: Set(Some(ToString::to_string(description))),
            ..Default::default()
        };
        if let Err(e) = row.insert(db).await {
            tracing::warn!("doc_seed: erreur site_config {key}: {e}");
        }
    }

    tracing::info!("doc_seed: site_config mis à jour");
}

/// Point d'entrée principal. Vide et re-seede `doc_section/page/block` à chaque démarrage.
pub async fn seed_docs(db: &DatabaseConnection) {
    seed_site_config(db).await;

    let docs_root = match find_docs_root() {
        Some(p) => p,
        None => {
            tracing::warn!("doc_seed: dossier docs/ introuvable, seed ignoré");
            return;
        }
    };

    // Nettoyage complet avant re-seed (ordre FK : block → page → section)
    let stmts = [
        "DELETE FROM doc_block",
        "DELETE FROM doc_page",
        "DELETE FROM doc_section",
    ];
    for sql in &stmts {
        if let Err(e) = db.execute_unprepared(sql).await {
            tracing::warn!("doc_seed: erreur nettoyage ({sql}): {e}");
            return;
        }
    }

    tracing::info!("doc_seed: démarrage depuis {:?}", docs_root);

    for lang in &["fr", "en"] {
        let lang_path = docs_root.join(lang);
        if lang_path.is_dir() {
            seed_language(lang, &lang_path, db).await;
        }
    }

    tracing::info!("doc_seed: terminé");
}

#[cfg(test)]
mod doc_link_validation {
    //! Vérifie qu'aucun lien interne `/docs/...` des fichiers markdown ne pointe
    //! vers une page inexistante. Le set d'URL valides est dérivé des MÊMES
    //! fonctions (`collect_sections`, `collect_section_pages`) que le seeder, donc
    //! ce test casse dès qu'un lien diverge de l'arborescence réelle.
    //!
    //! Pur filesystem : aucune base ni serveur requis.
    //!   cargo test -p demo-app internal_doc_links_resolve
    use super::{collect_section_pages, collect_sections, find_docs_root};
    use std::collections::{BTreeMap, BTreeSet};
    use std::fs;
    use std::path::{Path, PathBuf};

    // URL servies pour un slug donné, selon le routage réel :
    //   slug `{section}-index` ⇒ /docs/{lang}/{section} (+ /index explicite)
    //   slug `{section}-{page}` ⇒ /docs/{lang}/{section}/{page}
    fn slug_to_urls(lang: &str, section: &str, slug: &str) -> Vec<String> {
        if slug == format!("{section}-index") {
            vec![
                format!("/docs/{lang}/{section}"),
                format!("/docs/{lang}/{section}/index"),
            ]
        } else {
            let page = slug.strip_prefix(&format!("{section}-")).unwrap_or(slug);
            vec![format!("/docs/{lang}/{section}/{page}")]
        }
    }

    fn valid_urls(docs_root: &Path) -> BTreeSet<String> {
        let mut set = BTreeSet::new();
        for lang in ["fr", "en"] {
            let lang_path = docs_root.join(lang);
            if !lang_path.is_dir() {
                continue;
            }
            set.insert(format!("/docs/{lang}"));
            for (section, path) in collect_sections(&lang_path) {
                set.insert(format!("/docs/{lang}/{section}"));
                for (slug, _order, _path) in collect_section_pages(&section, &path) {
                    set.extend(slug_to_urls(lang, &section, &slug));
                }
            }
        }
        set
    }

    fn all_md(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(rd) = fs::read_dir(dir) else { return };
        for entry in rd.filter_map(|e| e.ok()) {
            let p = entry.path();
            if p.is_dir() {
                all_md(&p, out);
            } else if p.extension().is_some_and(|x| x == "md") {
                out.push(p);
            }
        }
    }

    // Cibles des liens markdown inline `](/docs/...)`.
    fn extract_doc_links(content: &str) -> Vec<String> {
        let mut links = Vec::new();
        let mut i = 0;
        while let Some(rel) = content[i..].find("](/docs/") {
            let start = i + rel + 2; // après "]("
            let end = content[start..]
                .find([')', ' ', '"', '\n'])
                .map_or(content.len(), |x| start + x);
            links.push(content[start..end].to_string());
            i = end;
        }
        links
    }

    fn normalize(link: &str) -> String {
        let link = link.split(['#', '?']).next().unwrap_or(link);
        link.trim_end_matches('/').to_string()
    }

    #[test]
    fn internal_doc_links_resolve() {
        let Some(docs_root) = find_docs_root() else {
            eprintln!("docs/ introuvable — test ignoré");
            return;
        };

        let valid = valid_urls(&docs_root);

        let mut md = Vec::new();
        all_md(&docs_root, &mut md);

        let mut dead: Vec<String> = Vec::new();
        for file in &md {
            let content = fs::read_to_string(file).unwrap_or_default();
            for link in extract_doc_links(&content) {
                if !valid.contains(&normalize(&link)) {
                    dead.push(format!("{}  →  {link}", file.display()));
                }
            }
        }
        dead.sort();

        assert!(
            dead.is_empty(),
            "{} lien(s) interne(s) mort(s) dans la doc :\n{}",
            dead.len(),
            dead.join("\n")
        );
    }

    // Dans un tableau de navigation (Table des matières, "Voir aussi"…), chaque
    // ligne doit pointer vers une page distincte. Deux lignes vers la MÊME URL
    // est la signature d'un lien valide-mais-erroné (ex. 4 sous-pages CSP
    // pointant toutes vers l'index `/middleware/csp`) — que le check d'existence
    // ne détecte pas, puisque l'index existe bel et bien.
    #[test]
    fn doc_tables_have_no_duplicate_internal_targets() {
        let Some(docs_root) = find_docs_root() else {
            eprintln!("docs/ introuvable — test ignoré");
            return;
        };

        let mut md = Vec::new();
        all_md(&docs_root, &mut md);

        let mut problems: Vec<String> = Vec::new();
        for file in &md {
            let content = fs::read_to_string(file).unwrap_or_default();
            let lines: Vec<&str> = content.lines().collect();
            let mut i = 0;
            while i < lines.len() {
                // Bloc de tableau : lignes consécutives commençant par '|'.
                if lines[i].trim_start().starts_with('|') {
                    let mut seen: BTreeMap<String, BTreeSet<usize>> = BTreeMap::new();
                    while i < lines.len() && lines[i].trim_start().starts_with('|') {
                        for link in extract_doc_links(lines[i]) {
                            seen.entry(normalize(&link)).or_default().insert(i + 1);
                        }
                        i += 1;
                    }
                    for (url, rows) in &seen {
                        if rows.len() >= 2 {
                            let rows: Vec<String> = rows.iter().map(usize::to_string).collect();
                            problems.push(format!(
                                "{} : {url} pointé par {} lignes du même tableau ({})",
                                file.display(),
                                rows.len(),
                                rows.join(", ")
                            ));
                        }
                    }
                } else {
                    i += 1;
                }
            }
        }
        problems.sort();

        assert!(
            problems.is_empty(),
            "{} tableau(x) avec cible interne dupliquée (probable dérive de lien) :\n{}",
            problems.len(),
            problems.join("\n")
        );
    }
}
