use std::sync::RwLock;

/// Registre global des rôles admin déclarés via `admin!{}`.
///
/// Peuplé une seule fois au boot par `admin_register()` (code généré par le daemon).
/// Lecture seule ensuite — pas de contention.
static ADMIN_ROLES: RwLock<Vec<String>> = RwLock::new(Vec::new());

/// Enregistre les rôles admin au boot (appelé par le code généré).
pub fn register_roles(roles: Vec<String>) {
    if let Ok(mut guard) = ADMIN_ROLES.write() {
        *guard = roles;
    }
}

/// Retourne une copie des rôles enregistrés.
pub fn get_roles() -> Vec<String> {
    ADMIN_ROLES.read().map(|g| g.clone()).unwrap_or_default()
}
