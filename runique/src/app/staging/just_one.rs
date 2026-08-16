//! `JustOne<T>` — a builder slot that accepts a single value.

/// A configuration slot that may be filled only once per builder instance.
///
/// Behaves like an `Option<T>`, but remembers how many times a caller tried to
/// fill it. Without it, a second `.with_admin(...)` or `.routes(...)` silently
/// overwrote the first one: the app booted half-configured with no diagnostic.
///
/// The count lives in the instance, never in a `static` — several builders may
/// legitimately exist in the same process (integration tests run in parallel),
/// and a process-wide latch would make the second one fail for nothing.
pub(crate) struct JustOne<T> {
    value: Option<T>,

    /// Saturating — only the 0 / 1 / 2+ distinction is ever read.
    calls: u8,

    /// Public method this slot backs (e.g. "with_admin") — quoted verbatim in
    /// the error, so the dev reads the name of what they actually wrote.
    label: &'static str,
}

impl<T> JustOne<T> {
    pub(crate) const fn new(label: &'static str) -> Self {
        Self {
            value: None,
            calls: 0,
            label,
        }
    }

    /// Fills the slot. Extra calls keep the FIRST value: `build()` fails either
    /// way, and keeping the first one makes the failure independent of whatever
    /// was written afterwards.
    pub(crate) fn claim(&mut self, value: T) {
        self.calls = self.calls.saturating_add(1);
        if self.value.is_none() {
            self.value = Some(value);
        }
    }

    /// `Some(label)` once the slot has been claimed more than once.
    pub(crate) fn duplicate(&self) -> Option<&'static str> {
        (self.calls > 1).then_some(self.label)
    }

    pub(crate) fn take(self) -> Option<T> {
        self.value
    }
}
