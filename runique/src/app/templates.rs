use crate::config::RuniqueConfig;
use crate::context::tera::static_tera;
use crate::utils::aliases::ARlockmap;
use crate::utils::constante::{ADMIN_TEMPLATES, ERROR_CORPS, FIELD_TEMPLATES, SIMPLE_TEMPLATES};
use regex::{Captures, Regex};
use std::path::Path;
use tera::Tera;

pub struct TemplateLoader;

impl TemplateLoader {
    /// Initialise Tera et traite tous les templates (internes + utilisateurs)
    pub fn init(
        config: &RuniqueConfig,
        url_registry: ARlockmap,
    ) -> Result<Tera, Box<dyn std::error::Error>> {
        let mut tera = Tera::default();

        // 1b. Enregistrer les filtres personnalisés (static, media, form, etc.)
        static_tera::register_asset_filters(
            &mut tera,
            config.static_files.static_url.clone(),
            config.static_files.media_url.clone(),
            config.static_files.static_runique_url.clone(),
            config.static_files.media_runique_url.clone(),
            url_registry.clone(),
        );

        // 2. Préparation des Regex de transformation (UNE fois)
        let balise_link =
            Regex::new(r#"\{%\s*(?P<tag>static|media)\s*['"](?P<link>[^'"]+)['"]\s*%}"#).unwrap();

        let link_regex = Regex::new(
            r#"\{%\s*link\s*['"](?P<name>[^'"]+)['"]\s*(?:,\s*)?(?P<params>[^%]*?)\s*%}"#,
        )
        .unwrap();

        let form_field_regex =
            Regex::new(r#"\{%\s*form\.([a-zA-Z0-9_]+)\.([a-zA-Z0-9_]+)\s*%}"#).unwrap();

        let form_full_regex = Regex::new(r#"\{%\s*form\.([a-zA-Z0-9_]+)\s*%}"#).unwrap();

        // 3. Chargement des templates internes du framework (AVEC preprocess)
        Self::load_internal_templates(
            &mut tera,
            &balise_link,
            &link_regex,
            &form_field_regex,
            &form_full_regex,
        )?;

        let mut all_templates = Vec::new();

        // 4. Boucle de traitement des dossiers templates configurés (dev) (AVEC preprocess)
        for dir_string in &config.static_files.templates_dir {
            let template_dir = Path::new(dir_string);
            let pattern = format!("{}/**/*.html", template_dir.display());

            if let Ok(paths) = glob::glob(&pattern) {
                for entry in paths.flatten() {
                    let content = std::fs::read_to_string(&entry)?;

                    let processed = Self::process_content(
                        content,
                        &balise_link,
                        &link_regex,
                        &form_field_regex,
                        &form_full_regex,
                    );

                    // Calcul du nom logique du template (chemin relatif)
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

    /// Applique toutes les transformations Runique sur un contenu de template
    fn process_content(
        mut content: String,
        balise_link: &Regex,
        link_regex: &Regex,
        form_field_regex: &Regex,
        form_full_regex: &Regex,
    ) -> String {
        // Remplacements simples (DSL Runique)
        content = content.replace("{% csrf %}", r#"{% include "csrf" %}"#);
        content = content.replace("{% messages %}", r#"{% include "message" %}"#);
        content = content.replace("{{ csp }}", r#"{% include "csp" %}"#);

        // Traitement Formulaires (Champs isolés)
        content = form_field_regex
            .replace_all(&content, |caps: &Captures| {
                format!(
                    r#"{{{{ {} | form(field='{}') | safe }}}}"#,
                    &caps[1], &caps[2]
                )
            })
            .to_string();

        // Traitement Formulaires (Full form)
        content = form_full_regex
            .replace_all(&content, |caps: &Captures| {
                format!("{{{{ {} | form | safe }}}}", &caps[1])
            })
            .to_string();

        // Traitement des liens nommés (link)
        content = link_regex
            .replace_all(&content, |caps: &Captures| {
                let name = &caps["name"];
                let params = caps
                    .name("params")
                    .map(|m| m.as_str().trim())
                    .filter(|s| !s.is_empty());
                match params {
                    Some(p) => format!(r#"{{{{ link(link='{}', {}) }}}}"#, name, p),
                    None => format!(r#"{{{{ link(link='{}') }}}}"#, name),
                }
            })
            .to_string();

        // Traitement Static/Media
        content = balise_link
            .replace_all(&content, |caps: &Captures| {
                format!(r#"{{{{ "{}" | {} }}}}"#, &caps["link"], &caps["tag"])
            })
            .to_string();

        content
    }

    /// Charge les templates HTML embarqués dans le binaire de Runique (AVEC preprocess)
    fn load_internal_templates(
        tera: &mut Tera,
        balise_link: &Regex,
        link_regex: &Regex,
        form_field_regex: &Regex,
        form_full_regex: &Regex,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for (name, content) in SIMPLE_TEMPLATES
            .iter()
            .chain(ERROR_CORPS.iter())
            .chain(FIELD_TEMPLATES.iter())
            .chain(ADMIN_TEMPLATES.iter())
        {
            let processed = Self::process_content(
                content.to_string(),
                balise_link,
                link_regex,
                form_field_regex,
                form_full_regex,
            );

            tera.add_raw_template(name, &processed)?;
        }
        Ok(())
    }
}
