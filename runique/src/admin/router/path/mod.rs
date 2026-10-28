//! The admin's paths. Each route's pattern is written once, here: the router
//! serves it and every link of a page is filled from it, so a link can't
//! point anywhere the router doesn't answer. Pages get their links built
//! (see `context.rs`), never a route name or a prefix to assemble.
use crate::admin::builder::site::AdminSite;
use crate::admin::router::AdminView;

/// The pattern of every admin route, under the admin's public base.
#[derive(Clone, Debug)]
pub struct AdminPaths {
    pub(crate) login: String,
    pub(crate) dashboard: String,
    pub(crate) dashboard_redirect: String,
    pub(crate) logout: String,
    pub(crate) toggle_template: String,
    pub(crate) list: String,
    pub(crate) detail: String,
    pub(crate) create: String,
    pub(crate) edit: String,
    pub(crate) delete: String,
    pub(crate) bulk: String,
    pub(crate) rank: String,
    pub(crate) reset_password: String,
    pub(crate) rights: String,
    pub(crate) group_select: String,
    pub(crate) history: String,
    pub(crate) history_entry: String,
}

impl AdminPaths {
    /// The paths under `prefix`, the admin's public base (mount prefix included).
    pub(crate) fn new(prefix: &str) -> Self {
        let prefix = prefix.trim_end_matches('/');
        Self {
            login: format!("{prefix}/login"),
            dashboard: format!("{prefix}/"),
            dashboard_redirect: prefix.to_string(),
            logout: format!("{prefix}/logout"),
            toggle_template: format!("{prefix}/toggle-template"),
            list: format!("{prefix}/{{resource}}/list"),
            detail: format!("{prefix}/{{resource}}/{{id}}/detail"),
            create: format!("{prefix}/{{resource}}/create"),
            edit: format!("{prefix}/{{resource}}/{{id}}/edit"),
            delete: format!("{prefix}/{{resource}}/{{id}}/delete"),
            bulk: format!("{prefix}/{{resource}}/bulk"),
            rank: format!("{prefix}/{{resource}}/rank"),
            reset_password: format!("{prefix}/{{resource}}/{{id}}/reset-password"),
            rights: format!("{prefix}/{{resource}}/{{id}}/rights"),
            group_select: format!(
                "{prefix}/{}/rights",
                crate::utils::constante::resource_key::GROUPES
            ),
            history: format!("{prefix}/history"),
            history_entry: format!("{prefix}/history/{{id}}"),
        }
    }

    pub(crate) fn list_of(&self, key: &str) -> String {
        fill(&self.list, &[("resource", key)])
    }

    /// The page of row `id` of model `key`: the route the developer declared
    /// to replace it, else the generated one. A replaced detail answers 404,
    /// so every link and redirect to a row goes through here.
    pub(crate) fn detail_of(&self, site: &AdminSite, key: &str, id: &str) -> String {
        match site.overridden_path(key, AdminView::Detail) {
            // Checked at boot: `{id}` is the route's only parameter.
            Some(path) => fill(path, &[("id", id)]),
            None => fill(&self.detail, &[("resource", key), ("id", id)]),
        }
    }

    pub(crate) fn create_of(&self, key: &str) -> String {
        fill(&self.create, &[("resource", key)])
    }

    pub(crate) fn edit_of(&self, key: &str, id: &str) -> String {
        fill(&self.edit, &[("resource", key), ("id", id)])
    }

    pub(crate) fn delete_of(&self, key: &str, id: &str) -> String {
        fill(&self.delete, &[("resource", key), ("id", id)])
    }

    pub(crate) fn bulk_of(&self, key: &str) -> String {
        fill(&self.bulk, &[("resource", key)])
    }

    pub(crate) fn rank_of(&self, key: &str) -> String {
        fill(&self.rank, &[("resource", key)])
    }

    pub(crate) fn reset_password_of(&self, key: &str, id: &str) -> String {
        fill(&self.reset_password, &[("resource", key), ("id", id)])
    }

    pub(crate) fn rights_of(&self, key: &str, id: &str) -> String {
        fill(&self.rights, &[("resource", key), ("id", id)])
    }

    /// The history, narrowed by `filters` (`resource`, `object`, `user`…).
    pub(crate) fn history_with(&self, filters: &[(&str, &str)]) -> String {
        let query: Vec<String> = filters
            .iter()
            .map(|(name, value)| format!("{name}={}", urlencoding::encode(value)))
            .collect();
        match query.is_empty() {
            true => self.history.clone(),
            false => format!("{}?{}", self.history, query.join("&")),
        }
    }

    pub(crate) fn history_entry_of(&self, id: i64) -> String {
        fill(&self.history_entry, &[("id", &id.to_string())])
    }
}

/// `pattern` with each `{name}` replaced by its value, percent-encoded: a
/// key holding `/`, `?` or `{` stays one segment.
fn fill(pattern: &str, params: &[(&str, &str)]) -> String {
    params
        .iter()
        .fold(pattern.to_string(), |path, (name, value)| {
            path.replace(&format!("{{{name}}}"), &urlencoding::encode(value))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_value_is_encoded_into_one_segment() {
        let paths = AdminPaths::new("/admin");
        assert_eq!(
            paths.edit_of("notes", "a/b?c"),
            "/admin/notes/a%2Fb%3Fc/edit"
        );
        assert_eq!(paths.list_of("notes"), "/admin/notes/list");
    }

    #[test]
    fn a_trailing_slash_of_the_prefix_is_dropped() {
        let paths = AdminPaths::new("/admin/");
        assert_eq!(paths.dashboard, "/admin/");
        assert_eq!(paths.dashboard_redirect, "/admin");
        assert_eq!(paths.history_entry_of(7), "/admin/history/7");
    }

    #[test]
    fn the_history_carries_its_filters_encoded_or_none() {
        let paths = AdminPaths::new("/admin");
        assert_eq!(paths.history_with(&[]), "/admin/history");
        assert_eq!(
            paths.history_with(&[("resource", "notes"), ("object", "a&b")]),
            "/admin/history?resource=notes&object=a%26b"
        );
    }

    #[test]
    fn the_detail_follows_a_declared_override() {
        let paths = AdminPaths::new("/admin");
        let mut site = AdminSite::default();
        assert_eq!(
            paths.detail_of(&site, "notes", "3"),
            "/admin/notes/3/detail"
        );
        site.override_view("notes".into(), AdminView::Detail, "/my/notes/{id}".into());
        assert_eq!(paths.detail_of(&site, "notes", "3"), "/my/notes/3");
        assert_eq!(
            paths.detail_of(&site, "other", "3"),
            "/admin/other/3/detail"
        );
    }
}
