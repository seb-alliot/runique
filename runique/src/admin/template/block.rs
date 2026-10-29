//! The blocks a page is made of, each with its HTML and, when it has one, its
//! stylesheet. The developer replaces either: a `Block` globally or for one
//! model, a `GlobalBlock` globally only (layout, and pages about no model).
use super::page::PathAdminTemplate;
use std::collections::HashMap;

/// A block of a model's page: replaceable globally or for one model.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Block {
    ListTable,
    ListFilters,
    ListSearch,
    ListPagination,
    ListBulk,
    /// The row's menu, on the list and the detail.
    Kebab,
    DetailFields,
    DetailActions,
    DetailGroups,
    DetailTransfer,
    FormFields,
    FormActions,
    DeleteWarning,
    DeleteActions,
}

/// A block of the layout, or of a page about no model: replaceable globally
/// only, a model has nothing to choose there.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GlobalBlock {
    Sidebar,
    Topbar,
    DashboardGroups,
    HistoryTable,
    HistoryFilters,
    HistoryActionBadge,
    RightsTable,
}

impl Block {
    pub const ALL: [Block; 14] = [
        Block::ListTable,
        Block::ListFilters,
        Block::ListSearch,
        Block::ListPagination,
        Block::ListBulk,
        Block::Kebab,
        Block::DetailFields,
        Block::DetailActions,
        Block::DetailGroups,
        Block::DetailTransfer,
        Block::FormFields,
        Block::FormActions,
        Block::DeleteWarning,
        Block::DeleteActions,
    ];

    /// The block's name, as `{% dev_block "…" %}` gives it.
    pub fn name(self) -> &'static str {
        match self {
            Block::ListTable => "list_table",
            Block::ListFilters => "list_filters",
            Block::ListSearch => "list_search",
            Block::ListPagination => "list_pagination",
            Block::ListBulk => "list_bulk",
            Block::Kebab => "kebab",
            Block::DetailFields => "detail_fields",
            Block::DetailActions => "detail_actions",
            Block::DetailGroups => "detail_groups",
            Block::DetailTransfer => "detail_transfer",
            Block::FormFields => "form_fields",
            Block::FormActions => "form_actions",
            Block::DeleteWarning => "delete_warning",
            Block::DeleteActions => "delete_actions",
        }
    }

    /// Runique's HTML for the block, as Tera knows it.
    pub fn runique(self) -> &'static str {
        runique_of(self.name())
    }

    /// Runique's stylesheet for the block, under Runique's static.
    pub fn runique_stylesheet(self) -> Option<&'static str> {
        match self {
            Block::DetailGroups => Some("css/admin/blocks/rights.css"),
            _ => None,
        }
    }
}

impl GlobalBlock {
    pub const ALL: [GlobalBlock; 7] = [
        GlobalBlock::Sidebar,
        GlobalBlock::Topbar,
        GlobalBlock::DashboardGroups,
        GlobalBlock::HistoryTable,
        GlobalBlock::HistoryFilters,
        GlobalBlock::HistoryActionBadge,
        GlobalBlock::RightsTable,
    ];

    /// The block's name, as `{% dev_block "…" %}` gives it.
    pub fn name(self) -> &'static str {
        match self {
            GlobalBlock::Sidebar => "sidebar",
            GlobalBlock::Topbar => "topbar",
            GlobalBlock::DashboardGroups => "dashboard_groups",
            GlobalBlock::HistoryTable => "history_table",
            GlobalBlock::HistoryFilters => "history_filters",
            GlobalBlock::HistoryActionBadge => "history_action_badge",
            GlobalBlock::RightsTable => "rights_table",
        }
    }

    pub fn runique(self) -> &'static str {
        runique_of(self.name())
    }

    pub fn runique_stylesheet(self) -> Option<&'static str> {
        None
    }
}

/// Runique's template for block `name`, from the one table of them.
fn runique_of(name: &str) -> &'static str {
    // Every variant's name is in the table: `every_block_names_a_template_runique_ships`.
    crate::utils::constante::template::admin_block(name).unwrap_or_default()
}

/// Where a block's stylesheet comes from: the tag that loads it differs,
/// each giving the file's integrity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Stylesheet {
    /// Under Runique's static: `{% runique_static %}`.
    Runique(&'static str),
    /// Under the developer's static: `{% static %}`.
    Dev(String),
}

/// A block as it is served: its HTML, and its stylesheet if any.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockTemplate {
    pub html: PathAdminTemplate,
    pub stylesheet: Option<Stylesheet>,
}

impl BlockTemplate {
    /// The block with the first override found in `layers`, highest first,
    /// for its HTML and for its stylesheet each.
    fn resolve<K: Copy + Eq + std::hash::Hash>(
        key: K,
        runique: &'static str,
        runique_stylesheet: Option<&'static str>,
        layers: &[&Overrides<K>],
    ) -> Self {
        let html = layers.iter().find_map(|l| l.html.get(&key).cloned());
        let stylesheet = layers
            .iter()
            .find_map(|l| l.stylesheet.get(&key).cloned())
            .map(Stylesheet::Dev)
            .or(runique_stylesheet.map(Stylesheet::Runique));
        Self {
            html: PathAdminTemplate { dev: html, runique },
            stylesheet,
        }
    }
}

/// The developer's replacements of blocks: the HTML of some, the stylesheet
/// of some, each by its own path (a template, a file of his static).
#[derive(Clone, Debug)]
pub struct Overrides<K> {
    html: HashMap<K, String>,
    stylesheet: HashMap<K, String>,
}

impl<K> Default for Overrides<K> {
    fn default() -> Self {
        Self {
            html: HashMap::new(),
            stylesheet: HashMap::new(),
        }
    }
}

impl<K: Copy + Eq + std::hash::Hash> Overrides<K> {
    /// Replaces the block's HTML with the template `path`.
    #[must_use]
    pub fn block(mut self, block: K, path: impl Into<String>) -> Self {
        self.html.insert(block, path.into());
        self
    }

    /// Replaces the block's stylesheet with `path`, under the developer's
    /// static. Its HTML stays Runique's unless replaced too.
    #[must_use]
    pub fn block_stylesheet(mut self, block: K, path: impl Into<String>) -> Self {
        self.stylesheet.insert(block, path.into());
        self
    }

    /// The template replacing the block's HTML, when one does.
    pub(crate) fn html_of(&self, block: K) -> Option<&str> {
        self.html.get(&block).map(String::as_str)
    }
}

/// A model's replacements, over the global ones.
pub type ModelBlocks = Overrides<Block>;

impl Block {
    /// The block as served for a model with `model`'s replacements, over the
    /// global `global` ones, over Runique's.
    pub(crate) fn resolve(
        self,
        model: Option<&ModelBlocks>,
        global: &ModelBlocks,
    ) -> BlockTemplate {
        let layers: Vec<&ModelBlocks> = model.into_iter().chain([global]).collect();
        BlockTemplate::resolve(self, self.runique(), self.runique_stylesheet(), &layers)
    }
}

impl GlobalBlock {
    pub(crate) fn resolve(self, global: &Overrides<GlobalBlock>) -> BlockTemplate {
        BlockTemplate::resolve(self, self.runique(), self.runique_stylesheet(), &[global])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_block_without_override_is_runiques() {
        let served = Block::DetailGroups.resolve(None, &ModelBlocks::default());
        assert_eq!(served.html.resolve(), "admin/blocks/detail/groups.html");
        assert_eq!(
            served.stylesheet,
            Some(Stylesheet::Runique("css/admin/blocks/rights.css"))
        );
        assert_eq!(
            Block::ListTable
                .resolve(None, &ModelBlocks::default())
                .stylesheet,
            None
        );
    }

    #[test]
    fn the_model_wins_over_the_global_which_wins_over_runique() {
        let global = ModelBlocks::default()
            .block(Block::ListTable, "theme/table.html")
            .block_stylesheet(Block::ListTable, "css/theme.css");
        let model = ModelBlocks::default().block(Block::ListTable, "notes/table.html");

        let served = Block::ListTable.resolve(Some(&model), &global);
        assert_eq!(served.html.resolve(), "notes/table.html");
        assert_eq!(
            served.stylesheet,
            Some(Stylesheet::Dev("css/theme.css".into())),
            "per part"
        );

        let other = Block::ListTable.resolve(None, &global);
        assert_eq!(other.html.resolve(), "theme/table.html");
    }

    #[test]
    fn a_stylesheet_alone_keeps_runiques_html() {
        let global = ModelBlocks::default().block_stylesheet(Block::DetailGroups, "css/mine.css");
        let served = Block::DetailGroups.resolve(None, &global);
        assert_eq!(served.html.resolve(), "admin/blocks/detail/groups.html");
        assert_eq!(
            served.stylesheet,
            Some(Stylesheet::Dev("css/mine.css".into()))
        );
    }

    #[test]
    fn every_block_names_a_template_runique_ships() {
        let shipped: Vec<&str> = crate::utils::constante::template::ADMIN_TEMPLATES
            .iter()
            .map(|(name, _)| *name)
            .collect();
        let names = Block::ALL
            .iter()
            .map(|b| b.runique())
            .chain(GlobalBlock::ALL.iter().map(|b| b.runique()));
        for name in names {
            assert!(shipped.contains(&name), "{name:?}");
        }
        let named = Block::ALL.len() + GlobalBlock::ALL.len();
        assert_eq!(
            named,
            crate::utils::constante::template::ADMIN_BLOCKS.len(),
            "one name each"
        );
    }
}
