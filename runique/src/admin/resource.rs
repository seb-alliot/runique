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

    /// Affiche uniquement les colonnes spécifiées
    Include(Vec<String>),

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
}

impl DisplayConfig {
    pub fn new() -> Self {
        Self {
            icon: None,
            columns: ColumnFilter::All,
            pagination: 25,
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

    pub fn columns_include(mut self, cols: Vec<&str>) -> Self {
        self.columns = ColumnFilter::Include(cols.iter().map(|s| s.to_string()).collect());
        self
    }

    pub fn columns_exclude(mut self, cols: Vec<&str>) -> Self {
        self.columns = ColumnFilter::Exclude(cols.iter().map(|s| s.to_string()).collect());
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
            display: DisplayConfig::new(),
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
            display: DisplayConfig::new(),
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
}
