use crate::app::error_build::BuildError;

// ═══════════════════════════════════════════════════════════════
// StaticStaging — Service de fichiers statiques
// ═══════════════════════════════════════════════════════════════
//
// Contrôle si les fichiers statiques (CSS, JS, media, assets
// internes Runique) sont servis par l'application.
//
// Activé par défaut. Peut être désactivé pour les API pures
// ou quand un CDN/reverse-proxy gère les fichiers statiques.
// ═══════════════════════════════════════════════════════════════

pub struct StaticStaging {
    /// Indique si le service de fichiers statiques est activé
    pub(crate) enabled: bool,
}

impl StaticStaging {
    /// Crée un StaticStaging (activé par défaut)
    pub fn new() -> Self {
        Self { enabled: true }
    }

    /// Valide la configuration des fichiers statiques
    pub fn validate(&self) -> Result<(), BuildError> {
        // Pour l'instant, rien à valider :
        // - Si désactivé, on ne sert tout simplement rien
        // - Si activé, les chemins sont vérifiés par la config
        //
        // Future : vérifier que les dossiers statiques existent
        // quand `enabled` est true
        Ok(())
    }

    /// Les fichiers statiques sont toujours prêts
    pub fn is_ready(&self) -> bool {
        true
    }
}

impl Default for StaticStaging {
    fn default() -> Self {
        Self::new()
    }
}
