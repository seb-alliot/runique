//! `env_or_default` — lecture d'une variable d'environnement avec valeur de repli.

/// Récupère la valeur d'une variable d'environnement ou retourne une valeur par défaut.
#[doc = include_str!("../../../doc-tests/read_env/lecture_env.md")]
#[must_use]
pub fn env_or_default(var: &str, default: &str) -> String {
    std::env::var(var).unwrap_or(default.to_string())
}
