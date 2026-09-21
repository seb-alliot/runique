//! `ContextHelper` — ergonomic wrapper around `tera::Context` with a chainable API.
use serde::Serialize;
use serde_json::Value;
use tera::Context;

/// Chainable wrapper around `tera::Context`, backing the `context!{}` macro.
/// Lets a template context be built as `ContextHelper::new().add(...).add(...)`
/// instead of mutating a `Context` in place; derefs to `Context` for anything
/// not covered by `.add()`/`.update()`.
pub struct ContextHelper {
    inner: Context,
}

impl Default for ContextHelper {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextHelper {
    /// Creates an empty context.
    pub fn new() -> Self {
        Self {
            inner: Context::new(),
        }
    }

    /// Inserts `value` under `key` and returns `self`, for chaining.
    pub fn add<T: Serialize>(mut self, key: &str, value: T) -> Self {
        // Tera 2 keys are `Cow<'static, str>`; owned here so the public signature
        // keeps accepting a borrowed `&str`.
        self.inner.insert(key.to_string(), &value);
        self
    }

    /// Merges the top-level keys of a JSON object into the context. Does
    /// nothing if `data` is not a JSON object.
    pub fn update(mut self, data: Value) -> Self {
        if let Some(obj) = data.as_object() {
            for (key, value) in obj {
                self.inner.insert(key.to_string(), value);
            }
        }
        self
    }
}

impl From<ContextHelper> for Context {
    fn from(helper: ContextHelper) -> Self {
        helper.inner
    }
}

impl std::ops::Deref for ContextHelper {
    type Target = Context;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for ContextHelper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
