//! `ContextBuilder` — ergonomic wrapper around `tera::Context` with a chainable API.
use serde::Serialize;
use serde_json::Value;
use std::ops::{Deref, DerefMut};
use tera::Context;

/// Chainable wrapper around `tera::Context`, backing the `context!{}` macro.
/// Lets a template context be built as `ContextBuilder::new().add(...).add(...)`
/// instead of mutating a `Context` in place; derefs to `Context` for anything
/// not covered by `.add()`/`.update()`.
pub struct ContextBuilder {
    inner: Context,
}

impl Default for ContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextBuilder {
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

impl From<ContextBuilder> for Context {
    fn from(helper: ContextBuilder) -> Self {
        helper.inner
    }
}

impl Deref for ContextBuilder {
    type Target = Context;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for ContextBuilder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
