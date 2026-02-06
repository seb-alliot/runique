use std::fmt;

/// Erreur principale de la phase de build
#[derive(Debug, Clone)]
pub struct BuildError {
    pub kind: BuildErrorKind,
    pub context: Option<String>,
}

/// Types d'erreurs possibles pendant le build
#[derive(Debug, Clone)]
pub enum BuildErrorKind {
    /// Validation structurelle échouée (composant manquant, config invalide)
    ValidationFailed(String),

    /// Les health checks ont détecté des problèmes
    CheckFailed(CheckReport),

    /// Erreur lors du chargement des templates
    TemplateLoadFailed(String),

    /// Base de données requise mais absente (feature orm activée)
    DatabaseMissing,

    /// Un composant n'est pas prêt
    ComponentNotReady(String),
}

/// Rapport complet des health checks
#[derive(Debug, Clone)]
pub struct CheckReport {
    pub errors: Vec<CheckError>,
}

/// Erreur individuelle d'un health check
#[derive(Debug, Clone)]
pub struct CheckError {
    /// Nom du composant testé (ex: "Database", "Templates", "Session")
    pub component: String,

    /// Description du problème
    pub message: String,

    /// Suggestion pour corriger le problème
    pub suggestion: Option<String>,
}

// ═══════════════════════════════════════════════════════════════
// IMPLÉMENTATIONS
// ═══════════════════════════════════════════════════════════════

impl BuildError {
    /// Crée une erreur de validation
    pub fn validation(msg: impl Into<String>) -> Self {
        Self {
            kind: BuildErrorKind::ValidationFailed(msg.into()),
            context: None,
        }
    }

    /// Crée une erreur de health check
    pub fn check(report: CheckReport) -> Self {
        Self {
            kind: BuildErrorKind::CheckFailed(report),
            context: None,
        }
    }

    /// Crée une erreur de template
    pub fn template(err: impl Into<String>) -> Self {
        Self {
            kind: BuildErrorKind::TemplateLoadFailed(err.into()),
            context: None,
        }
    }

    /// Crée une erreur de DB manquante
    pub fn database_missing() -> Self {
        Self {
            kind: BuildErrorKind::DatabaseMissing,
            context: None,
        }
    }

    /// Ajoute du contexte à l'erreur
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }
}

impl CheckReport {
    /// Crée un rapport vide
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    /// Ajoute une erreur au rapport
    pub fn add(&mut self, error: CheckError) {
        self.errors.push(error);
    }

    /// Vérifie si le rapport contient des erreurs
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Nombre d'erreurs dans le rapport
    pub fn count(&self) -> usize {
        self.errors.len()
    }
}

impl Default for CheckReport {
    fn default() -> Self {
        Self::new()
    }
}

impl CheckError {
    /// Crée une nouvelle erreur de health check
    pub fn new(component: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            component: component.into(),
            message: message.into(),
            suggestion: None,
        }
    }

    /// Ajoute une suggestion pour corriger l'erreur
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

// ═══════════════════════════════════════════════════════════════
// DISPLAY TRAITS (pour l'affichage dans le terminal)
// ═══════════════════════════════════════════════════════════════

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            BuildErrorKind::ValidationFailed(msg) => {
                write!(f, "Build validation failed: {}", msg)?;
            }
            BuildErrorKind::CheckFailed(report) => {
                writeln!(f, "Build failed with {} check error(s):", report.count())?;
                writeln!(f)?;
                for (i, err) in report.errors.iter().enumerate() {
                    write!(f, "  {}. {} - {}", i + 1, err.component, err.message)?;
                    if let Some(suggestion) = &err.suggestion {
                        write!(f, "\n     → {}", suggestion)?;
                    }
                    if i < report.errors.len() - 1 {
                        writeln!(f)?;
                    }
                }
            }
            BuildErrorKind::TemplateLoadFailed(msg) => {
                write!(f, "Template loading failed: {}", msg)?;
            }
            BuildErrorKind::DatabaseMissing => {
                write!(
                    f,
                    "Database connection required when 'orm' feature is enabled"
                )?;
            }
            BuildErrorKind::ComponentNotReady(component) => {
                write!(f, "Component '{}' is not ready", component)?;
            }
        }

        if let Some(ctx) = &self.context {
            write!(f, "\nContext: {}", ctx)?;
        }

        Ok(())
    }
}

impl fmt::Display for CheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.component, self.message)?;
        if let Some(suggestion) = &self.suggestion {
            write!(f, " (Suggestion: {})", suggestion)?;
        }
        Ok(())
    }
}

impl std::error::Error for BuildError {}
impl std::error::Error for CheckError {}

// ═══════════════════════════════════════════════════════════════
// CONVERSIONS
// ═══════════════════════════════════════════════════════════════

impl From<tera::Error> for BuildError {
    fn from(err: tera::Error) -> Self {
        Self::template(err.to_string())
    }
}
