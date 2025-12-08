use axum::{response::Response, http::StatusCode};
use tera::{Tera, Context};
use crate::settings::Settings;
use crate::middleware::error_handler::render_template;

/// Extension trait pour Tera qui ajoute des méthodes de rendu sécurisées
///
/// Ce trait ajoute une méthode `render_safe` qui intègre automatiquement
/// la gestion d'erreur sophistiquée de Rusti.
///
/// # Exemple
/// ```rust,no_run
/// use rusti::middleware::TeraSafe;
/// use rusti::{Tera, Context, StatusCode, Settings};
///
/// async fn my_handler(tera: std::sync::Arc<Tera>, config: std::sync::Arc<Settings>) -> axum::response::Response {
///     let mut context = Context::new();
///     context.insert("title", "Hello");
///
///     tera.render_safe("index.html", &context, StatusCode::OK, &config)
/// }
/// ```
pub trait TeraSafe {
    /// Rend un template avec gestion d'erreur intégrée
    ///
    /// Cette méthode remplace l'appel manuel à `render_template` et fournit:
    /// - Gestion automatique des erreurs de template
    /// - Pages de debug détaillées en mode développement
    /// - Pages d'erreur simples en production
    /// - Prévention des boucles infinies d'erreur
    fn render_safe(
        &self,
        template: &str,
        context: &Context,
        status: StatusCode,
        config: &Settings
    ) -> Response;
}

impl TeraSafe for Tera {
    fn render_safe(
        &self,
        template: &str,
        context: &Context,
        status: StatusCode,
        config: &Settings
    ) -> Response {
        render_template(self, template, context, status, config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tera_safe_trait_exists() {
        // Vérifie que le trait peut être importé et utilisé
        let _settings = Settings::default_values();
    }
}