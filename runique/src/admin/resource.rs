// ═══════════════════════════════════════════════════════════════
// AdminResource — Déclaration d'une ressource administrable
// ═══════════════════════════════════════════════════════════════
//
// Une ressource associe un Model SeaORM à un Form Runique/Prisme,
// avec des métadonnées d'affichage et des permissions granulaires.
//
// Créée par le daemon lors du parsing de `src/admin.rs` :
//
//   admin! {
//       users: users::Model => RegisterForm {
//           title: "Utilisateurs",
//           permissions: ["admin"]
//       }
//   }
//
// ═══════════════════════════════════════════════════════════════

// ───────────────────────────────────────────────
// ResourcePermissions — Contrôle CRUD granulaire
// ───────────────────────────────────────────────
//
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
#[derive(Debug, Clone)]
pub struct ResourcePermissions {
    /// Rôles autorisés pour lister les entrées (GET /admin/users)
    pub list: Vec<String>,

    /// Rôles autorisés pour voir une entrée (GET /admin/users/:id)
    pub view: Vec<String>,

    /// Rôles autorisés pour créer une entrée (POST /admin/users)
    pub create: Vec<String>,

    /// Rôles autorisés pour modifier une entrée (PUT /admin/users/:id)
    pub edit: Vec<String>,

    /// Rôles autorisés pour supprimer une entrée (DELETE /admin/users/:id)
    pub delete: Vec<String>,
}

impl ResourcePermissions {
    /// Crée des permissions uniformes — tous les rôles s'appliquent à toutes les opérations
    ///
    /// Cas d'usage : `permissions: ["admin"]` dans le macro admin!
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrudOperation {
    List,
    View,
    Create,
    Edit,
    Delete,
}

// ───────────────────────────────────────────────
// ColumnFilter — Contrôle des colonnes affichées
// ───────────────────────────────────────────────

/// Filtre les colonnes affichées dans la vue liste
#[derive(Debug, Clone, Default)]
pub enum ColumnFilter {
    /// Affiche toutes les colonnes du Model (défaut)
    #[default]
    All,

    /// Affiche uniquement les colonnes spécifiées
    Include(Vec<String>),

    /// Affiche toutes les colonnes sauf celles spécifiées
    Exclude(Vec<String>),
}

// ───────────────────────────────────────────────
// DisplayConfig — Configuration d'affichage
// ───────────────────────────────────────────────

/// Configuration de l'affichage d'une ressource dans l'interface admin
#[derive(Debug, Clone)]
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

// ───────────────────────────────────────────────
// AdminResource — La ressource complète
// ───────────────────────────────────────────────
//
// Contient toutes les métadonnées d'une ressource administrable.
// Créée automatiquement par le daemon lors du parsing de src/admin.rs.
//
// Note : Pas de générique <Model, Form> ici — c'est le rôle du code
// généré dans target/runique/admin/generated.rs d'être type-safe.
// AdminResource est la métadonnée pure (JSON-serializable).

/// Métadonnées d'une ressource administrable
#[derive(Debug, Clone)]
pub struct AdminResource {
    /// Clé unique de la ressource (ex: "users", "blog")
    ///
    /// Utilisée pour les routes : /admin/{key}/list
    pub key: &'static str,

    /// Chemin complet du Model SeaORM (pour diagnostics et génération)
    ///
    /// Ex: "crate::models::users::Model"
    pub model_path: &'static str,

    /// Chemin complet du Form Runique (pour diagnostics et génération)
    ///
    /// Ex: "crate::forms::RegisterForm"
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
