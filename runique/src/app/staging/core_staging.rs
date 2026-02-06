use crate::app::error_build::{BuildError, CheckError, CheckReport};
use crate::utils::aliases::{new_registry, ARlockmap};

#[cfg(feature = "orm")]
use crate::db::DatabaseConfig;
#[cfg(feature = "orm")]
use sea_orm::DatabaseConnection;

// ═══════════════════════════════════════════════════════════════
// CoreStaging — Composants obligatoires du cœur de l'application
// ═══════════════════════════════════════════════════════════════
//
// Inspiré du système Prisme des formulaires :
// - Collecte flexible (le dev ajoute dans l'ordre qu'il veut)
// - Validation stricte (tout est vérifié avant construction)
// - Signal OK (is_ready)
//
// Le CoreStaging accepte deux chemins pour la DB :
//   1. .with_database(db)         → connexion déjà établie par le dev
//   2. .with_database_config(cfg) → le staging valide le driver et
//                                    connecte pendant le build
// ═══════════════════════════════════════════════════════════════

pub struct CoreStaging {
    /// Connexion DB déjà établie (chemin 1)
    #[cfg(feature = "orm")]
    pub(crate) db: Option<DatabaseConnection>,

    /// Configuration DB pour connexion différée (chemin 2)
    #[cfg(feature = "orm")]
    pub(crate) db_config: Option<DatabaseConfig>,

    pub(crate) url_registry: ARlockmap,
}

impl CoreStaging {
    /// Crée un nouveau CoreStaging avec les valeurs par défaut
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "orm")]
            db: None,
            #[cfg(feature = "orm")]
            db_config: None,
            url_registry: new_registry(),
        }
    }

    /// Enregistre une connexion à la base de données déjà établie
    ///
    /// Le dev gère la connexion lui-même :
    /// ```rust,ignore
    /// let db = DatabaseConfig::from_env()?.build().connect().await?;
    /// builder.with_database(db)
    /// ```
    #[cfg(feature = "orm")]
    pub fn set_database(&mut self, db: DatabaseConnection) {
        self.db = Some(db);
    }

    /// Enregistre une configuration DB — la connexion sera établie pendant le build.
    ///
    /// Le staging valide le driver et connecte automatiquement :
    /// ```rust,ignore
    /// let config = DatabaseConfig::from_env()?.build();
    /// builder.with_database_config(config)
    /// ```
    #[cfg(feature = "orm")]
    pub fn set_database_config(&mut self, config: DatabaseConfig) {
        self.db_config = Some(config);
    }

    /// Valide que tous les composants obligatoires sont présents.
    ///
    /// Retourne un `BuildError::CheckFailed` avec un rapport détaillé
    /// incluant des suggestions de correction pour chaque composant manquant.
    pub fn validate(&self) -> Result<(), BuildError> {
        let mut report = CheckReport::new();

        #[cfg(feature = "orm")]
        if self.db.is_none() && self.db_config.is_none() {
            report.add(
                CheckError::new(
                    "Database",
                    "Connexion ou configuration de base de données requise (feature 'orm' activée)",
                )
                .with_suggestion(
                    "Ajoutez .with_database(db) ou .with_database_config(config) à votre chaîne de construction",
                ),
            );
        }

        if report.has_errors() {
            return Err(BuildError::check(report));
        }

        Ok(())
    }

    /// Si un `DatabaseConfig` a été fourni (chemin 2), valide le driver
    /// et établit la connexion. Si une connexion directe a été fournie
    /// (chemin 1), la retourne telle quelle.
    ///
    /// Appelé pendant `build()` — après `validate()`.
    #[cfg(feature = "orm")]
    pub(crate) async fn connect(&mut self) -> Result<DatabaseConnection, BuildError> {
        // Chemin 1 : connexion déjà fournie par le dev
        if let Some(db) = self.db.take() {
            return Ok(db);
        }

        // Chemin 2 : connexion depuis DatabaseConfig
        if let Some(config) = self.db_config.take() {
            let db = config.connect().await.map_err(|e| {
                BuildError::validation(format!("Échec de connexion à la base de données : {}", e))
            })?;
            return Ok(db);
        }

        // Ne devrait jamais arriver si validate() a été appelé avant
        Err(BuildError::database_missing())
    }

    /// Vérifie si le core est prêt pour la construction
    pub fn is_ready(&self) -> bool {
        #[cfg(feature = "orm")]
        {
            if self.db.is_none() && self.db_config.is_none() {
                return false;
            }
        }
        true
    }
}

impl Default for CoreStaging {
    fn default() -> Self {
        Self::new()
    }
}
