//! Form pipeline tracing — each field covers one stage of `#[form]` processing.
use tracing::Level;

/// Form pipeline tracing — each field covers one stage of `#[form]` processing.
#[derive(Debug, Clone, Default)]
pub struct FormTracing {
    /// Field-level events: type resolution, coercion, missing/extra fields.
    pub field: Option<Level>,
    /// `set_value()` calls: raw input → typed value assignment.
    pub set_value: Option<Level>,
    /// `validate()` results: per-field errors, required/length/format checks.
    pub validate: Option<Level>,
    /// HTML render events: widget selection, context injection.
    pub render: Option<Level>,
    /// `finalize()` per field: password hashing, file move to MEDIA_ROOT.
    pub finalize: Option<Level>,
}

impl FormTracing {
    pub fn new() -> Self {
        Self::default()
    }
    #[must_use]
    pub fn field(mut self, level: Level) -> Self {
        self.field = Some(level);
        self
    }
    #[must_use]
    pub fn set_value(mut self, level: Level) -> Self {
        self.set_value = Some(level);
        self
    }
    #[must_use]
    pub fn validate(mut self, level: Level) -> Self {
        self.validate = Some(level);
        self
    }
    #[must_use]
    pub fn render(mut self, level: Level) -> Self {
        self.render = Some(level);
        self
    }
    #[must_use]
    pub fn finalize(mut self, level: Level) -> Self {
        self.finalize = Some(level);
        self
    }
    pub fn dev(self) -> Self {
        self.field(Level::DEBUG)
            .set_value(Level::DEBUG)
            .validate(Level::DEBUG)
            .render(Level::DEBUG)
            .finalize(Level::DEBUG)
    }
}
