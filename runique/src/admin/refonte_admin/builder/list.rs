//! The list page of one model.
use crate::db::sql_dialect::{search_condition, text_cast_type_of};
use crate::utils::aliases::StrMap;
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
    pub filters: Vec<(String, String)>,
}

impl ListQuery {
    /// Reads the URL's query string. It only shapes the values: which column
    /// a name may reach is decided later, by `ListConfig`.
    pub(crate) fn from_params(mut params: StrMap, search_max_len: usize) -> Self {
        let mut non_empty = |key: &str| params.remove(key).filter(|v| !v.is_empty());
        let page = non_empty("page").and_then(|p| p.parse().ok()).unwrap_or(1);
        let sort_by = non_empty("sort_by");
        let sort_dir = non_empty("sort_dir");
        let search = non_empty("search").map(|s| s.chars().take(search_max_len).collect());
        let filters = params
            .into_iter()
            .filter_map(|(key, value)| {
                let col = key.strip_prefix("filter_")?.to_string();
                (!value.is_empty()).then_some((col, value))
            })
            .collect();

        Self {
            page,
            sort_by,
            sort_dir,
            search,
            filters,
        }
    }
}

/// Sets `col` to a value fixed here, on the rows checked or on one row.
/// The request only ever names the action, never the value.
pub(crate) struct Action<E: EntityTrait> {
    pub(crate) col: E::Column,
    pub(crate) value: Value,
    pub(crate) label: String,
}

/// Alias of the window column carrying the total on every row.
pub(crate) const TOTAL: &str = "__total";

/// Only what is declared here is shown or accepted from the URL
/// (e.g. `?sort_by=title&sort_dir=desc`); anything else is ignored.
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
            search_help: None,
            empty_value: None,
            actions: Vec::new(),
        }
    }
}

impl<E: EntityTrait> ListConfig<E> {
    pub fn display(mut self, cols: impl IntoIterator<Item = E::Column>) -> Self {
        self.display = cols.into_iter().collect();
        self
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
            lists.push("action")
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
        let cond = filters
            .iter()
            .filter_map(|(name, value)| {
                let col = self.filter.iter().find(|c| c.as_str() == name.as_str())?;
                Some(
                    Expr::col((E::default(), *col))
                        .cast_as(cast.clone())
                        .eq(value.as_str()),
                )
            })
            .fold(Condition::all(), |cond, expr| cond.add(expr));
        (!cond.is_empty()).then_some(cond)
    }
    /// Rows per page: the model's own `per_page`, otherwise the site's.
    pub(crate) fn page_size(&self, site_default: u64) -> u64 {
        std::cmp::max(self.per_page.unwrap_or(site_default), 1)
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
    use super::{ListConfig, ListQuery, decimal_fits};
    use crate::admin::refonte_admin::gate::test_support::fixtures::note::{Column, Entity};
    use crate::auth::user;
    use crate::utils::aliases::StrMap;
    use sea_orm::prelude::Decimal;
    use sea_orm::{Condition, DbBackend, EntityTrait, IdenStatic, Order, QueryFilter, QueryTrait};

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

    fn params(raw: &[(&str, &str)]) -> StrMap {
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
}
