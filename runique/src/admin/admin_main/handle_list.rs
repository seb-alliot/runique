use super::format_datetime;
use crate::admin::{
    helper::resource_entry::{ListParams, ResourceEntry, SortDir},
    resource::ColumnFilter,
};
use crate::auth::session::CurrentUser;
use crate::context::template::{AppError, Request};
use crate::errors::error::ErrorContext;
use crate::utils::{
    aliases::AppResult,
    constante::admin_context::list as list_ctx,
    trad::{current_lang, t},
};
use axum::response::Response;
use std::collections::HashMap;

pub(super) struct ListQuery {
    pub page: u64,
    pub sort_by: Option<String>,
    pub sort_dir: SortDir,
    pub search: Option<String>,
    pub column_filters: Vec<(String, String)>,
    pub filter_pages: HashMap<String, u64>,
}

pub(super) async fn handle_list(
    req: &mut Request,
    entry: &ResourceEntry,
    state: &super::PrototypeAdminState,
    query: ListQuery,
    current_user: &CurrentUser,
    is_htmx: bool,
) -> AppResult<Response> {
    if !current_user.can_access_resource(entry.meta.key) {
        return Ok(super::permission_denied_dashboard(&req.notices, &state.config.prefix).await);
    }
    super::inject_context(req, state, entry, current_user);
    let ListQuery {
        page,
        sort_by,
        sort_dir,
        search,
        column_filters,
        filter_pages,
    } = query;
    let page_size = state.config.page_size;
    let offset = page.saturating_sub(1).saturating_mul(page_size);
    let list_params = ListParams {
        offset,
        limit: page_size,
        sort_by: sort_by.clone(),
        sort_dir: sort_dir.clone(),
        search: search.clone(),
        column_filters: column_filters.clone(),
    };

    let (entries_result, count_result, filter_result) = tokio::join!(
        async {
            match &entry.list_fn {
                Some(f) => f(req.engine.db.clone(), list_params).await,
                None => Ok(Vec::new()),
            }
        },
        async {
            match &entry.count_fn {
                Some(f) => f(req.engine.db.clone(), search.clone()).await,
                None => Ok(0u64),
            }
        },
        async {
            match &entry.filter_fn {
                Some(f) => f(req.engine.db.clone(), filter_pages.clone())
                    .await
                    .unwrap_or_else(|e| {
                        if let Some(level) = crate::utils::runique_log::get_log().filter_fn {
                            crate::runique_log!(level, resource = entry.meta.key, error = %e, "filter_fn failed — list returned without sidebar filters");
                        }
                        HashMap::new()
                    }),
                None => HashMap::new(),
            }
        }
    );
    let entries = entries_result.map_err(|e| Box::new(AppError::new(ErrorContext::database(e))))?;
    let count = count_result.map_err(|e| Box::new(AppError::new(ErrorContext::database(e))))?;

    let filter_values: HashMap<String, Vec<String>> = filter_result
        .iter()
        .map(|(k, (vals, _))| (k.clone(), vals.clone()))
        .collect();
    let filter_totals: HashMap<String, u64> = filter_result
        .into_iter()
        .map(|(k, (_, total))| (k, total))
        .collect();

    let total = if entry.count_fn.is_some() {
        count
    } else {
        offset.saturating_add(entries.len() as u64)
    };

    let page_count = total.div_ceil(page_size);
    let page = page.min(page_count.max(1));

    let all_cols: Vec<String> = entries
        .first()
        .and_then(|v| v.as_object())
        .map(|obj| {
            obj.keys()
                .filter(|k| *k != "id" && !k.starts_with("password"))
                .cloned()
                .collect()
        })
        .unwrap_or_default();

    let (visible_columns, mut column_labels): (Vec<String>, HashMap<String, String>) =
        match &entry.meta.display.columns {
            ColumnFilter::All => (all_cols, HashMap::new()),
            ColumnFilter::Include(cols) => {
                let filtered: Vec<(String, String)> = cols
                    .iter()
                    .filter(|(c, _)| all_cols.contains(c))
                    .cloned()
                    .collect();
                let labels = filtered
                    .iter()
                    .map(|(c, l)| (c.clone(), l.clone()))
                    .collect();
                (filtered.into_iter().map(|(c, _)| c).collect(), labels)
            }
            ColumnFilter::Exclude(excluded) => (
                all_cols
                    .into_iter()
                    .filter(|c| !excluded.contains(c))
                    .collect(),
                HashMap::new(),
            ),
        };

    for col in &visible_columns {
        if !column_labels.contains_key(col) {
            let key = format!("permission.col.{col}");
            let translated = t(&key);
            if translated != key.as_str() {
                column_labels.insert(col.clone(), translated.into_owned());
            }
        }
    }

    let safe_sort_by = sort_by
        .filter(|s| s == "id" || visible_columns.contains(s))
        .unwrap_or_default();

    let mut active_filters: HashMap<String, String> = entry
        .meta
        .display
        .list_filter
        .iter()
        .map(|(col, _, _)| (col.clone(), String::new()))
        .collect();
    for (col, val) in &column_filters {
        active_filters.insert(col.clone(), val.clone());
    }

    let filter_qs: String = {
        let mut parts: Vec<String> = column_filters
            .iter()
            .map(|(col, val)| format!("&filter_{}={}", col, urlencoding::encode(val)))
            .collect();
        for (col, page) in &filter_pages {
            if *page > 0 {
                parts.push(format!("&fp_{}={}", col, page));
            }
        }
        parts.concat()
    };

    let base_qs: Vec<String> = {
        let mut parts = vec![];
        if !safe_sort_by.is_empty() {
            parts.push(format!("sort_by={}", safe_sort_by));
        }
        if sort_dir == SortDir::Desc {
            parts.push("sort_dir=desc".to_string());
        }
        if let Some(ref s) = search {
            parts.push(format!("search={}", urlencoding::encode(s)));
        }
        for (col, val) in &column_filters {
            parts.push(format!("filter_{}={}", col, urlencoding::encode(val)));
        }
        parts
    };
    let filter_meta: HashMap<String, serde_json::Value> = entry
        .meta
        .display
        .list_filter
        .iter()
        .map(|(col, _, col_limit)| {
            let cur_page = filter_pages.get(col).copied().unwrap_or(0);
            let total_distinct = filter_totals.get(col).copied().unwrap_or(0);
            let total_pages = total_distinct.div_ceil(*col_limit);
            let total_pages = total_pages.max(1);
            let has_prev = cur_page > 0;
            let has_next = cur_page.saturating_add(1) < total_pages;

            let build_qs = |fp_override: Option<u64>| -> String {
                let mut parts = base_qs.clone();
                for (other_col, other_page) in &filter_pages {
                    if other_col != col && *other_page > 0 {
                        parts.push(format!("fp_{}={}", other_col, other_page));
                    }
                }
                if let Some(fp) = fp_override
                    && fp > 0
                {
                    parts.push(format!("fp_{}={}", col, fp));
                }
                parts.join("&")
            };

            let prev_qs = if has_prev {
                build_qs(Some(cur_page.saturating_sub(1)))
            } else {
                String::new()
            };
            let next_qs = if has_next {
                build_qs(Some(cur_page.saturating_add(1)))
            } else {
                String::new()
            };

            let meta = serde_json::json!({
                "current_page": cur_page,
                "total_pages": total_pages,
                "has_prev": has_prev,
                "has_next": has_next,
                "prev_qs": prev_qs,
                "next_qs": next_qs,
            });
            (col.clone(), meta)
        })
        .collect();
    let entries: Vec<serde_json::Value> = entries
        .into_iter()
        .map(|mut v| {
            format_datetime(&mut v);
            v
        })
        .collect();

    macro_rules! ctx {
        ($($key:expr => $val:expr),* $(,)?) => {
            $( req.context.insert($key, &$val); )*
        };
    }

    ctx! {
        list_ctx::LANG              => current_lang().code(),
        list_ctx::ENTRIES           => entries,
        list_ctx::TOTAL             => total,
        list_ctx::PAGE              => page,
        list_ctx::PAGE_COUNT        => page_count,
        list_ctx::HAS_PREV          => (page > 1),
        list_ctx::HAS_NEXT          => (page < page_count),
        list_ctx::PREV_PAGE         => page.saturating_sub(1),
        list_ctx::NEXT_PAGE         => page.saturating_add(1),
        "current_page"              => "list",
        list_ctx::VISIBLE_COLUMNS   => visible_columns,
        list_ctx::COLUMN_LABELS     => column_labels,
        list_ctx::SORT_BY           => safe_sort_by,
        list_ctx::SORT_DIR          => sort_dir.as_str(),
        list_ctx::SORT_DIR_TOGGLE   => sort_dir.toggle(),
        list_ctx::SEARCH            => search.unwrap_or_default(),
        list_ctx::FILTER_VALUES     => filter_values,
        list_ctx::ACTIVE_FILTERS    => active_filters,
        list_ctx::FILTER_QS         => filter_qs,
        list_ctx::FILTER_META       => filter_meta,
    }

    let htmx_tpl = state.config.templates.htmx.resolve().to_string();
    let template = if is_htmx {
        htmx_tpl.as_str()
    } else {
        entry
            .meta
            .template_list
            .as_deref()
            .unwrap_or_else(|| state.config.templates.list.resolve())
    };
    req.render(template)
}
