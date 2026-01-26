use crate::config::RuniqueConfig;
use crate::context::tera::static_tera;
use regex::{Captures, Regex};
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::sync::{Arc, RwLock};
use tera::Tera;

pub struct TemplateLoader;

impl TemplateLoader {
    /// Initialise Tera et traite tous les templates (internes + utilisateurs)
    pub fn init(
        config: &RuniqueConfig,
        url_registry: Arc<RwLock<HashMap<String, String>>>,
    ) -> Result<Tera, Box<dyn std::error::Error>> {
        let mut tera = Tera::default();

        // 1. Chargement des templates internes du framework
        Self::load_internal_templates(&mut tera)?;

        // 1b. Enregistrer les filtres personnalisés (static, media, form, etc.)
        static_tera::register_all_asset_filters(
            &mut tera,
            config.static_files.static_url.clone(),
            config.static_files.media_url.clone(),
            config.static_files.static_runique_url.clone(),
            config.static_files.media_runique_url.clone(),
            url_registry,
        );

        // 2. Préparation des Regex de transformation
        let balise_link =
            Regex::new(r#"\{%\s*(?P<tag>static|media)\s*['"](?P<link>[^'"]+)['"]\s*%}"#).unwrap();
        let link_regex = Regex::new(
            r#"\{%\s*link\s*['"](?P<name>[^'"]+)['"]\s*(?:,\s*)?(?P<params>[^%]*?)\s*%}"#,
        )
        .unwrap();
        let form_field_regex =
            Regex::new(r#"\{%\s*form\.([a-zA-Z0-9_]+)\.([a-zA-Z0-9_]+)\s*%}"#).unwrap();
        let form_full_regex = Regex::new(r#"\{%\s*form\.([a-zA-Z0-9_]+)\s*%}"#).unwrap();

        let mut all_templates = Vec::new();

        // 3. Boucle de traitement des dossiers templates configurés
        for dir_string in &config.static_files.templates_dir {
            let template_dir = Path::new(dir_string);
            let pattern = format!("{}/**/*.html", template_dir.display());

            if let Ok(paths) = glob::glob(&pattern) {
                for entry in paths.flatten() {
                    let mut content = std::fs::read_to_string(&entry)?;

                    // Remplacements simples
                    content = content.replace("{% csrf %}", r#"{% include "csrf" %}"#);
                    content = content.replace("{% messages %}", r#"{% include "message" %}"#);
                    content = content.replace("{{ csp }}", r#"{% include "csp" %}"#);

                    // Traitement Formulaires (Champs isolés)
                    content = form_field_regex.replace_all(&content, |caps: &Captures| {
                        format!(r#"{{% set field = {} | form(field='{}') %}}{{% set input_type = field.field_type %}}{{% include "base_string" %}}"#, &caps[1], &caps[2])
                    }).to_string();

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

                    // Calcul du nom logique du template (chemin relatif)
                    let name = entry
                        .strip_prefix(template_dir)?
                        .to_string_lossy()
                        .replace("\\", "/");

                    all_templates.push((name, content));
                }
            }
        }

        tera.add_raw_templates(all_templates)?;
        Ok(tera)
    }

    /// Charge les templates HTML embarqués dans le binaire de Runique
    fn load_internal_templates(tera: &mut Tera) -> Result<(), Box<dyn Error>> {
        // Templates principaux + sécurité
        const SIMPLE_TEMPLATES: [(&str, &str); 7] = [
            (
                "base_index",
                include_str!("../../templates/runique_index/base_index.html"),
            ),
            (
                "message",
                include_str!("../../templates/message/message.html"),
            ),
            ("404", include_str!("../../templates/errors/404.html")),
            ("500", include_str!("../../templates/errors/500.html")),
            (
                "debug",
                include_str!("../../templates/errors/debug_error.html"),
            ),
            ("csrf", include_str!("../../templates/csrf/csrf.html")),
            ("csp", include_str!("../../templates/csp/csp.html")),
        ];

        // Corps des pages d'erreurs détaillées
        const ERROR_CORPS: [(&str, &str); 8] = [
            (
                "errors/corps-error/header-error.html",
                include_str!("../../templates/errors/corps-error/header-error.html"),
            ),
            (
                "errors/corps-error/message-error.html",
                include_str!("../../templates/errors/corps-error/message-error.html"),
            ),
            (
                "errors/corps-error/template-info.html",
                include_str!("../../templates/errors/corps-error/template-info.html"),
            ),
            (
                "errors/corps-error/stack-trace-error.html",
                include_str!("../../templates/errors/corps-error/stack-trace-error.html"),
            ),
            (
                "errors/corps-error/request-info.html",
                include_str!("../../templates/errors/corps-error/request-info.html"),
            ),
            (
                "errors/corps-error/environment-info.html",
                include_str!("../../templates/errors/corps-error/environment-info.html"),
            ),
            (
                "errors/corps-error/status-code-info.html",
                include_str!("../../templates/errors/corps-error/status-code-info.html"),
            ),
            (
                "errors/corps-error/footer-error.html",
                include_str!("../../templates/errors/corps-error/footer-error.html"),
            ),
        ];

        // Champs de formulaire
        const FIELD_TEMPLATES: [(&str, &str); 10] = [
            (
                "base_boolean",
                include_str!("../../templates/field_html/base_boolean.html"),
            ),
            (
                "base_checkbox",
                include_str!("../../templates/field_html/base_checkbox.html"),
            ),
            (
                "base_color",
                include_str!("../../templates/field_html/base_color.html"),
            ),
            (
                "base_datetime",
                include_str!("../../templates/field_html/base_datetime.html"),
            ),
            (
                "base_file",
                include_str!("../../templates/field_html/base_file.html"),
            ),
            (
                "base_number",
                include_str!("../../templates/field_html/base_number.html"),
            ),
            (
                "base_radio",
                include_str!("../../templates/field_html/base_radio.html"),
            ),
            (
                "base_select",
                include_str!("../../templates/field_html/base_select.html"),
            ),
            (
                "base_special",
                include_str!("../../templates/field_html/base_special.html"),
            ),
            (
                "base_string",
                include_str!("../../templates/field_html/base_string.html"),
            ),
        ];

        // Enregistrement centralisé
        for (name, content) in SIMPLE_TEMPLATES
            .iter()
            .chain(ERROR_CORPS.iter())
            .chain(FIELD_TEMPLATES.iter())
        {
            tera.add_raw_template(name, content)?;
        }

        Ok(())
    }
}
