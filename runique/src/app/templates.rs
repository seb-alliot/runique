//! Loading and initialization of the Tera template engine (internal + user).
use crate::config::RuniqueConfig;
use crate::context::tera::static_tera;
use crate::utils::aliases::ARlockmap;
use crate::utils::constante::*;
use regex::Captures;
use std::{collections::HashMap, path::Path};
use tera::Tera;

/// Loads and configures the Tera instance with internal framework templates and project templates.
pub(crate) struct TemplateLoader;

impl TemplateLoader {
    /// Initializes Tera and processes all templates (internal + users)
    pub fn init(
        config: &RuniqueConfig,
        url_registry: ARlockmap,
    ) -> Result<Tera, Box<dyn std::error::Error>> {
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

        let static_dir = Path::new(&config.static_files.staticfiles_dirs);
        let integrity_map = crate::utils::integrity::build_integrity_map(static_dir);

        // 3. Loading internal framework templates (WITH preprocess)
        Self::load_internal_templates(&mut tera, &integrity_map)?;

        let mut all_templates = Vec::new();

        // 4. Processing loop for configured template directories (dev) (WITH preprocess)
        for dir_string in &config.static_files.templates_dir {
            let template_dir = Path::new(dir_string);
            let pattern = format!("{}/**/*.html", template_dir.display());

            if let Ok(paths) = glob::glob(&pattern) {
                for entry in paths.flatten() {
                    let content = std::fs::read_to_string(&entry)?;

                    let processed = Self::process_content(content, &integrity_map);

                    // Calculation of the template's logical name (relative path)
                    let name = entry
                        .strip_prefix(template_dir)?
                        .to_string_lossy()
                        .replace("\\", "/");

                    all_templates.push((name, processed));
                }
            }
        }

        let user_count = all_templates.len();
        if let Err(e) = tera.add_raw_templates(all_templates) {
            // Tera's error message already contains the template name and line number
            tracing::error!(error = %e, "user template failed to load");
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

        Ok(tera)
    }

    /// Applies all Runique transformations on a template content
    fn process_content(mut content: String, integrity_map: &HashMap<String, String>) -> String {
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
                match integrity_map.get(path) {
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

    /// Loads HTML templates embedded in the Runique binary (WITH preprocess)
    ///
    /// Added as a single batch, never one by one: Tera 2 resolves `{% include %}`
    /// and `{% extends %}` when a template is added, so `add_raw_template` in a loop
    /// rejects any template whose dependency comes later in the iteration order
    /// (`debug.html` includes six partials declared after it). `add_raw_templates`
    /// inserts the whole set before validating it once, and rolls the batch back
    /// as a whole on failure.
    fn load_internal_templates(
        tera: &mut Tera,
        integrity_map: &HashMap<String, String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let templates: Vec<(&str, String)> = SIMPLE_TEMPLATES
            .iter()
            .chain(ERROR_CORPS.iter())
            .chain(FIELD_TEMPLATES.iter())
            .chain(AUTH_TEMPLATES.iter())
            .chain(ADMIN_TEMPLATES.iter())
            .map(|(name, content)| {
                (
                    *name,
                    Self::process_content(content.to_string(), integrity_map),
                )
            })
            .collect();

        if let Err(e) = tera.add_raw_templates(templates) {
            // Tera's error already names the offending template and line.
            tracing::error!(error = %e, "internal templates failed to load");
            return Err(Box::new(e));
        }
        Ok(())
    }
}
