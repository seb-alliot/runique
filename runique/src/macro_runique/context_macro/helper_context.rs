use serde::Serialize;
use serde_json::Value;
use tera::Context;

pub struct ContextHelper {
    inner: Context,
}

impl Default for ContextHelper {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextHelper {
    pub fn new() -> Self {
        Self {
            inner: Context::new(),
        }
    }

    pub fn add<T: Serialize>(mut self, key: &str, value: T) -> Self {
        self.inner.insert(key, &value);
        self
    }

    pub fn update(mut self, data: Value) -> Self {
        if let Some(obj) = data.as_object() {
            for (key, value) in obj {
                self.inner.insert(key, value);
            }
        }
        self
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
