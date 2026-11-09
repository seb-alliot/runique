use crate::entities::{chapitre, cour, cour_block};
use runique::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};

struct CourDef {
    slug: &'static str,
    title: &'static str,
    title_en: &'static str,
    theme: &'static str,
    difficulte: &'static str,
    sort_order: i32,
    ordre: i32,
}

const COURS: &[CourDef] = &[
    CourDef {
        slug: "cargo-dependances",
        title: "Cargo & dépendances",
        title_en: "Cargo & dependencies",
        theme: "Fondamentaux",
        difficulte: "debutant",
        sort_order: 1,
        ordre: 1,
    },
    CourDef {
        slug: "variables-et-fonctions",
        title: "Variables & fonctions",
        title_en: "Variables & functions",
        theme: "Fondamentaux",
        difficulte: "debutant",
        sort_order: 2,
        ordre: 2,
    },
    CourDef {
        slug: "structures-et-controle",
        title: "Structures & contrôle",
        title_en: "Control structures",
        theme: "Fondamentaux",
        difficulte: "debutant",
        sort_order: 3,
        ordre: 3,
    },
    CourDef {
        slug: "structures-enums",
        title: "Structures & enums",
        title_en: "Structs & enums",
        theme: "Fondamentaux",
        difficulte: "debutant",
        sort_order: 4,
        ordre: 4,
    },
    CourDef {
        slug: "pattern-matching",
        title: "Pattern matching",
        title_en: "Pattern matching",
        theme: "Fondamentaux",
        difficulte: "debutant",
        sort_order: 5,
        ordre: 5,
    },
    CourDef {
        slug: "collections",
        title: "Collections",
        title_en: "Collections",
        theme: "Fondamentaux",
        difficulte: "debutant",
        sort_order: 6,
        ordre: 6,
    },
    CourDef {
        slug: "modules",
        title: "Modules",
        title_en: "Modules",
        theme: "Fondamentaux",
        difficulte: "debutant",
        sort_order: 7,
        ordre: 7,
    },
    CourDef {
        slug: "tests-rust",
        title: "Tests en Rust",
        title_en: "Testing in Rust",
        theme: "Fondamentaux",
        difficulte: "debutant",
        sort_order: 8,
        ordre: 8,
    },
    CourDef {
        slug: "borrow-bases",
        title: "Borrow & ownership",
        title_en: "Borrowing & ownership",
        theme: "Fondamentaux",
        difficulte: "debutant",
        sort_order: 9,
        ordre: 29,
    },
    CourDef {
        slug: "closures-iterateurs",
        title: "Closures & itérateurs",
        title_en: "Closures & iterators",
        theme: "Mémoire & sûreté",
        difficulte: "intermediaire",
        sort_order: 1,
        ordre: 9,
    },
    CourDef {
        slug: "gestion-des-erreurs",
        title: "Gestion des erreurs",
        title_en: "Error handling",
        theme: "Mémoire & sûreté",
        difficulte: "intermediaire",
        sort_order: 2,
        ordre: 10,
    },
    CourDef {
        slug: "generics",
        title: "Generics",
        title_en: "Generics",
        theme: "Mémoire & sûreté",
        difficulte: "intermediaire",
        sort_order: 3,
        ordre: 11,
    },
    CourDef {
        slug: "type-aliases",
        title: "Type aliases",
        title_en: "Type aliases",
        theme: "Mémoire & sûreté",
        difficulte: "intermediaire",
        sort_order: 4,
        ordre: 12,
    },
    CourDef {
        slug: "lifetimes",
        title: "Lifetimes",
        title_en: "Lifetimes",
        theme: "Mémoire & sûreté",
        difficulte: "intermediaire",
        sort_order: 5,
        ordre: 13,
    },
    CourDef {
        slug: "box-dynamiques",
        title: "Box & types dynamiques",
        title_en: "Box & dynamic types",
        theme: "Mémoire & sûreté",
        difficulte: "intermediaire",
        sort_order: 6,
        ordre: 14,
    },
    CourDef {
        slug: "smart-pointers",
        title: "Smart pointers",
        title_en: "Smart pointers",
        theme: "Mémoire & sûreté",
        difficulte: "intermediaire",
        sort_order: 7,
        ordre: 15,
    },
    CourDef {
        slug: "send-sync",
        title: "Send & Sync",
        title_en: "Send & Sync",
        theme: "Mémoire & sûreté",
        difficulte: "intermediaire",
        sort_order: 8,
        ordre: 16,
    },
    CourDef {
        slug: "borrow-avance",
        title: "Borrow avancé — Cow & AsRef",
        title_en: "Advanced borrowing — Cow & AsRef",
        theme: "Mémoire & sûreté",
        difficulte: "intermediaire",
        sort_order: 9,
        ordre: 30,
    },
    CourDef {
        slug: "traits-basics",
        title: "Traits — Les bases",
        title_en: "Traits — The basics",
        theme: "Avancé",
        difficulte: "avance",
        sort_order: 1,
        ordre: 27,
    },
    CourDef {
        slug: "traits-avances",
        title: "Traits avancés",
        title_en: "Advanced traits",
        theme: "Avancé",
        difficulte: "avance",
        sort_order: 2,
        ordre: 17,
    },
    CourDef {
        slug: "macros-declaratives",
        title: "Macros déclaratives",
        title_en: "Declarative macros",
        theme: "Avancé",
        difficulte: "avance",
        sort_order: 3,
        ordre: 18,
    },
    CourDef {
        slug: "macros-export",
        title: "Macros — Visibilité et export",
        title_en: "Macros — Visibility and export",
        theme: "Avancé",
        difficulte: "avance",
        sort_order: 4,
        ordre: 23,
    },
    CourDef {
        slug: "macros-derive",
        title: "Macros procédurales — Derive",
        title_en: "Procedural macros — Derive",
        theme: "Avancé",
        difficulte: "avance",
        sort_order: 5,
        ordre: 24,
    },
    CourDef {
        slug: "macros-attribut",
        title: "Macros procédurales — Attribute",
        title_en: "Procedural macros — Attribute",
        theme: "Avancé",
        difficulte: "avance",
        sort_order: 6,
        ordre: 25,
    },
    CourDef {
        slug: "macros-function-like",
        title: "Macros procédurales — Function-like",
        title_en: "Procedural macros — Function-like",
        theme: "Avancé",
        difficulte: "avance",
        sort_order: 7,
        ordre: 26,
    },
    CourDef {
        slug: "async-tokio",
        title: "Async & Tokio",
        title_en: "Async & Tokio",
        theme: "Avancé",
        difficulte: "avance",
        sort_order: 8,
        ordre: 19,
    },
    CourDef {
        slug: "concurrence",
        title: "Concurrence & état partagé",
        title_en: "Concurrency & shared state",
        theme: "Avancé",
        difficulte: "avance",
        sort_order: 9,
        ordre: 28,
    },
    CourDef {
        slug: "serde",
        title: "Sérialisation — serde",
        title_en: "Serialization — serde",
        theme: "Indispensables",
        difficulte: "intermediaire",
        sort_order: 1,
        ordre: 31,
    },
    CourDef {
        slug: "gestion-erreurs-avancee",
        title: "Gestion erreurs avancée",
        title_en: "Advanced error handling",
        theme: "Indispensables",
        difficulte: "intermediaire",
        sort_order: 2,
        ordre: 32,
    },
    CourDef {
        slug: "orm",
        title: "ORM — SeaORM",
        title_en: "ORM — SeaORM",
        theme: "Runique",
        difficulte: "specifique",
        sort_order: 1,
        ordre: 20,
    },
    CourDef {
        slug: "filtre-admin",
        title: "Filtre admin",
        title_en: "Admin filters",
        theme: "Runique",
        difficulte: "specifique",
        sort_order: 2,
        ordre: 21,
    },
    CourDef {
        slug: "middleware-ordre",
        title: "Middlewares — Pièges et Solutions",
        title_en: "Middlewares — Pitfalls and solutions",
        theme: "Runique",
        difficulte: "specifique",
        sort_order: 3,
        ordre: 22,
    },
];

const LANGS: [&str; 2] = ["fr", "en"];

fn find_cour_dir(lang: &str) -> Option<PathBuf> {
    [
        format!("docs/{lang}/cour"),
        format!("../docs/{lang}/cour"),
        format!("../../docs/{lang}/cour"),
        format!("/app/docs/{lang}/cour"),
        format!("/var/www/runique/docs/{lang}/cour"),
    ]
    .into_iter()
    .map(PathBuf::from)
    .find(|p| p.is_dir())
}

fn slugify(s: &str) -> String {
    let s = s.to_lowercase();
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            'é' | 'è' | 'ê' | 'ë' => result.push('e'),
            'à' | 'â' | 'ä' => result.push('a'),
            'î' | 'ï' => result.push('i'),
            'ô' => result.push('o'),
            'ù' | 'û' | 'ü' => result.push('u'),
            'ç' => result.push('c'),
            '<' | '>' | '`' | '(' | ')' | '!' | '?' | ',' | '.' | ':' => {}
            '&' => result.push_str("et"),
            ' ' | '\t' | '_' | '/' => result.push('-'),
            c if c.is_ascii_alphanumeric() || c == '-' => result.push(c),
            _ => {}
        }
    }
    // Collapse consecutive dashes
    let mut out = String::with_capacity(result.len());
    let mut prev_dash = false;
    for c in result.chars() {
        if c == '-' {
            if !prev_dash {
                out.push('-');
            }
            prev_dash = true;
        } else {
            out.push(c);
            prev_dash = false;
        }
    }
    let out = out.trim_matches('-');
    out[..out.len().min(80)].to_string()
}

fn detect_block_type(content: &str) -> &'static str {
    let stripped = content.trim();
    if stripped.starts_with("```") {
        return "code";
    }
    // Table: line matching |...|
    if stripped.starts_with("| ") || stripped.starts_with("|") && stripped.contains('|') {
        let first_line = stripped.lines().next().unwrap_or("");
        if first_line.starts_with('|') && first_line.ends_with('|') {
            return "table";
        }
    }
    // List: starts with - , * or digit.
    if stripped.starts_with("- ") || stripped.starts_with("* ") {
        return "list";
    }
    if stripped.len() >= 3 {
        let mut chars = stripped.chars();
        let first = chars.next();
        let second = chars.next();
        let third = chars.next();
        if first.is_some_and(|c| c.is_ascii_digit()) && second == Some('.') && third == Some(' ') {
            return "list";
        }
    }
    // Warning: blockquote
    if stripped.starts_with("> ") {
        return "warning";
    }
    "text"
}

/// Découpe le contenu markdown en chapitres délimités par `## Titre`.
/// Retourne vec de (titre_chapitre, contenu_brut).
/// Filtre les sections de navigation (table des matières, objectifs), en FR et en EN.
fn split_chapitres(content: &str) -> Vec<(String, String)> {
    let mut chapitres: Vec<(String, String)> = Vec::new();
    let mut current_title: Option<String> = None;
    let mut current_lines: Vec<&str> = Vec::new();

    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("## ") {
            // Save previous chapter
            if let Some(title) = current_title.take() {
                let raw = current_lines.join("\n").trim().to_string();
                chapitres.push((title, raw));
            }
            current_title = Some(rest.trim().to_string());
            current_lines = Vec::new();
        } else if current_title.is_some() {
            current_lines.push(line);
        }
    }

    // Flush last chapter
    if let Some(title) = current_title {
        let raw = current_lines.join("\n").trim().to_string();
        chapitres.push((title, raw));
    }

    // Filter navigation chapters and empty ones
    chapitres
        .into_iter()
        .filter(|(title, raw)| {
            let title_low = title.to_lowercase();
            let is_nav = title_low.contains("table des matières")
                || title_low.contains("objectifs")
                || title_low.contains("objectives")
                || title_low.contains("table of contents");
            !is_nav && !raw.trim().is_empty()
        })
        .collect()
}

/// Découpe le contenu d'un chapitre en blocs.
/// Retourne vec de (heading, content, block_type).
fn split_blocks(raw: &str) -> Vec<(Option<String>, String, &'static str)> {
    let mut blocks: Vec<(Option<String>, String, &'static str)> = Vec::new();
    let lines: Vec<&str> = raw.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        // Sous-titre ### → heading du bloc suivant
        if let Some(rest) = line.strip_prefix("### ") {
            let heading = rest.trim().to_string();
            i += 1;
            // Collect body until next ### or end
            let mut body_lines: Vec<&str> = Vec::new();
            while i < lines.len() && !lines[i].starts_with("### ") {
                body_lines.push(lines[i]);
                i += 1;
            }
            let body = body_lines.join("\n").trim().to_string();
            if !body.is_empty() {
                let btype = detect_block_type(&body);
                blocks.push((Some(heading), body, btype));
            }
            continue;
        }

        // Bloc de code ```
        if line.starts_with("```") {
            let mut code_lines: Vec<&str> = vec![line];
            i += 1;
            while i < lines.len() && !lines[i].starts_with("```") {
                code_lines.push(lines[i]);
                i += 1;
            }
            if i < lines.len() {
                code_lines.push(lines[i]);
                i += 1;
            }
            let content = code_lines.join("\n").trim().to_string();
            blocks.push((None, content, "code"));
            continue;
        }

        // Ligne vide → skip
        if line.trim().is_empty() {
            i += 1;
            continue;
        }

        // Bloc texte / liste / table / warning : collecte jusqu'à ligne vide
        let mut block_lines: Vec<&str> = Vec::new();
        while i < lines.len() && !lines[i].trim().is_empty() {
            // Stop if we hit ``` or ###
            if lines[i].starts_with("```") || lines[i].starts_with("### ") {
                break;
            }
            block_lines.push(lines[i]);
            i += 1;
        }

        if !block_lines.is_empty() {
            let content = block_lines.join("\n").trim().to_string();
            if !content.is_empty() {
                let btype = detect_block_type(&content);
                blocks.push((None, content, btype));
            }
        }
    }

    blocks
        .into_iter()
        .filter(|(_, content, _)| !content.trim().is_empty())
        .collect()
}

/// Point d'entrée principal. Vide et re-seede cour/chapitre/cour_block à chaque démarrage.
pub async fn seed_cours(db: &DatabaseConnection) {
    tracing::info!("cour_seed: démarrage");

    // Ordre FK : cour_block → chapitre → cour
    if let Err(e) = cour_block::Entity::delete_many().exec(db).await {
        tracing::warn!("cour_seed: erreur nettoyage cour_block: {e}");
        return;
    }
    if let Err(e) = chapitre::Entity::delete_many().exec(db).await {
        tracing::warn!("cour_seed: erreur nettoyage chapitre: {e}");
        return;
    }
    if let Err(e) = cour::Entity::delete_many().exec(db).await {
        tracing::warn!("cour_seed: erreur nettoyage cour: {e}");
        return;
    }

    for lang in LANGS {
        match find_cour_dir(lang) {
            Some(dir) => seed_lang(db, lang, &dir).await,
            None => tracing::warn!("cour_seed: dossier docs/{lang}/cour/ introuvable, ignoré"),
        }
    }

    tracing::info!("cour_seed: terminé");
}

async fn seed_lang(db: &DatabaseConnection, lang: &str, cour_dir: &Path) {
    tracing::info!("cour_seed: lecture depuis {:?}", cour_dir);

    for def in COURS {
        let md_path = cour_dir.join(format!("{}.md", def.slug));

        let content = match fs::read_to_string(&md_path) {
            Ok(c) => c,
            Err(_) => {
                tracing::warn!("cour_seed: fichier introuvable — {lang}/{}.md", def.slug);
                continue;
            }
        };

        // Insère le cours
        let cour_model = cour::ActiveModel {
            slug: Set(def.slug.to_string()),
            lang: Set(lang.to_string()),
            title: Set(if lang == "fr" { def.title } else { def.title_en }.to_string()),
            theme: Set(def.theme.parse::<cour::CourTheme>().unwrap_or_default()),
            difficulte: Set(def
                .difficulte
                .parse::<cour::Difficulte>()
                .unwrap_or_default()),
            sort_order: Set(def.sort_order),
            ordre: Set(def.ordre),
            ..Default::default()
        };

        let inserted_cour = match cour_model.insert(db).await {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("cour_seed: erreur insertion cour '{}': {e}", def.slug);
                continue;
            }
        };

        let chapitres = split_chapitres(&content);

        for (chap_order, (chap_title, chap_raw)) in chapitres.into_iter().enumerate() {
            let chap_slug = format!("{}-{}", def.slug, slugify(&chap_title));

            let chapitre_model = chapitre::ActiveModel {
                cour_id: Set(inserted_cour.id),
                slug: Set(chap_slug.clone()),
                title: Set(chap_title.clone()),
                lead: Set(None),
                sort_order: Set(chap_order as i32 + 1),
                ..Default::default()
            };

            let inserted_chapitre = match chapitre_model.insert(db).await {
                Ok(c) => c,
                Err(e) => {
                    tracing::warn!("cour_seed: erreur insertion chapitre '{}': {e}", chap_slug);
                    continue;
                }
            };

            let blocs = split_blocks(&chap_raw);

            for (blk_order, (heading, blk_content, blk_type)) in blocs.into_iter().enumerate() {
                let block_model = cour_block::ActiveModel {
                    chapitre_id: Set(inserted_chapitre.id),
                    heading: Set(heading),
                    content: Set(blk_content),
                    block_type: Set(blk_type
                        .parse::<cour_block::CourBlockType>()
                        .unwrap_or_default()),
                    sort_order: Set(blk_order as i32 + 1),
                    ..Default::default()
                };

                if let Err(e) = block_model.insert(db).await {
                    tracing::warn!(
                        "cour_seed: erreur insertion bloc pour chapitre '{}': {e}",
                        chap_slug
                    );
                }
            }
        }

        tracing::info!("cour_seed: cours seedé — {lang}/{}", def.slug);
    }
}
