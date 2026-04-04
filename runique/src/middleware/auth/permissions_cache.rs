//! Cache global des permissions utilisateur (droits et groupes) par `UserId`.
use crate::admin::permissions::{Droit, Groupe};
use crate::utils::pk::UserId;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock};

// ─────────────────────────────────────────────────────────────────────────────
// Cache global des permissions par user_id
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct CachedPermissions {
    pub droits: Vec<Droit>,
    pub groupes: Vec<Groupe>,
}

static PERMISSIONS_CACHE: LazyLock<RwLock<HashMap<UserId, Arc<CachedPermissions>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Insère ou met à jour les permissions d'un utilisateur dans le cache.
/// Appelé au login et lors d'un signal de changement de droits.
pub fn cache_permissions(user_id: UserId, droits: Vec<Droit>, groupes: Vec<Groupe>) {
    if let Ok(mut cache) = PERMISSIONS_CACHE.write() {
        cache.insert(user_id, Arc::new(CachedPermissions { droits, groupes }));
    }
}

/// Retourne les permissions cachées pour un utilisateur.
pub fn get_permissions(user_id: UserId) -> Option<Arc<CachedPermissions>> {
    PERMISSIONS_CACHE.read().ok()?.get(&user_id).cloned()
}

/// Supprime les permissions d'un utilisateur du cache (logout).
pub fn evict_permissions(user_id: UserId) {
    if let Ok(mut cache) = PERMISSIONS_CACHE.write() {
        cache.remove(&user_id);
    }
}

/// Vide entièrement le cache (redémarrage, maintenance).
pub fn clear_cache() {
    if let Ok(mut cache) = PERMISSIONS_CACHE.write() {
        cache.clear();
    }
}
