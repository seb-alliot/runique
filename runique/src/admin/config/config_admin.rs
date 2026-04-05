//! Configuration du panneau d'administration : préfixe, titre, hot reload, auth et templates.
use std::sync::Arc;

use crate::admin::template::AdminTemplate;
use crate::middleware::auth::AdminAuth;
use crate::utils::env::is_debug;

pub struct AdminConfig {
    /// Préfixe des routes admin (défaut : "/admin")
    pub prefix: String,

    /// Active le daemon de hot reload en développement
    pub hot_reload: bool,

    /// Titre affiché dans l'interface admin
    pub site_title: String,

    /// URL de retour vers le site principal (défaut : "/")
    pub site_url: String,

    /// Active ou désactive entièrement l'AdminPanel
    pub enabled: bool,

    /// Handler de vérification du login admin
    ///
    /// Voir `crate::middleware::auth::AdminAuth`.
    pub auth: Option<Arc<dyn AdminAuth>>,

    /// Surcharges de templates admin (dashboard, login, list, etc.)
    pub templates: AdminTemplate,

    /// Nombre d'entrées par page dans la vue liste (défaut : 10)
    pub page_size: u64,

    /// URL de base pour la réinitialisation de mot de passe (défaut : None)
    /// Le token sera ajouté automatiquement : `{reset_password_url}/{token}`
    ///
    /// En production avec mailer, doit être une URL absolue :
    /// `"https://monsite.fr/reset-password"`
    ///
    /// Si None, le lien est affiché dans le flash message (dev sans mailer).
    pub reset_password_url: Option<String>,

    /// Ressources "utilisateur" : clé de ressource → template email optionnel.
    /// Activer via `.user_resource("users")`.
    /// À la création, génère un mot de passe aléatoire hashé et envoie un email de reset.
    pub user_resources: std::collections::HashMap<String, Option<String>>,

    /// Template Tera pour l'email de reset de mot de passe depuis l'admin.
    /// Défaut : "admin/reset_password_email.html"
    /// Contexte disponible : `username`, `email`, `reset_url`
    pub reset_password_email_template: Option<String>,

    /// Ordre d'affichage des ressources dans la nav (clés URL).
    /// Les clés non listées apparaissent à la fin dans leur ordre d'insertion.
    pub resource_order: Vec<String>,
}

impl Clone for AdminConfig {
    fn clone(&self) -> Self {
        Self {
            prefix: self.prefix.clone(),
            hot_reload: self.hot_reload,
            site_title: self.site_title.clone(),
            site_url: self.site_url.clone(),
            enabled: self.enabled,
            auth: self.auth.clone(),
            templates: self.templates.clone(),
            page_size: self.page_size,
            reset_password_url: self.reset_password_url.clone(),
            user_resources: self.user_resources.clone(),
            reset_password_email_template: self.reset_password_email_template.clone(),
            resource_order: self.resource_order.clone(),
        }
    }
}

impl std::fmt::Debug for AdminConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AdminConfig")
            .field("prefix", &self.prefix)
            .field("hot_reload", &self.hot_reload)
            .field("site_title", &self.site_title)
            .field("site_url", &self.site_url)
            .field("enabled", &self.enabled)
            .field("auth", &self.auth.as_ref().map(|_| "<AdminAuth>"))
            .field("templates", &self.templates)
            .finish()
    }
}

impl AdminConfig {
    pub fn new() -> Self {
        Self {
            prefix: "/admin".to_string(),
            hot_reload: is_debug(),
            site_title: "Administration".to_string(),
            site_url: "/".to_string(),
            enabled: true,
            auth: None,
            templates: AdminTemplate::new(),
            page_size: 10,
            reset_password_url: None,
            user_resources: std::collections::HashMap::new(),
            reset_password_email_template: None,
            resource_order: Vec::new(),
        }
    }

    pub fn page_size(mut self, size: u64) -> Self {
        self.page_size = size.max(1);
        self
    }

    pub fn prefix(mut self, prefix: &str) -> Self {
        self.prefix = prefix.to_string();
        self
    }

    pub fn hot_reload(mut self, enabled: bool) -> Self {
        self.hot_reload = enabled;
        self
    }

    pub fn site_title(mut self, title: &str) -> Self {
        self.site_title = title.to_string();
        self
    }

    pub fn site_url(mut self, url: &str) -> Self {
        self.site_url = url.to_string();
        self
    }

    /// URL de base pour le reset de mot de passe côté projet.
    /// Le token sera ajouté automatiquement : `{url}/{token}`
    ///
    /// En production (avec mailer), passer une URL absolue :
    /// ```rust,ignore
    /// .with_admin(|a| a.reset_password_url("https://monsite.fr/reset-password"))
    /// ```
    ///
    /// Pour lire depuis l'environnement dans `main.rs` :
    /// ```rust,ignore
    /// let reset_url = std::env::var("RESET_PASSWORD_URL").ok();
    /// .with_admin(|a| {
    ///     let a = match &reset_url { Some(u) => a.reset_password_url(u), None => a };
    ///     a
    /// })
    /// ```
    pub fn reset_password_url(mut self, url: &str) -> Self {
        self.reset_password_url = Some(url.to_string());
        self
    }

    /// Branche le handler d'authentification admin
    ///
    /// ```rust,ignore
    /// AdminConfig::new().auth(RuniqueAdminAuth::new())
    ///
    /// AdminConfig::new().auth(DefaultAdminAuth::<users::Entity>::new())
    /// ```
    pub fn auth<A: AdminAuth>(mut self, handler: A) -> Self {
        self.auth = Some(Arc::new(handler));
        self
    }

    pub fn disable(mut self) -> Self {
        self.enabled = false;
        self
    }

    /// Déclare une ressource comme "utilisateur".
    /// À la création : mot de passe aléatoire hashé + email de reset envoyé automatiquement.
    /// Le champ email du formulaire doit s'appeler "email".
    ///
    /// ```rust,ignore
    /// AdminConfig::new().user_resource("users")
    /// ```
    pub fn user_resource(mut self, resource_key: &str) -> Self {
        self.user_resources.insert(resource_key.to_string(), None);
        self
    }

    /// Template Tera pour l'email de reset de mot de passe depuis l'admin.
    /// Contexte : `username`, `email`, `reset_url`
    pub fn reset_password_email_template(mut self, path: &str) -> Self {
        self.reset_password_email_template = Some(path.to_string());
        self
    }

    /// Comme `user_resource` mais avec un template email personnalisé.
    ///
    /// ```rust,ignore
    /// AdminConfig::new().user_resource_with_template("users", "emails/welcome.html")
    /// ```
    pub fn user_resource_with_template(mut self, resource_key: &str, email_template: &str) -> Self {
        self.user_resources
            .insert(resource_key.to_string(), Some(email_template.to_string()));
        self
    }

    /// Définit l'ordre d'affichage des ressources dans la nav admin.
    ///
    /// ```rust,ignore
    /// AdminConfig::new().resource_order(["users", "blog", "droits", "groupes"])
    /// ```
    /// Les clés non listées apparaissent à la fin dans leur ordre d'insertion.
    pub fn resource_order<I, S>(mut self, order: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.resource_order = order.into_iter().map(Into::into).collect();
        self
    }
}

impl Default for AdminConfig {
    fn default() -> Self {
        Self::new()
    }
}
