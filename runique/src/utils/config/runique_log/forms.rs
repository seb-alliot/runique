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
    /// Creates a config with all form pipeline channels disabled.
    pub fn new() -> Self {
        Self::default()
    }
    /// Sets the level for field-level events (type resolution, coercion, missing/extra fields).
    #[must_use]
    pub fn field(mut self, level: Level) -> Self {
        self.field = Some(level);
        self
    }
    /// Sets the level for `set_value()` calls (raw input to typed value assignment).
    #[must_use]
    pub fn set_value(mut self, level: Level) -> Self {
        self.set_value = Some(level);
        self
    }
    /// Sets the level for `validate()` results (per-field errors, required/length/format checks).
    #[must_use]
    pub fn validate(mut self, level: Level) -> Self {
        self.validate = Some(level);
        self
    }
    /// Sets the level for HTML render events (widget selection, context injection).
    #[must_use]
    pub fn render(mut self, level: Level) -> Self {
        self.render = Some(level);
        self
    }
    /// Sets the level for `finalize()` per-field events (password hashing, file move to `MEDIA_ROOT`).
    #[must_use]
    pub fn finalize(mut self, level: Level) -> Self {
        self.finalize = Some(level);
        self
    }
    /// Enables every form pipeline channel at `Level::DEBUG`.
    pub fn dev(self) -> Self {
        self.field(Level::DEBUG)
            .set_value(Level::DEBUG)
            .validate(Level::DEBUG)
            .render(Level::DEBUG)
            .finalize(Level::DEBUG)
    }
}
