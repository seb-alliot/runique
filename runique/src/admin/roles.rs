use std::sync::RwLock;

/// Registre global des rôles admin déclarés via `admin!{}`.
///
/// Peuplé une seule fois au boot par `admin_register()` (code généré par le daemon).
/// Lecture seule ensuite — pas de contention.
static ADMIN_ROLES: RwLock<Vec<String>> = RwLock::new(Vec::new());

/// Enregistre les rôles admin au boot (appelé par le code généré).
pub fn register_roles(roles: Vec<String>) {
    match ADMIN_ROLES.write() {
        Ok(mut guard) => *guard = roles,
        Err(_) => {
            if let Some(level) = crate::utils::runique_log::get_log().roles {
                crate::runique_log!(
                    level,
                    "register_roles() : impossible d'acquérir le lock en écriture — rôles non enregistrés"
                );
            }
        }
    }
}

/// Retourne une copie des rôles enregistrés.
pub fn get_roles() -> Vec<String> {
    match ADMIN_ROLES.read() {
        Ok(guard) => guard.clone(),
        Err(_) => {
            if let Some(level) = crate::utils::runique_log::get_log().roles {
                crate::runique_log!(level, "get_roles() : lock poisonné — rôles retournés vides");
            }
            Vec::new()
        }
    }
}
