/// Récupère la valeur d'une variable d'environnement ou retourne une valeur par défaut.
///
/// # Exemple
///
/// ```rust
/// use std::env;
/// // Définit une variable d'environnement temporaire
/// env::set_var("TEST_ENV_VAR", "valeur");
/// assert_eq!(runique::utils::config::lecture_env::env_or_default("TEST_ENV_VAR", "defaut"), "valeur");
/// // Variable non définie
/// assert_eq!(runique::utils::config::lecture_env::env_or_default("INEXISTANTE", "defaut"), "defaut");
/// ```
pub fn env_or_default(var: &str, default: &str) -> String {
    std::env::var(var).unwrap_or(default.to_string())
}
