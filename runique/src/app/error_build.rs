//! Application construction phase errors (build-time).
use std::fmt;

/// Main build phase error
#[derive(Debug, Clone)]
pub struct BuildError {
    pub kind: BuildErrorKind,
    pub context: Option<String>,
}

/// Possible error types during build
#[derive(Debug, Clone)]
pub enum BuildErrorKind {
    /// Structural validation failed (missing component, invalid config)
    ValidationFailed(String),

    /// Health checks detected problems
    CheckFailed(CheckReport),

    /// Error during template loading
    TemplateLoadFailed(String),

    /// Database required but absent (`orm` feature enabled)
    DatabaseMissing,

    /// A component is not ready
    ComponentNotReady(String),
}

/// Complete health checks report
#[derive(Debug, Clone)]
pub struct CheckReport {
    pub errors: Vec<CheckError>,
}

/// Individual health check error
#[derive(Debug, Clone)]
pub struct CheckError {
    /// Name of the tested component (e.g., "Database", "Templates", "Session")
    pub component: String,

    /// Description of the problem
    pub message: String,

    /// Suggestion to fix the problem
    pub suggestion: Option<String>,
}

// ═══════════════════════════════════════════════════════════════
// IMPLEMENTATIONS
// ═══════════════════════════════════════════════════════════════

impl BuildError {
    /// Creates a validation error
    pub fn validation(msg: impl Into<String>) -> Self {
        Self {
            kind: BuildErrorKind::ValidationFailed(msg.into()),
            context: None,
        }
    }

    /// Creates a health check error
    pub fn check(report: CheckReport) -> Self {
        Self {
            kind: BuildErrorKind::CheckFailed(report),
            context: None,
        }
    }

    /// Creates a template error
    pub fn template(err: impl Into<String>) -> Self {
        Self {
            kind: BuildErrorKind::TemplateLoadFailed(err.into()),
            context: None,
        }
    }

    /// Creates a missing DB error
    pub fn database_missing() -> Self {
        Self {
            kind: BuildErrorKind::DatabaseMissing,
            context: None,
        }
    }

    /// Adds context to the error
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }
}

impl CheckReport {
    /// Creates an empty report
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    /// Adds an error to the report
    pub fn add(&mut self, error: CheckError) {
        self.errors.push(error);
    }

    /// Checks if the report contains errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Number of errors in the report
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
    /// Creates a new health check error
    pub fn new(component: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            component: component.into(),
            message: message.into(),
            suggestion: None,
        }
    }

    /// Adds a suggestion to fix the error
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

// ═══════════════════════════════════════════════════════════════
// DISPLAY TRAITS (for terminal display)
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
                    write!(
                        f,
                        "  {}. {} - {}",
                        i.saturating_add(1),
                        err.component,
                        err.message
                    )?;
                    if let Some(suggestion) = &err.suggestion {
                        write!(f, "\n     → {}", suggestion)?;
                    }
                    if i < report.errors.len().saturating_sub(1) {
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
