//! Types for admin resources: columns, operations, display configuration.
//
// Resource access permissions are managed in the database via scoped rights
// (eihwaz_droits with resource_key + access_type), and not in admin!{}.
// See: runique::auth::permissions_cache

/// Type of the primary key for an admin resource
#[derive(Debug, Clone, Copy, Default, serde::Serialize)]
pub enum AdminIdType {
    /// i32 (SeaORM default)
    #[default]
    I32,
    /// i64
    I64,
    /// UUID
    Uuid,
}

impl AdminIdType {
    /// Generates the Rust code for conversion from a `String` captured in the route
    pub fn parse_expr(&self) -> &'static str {
        match self {
            AdminIdType::I32 => {
                "let id = id.parse::<i32>().map_err(|_| DbErr::Custom(\"invalid id\".into()))?;"
            }
            AdminIdType::I64 => {
                "let id = id.parse::<i64>().map_err(|_| DbErr::Custom(\"invalid id\".into()))?;"
            }
            AdminIdType::Uuid => {
                "let id = uuid::Uuid::parse_str(&id).map_err(|_| DbErr::Custom(\"invalid id\".into()))?;"
            }
        }
    }
}

/// Granular permissions per CRUD operation
#[derive(Debug, Clone, serde::Serialize)]
pub struct ResourcePermissions {
    // Authorized roles for each operation
    pub list: Vec<String>,

    pub view: Vec<String>,

    pub create: Vec<String>,

    pub edit: Vec<String>,

    pub delete: Vec<String>,
}

impl ResourcePermissions {
    /// Creates uniform permissions for all actions
    pub fn uniform(roles: Vec<String>) -> Self {
        Self {
            list: roles.clone(),
            view: roles.clone(),
            create: roles.clone(),
            edit: roles.clone(),
            delete: roles,
        }
    }

    /// Checks if a role is authorized for a given operation
    pub fn can(&self, operation: CrudOperation, role: &str) -> bool {
        let allowed = match operation {
            CrudOperation::List => &self.list,
            CrudOperation::View => &self.view,
            CrudOperation::Create => &self.create,
            CrudOperation::Edit => &self.edit,
            CrudOperation::Delete => &self.delete,
        };
        allowed.iter().any(|r| r == role)
    }

    /// Checks if any of the provided roles are authorized for an operation
    pub fn can_any(&self, operation: CrudOperation, roles: &[&str]) -> bool {
        roles.iter().any(|role| self.can(operation, role))
    }
}

/// Available CRUD operations on an admin resource
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
    pub fn new() -> Self {
        Self {
            icon: None,
            columns: ColumnFilter::All,
            pagination: 25,
            list_filter: Vec::new(),
        }
    }

    pub fn icon(mut self, icon: &str) -> Self {
        self.icon = Some(icon.to_string());
        self
    }

    pub fn pagination(mut self, per_page: usize) -> Self {
        self.pagination = per_page;
        self
    }

    pub fn columns_include(mut self, cols: Vec<(&str, &str)>) -> Self {
        self.columns = ColumnFilter::Include(
            cols.iter()
                .map(|(c, l)| (c.to_string(), l.to_string()))
                .collect(),
        );
        self
    }

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

    /// CRUD permissions for this resource
    pub permissions: ResourcePermissions,

    /// Display configuration (columns, pagination, icon)
    pub display: DisplayConfig,

    /// Template overrides per operation (None = default Runique template)
    pub template_list: Option<String>,
    pub template_create: Option<String>,
    pub template_edit: Option<String>,
    pub template_detail: Option<String>,
    pub template_delete: Option<String>,

    /// Primary key type (for /{id}/ routes)
    pub id_type: AdminIdType,

    /// Custom keys injected into the Tera context (defined via extra: {} in admin!{})
    pub extra_context: std::collections::HashMap<String, String>,

    /// If true: injects a random hash into the empty "password" field upon creation.
    /// Automatically set by the daemon when `create_form:` is declared.
    pub inject_password: bool,
}

impl AdminResource {
    pub fn new(
        key: &'static str,
        model_path: &'static str,
        form_path: &'static str,
        title: &'static str,
        roles: Vec<String>,
    ) -> Self {
        Self {
            key,
            model_path,
            form_path,
            title,
            permissions: ResourcePermissions::uniform(roles),
            id_type: AdminIdType::I32,
            display: DisplayConfig::new(),
            template_list: None,
            template_create: None,
            template_edit: None,
            template_detail: None,
            template_delete: None,
            extra_context: std::collections::HashMap::new(),
            inject_password: false,
        }
    }

    /// Creates a resource with granular permissions
    pub fn with_permissions(
        key: &'static str,
        model_path: &'static str,
        form_path: &'static str,
        title: &'static str,
        permissions: ResourcePermissions,
    ) -> Self {
        Self {
            key,
            model_path,
            form_path,
            title,
            permissions,
            id_type: AdminIdType::I32,
            display: DisplayConfig::new(),
            template_list: None,
            template_create: None,
            template_edit: None,
            template_detail: None,
            template_delete: None,
            extra_context: std::collections::HashMap::new(),
            inject_password: false,
        }
    }

    /// Enables automatic injection of a random hash into the empty "password" field upon creation.
    pub fn inject_password(mut self, v: bool) -> Self {
        self.inject_password = v;
        self
    }

    /// Configures the display of this resource
    pub fn display(mut self, display: DisplayConfig) -> Self {
        self.display = display;
        self
    }

    /// Returns the list route path for this resource
    ///
    /// Ex: resource.key = "users" → "/users/list"
    pub fn list_route(&self) -> String {
        format!("/{}/list", self.key)
    }

    /// Returns the creation route path for this resource
    pub fn create_route(&self) -> String {
        format!("/{}/create", self.key)
    }

    /// Returns the detail/edit route path for this resource
    pub fn detail_route(&self) -> String {
        format!("/{}/{{id}}", self.key)
    }

    /// Returns the delete route path for this resource
    pub fn delete_route(&self) -> String {
        format!("/{}/{{id}}/delete", self.key)
    }

    // ─── Template resolution (fallback to Runique defaults) ───

    pub fn resolve_list(&self) -> &str {
        self.template_list.as_deref().unwrap_or("admin/list.html")
    }

    pub fn resolve_create(&self) -> &str {
        self.template_create
            .as_deref()
            .unwrap_or("admin/create.html")
    }

    pub fn resolve_edit(&self) -> &str {
        self.template_edit.as_deref().unwrap_or("admin/edit.html")
    }

    pub fn resolve_detail(&self) -> &str {
        self.template_detail
            .as_deref()
            .unwrap_or("admin/detail.html")
    }

    pub fn resolve_delete(&self) -> &str {
        self.template_delete
            .as_deref()
            .unwrap_or("admin/delete.html")
    }

    // ─── Builder methods ──────────────────────────────────────────

    pub fn template_list(mut self, path: &str) -> Self {
        self.template_list = Some(path.to_string());
        self
    }

    pub fn template_create(mut self, path: &str) -> Self {
        self.template_create = Some(path.to_string());
        self
    }

    pub fn template_edit(mut self, path: &str) -> Self {
        self.template_edit = Some(path.to_string());
        self
    }

    pub fn template_detail(mut self, path: &str) -> Self {
        self.template_detail = Some(path.to_string());
        self
    }

    pub fn template_delete(mut self, path: &str) -> Self {
        self.template_delete = Some(path.to_string());
        self
    }

    pub fn id_type(mut self, id_type: AdminIdType) -> Self {
        self.id_type = id_type;
        self
    }

    pub fn extra(mut self, key: &str, value: &str) -> Self {
        self.extra_context
            .insert(key.to_string(), value.to_string());
        self
    }

    pub fn extra_map(mut self, map: std::collections::HashMap<String, String>) -> Self {
        self.extra_context.extend(map);
        self
    }
}
