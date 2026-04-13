//! Loading and initialization of the Tera template engine (internal + user).
use crate::config::RuniqueConfig;
use crate::context::tera::static_tera;
use crate::utils::aliases::ARlockmap;
use crate::utils::constante::*;
use regex::Captures;
use std::path::Path;
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

        // 3. Loading internal framework templates (WITH preprocess)
        Self::load_internal_templates(&mut tera)?;

        let mut all_templates = Vec::new();

        // 4. Processing loop for configured template directories (dev) (WITH preprocess)
        for dir_string in &config.static_files.templates_dir {
            let template_dir = Path::new(dir_string);
            let pattern = format!("{}/**/*.html", template_dir.display());

            if let Ok(paths) = glob::glob(&pattern) {
                for entry in paths.flatten() {
                    let content = std::fs::read_to_string(&entry)?;

                    let processed = Self::process_content(content);

                    // Calculation of the template's logical name (relative path)
                    let name = entry
                        .strip_prefix(template_dir)?
                        .to_string_lossy()
                        .replace("\\", "/");

                    all_templates.push((name, processed));
                }
            }
        }

        tera.add_raw_templates(all_templates)?;
        Ok(tera)
    }

    /// Applies all Runique transformations on a template content
    fn process_content(mut content: String) -> String {
        // Simple replacements (Runique DSL)
        content = content.replace("{% csrf %}", r#"{% include "csrf" %}"#);
        content = content.replace("{% messages %}", r#"{% include "message" %}"#);
        content = content.replace("{% csp %}", r#"{% include "csp" %}"#);

        // Form processing (Isolated fields)
        content = FORM_FIELD_REGEX
            .replace_all(&content, |caps: &Captures| {
                format!(
                    r"{{{{ {} | form(field='{}') | safe }}}}",
                    &caps[1], &caps[2]
                )
            })
            .to_string();

        // Form processing (Full form)
        content = FORM_FULL_REGEX
            .replace_all(&content, |caps: &Captures| {
                format!("{{{{ {} | form | safe }}}}", &caps[1])
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

        // Markdown processing ({{ var | markdown }} → {{ var | markdown | safe }})
        content = MARKDOWN_REGEX
            .replace_all(&content, |caps: &Captures| {
                format!("{{{{ {} | markdown | safe }}}}", &caps[1])
            })
            .to_string();

        // Static/Media processing
        content = BALISE_LINK
            .replace_all(&content, |caps: &Captures| {
                format!(r#"{{{{ "{}" | {} }}}}"#, &caps["link"], &caps["tag"])
            })
            .to_string();

        content
    }

    /// Loads HTML templates embedded in the Runique binary (WITH preprocess)
    fn load_internal_templates(tera: &mut Tera) -> Result<(), Box<dyn std::error::Error>> {
        for (name, content) in SIMPLE_TEMPLATES
            .iter()
            .chain(ERROR_CORPS.iter())
            .chain(FIELD_TEMPLATES.iter())
            .chain(AUTH_TEMPLATES.iter())
            .chain(ADMIN_TEMPLATES.iter())
        {
            let processed = Self::process_content(content.to_string());

            tera.add_raw_template(name, &processed)?;
        }
        Ok(())
    }
}
