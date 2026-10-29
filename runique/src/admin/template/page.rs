//! The page each admin view renders: Runique's, or the developer's override.

/// Admin template path — with fallback to Runique default.
///
/// `runique` : Tera key of the framework template (immutable)
/// `dev`     : optional dev path — replaces `runique` if defined
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathAdminTemplate {
    pub dev: Option<String>,
    pub runique: &'static str,
}

impl PathAdminTemplate {
    /// Returns the resolved path: dev has priority, otherwise Runique default.
    #[must_use]
    pub fn resolve(&self) -> &str {
        self.dev.as_deref().unwrap_or(self.runique)
    }
    /// Default template for the admin dashboard page (`admin/dashboard.html`), no dev override.
    pub fn dashboard() -> Self {
        Self {
            dev: None,
            runique: "admin/dashboard.html",
        }
    }
    /// Default template for the admin login page (`admin/login.html`), no dev override.
    pub fn login() -> Self {
        Self {
            dev: None,
            runique: "admin/login.html",
        }
    }
    /// Default template for the admin list view (`admin/list.html`), no dev override.
    pub fn list() -> Self {
        Self {
            dev: None,
            runique: "admin/list.html",
        }
    }
    /// Default template for the admin create view (`admin/create.html`), no dev override.
    pub fn create() -> Self {
        Self {
            dev: None,
            runique: "admin/create.html",
        }
    }
    /// Default template for the admin edit view (`admin/edit.html`), no dev override.
    pub fn edit() -> Self {
        Self {
            dev: None,
            runique: "admin/edit.html",
        }
    }
    /// Default template for the admin detail view (`admin/detail.html`), no dev override.
    pub fn detail() -> Self {
        Self {
            dev: None,
            runique: "admin/detail.html",
        }
    }
    /// Default template for the admin delete confirmation view (`admin/delete.html`), no dev override.
    pub fn delete() -> Self {
        Self {
            dev: None,
            runique: "admin/delete.html",
        }
    }
    /// Default base layout template (`admin_base.html`) that other admin templates extend.
    pub fn base() -> Self {
        Self {
            dev: None,
            runique: "admin_base.html",
        }
    }
    /// Default template for HTMX partial list responses (fragment only, no layout).
    pub fn htmx() -> Self {
        Self {
            dev: None,
            runique: "admin/list_partial.html",
        }
    }
    /// Default template for the admin bulk edit view (`admin/bulk_edit.html`), no dev override.
    pub fn bulk_edit() -> Self {
        Self {
            dev: None,
            runique: "admin/bulk_edit.html",
        }
    }
    /// Default template for the admin history (`admin/history.html`), no dev override.
    pub fn history() -> Self {
        Self {
            dev: None,
            runique: "admin/history.html",
        }
    }
    /// Default template for one history entry (`admin/history_diff.html`), no dev override.
    pub fn history_diff() -> Self {
        Self {
            dev: None,
            runique: "admin/history_diff.html",
        }
    }
}

/// Global configuration for admin templates.
///
/// Resolution (decreasing priority):
/// 1. `AdminTemplate.list.dev`     — the developer's override
/// 2. `AdminTemplate.list.runique` — Runique's page
///
/// ## Example
/// ```rust,ignore
/// .with_admin(|a| a
///     .templates(|t| t
///         .with_list("my_theme/list")
///         .with_dashboard("my_theme/dashboard")
///     )
/// )
/// ```
#[derive(Debug, Clone)]
pub struct AdminTemplate {
    pub dashboard: PathAdminTemplate,
    pub list: PathAdminTemplate,
    pub create: PathAdminTemplate,
    pub edit: PathAdminTemplate,
    pub detail: PathAdminTemplate,
    pub delete: PathAdminTemplate,
    pub login: PathAdminTemplate,
    pub base: PathAdminTemplate,
    /// Template for HTMX partial responses (fragment only, no layout).
    pub htmx: PathAdminTemplate,
    pub bulk_edit: PathAdminTemplate,
    pub history: PathAdminTemplate,
    pub history_diff: PathAdminTemplate,
    /// The model pages' blocks replaced for every model.
    pub blocks: super::block::ModelBlocks,
    /// The layout's blocks, and those of pages about no model, replaced.
    pub global_blocks: super::block::Overrides<super::block::GlobalBlock>,
    /// The sheet of a page the developer replaced: loaded alone, after the
    /// variables. Without one, his page gets Runique's sheets.
    pub page_stylesheets: std::collections::HashMap<super::stylesheet::Page, String>,
}

impl AdminTemplate {
    /// The sheet of the page `page` replaces, under the developer's static:
    /// it then stands for all of Runique's on that page but the variables.
    /// Only read once the page itself is replaced.
    #[must_use]
    pub fn page_stylesheet(mut self, page: super::stylesheet::Page, path: &str) -> Self {
        self.page_stylesheets.insert(page, path.to_string());
        self
    }

    /// The template a page renders, for the pages a developer may replace.
    pub(crate) fn page_template(
        &self,
        page: super::stylesheet::Page,
    ) -> Option<&PathAdminTemplate> {
        use super::stylesheet::Page;
        Some(match page {
            Page::Login => &self.login,
            Page::Dashboard => &self.dashboard,
            Page::List => &self.list,
            Page::Create => &self.create,
            Page::Edit => &self.edit,
            Page::Detail => &self.detail,
            Page::Delete => &self.delete,
            Page::History => &self.history,
            Page::HistoryDiff => &self.history_diff,
            Page::Rights | Page::Transfer => return None,
        })
    }

    /// Replaces a model page's block, for every model, with the template `path`.
    #[must_use]
    pub fn block(mut self, block: super::block::Block, path: &str) -> Self {
        self.blocks = self.blocks.block(block, path);
        self
    }

    /// Replaces a model page's block's stylesheet, for every model, with
    /// `path` under the developer's static.
    #[must_use]
    pub fn block_stylesheet(mut self, block: super::block::Block, path: &str) -> Self {
        self.blocks = self.blocks.block_stylesheet(block, path);
        self
    }

    /// Replaces a layout block, or one of a page about no model.
    #[must_use]
    pub fn global_block(mut self, block: super::block::GlobalBlock, path: &str) -> Self {
        self.global_blocks = self.global_blocks.block(block, path);
        self
    }

    /// Replaces a layout block's stylesheet, or one of a page about no model.
    #[must_use]
    pub fn global_block_stylesheet(mut self, block: super::block::GlobalBlock, path: &str) -> Self {
        self.global_blocks = self.global_blocks.block_stylesheet(block, path);
        self
    }

    /// Builds the default `AdminTemplate` configuration: every page points at
    /// its framework template with no dev override.
    pub fn new() -> Self {
        Self {
            dashboard: PathAdminTemplate::dashboard(),
            list: PathAdminTemplate::list(),
            create: PathAdminTemplate::create(),
            edit: PathAdminTemplate::edit(),
            detail: PathAdminTemplate::detail(),
            delete: PathAdminTemplate::delete(),
            login: PathAdminTemplate::login(),
            base: PathAdminTemplate::base(),
            htmx: PathAdminTemplate::htmx(),
            bulk_edit: PathAdminTemplate::bulk_edit(),
            history: PathAdminTemplate::history(),
            history_diff: PathAdminTemplate::history_diff(),
            blocks: Default::default(),
            global_blocks: Default::default(),
            page_stylesheets: Default::default(),
        }
    }
    /// Overrides the dashboard template with a developer-provided path.
    #[must_use]
    pub fn with_dashboard(mut self, path: &str) -> Self {
        self.dashboard.dev = Some(path.to_string());
        self
    }
    /// Overrides the list view template with a developer-provided path.
    #[must_use]
    pub fn with_list(mut self, path: &str) -> Self {
        self.list.dev = Some(path.to_string());
        self
    }
    /// Overrides the create view template with a developer-provided path.
    #[must_use]
    pub fn with_create(mut self, path: &str) -> Self {
        self.create.dev = Some(path.to_string());
        self
    }
    /// Overrides the edit view template with a developer-provided path.
    #[must_use]
    pub fn with_edit(mut self, path: &str) -> Self {
        self.edit.dev = Some(path.to_string());
        self
    }
    /// Overrides the detail view template with a developer-provided path.
    #[must_use]
    pub fn with_detail(mut self, path: &str) -> Self {
        self.detail.dev = Some(path.to_string());
        self
    }
    /// Overrides the delete confirmation template with a developer-provided path.
    #[must_use]
    pub fn with_delete(mut self, path: &str) -> Self {
        self.delete.dev = Some(path.to_string());
        self
    }
    /// Overrides the login page template with a developer-provided path.
    #[must_use]
    pub fn with_login(mut self, path: &str) -> Self {
        self.login.dev = Some(path.to_string());
        self
    }
    /// Overrides the base layout template with a developer-provided path.
    #[must_use]
    pub fn with_base(mut self, path: &str) -> Self {
        self.base.dev = Some(path.to_string());
        self
    }
    /// Overrides the HTMX partial-list template with a developer-provided path.
    #[must_use]
    pub fn with_htmx(mut self, path: &str) -> Self {
        self.htmx.dev = Some(path.to_string());
        self
    }
    /// Overrides the bulk edit view template with a developer-provided path.
    #[must_use]
    pub fn with_bulk_edit(mut self, path: &str) -> Self {
        self.bulk_edit.dev = Some(path.to_string());
        self
    }
    /// Overrides the history template with a developer-provided path.
    #[must_use]
    pub fn with_history(mut self, path: &str) -> Self {
        self.history.dev = Some(path.to_string());
        self
    }
    /// Overrides one history entry's template with a developer-provided path.
    #[must_use]
    pub fn with_history_diff(mut self, path: &str) -> Self {
        self.history_diff.dev = Some(path.to_string());
        self
    }
}

impl Default for AdminTemplate {
    fn default() -> Self {
        Self::new()
    }
}

/// A model's replacements: blocks of its pages, or whole pages, over the
/// global ones.
#[derive(Clone, Debug, Default)]
pub struct ModelTemplates {
    pub(crate) blocks: super::block::ModelBlocks,
    pub(crate) pages: std::collections::HashMap<super::stylesheet::Page, String>,
    pub(crate) page_stylesheets: std::collections::HashMap<super::stylesheet::Page, String>,
}

impl ModelTemplates {
    /// Replaces a block of this model's pages with the template `path`.
    #[must_use]
    pub fn block(mut self, block: super::block::Block, path: &str) -> Self {
        self.blocks = self.blocks.block(block, path);
        self
    }

    /// Replaces a block's stylesheet on this model's pages.
    #[must_use]
    pub fn block_stylesheet(mut self, block: super::block::Block, path: &str) -> Self {
        self.blocks = self.blocks.block_stylesheet(block, path);
        self
    }

    /// Replaces one of this model's pages (list, create, edit, detail,
    /// delete) with the template `path`. Another page refuses the boot.
    #[must_use]
    pub fn page(mut self, page: super::stylesheet::Page, path: &str) -> Self {
        self.pages.insert(page, path.to_string());
        self
    }

    /// The sheet of this model's replaced page, standing for all of
    /// Runique's but the variables.
    #[must_use]
    pub fn page_stylesheet(mut self, page: super::stylesheet::Page, path: &str) -> Self {
        self.page_stylesheets.insert(page, path.to_string());
        self
    }
}

impl AdminTemplate {
    /// The template `page` renders for a model with `model`'s replacements:
    /// its own page, else the global one, else Runique's.
    pub(crate) fn page_for<'a>(
        &'a self,
        page: super::stylesheet::Page,
        model: Option<&'a ModelTemplates>,
    ) -> &'a str {
        if let Some(own) = model.and_then(|m| m.pages.get(&page)) {
            return own;
        }
        self.page_template(page)
            .map_or("", PathAdminTemplate::resolve)
    }
}
