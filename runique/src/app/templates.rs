//! Loading and initialization of the Tera template engine (internal + user).
use crate::config::RuniqueConfig;
use crate::context::tera::static_tera;
use crate::utils::aliases::{ARlockmap, StrMap};
use crate::utils::constante::*;
use regex::Captures;
use std::path::Path;
use tera::Tera;

/// Loads and configures the Tera instance with internal framework templates and project templates.
pub(crate) struct TemplateLoader;

/// `content` rewritten as Runique loads its templates (`{% csrf %}`,
/// `{% static %}`, `{% runique_static %}`, `{% link %}`…), without the SRI
/// hashes: for a test that builds its own Tera from Runique's templates.
#[doc(hidden)]
pub fn preprocess(content: &str) -> String {
    TemplateLoader::process_content(content.to_string(), &Integrity::default(), &[])
}

/// The SRI hashes of the static files, by folder: the project's
/// (`{% static %}`) and Runique's own (`{% runique_static %}`, the admin's
/// sheets and scripts).
#[derive(Default)]
pub(crate) struct Integrity {
    pub site: StrMap,
    pub runique: StrMap,
}

/// A block the developer replaced: what `{% dev_block "<name>" %}` becomes,
/// in place of the include of Runique's template for it.
#[doc(hidden)]
#[derive(Debug, Clone, Default)]
pub struct BlockDispatch {
    pub name: String,
    pub content: String,
}

impl TemplateLoader {
    /// Initializes Tera and processes all templates (internal + users)
    pub fn init(
        config: &RuniqueConfig,
        url_registry: ARlockmap,
    ) -> Result<Tera, Box<dyn std::error::Error>> {
        Self::init_with_integrity(config, url_registry).map(|(tera, _)| tera)
    }

    /// `init`, also returning the static files' hashes it computed: the boot
    /// reuses them for the admin's stylesheets instead of hashing twice.
    pub(crate) fn init_with_integrity(
        config: &RuniqueConfig,
        url_registry: ARlockmap,
    ) -> Result<(Tera, Integrity), Box<dyn std::error::Error>> {
        let mut tera = Tera::default();
        tera.autoescape_on(vec!["html", "xml"]);

        // 1b. Register custom filters (static, media, form, etc.)
        static_tera::register_asset_filters(
            &mut tera,
            config.static_files.static_url.clone(),
            config.static_files.media_url.clone(),
            config.static_files.static_runique_url.clone(),
            config.static_files.media_runique.clone(),
            url_registry.clone(),
        );

        let integrity_map = Integrity {
            site: crate::utils::integrity::build_integrity_map(Path::new(
                &config.static_files.staticfiles_dirs,
            )),
            runique: crate::utils::integrity::build_integrity_map(Path::new(
                &config.static_files.static_runique_path,
            )),
        };

        // 3. Runique's templates, the admin's with their block includes
        //    rewritten, then the developer's (WITH preprocess). One batch:
        //    Tera checks every include once all are in, so a rewritten include
        //    naming a developer's template resolves.
        let mut all_templates = Self::internal_templates(&integrity_map, &config.block_dispatch);
        let internal_loaded = all_templates.len();

        // 4. Processing loop for configured template directories (dev) (WITH preprocess)
        // The extensions Tera escapes (`autoescape_on` above), and no other.
        for dir_string in &config.static_files.templates_dir {
            let template_dir = Path::new(dir_string);
            for extension in ["html", "xml"] {
                let pattern = format!("{}/**/*.{extension}", template_dir.display());

                if let Ok(paths) = glob::glob(&pattern) {
                    for entry in paths.flatten() {
                        let content = std::fs::read_to_string(&entry)?;

                        let processed =
                            Self::process_content(content, &integrity_map, &config.block_dispatch);

                        // Calculation of the template's logical name (relative path)
                        let name = entry
                            .strip_prefix(template_dir)?
                            .to_string_lossy()
                            .replace("\\", "/");

                        all_templates.push((name, processed));
                    }
                }
            }
        }

        let user_count = all_templates.len() - internal_loaded;
        if let Err(e) = tera.add_raw_templates(all_templates) {
            // Tera's error message already contains the template name and line number
            tracing::error!(error = %e, "templates failed to load");
            return Err(Box::new(e));
        }

        if let Some(level) = crate::utils::runique_log::get_log()
            .builder
            .as_ref()
            .and_then(|b| b.templates)
        {
            let internal_count = SIMPLE_TEMPLATES.len()
                + ERROR_CORPS.len()
                + FIELD_TEMPLATES.len()
                + AUTH_TEMPLATES.len()
                + ADMIN_TEMPLATES.len();
            crate::runique_log!(
                level,
                internal = internal_count,
                user = user_count,
                total = internal_count + user_count,
                "templates loaded"
            );
        }

        Ok((tera, integrity_map))
    }

    /// Applies all Runique transformations on a template content
    fn process_content(
        mut content: String,
        integrity_map: &Integrity,
        blocks: &[BlockDispatch],
    ) -> String {
        // A block the developer may replace, by its name: what he declared
        // for it, else Runique's template (written before Tera reads it, Tera
        // naming an include once). An unknown name stays, refused by Tera.
        content = DEV_BLOCK_REGEX
            .replace_all(&content, |caps: &Captures| {
                let name = &caps["name"];
                match blocks.iter().find(|b| b.name == name) {
                    Some(block) => block.content.clone(),
                    None => match admin_block(name) {
                        Some(path) => format!(r#"{{% include "{path}" %}}"#),
                        None => caps[0].to_string(),
                    },
                }
            })
            .to_string();

        // Simple replacements (Runique DSL)
        content = content.replace("{% csrf %}", r#"{% include "csrf.html" %}"#);
        content = content.replace("{% messages %}", r#"{% include "message.html" %}"#);
        content = content.replace("{% csp %}", r#"{% include "csp.html" %}"#);

        // Form processing (Isolated fields)
        // No `| safe` is appended: the `form` filter declares `is_safe()`, so the
        // HTML it returns is emitted unescaped on its own authority. Same for
        // `| markdown` and `| sanitize`, which no longer need a rewrite at all.
        content = FORM_FIELD_REGEX
            .replace_all(&content, |caps: &Captures| {
                format!(r"{{{{ {} | form(field='{}') }}}}", &caps[1], &caps[2])
            })
            .to_string();

        // Form processing (Full form)
        content = FORM_FULL_REGEX
            .replace_all(&content, |caps: &Captures| {
                format!("{{{{ {} | form }}}}", &caps[1])
            })
            .to_string();

        // Named link processing (link)
        content = LINK_REGEX
            .replace_all(&content, |caps: &Captures| {
                let name = &caps["name"];
                let params = caps
                    .name("params")
                    .map(|m| m.as_str().trim())
                    .filter(|s| !s.is_empty());
                match params {
                    Some(p) => format!(r"{{{{ link(link='{}', {}) }}}}", name, p),
                    None => format!(r"{{{{ link(link='{}') }}}}", name),
                }
            })
            .to_string();

        // Admin form HTML ({{ form_fields.html }} → {{ form_fields.html | safe }})
        // form_fields.html is always Runique-generated HTML, never raw user input.
        content = ADMIN_FORM_HTML_REGEX
            .replace_all(&content, "{{ form_fields.html | safe }}")
            .to_string();

        // Static/Media processing — literal strings: {% static "path" %} / {% media "path" %}
        content = BALISE_LINK
            .replace_all(&content, |caps: &Captures| {
                let path = &caps["link"];
                let tag = &caps["tag"];
                let q = &caps["q"];
                let url = format!(r#"{{{{ "{}" | {} }}}}"#, path, tag);
                // Each tag's files are hashed from its own folder.
                let map = match tag {
                    "runique_static" => &integrity_map.runique,
                    _ => &integrity_map.site,
                };
                match map.get(path) {
                    Some(hash) => {
                        format!(
                            r#"{}{}{} integrity="{}" crossorigin="anonymous""#,
                            q, url, q, hash
                        )
                    }
                    None => format!("{}{}{}", q, url, q),
                }
            })
            .to_string();

        // Static/Media processing — Tera variables: {% media var %} / {% static var %}
        content = BALISE_LINK_VAR
            .replace_all(&content, |caps: &Captures| {
                format!("{{{{ {} | {} }}}}", &caps["var"], &caps["tag"])
            })
            .to_string();

        content
    }

    /// The HTML templates embedded in the Runique binary (WITH preprocess),
    /// each `{% dev_block %}` written from `blocks`.
    ///
    /// Returned to be added in the same single batch as the developer's:
    /// Tera 2 resolves `{% include %}` and `{% extends %}` when a template is
    /// added, so templates added one by one reject any whose dependency comes
    /// later (`debug.html` includes six partials declared after it), and a
    /// `{% dev_block %}` naming a developer's template needs his in the set.
    fn internal_templates(
        integrity_map: &Integrity,
        blocks: &[BlockDispatch],
    ) -> Vec<(String, String)> {
        SIMPLE_TEMPLATES
            .iter()
            .chain(ERROR_CORPS.iter())
            .chain(FIELD_TEMPLATES.iter())
            .chain(AUTH_TEMPLATES.iter())
            .chain(ADMIN_TEMPLATES.iter())
            .map(|(name, content)| {
                (
                    name.to_string(),
                    Self::process_content(content.to_string(), integrity_map, blocks),
                )
            })
            .collect()
    }
}

/// Written from cargo-mutants survivors (2026-10-02): the preprocessor's
/// rewrites, checked on their exact output.
#[cfg(test)]
mod process_content_tests {
    use super::TemplateLoader;
    use crate::utils::aliases::StrMap;

    fn process(src: &str) -> String {
        TemplateLoader::process_content(src.to_string(), &super::Integrity::default(), &[])
    }

    fn with_runique(path: &str) -> super::Integrity {
        let mut runique = StrMap::new();
        runique.insert(path.to_string(), "sha384-abc".to_string());
        super::Integrity {
            runique,
            ..Default::default()
        }
    }

    /// Runique's own files get their SRI from Runique's folder.
    #[test]
    fn a_runique_static_file_gets_its_integrity() {
        let out = TemplateLoader::process_content(
            r#"<link href="{% runique_static "css/a.css" %}">"#.to_string(),
            &with_runique("css/a.css"),
            &[],
        );
        assert!(
            out.contains(r#"{{ "css/a.css" | runique_static }}"#),
            "{out}"
        );
        assert!(
            out.contains(r#"integrity="sha384-abc" crossorigin="anonymous""#),
            "{out}"
        );
    }

    /// Each tag is hashed from its own folder: a project file of the same
    /// path never takes Runique's hash, nor the reverse.
    #[test]
    fn the_folders_never_lend_each_other_a_hash() {
        let out = TemplateLoader::process_content(
            r#"<link href="{% static "css/a.css" %}">"#.to_string(),
            &with_runique("css/a.css"),
            &[],
        );
        assert!(!out.contains("integrity"), "{out}");
    }

    /// A replaceable block is Runique's include, or the developer's dispatch
    /// when he replaced it; the other blocks stay Runique's.
    #[test]
    fn a_dev_block_is_runiques_unless_replaced() {
        let src = r#"{% dev_block "list_table" %}|{% dev_block "sidebar" %}"#;
        let plain = process(src);
        assert_eq!(
            plain,
            r#"{% include "admin/blocks/list/table.html" %}|{% include "admin/blocks/layout/sidebar.html" %}"#
        );
        let replaced = TemplateLoader::process_content(
            src.to_string(),
            &super::Integrity::default(),
            &[super::BlockDispatch {
                name: "list_table".into(),
                content: r#"{% include "mine.html" %}"#.into(),
            }],
        );
        assert_eq!(
            replaced,
            r#"{% include "mine.html" %}|{% include "admin/blocks/layout/sidebar.html" %}"#
        );
        assert_eq!(
            process(r#"{% dev_block "nope" %}"#),
            r#"{% dev_block "nope" %}"#,
            "left for Tera to refuse"
        );
    }

    #[test]
    fn runique_tags_become_tera() {
        assert_eq!(process("{% csrf %}"), r#"{% include "csrf.html" %}"#);
        assert_eq!(process("{% messages %}"), r#"{% include "message.html" %}"#);
        assert_eq!(process("{% csp %}"), r#"{% include "csp.html" %}"#);
        assert_eq!(
            process("{% form.contact.email %}"),
            "{{ contact | form(field='email') }}"
        );
        assert_eq!(process("{% form.contact %}"), "{{ contact | form }}");
        assert_eq!(
            process("{{ form_fields.html }}"),
            "{{ form_fields.html | safe }}"
        );
    }

    #[test]
    fn link_passes_its_parameters_only_when_there_are_some() {
        assert_eq!(process(r#"{% link "home" %}"#), "{{ link(link='home') }}");
        assert_eq!(
            process(r#"{% link "post", id=post.id %}"#),
            "{{ link(link='post', id=post.id) }}"
        );
    }
}
