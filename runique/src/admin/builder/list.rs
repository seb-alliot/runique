//! The list page of one model.
use crate::auth::gate::rights::ReadableColumns;
use crate::db::sql_dialect::{search_condition, text_cast_type_of};
use sea_orm::{
    sea_query::{Alias, Asterisk, Expr, ExprTrait, Func, StringLen, WindowStatement},
    {
        ActiveModelTrait, ColumnTrait, ColumnType, Condition, DbBackend, EntityTrait, IdenStatic,
        Iterable, Order, PrimaryKeyToColumn, QueryFilter, QueryOrder, QuerySelect, QueryTrait,
        Select, Value,
    },
};

/// What the URL asks of a list page, as received. Nothing here is trusted:
/// every field goes through `ListConfig` before touching the query.
#[derive(Default)]
pub(crate) struct ListQuery {
    pub page: u64,
    pub sort_by: Option<String>,
    pub sort_dir: Option<String>,
    pub search: Option<String>,
    /// `(column, value)`, several per column: a row matches one of a column's
    /// values, and every column.
    pub filters: Vec<(String, String)>,
    /// The page size asked in the URL, kept only when it is one of
    /// [`PER_PAGE_OPTIONS`]: a free value would let a request pull the whole
    /// table in one page.
    pub per_page: Option<u64>,
    /// Values per page of the filter groups (`fsize`), bound to
    /// [`PER_PAGE_OPTIONS`] like `per_page`.
    pub filter_size: Option<u64>,
    /// The page each filter group shows (`fpage_{col}`). Any name is kept
    /// here; only the list's own filter columns ever read it.
    pub filter_pages: Vec<(String, u64)>,
}

/// The page sizes the list offers, and the only ones the URL may set.
pub const PER_PAGE_OPTIONS: [u64; 4] = [10, 25, 50, 100];

/// Values per filter group without a choice in the URL.
pub(crate) const FILTER_PAGE_SIZE: u64 = 10;

/// Values one column's filter accepts from the URL: past it, the rest is
/// ignored, so a forged query can't grow the `WHERE` without bound.
const FILTER_CHOSEN_MAX: usize = 100;

/// Shorter terms search nothing: one or two letters match almost every row
/// and scan the whole table, while the list searches as one types.
pub(crate) const SEARCH_MIN_LEN: usize = 3;

impl ListQuery {
    /// Reads the URL's query string. It only shapes the values: which column
    /// a name may reach is decided later, by `ListConfig`.
    pub(crate) fn from_params(params: Vec<(String, String)>, search_max_len: usize) -> Self {
        // A name sent twice: the last one counts, except for a filter's values.
        let non_empty = |key: &str| {
            params
                .iter()
                .rev()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.clone())
                .filter(|v| !v.is_empty())
        };
        let page = non_empty("page").and_then(|p| p.parse().ok()).unwrap_or(1);
        let per_page = non_empty("per_page")
            .and_then(|v| v.parse().ok())
            .filter(|v| PER_PAGE_OPTIONS.contains(v));
        let filter_size = non_empty("fsize")
            .and_then(|v| v.parse().ok())
            .filter(|v| PER_PAGE_OPTIONS.contains(v));
        let sort_by = non_empty("sort_by");
        let sort_dir = non_empty("sort_dir");
        let search = non_empty("search")
            .filter(|s| s.trim().chars().count() >= SEARCH_MIN_LEN)
            .map(|s| s.chars().take(search_max_len).collect());
        let mut filters: Vec<(String, String)> = Vec::new();
        let mut filter_pages = Vec::new();
        for (key, value) in params {
            if let Some(col) = key.strip_prefix("filter_")
                && !value.is_empty()
                && !filters.iter().any(|(c, v)| c == col && *v == value)
                && filters.iter().filter(|(c, _)| c == col).count() < FILTER_CHOSEN_MAX
            {
                filters.push((col.to_string(), value));
            } else if let Some(col) = key.strip_prefix("fpage_")
                && let Ok(n) = value.parse::<u64>()
                && n > 1
            {
                filter_pages.push((col.to_string(), n));
            }
        }

        Self {
            page,
            per_page,
            sort_by,
            sort_dir,
            search,
            filters,
            filter_size,
            filter_pages,
        }
    }

    /// The page the filter group of `col` shows, 1 when the URL names none.
    pub(crate) fn filter_page(&self, col: &str) -> u64 {
        self.filter_pages
            .iter()
            .rev()
            .find(|(c, _)| c == col)
            .map_or(1, |(_, n)| *n)
    }
}

/// Sets `col` to a value fixed here, on the rows checked or on one row.
/// The request only ever names the action, never the value.
#[derive(Clone)]
pub(crate) struct Action<E: EntityTrait> {
    pub(crate) col: E::Column,
    pub(crate) value: Value,
    pub(crate) label: String,
}

/// Alias of the value a filter offers.
pub(crate) const FILTER_VALUE: &str = "value";

/// Alias of the window column carrying the total on every row.
pub(crate) const TOTAL: &str = "__total";

/// Only what is declared here is shown or accepted from the URL
/// (e.g. `?sort_by=title&sort_dir=desc`); anything else is ignored.
///
#[doc = include_str!("../../../doc-tests/admin/list_config.md")]
#[derive(Clone)]
pub struct ListConfig<E: EntityTrait> {
    /// Columns shown, in this order.
    display: Vec<E::Column>,
    /// Columns linking to the detail page (empty: the first one shown).
    links: Vec<E::Column>,
    /// The only columns sortable from the URL.
    sortable: Vec<E::Column>,
    /// The default order, direction included.
    ordering: Vec<(E::Column, Order)>,
    /// `None`: the text columns shown; `Some(vec![])`: search disabled.
    search: Option<Vec<E::Column>>,
    /// The only columns filterable from the URL (empty: no filter).
    filter: Vec<E::Column>,
    /// The default number of items per page.
    per_page: Option<u64>,
    /// The default number of values per filter group.
    filter_size: Option<u64>,
    /// Text under the search bar
    pub(super) search_help: Option<String>,
    /// Shown in place of an empty value (`None`: the template's default).
    pub(super) empty_value: Option<String>,
    /// Declared in order: an action is named by its place in this list
    pub(super) actions: Vec<Action<E>>,
}

impl<E: EntityTrait> Default for ListConfig<E> {
    fn default() -> Self {
        Self {
            display: Vec::new(),
            links: Vec::new(),
            sortable: Vec::new(),
            ordering: Vec::new(),
            search: None,
            filter: Vec::new(),
            per_page: None,
            filter_size: None,
            search_help: None,
            empty_value: None,
            actions: Vec::new(),
        }
    }
}

impl<E: EntityTrait> ListConfig<E> {
    /// This list as one user may run it: every column list cut to what the proof lets out.
    pub(crate) fn readable_only(&self, readable: &ReadableColumns) -> Self {
        Self {
            display: readable.preserve::<E>(&self.display),
            links: readable.preserve::<E>(&self.links),
            sortable: readable.preserve::<E>(&self.sortable),
            ordering: self
                .ordering
                .iter()
                .filter(|(c, _)| !readable.preserve::<E>(&[*c]).is_empty())
                .cloned()
                .collect(),
            search: self
                .search
                .as_ref()
                .map(|cols| readable.preserve::<E>(cols)),
            filter: readable.preserve::<E>(&self.filter),
            ..self.clone()
        }
    }

    pub fn display(mut self, cols: impl IntoIterator<Item = E::Column>) -> Self {
        self.display = cols.into_iter().collect();
        self
    }

    /// Whether `display` was declared.
    /// The options set on this list that no page reads yet.
    pub(super) fn options_without_effect(&self) -> Vec<&'static str> {
        let mut names = Vec::new();
        if !self.links.is_empty() {
            names.push("links");
        }
        if self.search_help.is_some() {
            names.push("search_help");
        }
        if self.empty_value.is_some() {
            names.push("empty_value");
        }
        names
    }

    pub(super) fn has_display(&self) -> bool {
        !self.display.is_empty()
    }

    /// The columns shown, in their declared order.
    pub(super) fn display_columns(&self) -> &[E::Column] {
        &self.display
    }

    /// The columns filterable from the URL, in their declared order.
    pub(super) fn filter_columns(&self) -> &[E::Column] {
        &self.filter
    }

    /// The columns sortable from the URL, in their declared order: the only
    /// ones `resolve_sort` honours, so the only ones worth offering.
    pub(super) fn sortable_columns(&self) -> &[E::Column] {
        &self.sortable
    }

    /// The values a filter offers for `col`: the distinct ones of the rows in
    /// `scope`, as text (how `resolve_filters` compares them), sorted, NULL
    /// left out, `page` of `size` of them. One more than `size` is read: its
    /// presence says a next page exists, without counting the values.
    pub(crate) fn filter_values(
        backend: DbBackend,
        scope: Condition,
        col: E::Column,
        page: u64,
        size: u64,
    ) -> Select<E> {
        let cast = Alias::new(text_cast_type_of(backend));
        // The databases take a signed 64-bit offset: a huge page from the URL
        // stays in range instead of failing the query.
        let offset = std::cmp::Ord::min(
            page.saturating_sub(1).saturating_mul(size),
            i64::MAX.unsigned_abs(),
        );
        E::find()
            .filter(scope)
            .filter(col.is_not_null())
            .select_only()
            .column_as(Expr::col((E::default(), col)).cast_as(cast), FILTER_VALUE)
            .distinct()
            .order_by_asc(Expr::col(Alias::new(FILTER_VALUE)))
            .offset(offset)
            .limit(size.saturating_add(1))
    }

    pub fn links(mut self, cols: impl IntoIterator<Item = E::Column>) -> Self {
        self.links = cols.into_iter().collect();
        self
    }

    pub fn sortable(mut self, cols: impl IntoIterator<Item = E::Column>) -> Self {
        self.sortable = cols.into_iter().collect();
        self
    }

    pub fn ordering(mut self, cols: impl IntoIterator<Item = (E::Column, Order)>) -> Self {
        self.ordering = cols.into_iter().collect();
        self
    }

    pub fn search(mut self, cols: impl IntoIterator<Item = E::Column>) -> Self {
        self.search = Some(cols.into_iter().collect());
        self
    }

    pub fn filter(mut self, cols: impl IntoIterator<Item = E::Column>) -> Self {
        self.filter = cols.into_iter().collect();
        self
    }

    pub fn per_page(mut self, per_page: u64) -> Self {
        self.per_page = Some(per_page);
        self
    }

    /// Values each filter group shows before its arrows, over the site's
    /// `filter_size` (at least 1).
    pub fn filter_size(mut self, size: u64) -> Self {
        self.filter_size = Some(std::cmp::max(size, 1));
        self
    }

    /// Values per page of the filter groups: the URL's choice (bound to
    /// [`PER_PAGE_OPTIONS`]), otherwise the model's, otherwise the site's.
    pub(crate) fn filter_page_size(&self, query: &ListQuery, site_default: u64) -> u64 {
        std::cmp::max(
            query
                .filter_size
                .unwrap_or_else(|| self.filter_size.unwrap_or(site_default)),
            1,
        )
    }

    pub fn search_help(mut self, text: impl Into<String>) -> Self {
        self.search_help = Some(text.into());
        self
    }

    pub fn empty_value(mut self, text: impl Into<String>) -> Self {
        self.empty_value = Some(text.into());
        self
    }

    pub fn action(
        mut self,
        col: E::Column,
        value: impl Into<Value>,
        label: impl Into<String>,
    ) -> Self {
        self.actions.push(Action {
            col,
            value: value.into(),
            label: label.into(),
        });
        self
    }

    /// The lists of this block that name the column `name`.
    pub(super) fn lists_using(&self, name: &str) -> Vec<&'static str> {
        let named = |cols: &[E::Column]| cols.iter().any(|c| c.as_str() == name);
        let mut lists = Vec::new();

        if named(&self.display) {
            lists.push("display");
        }

        if self.actions.iter().any(|a| a.col.as_str() == name) {
            lists.push("action");
        }

        if named(&self.links) {
            lists.push("links");
        }
        if named(&self.sortable) {
            lists.push("sortable");
        }
        if self.ordering.iter().any(|(c, _)| c.as_str() == name) {
            lists.push("ordering");
        }
        if let Some(search) = &self.search
            && named(search)
        {
            lists.push("search");
        }
        if named(&self.filter) {
            lists.push("filter");
        }
        lists
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        let mut list_error = Vec::new();
        for (field, cols) in [("links", &self.links), ("sortable", &self.sortable)] {
            for col in cols {
                if !self.display.iter().any(|c| c.as_str() == col.as_str()) {
                    list_error.push(format!(
                        "{field}: column `{}` is not in display",
                        col.as_str()
                    ));
                }
            }
        }

        if self.search_help.is_some() && self.search.as_ref().is_some_and(Vec::is_empty) {
            list_error.push("search_help: search is disabled (`search([])`)".to_string());
        }

        for (i, action) in self.actions.iter().enumerate() {
            let name = action.col.as_str();
            if E::PrimaryKey::iter().any(|k| k.into_column().as_str() == name) {
                list_error.push(format!(
                    "action `{}`: `{name}` is the primary key",
                    action.label
                ));
            }
            if !value_fits::<E>(action.col, &action.value) {
                list_error.push(format!(
                    "action `{}`: the value does not fit the column `{name}`",
                    action.label
                ));
            }
            if self.actions[..i]
                .iter()
                .any(|a| a.col.as_str() == name && a.value == action.value)
            {
                list_error.push(format!(
                    "action `{}`: `{name}` already set to this value",
                    action.label
                ));
            }
        }

        if !list_error.is_empty() {
            Err(list_error.join("\n"))
        } else {
            Ok(())
        }
    }

    pub(crate) fn resolve_sort(
        &self,
        sort_by: Option<&str>,
        sort_dir: Option<&str>,
    ) -> Vec<(E::Column, Order)> {
        let dir = match sort_dir {
            Some("desc") => Order::Desc,
            _ => Order::Asc,
        };
        if let Some(name) = sort_by
            && let Some(col) = self.sortable.iter().find(|c| c.as_str() == name)
        {
            vec![(*col, dir)]
        } else {
            self.ordering.clone()
        }
    }

    pub(crate) fn resolve_search(&self, backend: DbBackend, term: &str) -> Option<Condition> {
        match &self.search {
            Some(cols) => search_condition::<E>(backend, cols, term),
            None => {
                let text: Vec<E::Column> =
                    self.display.iter().copied().filter(is_text::<E>).collect();
                search_condition::<E>(backend, &text, term)
            }
        }
    }

    pub(crate) fn resolve_filters(
        &self,
        backend: DbBackend,
        filters: &[(String, String)],
    ) -> Option<Condition> {
        let cast = Alias::new(text_cast_type_of(backend));
        let mut cond = Condition::all();
        for col in &self.filter {
            let one_of = filters
                .iter()
                .filter(|(name, _)| name == col.as_str())
                .fold(Condition::any(), |any, (_, value)| {
                    any.add(
                        Expr::col((E::default(), *col))
                            .cast_as(cast.clone())
                            .eq(value.as_str()),
                    )
                });
            if !one_of.is_empty() {
                cond = cond.add(one_of);
            }
        }
        (!cond.is_empty()).then_some(cond)
    }
    /// Rows per page without a viewer's choice: the model's `per_page`,
    /// otherwise the site's. The bulk guard caps its ids to this.
    pub(crate) fn default_page_size(&self, site_default: u64) -> u64 {
        std::cmp::max(self.per_page.unwrap_or(site_default), 1)
    }

    /// Rows per page: the URL's choice (already bound to [`PER_PAGE_OPTIONS`]),
    /// otherwise [`default_page_size`](Self::default_page_size).
    pub(crate) fn page_size(&self, query: &ListQuery, site_default: u64) -> u64 {
        std::cmp::max(
            query
                .per_page
                .unwrap_or_else(|| self.default_page_size(site_default)),
            1,
        )
    }

    /// The rows the page draws from: the gate's scope, then search and filters.
    pub(crate) fn filtered(
        &self,
        backend: DbBackend,
        scope: Condition,
        query: &ListQuery,
    ) -> Select<E> {
        let mut select = E::find().filter(scope);
        if let Some(term) = &query.search
            && let Some(cond) = self.resolve_search(backend, term)
        {
            select = select.filter(cond);
        }
        if let Some(cond) = self.resolve_filters(backend, &query.filters) {
            select = select.filter(cond);
        }
        select
    }

    /// The page query: `filtered`, ordered, reduced to the columns shown, with
    /// the total of matching rows on every row.
    pub(crate) fn select(
        &self,
        backend: DbBackend,
        scope: Condition,
        query: &ListQuery,
    ) -> Select<E> {
        let mut select = self.filtered(backend, scope, query);
        for (col, order) in self.resolve_sort(query.sort_by.as_deref(), query.sort_dir.as_deref()) {
            select = select.order_by(col, order);
        }
        for key in E::PrimaryKey::iter() {
            select = select.order_by(key.into_column(), Order::Asc);
        }
        let mut select = select.select_only().columns(self.read_columns());
        QueryTrait::query(&mut select).expr_window_as(
            Func::count(Expr::col(Asterisk)),
            WindowStatement::new(),
            Alias::new(TOTAL),
        );
        select
    }

    /// The columns read from the database: those shown, plus the primary key
    /// for the row links. Nothing else leaves the table.
    fn read_columns(&self) -> Vec<E::Column> {
        let mut cols = self.display.clone();
        for key in E::PrimaryKey::iter() {
            let col = key.into_column();
            if !cols.iter().any(|c| c.as_str() == col.as_str()) {
                cols.push(col);
            }
        }
        cols
    }
}

/// Text in the database: the only columns searched when `search` is not declared.
fn is_text<E: EntityTrait>(col: &E::Column) -> bool {
    matches!(
        col.def().get_column_type(),
        ColumnType::Char(_) | ColumnType::String(_) | ColumnType::Text
    )
}

/// The model itself says whether `value` converts to the column's Rust type,
/// enum variants included, on every engine. The column's declared bounds come
/// on top: a decimal's precision and scale, a text's length. Never NULL.
fn value_fits<E: EntityTrait>(col: E::Column, value: &Value) -> bool {
    if *value == value.as_null() {
        return false;
    }
    let mut probe = <E::ActiveModel as ActiveModelTrait>::default();
    if probe.try_set(col, value.clone()).is_err() {
        return false;
    }
    match (col.def().get_column_type(), value) {
        (ColumnType::Decimal(len), Value::Decimal(Some(d))) => decimal_fits(*len, d),
        (ColumnType::String(StringLen::N(n)), Value::String(Some(s))) => {
            s.chars().count() <= *n as usize
        }
        _ => true,
    }
}

/// A price fits its column exactly: no more decimals than its scale, no more
/// integer digits than precision minus scale. Otherwise each engine would
/// round or keep it its own way.
fn decimal_fits(len: Option<(u32, u32)>, d: &sea_orm::prelude::Decimal) -> bool {
    let Some((precision, scale)) = len else {
        return true;
    };
    let integer_digits = d.trunc().abs().to_string().trim_start_matches('0').len() as u32;
    d.scale() <= scale && integer_digits <= precision.saturating_sub(scale)
}

#[cfg(feature = "sqlite")]
#[cfg(test)]
mod tests {
    use super::{FILTER_CHOSEN_MAX, ListConfig, ListQuery, decimal_fits};
    use crate::auth::gate::test_support::fixtures::note::{Column, Entity};
    use crate::auth::user;
    use sea_orm::prelude::Decimal;
    use sea_orm::{Condition, DbBackend, EntityTrait, IdenStatic, Order, QueryFilter, QueryTrait};

    mod product {
        use sea_orm::entity::prelude::*;

        #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
        #[sea_orm(table_name = "products")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub id: i32,
            #[sea_orm(column_type = "Decimal(Some((10, 2)))")]
            pub price: Decimal,
            #[sea_orm(column_type = "String(StringLen::N(5))")]
            pub code: String,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {}

        impl ActiveModelBehavior for ActiveModel {}
    }

    #[test]
    fn lists_using_names_exactly_the_lists_holding_the_column() {
        let list = ListConfig::<user::Entity>::default()
            .display([user::Column::Username, user::Column::Email])
            .links([user::Column::Username])
            .sortable([user::Column::Email])
            .ordering([(user::Column::Username, Order::Asc)])
            .search([user::Column::Email])
            .filter([user::Column::IsActive])
            .action(user::Column::IsActive, false, "Désactiver");
        assert_eq!(
            list.lists_using("username"),
            ["display", "links", "ordering"]
        );
        assert_eq!(list.lists_using("email"), ["display", "sortable", "search"]);
        assert_eq!(list.lists_using("is_active"), ["action", "filter"]);
        assert!(list.lists_using("is_admin").is_empty());
    }

    #[test]
    fn links_and_sortable_inside_display_are_accepted() {
        let list = ListConfig::<Entity>::default()
            .display([Column::Id, Column::OwnerId])
            .links([Column::Id])
            .sortable([Column::OwnerId]);
        assert!(list.validate().is_ok());
    }

    #[test]
    fn a_link_outside_display_is_refused() {
        let list = ListConfig::<Entity>::default()
            .display([Column::Id])
            .links([Column::OwnerId]);
        let error = list.validate().unwrap_err();
        assert!(error.contains("links"), "{error}");
        assert!(error.contains("owner_id"), "{error}");
    }
    #[test]
    fn search_help_on_a_disabled_search_is_refused() {
        let list = ListConfig::<Entity>::default()
            .search([])
            .search_help("Titre ou auteur");
        let error = list.validate().unwrap_err();
        assert!(error.contains("search_help"), "{error}");
    }

    #[test]
    fn search_help_on_the_default_search_is_accepted() {
        let list = ListConfig::<Entity>::default().search_help("Titre ou auteur");
        assert!(list.validate().is_ok());
    }

    #[test]
    fn search_help_on_declared_columns_is_accepted() {
        let list = ListConfig::<Entity>::default()
            .search([Column::OwnerId])
            .search_help("Propriétaire");
        assert!(list.validate().is_ok());
    }

    #[test]
    fn a_sortable_column_outside_display_is_refused() {
        let list = ListConfig::<Entity>::default()
            .display([Column::Id])
            .sortable([Column::OwnerId]);
        let error = list.validate().unwrap_err();
        assert!(error.contains("sortable"), "{error}");
    }

    fn sortable_list() -> ListConfig<Entity> {
        ListConfig::<Entity>::default()
            .display([Column::Id, Column::OwnerId])
            .sortable([Column::OwnerId])
            .ordering([(Column::Id, Order::Desc)])
    }

    #[test]
    fn a_sortable_column_from_the_url_is_applied() {
        let sort = sortable_list().resolve_sort(Some("owner_id"), Some("desc"));
        assert_eq!(sort.len(), 1);
        assert_eq!(sort[0].0.as_str(), "owner_id");
        assert_eq!(sort[0].1, Order::Desc);
    }

    #[test]
    fn a_shown_but_not_sortable_column_falls_back_to_the_default() {
        let sort = sortable_list().resolve_sort(Some("id"), Some("asc"));
        assert_eq!(sort[0].0.as_str(), "id");
        assert_eq!(sort[0].1, Order::Desc);
    }

    #[test]
    fn an_unknown_column_falls_back_to_the_default() {
        let sort = sortable_list().resolve_sort(Some("id; DROP TABLE notes"), None);
        assert_eq!(sort.len(), 1);
        assert_eq!(sort[0].0.as_str(), "id");
    }

    #[test]
    fn any_direction_other_than_desc_is_ascending() {
        let sort = sortable_list().resolve_sort(Some("owner_id"), Some("DESC"));
        assert_eq!(sort[0].1, Order::Asc);
    }

    /// The `WHERE` part only, or `None` when no search applies.
    fn search_where(list: &ListConfig<user::Entity>, term: &str) -> Option<String> {
        let cond = list.resolve_search(DbBackend::Sqlite, term)?;
        let sql = user::Entity::find()
            .filter(cond)
            .build(DbBackend::Sqlite)
            .to_string();
        sql.split_once("WHERE").map(|(_, w)| w.to_string())
    }

    #[test]
    fn by_default_only_the_text_columns_shown_are_searched() {
        let list = ListConfig::<user::Entity>::default().display([
            user::Column::Id,
            user::Column::Username,
            user::Column::IsSuperuser,
        ]);
        let w = search_where(&list, "bob").unwrap();
        assert!(w.contains(r#""username""#), "{w}");
        assert!(!w.contains("is_superuser"), "{w}");
        assert!(!w.contains(r#""id""#), "{w}");
    }

    #[test]
    fn declared_search_columns_replace_the_default() {
        let list = ListConfig::<user::Entity>::default()
            .display([user::Column::Username])
            .search([user::Column::Email]);
        let w = search_where(&list, "bob").unwrap();
        assert!(w.contains(r#""email""#), "{w}");
        assert!(!w.contains("username"), "{w}");
    }

    #[test]
    fn an_empty_search_disables_it() {
        let list = ListConfig::<user::Entity>::default()
            .display([user::Column::Username])
            .search([]);
        assert!(search_where(&list, "bob").is_none());
    }

    #[test]
    fn no_text_column_shown_means_no_search() {
        let list = ListConfig::<user::Entity>::default()
            .display([user::Column::Id, user::Column::IsSuperuser]);
        assert!(search_where(&list, "bob").is_none());
    }
    fn pairs(raw: &[(&str, &str)]) -> Vec<(String, String)> {
        raw.iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    fn note_where(cond: Condition) -> String {
        let sql = Entity::find()
            .filter(cond)
            .build(DbBackend::Sqlite)
            .to_string();
        sql.split_once("WHERE").map(|(_, w)| w.to_string()).unwrap()
    }

    fn filtered_list() -> ListConfig<Entity> {
        ListConfig::<Entity>::default()
            .display([Column::Id, Column::OwnerId])
            .filter([Column::Id, Column::OwnerId])
    }

    #[test]
    fn a_declared_filter_is_applied_as_text() {
        let cond = filtered_list()
            .resolve_filters(DbBackend::Sqlite, &pairs(&[("owner_id", "10")]))
            .unwrap();
        let w = note_where(cond);
        assert!(
            w.contains(r#"CAST("notes"."owner_id" AS TEXT) = '10'"#),
            "{w}"
        );
    }

    #[test]
    fn several_filters_are_combined_with_and() {
        let cond = filtered_list()
            .resolve_filters(
                DbBackend::Sqlite,
                &pairs(&[("id", "1"), ("owner_id", "10")]),
            )
            .unwrap();
        assert!(note_where(cond).contains(" AND "));
    }

    #[test]
    fn the_values_of_one_column_are_alternatives() {
        let cond = filtered_list()
            .resolve_filters(
                DbBackend::Sqlite,
                &pairs(&[("owner_id", "10"), ("owner_id", "20"), ("id", "1")]),
            )
            .unwrap();
        let w = note_where(cond);
        assert!(w.contains(" OR "), "the column's two values: {w}");
        assert!(w.contains(" AND "), "the two columns: {w}");
        assert!(w.contains("'10'") && w.contains("'20'"), "{w}");
    }

    #[test]
    fn a_column_keeps_each_of_its_values_once_up_to_the_cap() {
        let q = ListQuery::from_params(
            params(&[
                ("filter_id", "1"),
                ("filter_id", "2"),
                ("filter_id", "1"),
                ("filter_id", ""),
            ]),
            150,
        );
        assert_eq!(q.filters, pairs(&[("id", "1"), ("id", "2")]));

        let many: Vec<(String, String)> = (0..=FILTER_CHOSEN_MAX)
            .map(|n| ("filter_id".to_string(), n.to_string()))
            .collect();
        let q = ListQuery::from_params(many, 150);
        assert_eq!(q.filters.len(), FILTER_CHOSEN_MAX);
    }

    #[test]
    fn a_name_sent_twice_keeps_its_last_value() {
        let q = ListQuery::from_params(params(&[("page", "2"), ("page", "3")]), 150);
        assert_eq!(q.page, 3);
    }

    #[test]
    fn an_undeclared_column_is_dropped() {
        let list = ListConfig::<Entity>::default().filter([Column::OwnerId]);
        let filters = pairs(&[("id", "1")]);
        assert!(list.resolve_filters(DbBackend::Sqlite, &filters).is_none());
    }

    #[test]
    fn only_the_declared_part_of_a_mix_is_kept() {
        let list = ListConfig::<Entity>::default().filter([Column::OwnerId]);
        let cond = list
            .resolve_filters(
                DbBackend::Sqlite,
                &pairs(&[("id", "1"), ("owner_id", "10")]),
            )
            .unwrap();
        let w = note_where(cond);
        assert!(w.contains("owner_id"), "{w}");
        assert!(!w.contains(r#""id""#), "{w}");
    }

    #[test]
    fn the_value_stays_a_bound_literal() {
        let cond = filtered_list()
            .resolve_filters(DbBackend::Sqlite, &pairs(&[("id", "1' OR '1'='1")]))
            .unwrap();
        let w = note_where(cond);
        assert!(w.contains("'1'' OR ''1''=''1'"), "{w}");
    }

    #[test]
    fn no_filter_in_the_url_is_no_condition() {
        assert!(
            filtered_list()
                .resolve_filters(DbBackend::Sqlite, &[])
                .is_none()
        );
    }

    fn params(raw: &[(&str, &str)]) -> Vec<(String, String)> {
        raw.iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn the_query_string_is_read_as_given() {
        let query = ListQuery::from_params(
            params(&[
                ("page", "3"),
                ("sort_by", "title"),
                ("sort_dir", "desc"),
                ("search", "bob"),
                ("filter_owner_id", "10"),
            ]),
            150,
        );
        assert_eq!(query.page, 3);
        assert_eq!(query.sort_by.as_deref(), Some("title"));
        assert_eq!(query.sort_dir.as_deref(), Some("desc"));
        assert_eq!(query.search.as_deref(), Some("bob"));
        assert_eq!(query.filters, [("owner_id".to_string(), "10".to_string())]);
    }

    #[test]
    fn empty_values_are_absent() {
        let query = ListQuery::from_params(params(&[("search", ""), ("filter_id", "")]), 150);
        assert!(query.search.is_none());
        assert!(query.filters.is_empty());
    }

    #[test]
    fn a_per_page_in_the_options_is_read() {
        let q = ListQuery::from_params(params(&[("per_page", "25")]), 150);
        assert_eq!(q.per_page, Some(25));
    }

    #[test]
    fn a_per_page_outside_the_options_is_ignored() {
        for v in ["1000000", "7", "0", "abc", "-5"] {
            let q = ListQuery::from_params(params(&[("per_page", v)]), 150);
            assert_eq!(q.per_page, None, "{v}");
        }
    }

    #[test]
    fn a_filter_size_is_bound_to_the_options() {
        let bare = ListConfig::<Entity>::default();
        let q = ListQuery::from_params(params(&[("fsize", "50")]), 150);
        assert_eq!(bare.filter_page_size(&q, 10), 50);
        for v in ["1000000", "7", "0", "abc"] {
            let q = ListQuery::from_params(params(&[("fsize", v)]), 150);
            assert_eq!(bare.filter_page_size(&q, 10), 10, "{v}");
        }
    }

    /// A selector resubmitted over a URL that already names a size: the new
    /// one counts, or picking 50 would fall back on the previous 10.
    #[test]
    fn the_last_filter_size_and_page_of_the_url_count() {
        let bare = ListConfig::<Entity>::default();
        let q = ListQuery::from_params(
            params(&[
                ("fsize", "10"),
                ("fsize", "50"),
                ("fpage_status", "2"),
                ("fpage_status", "3"),
            ]),
            150,
        );
        assert_eq!(bare.filter_page_size(&q, 5), 50);
        assert_eq!(q.filter_page("status"), 3);
    }

    #[test]
    fn the_url_filter_size_wins_over_the_model_and_site_defaults() {
        let model = ListConfig::<Entity>::default().filter_size(15);
        let picked = ListQuery::from_params(params(&[("fsize", "25")]), 150);
        let none = ListQuery::default();
        assert_eq!(model.filter_page_size(&picked, 5), 25, "the url");
        assert_eq!(model.filter_page_size(&none, 5), 15, "the model");
        let bare = ListConfig::<Entity>::default();
        assert_eq!(bare.filter_page_size(&none, 5), 5, "the site");
        assert_eq!(
            ListConfig::<Entity>::default()
                .filter_size(0)
                .filter_page_size(&none, 5),
            1,
            "a zero is raised to one"
        );
    }

    #[test]
    fn a_filter_page_is_read_per_column() {
        let q = ListQuery::from_params(params(&[("fpage_status", "3"), ("fpage_lang", "x")]), 150);
        assert_eq!(q.filter_page("status"), 3);
        assert_eq!(q.filter_page("lang"), 1, "not a number");
        assert_eq!(q.filter_page("other"), 1, "not in the url");
        // `fpage_*` / `fsize` are not filters: they must not narrow the rows.
        assert!(q.filters.is_empty());
        let sized = ListQuery::from_params(params(&[("fsize", "25")]), 150);
        assert!(sized.filters.is_empty());
        let one = ListQuery::from_params(params(&[("fpage_status", "1")]), 150);
        assert!(
            one.filter_pages.is_empty(),
            "page 1 is the default, not kept"
        );
    }

    #[test]
    fn the_url_page_size_wins_over_the_model_and_site_defaults() {
        let list = ListConfig::<Entity>::default().per_page(10);
        let picked = ListQuery {
            per_page: Some(50),
            ..Default::default()
        };
        assert_eq!(list.page_size(&picked, 5), 50, "the url");
        assert_eq!(list.page_size(&ListQuery::default(), 5), 10, "the model");
        let bare = ListConfig::<Entity>::default();
        assert_eq!(bare.page_size(&ListQuery::default(), 5), 5, "the site");
        assert_eq!(
            bare.default_page_size(5),
            5,
            "the bulk guard ignores the url"
        );
    }

    #[test]
    fn a_bad_or_missing_page_is_the_first() {
        assert_eq!(
            ListQuery::from_params(params(&[("page", "abc")]), 150).page,
            1
        );
        assert_eq!(ListQuery::from_params(params(&[]), 150).page, 1);
    }

    #[test]
    fn keys_without_the_filter_prefix_are_not_filters() {
        let query =
            ListQuery::from_params(params(&[("csrf_token", "x"), ("fp_owner_id", "2")]), 150);
        assert!(query.filters.is_empty());
    }
    #[test]
    fn a_long_search_term_is_cut_to_the_limit() {
        let query = ListQuery::from_params(params(&[("search", "aaaaaaaa")]), 5);
        assert_eq!(query.search.as_deref(), Some("aaaaa"));
    }

    #[test]
    fn a_term_below_three_characters_searches_nothing() {
        assert!(
            ListQuery::from_params(params(&[("search", "ab")]), 150)
                .search
                .is_none()
        );
        assert!(
            ListQuery::from_params(params(&[("search", " ab ")]), 150)
                .search
                .is_none()
        );
        assert_eq!(
            ListQuery::from_params(params(&[("search", "Léo")]), 150)
                .search
                .as_deref(),
            Some("Léo"),
            "three characters, accent included"
        );
    }

    #[test]
    fn a_term_at_the_limit_is_kept_whole() {
        let query = ListQuery::from_params(params(&[("search", "abcde")]), 5);
        assert_eq!(query.search.as_deref(), Some("abcde"));
    }

    #[test]
    fn an_accented_term_is_cut_on_characters_not_bytes() {
        let query = ListQuery::from_params(params(&[("search", "éééééé")]), 5);
        assert_eq!(query.search.as_deref(), Some("ééééé"));
    }

    #[test]
    fn a_price_at_the_column_scale_is_accepted() {
        // 99999999.99: the highest value of a decimal(10, 2)
        assert!(decimal_fits(Some((10, 2)), &Decimal::new(9_999_999_999, 2)));
        assert!(decimal_fits(Some((10, 2)), &Decimal::new(50, 2)));
    }

    #[test]
    fn a_price_with_too_many_decimals_is_refused() {
        assert!(!decimal_fits(Some((10, 2)), &Decimal::new(9_999, 3)));
    }

    #[test]
    fn a_price_with_too_many_integer_digits_is_refused() {
        // 100000000.00: 9 integer digits, the column holds 8
        assert!(!decimal_fits(
            Some((10, 2)),
            &Decimal::new(10_000_000_000, 2)
        ));
    }

    #[test]
    fn an_action_on_the_primary_key_is_refused() {
        let list = ListConfig::<user::Entity>::default().action(user::Column::Id, 1, "Renuméroter");
        let error = list.validate().unwrap_err();
        assert!(error.contains("primary key"), "{error}");
        assert!(error.contains("Renuméroter"), "{error}");
    }

    #[test]
    fn a_value_of_the_wrong_type_is_refused() {
        let list = ListConfig::<user::Entity>::default().action(
            user::Column::IsSuperuser,
            "yes",
            "Promouvoir",
        );
        let error = list.validate().unwrap_err();
        assert!(error.contains("does not fit"), "{error}");
    }

    #[test]
    fn a_null_value_is_refused() {
        let list = ListConfig::<user::Entity>::default().action(
            user::Column::IsActive,
            None::<bool>,
            "Vider",
        );
        assert!(list.validate().is_err());
    }

    #[test]
    fn a_single_well_typed_action_is_accepted() {
        let list = ListConfig::<user::Entity>::default().action(
            user::Column::IsActive,
            false,
            "Désactiver",
        );
        assert!(list.validate().is_ok());
    }

    #[test]
    fn the_same_column_set_twice_to_the_same_value_is_refused() {
        let list = ListConfig::<user::Entity>::default()
            .action(user::Column::IsActive, false, "Désactiver")
            .action(user::Column::IsActive, false, "Suspendre");
        let error = list.validate().unwrap_err();
        assert!(error.contains("already set"), "{error}");
        assert!(
            error.contains("Suspendre"),
            "the second one is named: {error}"
        );
        assert!(!error.contains("Désactiver"), "{error}");
    }

    #[test]
    fn the_same_column_with_another_value_is_accepted() {
        let list = ListConfig::<user::Entity>::default()
            .action(user::Column::IsActive, false, "Désactiver")
            .action(user::Column::IsActive, true, "Activer");
        assert!(list.validate().is_ok());
    }

    #[test]
    fn the_same_value_on_another_column_is_accepted() {
        let list = ListConfig::<user::Entity>::default()
            .action(user::Column::IsActive, false, "Désactiver")
            .action(user::Column::IsAdmin, false, "Retirer admin");
        assert!(list.validate().is_ok());
    }

    #[test]
    fn a_price_at_the_limit_of_its_column_is_accepted() {
        let list = ListConfig::<product::Entity>::default().action(
            product::Column::Price,
            Decimal::new(9_999_999_999, 2),
            "Prix max",
        );
        assert!(list.validate().is_ok());
    }

    #[test]
    fn a_price_over_its_column_is_refused() {
        let list = ListConfig::<product::Entity>::default().action(
            product::Column::Price,
            Decimal::new(10_000_000_000, 2),
            "Trop cher",
        );
        assert!(list.validate().is_err());
    }

    #[test]
    fn a_text_at_its_length_is_accepted() {
        let list =
            ListConfig::<product::Entity>::default().action(product::Column::Code, "abcde", "Code");
        assert!(list.validate().is_ok());
    }

    #[test]
    fn a_text_over_its_length_is_refused() {
        let list = ListConfig::<product::Entity>::default().action(
            product::Column::Code,
            "abcdef",
            "Code",
        );
        assert!(list.validate().is_err());
    }

    #[test]
    fn a_text_length_is_counted_in_characters() {
        let list =
            ListConfig::<product::Entity>::default().action(product::Column::Code, "ééééé", "Code");
        assert!(list.validate().is_ok());
    }

    #[test]
    fn empty_value_is_kept_as_given() {
        let list = ListConfig::<Entity>::default().empty_value("—");
        assert_eq!(list.empty_value.as_deref(), Some("—"));
    }

    #[test]
    fn the_column_of_an_action_is_listed_as_used_by_action() {
        let list = ListConfig::<user::Entity>::default().action(
            user::Column::IsActive,
            false,
            "Désactiver",
        );
        assert_eq!(list.lists_using("is_active"), ["action"]);
        assert!(list.lists_using("is_admin").is_empty());
    }
}
