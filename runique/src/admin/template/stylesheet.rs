//! The stylesheets each admin page loads, all in its `<head>`. Whatever the
//! developer doesn't replace stays Runique's: a block's sheet replaced takes
//! that block's place; a page replaced with its own sheet loads it alone,
//! after the variables its included blocks still read.
use super::block::{Block, GlobalBlock, Stylesheet};
use super::page::{AdminTemplate, ModelTemplates};
use crate::utils::aliases::StrMap;
use serde::Serialize;
use std::collections::HashMap;

/// The variables every other sheet reads, and nothing else (`:root` and
/// the light theme): kept first, under any replacement.
const VARIABLES: &str = "css/admin/variables.css";

/// The layout's sheets, in their order.
const LAYOUT: [&str; 6] = [
    "css/normalize/normalize.css",
    VARIABLES,
    "css/admin/layout.css",
    "css/admin/components.css",
    "css/admin/buttons.css",
    "css/admin/forms.css",
];

/// The login's, outside the layout.
const LOGIN: [&str; 6] = [
    VARIABLES,
    "css/admin/layout.css",
    "css/admin/components.css",
    "css/admin/buttons.css",
    "css/admin/forms.css",
    "css/admin/login.css",
];

/// An admin page, as its stylesheets know it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Page {
    Login,
    Dashboard,
    List,
    Create,
    Edit,
    Detail,
    Delete,
    History,
    HistoryDiff,
    Rights,
    Transfer,
}

impl Page {
    pub const ALL: [Page; 11] = [
        Page::Login,
        Page::Dashboard,
        Page::List,
        Page::Create,
        Page::Edit,
        Page::Detail,
        Page::Delete,
        Page::History,
        Page::HistoryDiff,
        Page::Rights,
        Page::Transfer,
    ];

    /// Whether its pages are about one model: its blocks may then be
    /// replaced for that model.
    pub(crate) fn per_model(self) -> bool {
        matches!(
            self,
            Page::List | Page::Create | Page::Edit | Page::Detail | Page::Delete
        )
    }

    /// The model blocks it shows.
    fn blocks(self) -> &'static [Block] {
        match self {
            Page::List => &[
                Block::ListTable,
                Block::ListFilters,
                Block::ListSearch,
                Block::ListPagination,
                Block::ListBulk,
                Block::Kebab,
            ],
            Page::Create | Page::Edit => &[Block::FormFields, Block::FormActions],
            Page::Detail => &[
                Block::DetailFields,
                Block::DetailActions,
                Block::DetailGroups,
                Block::DetailTransfer,
                Block::Kebab,
            ],
            Page::Delete => &[Block::DeleteWarning, Block::DeleteActions],
            _ => &[],
        }
    }

    /// The global blocks it shows, the layout's included.
    fn global_blocks(self) -> &'static [GlobalBlock] {
        const LAYOUT_BLOCKS: [GlobalBlock; 2] = [GlobalBlock::Sidebar, GlobalBlock::Topbar];
        match self {
            Page::Login => &[],
            Page::Dashboard => &[
                GlobalBlock::Sidebar,
                GlobalBlock::Topbar,
                GlobalBlock::DashboardTable,
                GlobalBlock::DashboardGroups,
            ],
            Page::History => &[
                GlobalBlock::Sidebar,
                GlobalBlock::Topbar,
                GlobalBlock::HistoryTable,
                GlobalBlock::HistoryFilters,
                GlobalBlock::HistoryActionBadge,
            ],
            Page::HistoryDiff => &[
                GlobalBlock::Sidebar,
                GlobalBlock::Topbar,
                GlobalBlock::HistoryActionBadge,
            ],
            Page::Rights => &[
                GlobalBlock::Sidebar,
                GlobalBlock::Topbar,
                GlobalBlock::RightsTable,
            ],
            _ => &LAYOUT_BLOCKS,
        }
    }

    /// Runique's sheets for the page itself, before its blocks': the
    /// layout's (the login's own), then the page's.
    fn runique_sheets(self) -> impl Iterator<Item = &'static str> {
        let base: &[&str] = if self == Page::Login { &LOGIN } else { &LAYOUT };
        let own = match self {
            Page::Rights => Some("css/admin/blocks/rights.css"),
            Page::Transfer => Some("css/admin/blocks/transfer.css"),
            _ => None,
        };
        base.iter().copied().chain(own)
    }
}

/// The sheets `page` loads, in order, with `model`'s replacements over the
/// global ones. A page the developer replaced with a sheet of its own loads
/// the variables, then that sheet alone.
pub(crate) fn sheets(
    page: Page,
    templates: &AdminTemplate,
    model: Option<&ModelTemplates>,
) -> Vec<Stylesheet> {
    let model = model.filter(|_| page.per_model());
    // The page the model replaced takes its own sheet; else the global one.
    let (replaced, own) = match model.filter(|m| m.pages.contains_key(&page)) {
        Some(m) => (true, m.page_stylesheets.get(&page)),
        None => (
            templates
                .page_template(page)
                .is_some_and(|t| t.dev.is_some()),
            templates.page_stylesheets.get(&page),
        ),
    };
    if replaced && let Some(own) = own {
        return vec![Stylesheet::Runique(VARIABLES), Stylesheet::Dev(own.clone())];
    }
    let mut out: Vec<Stylesheet> = page.runique_sheets().map(Stylesheet::Runique).collect();
    let blocks = page
        .global_blocks()
        .iter()
        .filter_map(|b| b.resolve(&templates.global_blocks).stylesheet)
        .chain(page.blocks().iter().filter_map(|b| {
            b.resolve(model.map(|m| &m.blocks), &templates.blocks)
                .stylesheet
        }));
    for sheet in blocks {
        if !out.contains(&sheet) {
            out.push(sheet);
        }
    }
    out
}

/// A stylesheet as the `<head>` writes it: its URL, built as `{% static %}`
/// builds it, and its integrity when the file was found.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ServedStylesheet {
    pub href: String,
    pub integrity: Option<String>,
}

/// Where each static folder is served, and the hashes of its files.
pub(crate) struct Statics<'a> {
    pub site_url: &'a str,
    pub site: &'a StrMap,
    pub runique_url: &'a str,
    pub runique: &'a StrMap,
    /// The cache-busting token both folders' URLs carry.
    pub version: &'a str,
}

/// Every page's sheets, for every model (`""` for none), integrity looked up
/// once at boot.
#[derive(Clone, Debug, Default)]
pub struct PageSheets(HashMap<(Page, String), Vec<ServedStylesheet>>);

impl PageSheets {
    /// Resolves and hashes every page's sheets. A developer's sheet not
    /// found under his static (a typo, a path outside it) refuses the boot:
    /// served without its integrity, or not at all, it would go unnoticed.
    pub(crate) fn build(
        templates: &AdminTemplate,
        models: &[(&'static str, &ModelTemplates)],
        statics: &Statics,
    ) -> Result<Self, String> {
        use crate::context::tera::static_tera::asset_url;
        let serve = |sheet: Stylesheet| -> Result<ServedStylesheet, String> {
            Ok(match sheet {
                Stylesheet::Runique(path) => ServedStylesheet {
                    href: asset_url(statics.runique_url, path, statics.version),
                    integrity: statics.runique.get(path).cloned(),
                },
                Stylesheet::Dev(path) => {
                    let Some(hash) = statics.site.get(&path) else {
                        return Err(format!(
                            "admin stylesheet `{path}` is not a file of the static folder"
                        ));
                    };
                    ServedStylesheet {
                        href: asset_url(statics.site_url, &path, statics.version),
                        integrity: Some(hash.clone()),
                    }
                }
            })
        };
        let mut all = HashMap::new();
        for page in Page::ALL {
            let mut keys: Vec<(String, Option<&ModelTemplates>)> = vec![(String::new(), None)];
            if page.per_model() {
                keys.extend(
                    models
                        .iter()
                        .map(|(key, blocks)| (key.to_string(), Some(*blocks))),
                );
            }
            for (key, blocks) in keys {
                let served = sheets(page, templates, blocks)
                    .into_iter()
                    .map(serve)
                    .collect::<Result<Vec<_>, _>>()?;
                all.insert((page, key), served);
            }
        }
        Ok(Self(all))
    }

    /// The sheets of `page` for model `key` (`""` for none).
    pub(crate) fn of(&self, page: Page, key: &str) -> &[ServedStylesheet] {
        self.0
            .get(&(page, key.to_string()))
            .or_else(|| self.0.get(&(page, String::new())))
            .map_or(&[], Vec::as_slice)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths(sheets: &[Stylesheet]) -> Vec<String> {
        sheets
            .iter()
            .map(|s| match s {
                Stylesheet::Runique(p) => format!("runique:{p}"),
                Stylesheet::Dev(p) => format!("dev:{p}"),
            })
            .collect()
    }

    #[test]
    fn a_page_loads_the_layout_then_its_blocks_once_each() {
        let detail = paths(&sheets(Page::Detail, &AdminTemplate::new(), None));
        assert_eq!(detail[0], "runique:css/normalize/normalize.css");
        assert_eq!(detail[1], format!("runique:{VARIABLES}"));
        assert_eq!(
            detail.last().unwrap(),
            "runique:css/admin/blocks/rights.css"
        );
        let mut unique = detail.clone();
        unique.dedup();
        assert_eq!(unique.len(), detail.len());
    }

    #[test]
    fn a_blocks_sheet_replaced_takes_its_place() {
        let t = AdminTemplate::new().block_stylesheet(Block::DetailGroups, "css/mine.css");
        let detail = paths(&sheets(Page::Detail, &t, None));
        assert!(detail.contains(&"dev:css/mine.css".to_string()));
        assert!(!detail.contains(&"runique:css/admin/blocks/rights.css".to_string()));
    }

    #[test]
    fn a_models_sheet_only_reaches_its_own_pages() {
        let model = ModelTemplates::default().block_stylesheet(Block::ListTable, "css/notes.css");
        let t = AdminTemplate::new();
        assert!(
            paths(&sheets(Page::List, &t, Some(&model))).contains(&"dev:css/notes.css".to_string())
        );
        assert!(
            !paths(&sheets(Page::Dashboard, &t, Some(&model)))
                .contains(&"dev:css/notes.css".to_string())
        );
    }

    #[test]
    fn a_page_replaced_with_its_sheet_keeps_only_the_variables() {
        let t = AdminTemplate::new()
            .with_list("theme/list.html")
            .page_stylesheet(Page::List, "css/theme.css");
        assert_eq!(
            paths(&sheets(Page::List, &t, None)),
            [
                format!("runique:{VARIABLES}"),
                "dev:css/theme.css".to_string()
            ]
        );
        // Without its page replaced, a page sheet changes nothing.
        let t = AdminTemplate::new().page_stylesheet(Page::List, "css/theme.css");
        assert!(!paths(&sheets(Page::List, &t, None)).contains(&"dev:css/theme.css".to_string()));
        // A model's own page takes its own sheet, other models keep Runique's.
        let notes = ModelTemplates::default()
            .page(Page::List, "notes/list.html")
            .page_stylesheet(Page::List, "css/notes.css");
        assert_eq!(
            paths(&sheets(Page::List, &AdminTemplate::new(), Some(&notes))),
            [
                format!("runique:{VARIABLES}"),
                "dev:css/notes.css".to_string()
            ]
        );
        // Replaced without a sheet: Runique's whole.
        let t = AdminTemplate::new().with_list("theme/list.html");
        assert_eq!(
            sheets(Page::List, &t, None),
            sheets(Page::List, &AdminTemplate::new(), None)
        );
    }

    fn statics<'a>(site: &'a StrMap, runique: &'a StrMap) -> Statics<'a> {
        Statics {
            site_url: "/static",
            site,
            runique_url: "/runique/static",
            runique,
            version: "v1",
        }
    }

    #[test]
    fn a_developers_sheet_outside_his_static_refuses_the_boot() {
        let none = StrMap::new();
        let t = AdminTemplate::new().block_stylesheet(Block::ListTable, "../secret.css");
        let err = PageSheets::build(&t, &[], &statics(&none, &none)).unwrap_err();
        assert!(err.contains("../secret.css"), "{err}");

        let mut site = StrMap::new();
        site.insert("css/ok.css".into(), "sha384-x".into());
        let t = AdminTemplate::new().block_stylesheet(Block::ListTable, "css/ok.css");
        let built = PageSheets::build(&t, &[], &statics(&site, &none)).unwrap();
        let list = built.of(Page::List, "notes");
        let mine = list
            .iter()
            .find(|s| s.href.starts_with("/static/"))
            .unwrap();
        assert_eq!(mine.href, "/static/css/ok.css?v=v1");
        assert_eq!(mine.integrity.as_deref(), Some("sha384-x"));
        assert!(list[0].href.starts_with("/runique/static/css/"), "{list:?}");
    }

    /// A block's sheet reaches every page showing the block, and no other:
    /// the table of pages and blocks, page by page.
    #[test]
    fn a_blocks_sheet_reaches_the_pages_that_show_it() {
        use super::super::block::GlobalBlock;
        let model = |block: Block| AdminTemplate::new().block_stylesheet(block, "css/x.css");
        let global = |block: GlobalBlock| AdminTemplate::new().global_block_stylesheet(block, "css/x.css");
        let has = |page: Page, t: &AdminTemplate| paths(&sheets(page, t, None)).contains(&"dev:css/x.css".to_string());

        for page in [Page::Create, Page::Edit] {
            assert!(has(page, &model(Block::FormFields)), "{page:?}");
        }
        assert!(has(Page::Delete, &model(Block::DeleteWarning)));
        assert!(!has(Page::List, &model(Block::DeleteWarning)));
        assert!(has(Page::Dashboard, &global(GlobalBlock::DashboardTable)));
        assert!(!has(Page::List, &global(GlobalBlock::DashboardTable)));
        assert!(has(Page::Dashboard, &global(GlobalBlock::DashboardGroups)));
        assert!(has(Page::History, &global(GlobalBlock::HistoryTable)));
        assert!(has(Page::HistoryDiff, &global(GlobalBlock::HistoryActionBadge)));
        assert!(!has(Page::HistoryDiff, &global(GlobalBlock::HistoryTable)));
        assert!(has(Page::Rights, &global(GlobalBlock::RightsTable)));
        for page in [Page::Dashboard, Page::List, Page::History, Page::Rights, Page::Transfer] {
            assert!(has(page, &global(GlobalBlock::Sidebar)), "{page:?}");
        }
        assert!(!has(Page::Login, &global(GlobalBlock::Sidebar)), "the login has no layout");
    }

}
