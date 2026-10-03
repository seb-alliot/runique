//! Types for admin resources: columns, operations, display configuration.
//
// Resource access permissions are managed in the database via per-group scoped rights
// (eihwaz_groupes_droits: groupe_id + resource_key + CRUD matrix), and not in admin!{}.

/// An operation on an admin resource — what a custom admin route declares it
/// performs (`extra_routes`), to check the matching right.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum CrudOperation {
    List,
    View,
    Create,
    Edit,
    Delete,
}

/// Filters columns displayed in the list view
#[derive(Debug, Clone, Default, serde::Serialize)]
pub enum ColumnFilter {
    /// Displays all columns of the Model (default)
    #[default]
    All,

    /// Displays only specified columns with their labels: (col_sql, displayed_label)
    Include(Vec<(String, String)>),

    /// Displays all columns except specified ones
    Exclude(Vec<String>),
}

/// Declares that a resource is a **scoped child** of another resource, reached
/// only through its parent (`/{parent}/{parent_id}/{child}/...`), never at the
/// top level.
///
/// The child carries its own scoping contract (single source of truth for the FK):
/// - its list is filtered `WHERE {fk_col} = {parent_id}`,
/// - its create/edit forms fix `{fk_col}` to the parent id and hide the picker,
/// - the parent detail screen renders it as an inline sub-list.
///
/// `local_key` distinguishes two child shapes:
/// - `Some(col)` — junction table whose closure-id is `"{parent_id}:{col_value}"`
///   (e.g. `groupes_droits`, keyed by `(groupe_id, resource_key)` → `local_key =
///   Some("resource_key")`). The nested handler rebuilds the composite id from the
///   path so the CRUD closures stay unchanged, exposes only the local key in child
///   URLs, and fixes/hides both `fk_col` and `local_key` in the edit form (both are
///   the row's identity).
/// - `None` — child owns its own primary key. The closure-id is that key; the
///   handler additionally verifies the fetched row's `{fk_col} == parent_id`
///   (IDOR guard) so a child of another parent can't be reached through this one.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ParentScope {
    /// Registry key of the parent resource (e.g. `"groupes"`).
    pub parent_key: &'static str,
    /// Child column holding the FK to the parent (e.g. `"groupe_id"`).
    pub fk_col: &'static str,
    /// `Some(col)` for a composite/junction child keyed by `(fk_col, col)`;
    /// `None` when the child owns its own primary key.
    pub local_key: Option<&'static str>,
}

impl ParentScope {
    /// Whether the child's closure-id is composite `"{parent_id}:{local_key}"`.
    #[must_use]
    pub fn is_composite(&self) -> bool {
        self.local_key.is_some()
    }
}

/// Configuration of resource display in the admin interface
#[derive(Debug, Clone, serde::Serialize)]
pub struct DisplayConfig {
    /// Icon displayed in navigation (icon name, e.g., "user", "file")
    pub icon: Option<String>,

    /// Columns to display in the list view
    pub columns: ColumnFilter,

    /// Number of entries per page
    pub pagination: usize,

    /// Sidebar filters: [(col_sql, displayed_label, limit_per_page)]
    pub list_filter: Vec<(String, String, u64)>,
}

impl DisplayConfig {
    /// Creates a `DisplayConfig` with framework defaults: no icon, all columns
    /// shown, 25 rows per page, no sidebar filters.
    pub fn new() -> Self {
        Self {
            icon: None,
            columns: ColumnFilter::All,
            pagination: 25,
            list_filter: Vec::new(),
        }
    }

    /// Sets the icon shown next to this resource in the admin navigation.
    pub fn icon(mut self, icon: &str) -> Self {
        self.icon = Some(icon.to_string());
        self
    }

    /// Sets the number of rows shown per page in the list view.
    pub fn pagination(mut self, per_page: usize) -> Self {
        self.pagination = per_page;
        self
    }

    /// Restricts the list view to the given `(column, label)` pairs, in order,
    /// replacing the default of showing every column.
    pub fn columns_include(mut self, cols: Vec<(&str, &str)>) -> Self {
        self.columns = ColumnFilter::Include(
            cols.iter()
                .map(|(c, l)| (c.to_string(), l.to_string()))
                .collect(),
        );
        self
    }

    /// Displays every column in the list view except the given ones.
    pub fn columns_exclude(mut self, cols: Vec<&str>) -> Self {
        self.columns = ColumnFilter::Exclude(cols.iter().map(|s| s.to_string()).collect());
        self
    }

    /// Sidebar filters: [("col_sql", "Label", limit_per_page), ...]
    pub fn list_filter(mut self, filters: Vec<(&str, &str, u64)>) -> Self {
        self.list_filter = filters
            .iter()
            .map(|(c, l, limit)| (c.to_string(), l.to_string(), *limit))
            .collect();
        self
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self::new()
    }
}

// Created by the daemon during parsing of src/admin.rs.
//
// generated in target/runique/admin/generated.rs to be type-safe.

/// Metadata of an administrable resource
#[derive(Debug, Clone, serde::Serialize)]
pub struct AdminResource {
    /// Used for routes: /admin/{key}/list
    pub key: &'static str,

    /// We retrieve the paths for model and form
    pub model_path: &'static str,

    pub form_path: &'static str,

    /// Title displayed in the admin interface
    pub title: &'static str,

    /// Display configuration (columns, pagination, icon)
    pub display: DisplayConfig,

    /// Template overrides per operation (None = default Runique template)
    pub template_list: Option<String>,
    pub template_create: Option<String>,
    pub template_edit: Option<String>,
    pub template_detail: Option<String>,
    pub template_delete: Option<String>,

    /// Custom keys injected into the Tera context (defined via extra: {} in admin!{})
    pub extra_context: crate::utils::aliases::StrMap,

    /// If true: injects a random hash into the empty "password" field upon creation.
    /// Automatically set by the daemon when `create_form:` is declared.
    pub inject_password: bool,

    /// FK columns to resolve to a related label in **display** views (list,
    /// detail, delete) — `[(col, fk_table, label_col)]`. Resolution happens at
    /// the display layer (never in `get_fn`/`list_fn`) so edit forms keep the
    /// raw id and pre-select the right option. Emitted by the daemon.
    pub fk_display: Vec<(String, String, String)>,

    /// When set, this resource is a scoped child reached only through its parent
    /// (`/{parent}/{parent_id}/{child}/...`). See [`ParentScope`]. `None` = a
    /// normal top-level resource.
    pub parent_scope: Option<ParentScope>,
}

impl AdminResource {
    /// Creates a new admin resource with framework defaults for display,
    /// templates and FK resolution. Who may use it is decided by the groups'
    /// rights in the database (`eihwaz_groupes_droits`), not here.
    pub fn new(
        key: &'static str,
        model_path: &'static str,
        form_path: &'static str,
        title: &'static str,
    ) -> Self {
        Self {
            key,
            model_path,
            form_path,
            title,
            display: DisplayConfig::new(),
            template_list: None,
            template_create: None,
            template_edit: None,
            template_detail: None,
            template_delete: None,
            extra_context: std::collections::HashMap::new(),
            inject_password: false,
            fk_display: Vec::new(),
            parent_scope: None,
        }
    }

    /// Enables automatic injection of a random hash into the empty "password" field upon creation.
    pub fn inject_password(mut self, v: bool) -> Self {
        self.inject_password = v;
        self
    }

    /// Declares the FK columns resolved to a label in display views.
    /// `specs` = `[(col, fk_table, label_col)]`. Emitted by the daemon.
    #[must_use]
    pub fn fk_display(mut self, specs: Vec<(String, String, String)>) -> Self {
        self.fk_display = specs;
        self
    }

    /// Declares this resource as a scoped child of `parent_key`, reached only
    /// through `/{parent_key}/{parent_id}/{key}/...`. See [`ParentScope`].
    ///
    /// `local_key`: `Some(col)` for a composite/junction child keyed by
    /// `(fk_col, col)`; `None` when the child owns its own primary key.
    #[must_use]
    pub fn parent_scope(
        mut self,
        parent_key: &'static str,
        fk_col: &'static str,
        local_key: Option<&'static str>,
    ) -> Self {
        self.parent_scope = Some(ParentScope {
            parent_key,
            fk_col,
            local_key,
        });
        self
    }

    /// Configures the display of this resource
    pub fn display(mut self, display: DisplayConfig) -> Self {
        self.display = display;
        self
    }

    // ─── Builder methods ──────────────────────────────────────────

    /// Overrides the template used for the list view.
    pub fn template_list(mut self, path: &str) -> Self {
        self.template_list = Some(path.to_string());
        self
    }

    /// Overrides the template used for the create view.
    pub fn template_create(mut self, path: &str) -> Self {
        self.template_create = Some(path.to_string());
        self
    }

    /// Overrides the template used for the edit view.
    pub fn template_edit(mut self, path: &str) -> Self {
        self.template_edit = Some(path.to_string());
        self
    }

    /// Overrides the template used for the detail view.
    pub fn template_detail(mut self, path: &str) -> Self {
        self.template_detail = Some(path.to_string());
        self
    }

    /// Overrides the template used for the delete confirmation view.
    pub fn template_delete(mut self, path: &str) -> Self {
        self.template_delete = Some(path.to_string());
        self
    }

    /// Adds a single custom key/value pair to the Tera context injected for
    /// this resource (mirrors `extra: {}` in `admin!{}`).
    pub fn extra(mut self, key: &str, value: &str) -> Self {
        self.extra_context
            .insert(key.to_string(), value.to_string());
        self
    }

    /// Merges a map of custom key/value pairs into the Tera context injected
    /// for this resource.
    pub fn extra_map(mut self, map: crate::utils::aliases::StrMap) -> Self {
        self.extra_context.extend(map);
        self
    }
}
