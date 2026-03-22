// Créée par le daemon lors du parsing de `src/admin.rs` :
//
//   admin! {
//       users: users::Model => RegisterForm {
//           title: "Utilisateurs",
//           permissions: ["admin"]
//       }
//   }

// Chaque opération CRUD peut avoir ses propres rôles autorisés.
// Permet une sécurité fine sans sacrifier la lisibilité.
//
// Si une seule liste est fournie dans admin! (permissions: ["admin"]),
// elle s'applique à toutes les opérations.
//
// Exemple avancé :
//   permissions: {
//       list:   ["admin", "staff"],
//       view:   ["admin", "staff"],
//       create: ["admin"],
//       edit:   ["admin"],
//       delete: ["admin"],
//   }

/// Type de la clé primaire d'une ressource admin
#[derive(Debug, Clone, Copy, Default, serde::Serialize)]
pub enum AdminIdType {
    /// i32 (défaut SeaORM)
    #[default]
    I32,
    /// i64
    I64,
    /// UUID
    Uuid,
}

impl AdminIdType {
    /// Génère le code Rust de conversion depuis un `String` capturé dans la route
    pub fn parse_expr(&self) -> &'static str {
        match self {
            AdminIdType::I32 => {
                "let id = id.parse::<i32>().map_err(|_| DbErr::Custom(\"id invalide\".into()))?;"
            }
            AdminIdType::I64 => {
                "let id = id.parse::<i64>().map_err(|_| DbErr::Custom(\"id invalide\".into()))?;"
            }
            AdminIdType::Uuid => {
                "let id = uuid::Uuid::parse_str(&id).map_err(|_| DbErr::Custom(\"id invalide\".into()))?;"
            }
        }
    }
}

/// Permissions granulaires par opération CRUD
#[derive(Debug, Clone, serde::Serialize)]
pub struct ResourcePermissions {
    // Rôles autorisés pour chaque opération
    pub list: Vec<String>,

    pub view: Vec<String>,

    pub create: Vec<String>,

    pub edit: Vec<String>,

    pub delete: Vec<String>,
}

impl ResourcePermissions {
    /// Crée des permissions uniformes a toutes les actions
    pub fn uniform(roles: Vec<String>) -> Self {
        Self {
            list: roles.clone(),
            view: roles.clone(),
            create: roles.clone(),
            edit: roles.clone(),
            delete: roles,
        }
    }

    /// Vérifie si un rôle est autorisé pour une opération donnée
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

    /// Vérifie si l'un des rôles fournis est autorisé pour une opération
    pub fn can_any(&self, operation: CrudOperation, roles: &[&str]) -> bool {
        roles.iter().any(|role| self.can(operation, role))
    }
}

/// Opérations CRUD disponibles sur une ressource admin
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum CrudOperation {
    List,
    View,
    Create,
    Edit,
    Delete,
}

/// Filtre les colonnes affichées dans la vue liste
#[derive(Debug, Clone, Default, serde::Serialize)]
pub enum ColumnFilter {
    /// Affiche toutes les colonnes du Model (défaut)
    #[default]
    All,

    /// Affiche uniquement les colonnes spécifiées avec leurs labels : (col_sql, label_affiché)
    Include(Vec<(String, String)>),

    /// Affiche toutes les colonnes sauf celles spécifiées
    Exclude(Vec<String>),
}

/// Configuration de l'affichage d'une ressource dans l'interface admin
#[derive(Debug, Clone, serde::Serialize)]
pub struct DisplayConfig {
    /// Icône affichée dans la navigation (nom d'icône, ex: "user", "file")
    pub icon: Option<String>,

    /// Colonnes à afficher dans la vue liste
    pub columns: ColumnFilter,

    /// Nombre d'entrées par page
    pub pagination: usize,

    /// Filtres sidebar : [(col_sql, label_affiché)]
    pub list_filter: Vec<(String, String)>,
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

    /// Filtres sidebar : [("col_sql", "Label"), ...]
    pub fn list_filter(mut self, filters: Vec<(&str, &str)>) -> Self {
        self.list_filter = filters
            .iter()
            .map(|(c, l)| (c.to_string(), l.to_string()))
            .collect();
        self
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self::new()
    }
}

// Créée par le daemon lors du parsing de src/admin.rs.
//
// généré dans target/runique/admin/generated.rs d'être type-safe.

/// Métadonnées d'une ressource administrable
#[derive(Debug, Clone, serde::Serialize)]
pub struct AdminResource {
    /// Utilisée pour les routes : /admin/{key}/list
    pub key: &'static str,

    /// On récupere les chemins pour model et form
    pub model_path: &'static str,

    pub form_path: &'static str,

    /// Titre affiché dans l'interface admin
    pub title: &'static str,

    /// Permissions CRUD de cette ressource
    pub permissions: ResourcePermissions,

    /// Configuration d'affichage (colonnes, pagination, icône)
    pub display: DisplayConfig,

    /// Surcharges de templates par opération (None = template Runique par défaut)
    pub template_list: Option<String>,
    pub template_create: Option<String>,
    pub template_edit: Option<String>,
    pub template_detail: Option<String>,
    pub template_delete: Option<String>,

    /// Type de la clé primaire (pour les routes /{id}/)
    pub id_type: AdminIdType,

    /// Clés custom injectées dans le contexte Tera (définies via extra: {} dans admin!{})
    pub extra_context: std::collections::HashMap<String, String>,
}

impl AdminResource {
    /// Crée une ressource avec des permissions uniformes
    ///
    /// Utilisé par le code généré pour les déclarations simples :
    /// ```rust,ignore
    /// admin! {
    ///     users: users::Model => RegisterForm {
    ///         title: "Utilisateurs",
    ///         permissions: ["admin"]
    ///     }
    /// }
    /// ```
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
        }
    }

    /// Crée une ressource avec des permissions granulaires
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
        }
    }

    /// Configure l'affichage de cette ressource
    pub fn display(mut self, display: DisplayConfig) -> Self {
        self.display = display;
        self
    }

    /// Retourne le chemin de la route liste pour cette ressource
    ///
    /// Ex: resource.key = "users" → "/users/list"
    pub fn list_route(&self) -> String {
        format!("/{}/list", self.key)
    }

    /// Retourne le chemin de la route création pour cette ressource
    pub fn create_route(&self) -> String {
        format!("/{}/create", self.key)
    }

    /// Retourne le chemin de la route détail/édition pour cette ressource
    pub fn detail_route(&self) -> String {
        format!("/{}/{{id}}", self.key)
    }

    /// Retourne le chemin de la route suppression pour cette ressource
    pub fn delete_route(&self) -> String {
        format!("/{}/{{id}}/delete", self.key)
    }

    // ─── Résolution de templates (fallback sur défauts Runique) ───

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
