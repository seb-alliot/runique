//! Initialisation du subscriber tracing — utilisé par le CLI ; les apps web passent par `RuniqueLog`.
use crate::utils::runique_log::RuniqueLog;

/// Initialise le subscriber tracing avec les paramètres par défaut.
/// Utilisé par le CLI Runique — les applications web n'ont pas à l'appeler
/// (le builder le fait automatiquement via `RuniqueLog::init_subscriber`).
pub fn init_logging() {
    RuniqueLog::default().init_subscriber();
}
