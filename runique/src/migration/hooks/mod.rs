//! SeaORM Hooks — before/after save/delete, ordered by execution slot.
//!
//! [`HooksDef`] is optional in [`crate::migration::ModelSchema`]:
//! absent = empty `impl ActiveModelBehavior for ActiveModel {}`.

/// SeaORM hook type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HookType {
    BeforeSave,
    AfterSave,
    BeforeDelete,
    AfterDelete,
}

/// A hook with its slot (execution order)
#[derive(Debug, Clone)]
pub struct Hook {
    pub hook_type: HookType,
    pub slot: u8,
    pub handler_path: String, // path to the handler function
}

/// Definition of hooks — optional, None = empty ActiveModelBehavior
#[derive(Debug, Clone, Default)]
pub struct HooksDef {
    pub hooks: Vec<Hook>,
    pub file_path: Option<String>, // src/hooks/article.rs
}

impl HooksDef {
    /// Creates an empty hook set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Points to an external file containing business logic
    pub fn from_file(path: impl Into<String>) -> Self {
        Self {
            hooks: Vec::new(),
            file_path: Some(path.into()),
        }
    }

    /// Registers a hook of `hook_type` at `slot`, pointing to `handler`. Hooks
    /// are kept sorted by slot after every insertion, so execution order is
    /// always consistent regardless of registration order.
    pub fn add(mut self, hook_type: HookType, slot: u8, handler: impl Into<String>) -> Self {
        self.hooks.push(Hook {
            hook_type,
            slot,
            handler_path: handler.into(),
        });
        // Sort by slot to guarantee execution order
        self.hooks.sort_by_key(|h| h.slot);
        self
    }

    /// Registers a `BeforeSave` hook at `slot`.
    pub fn before_save(self, slot: u8, handler: impl Into<String>) -> Self {
        self.add(HookType::BeforeSave, slot, handler)
    }

    /// Registers an `AfterSave` hook at `slot`.
    pub fn after_save(self, slot: u8, handler: impl Into<String>) -> Self {
        self.add(HookType::AfterSave, slot, handler)
    }

    /// Registers a `BeforeDelete` hook at `slot`.
    pub fn before_delete(self, slot: u8, handler: impl Into<String>) -> Self {
        self.add(HookType::BeforeDelete, slot, handler)
    }

    /// Registers an `AfterDelete` hook at `slot`.
    pub fn after_delete(self, slot: u8, handler: impl Into<String>) -> Self {
        self.add(HookType::AfterDelete, slot, handler)
    }
}
