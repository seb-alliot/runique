//! Cache global des permissions utilisateur (droits et groupes) par `Pk`.
use crate::admin::permissions::Groupe;
use crate::utils::pk::Pk;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock};

// ─────────────────────────────────────────────────────────────────────────────
// Cache global des permissions par user_id
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct CachedPermissions {
    pub groupes: Vec<Groupe>,
}

static PERMISSIONS_CACHE: LazyLock<RwLock<HashMap<Pk, Arc<CachedPermissions>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Insère ou met à jour les permissions d'un utilisateur dans le cache.
/// Appelé au login et lors d'un signal de changement de droits.
pub(crate) fn cache_permissions(user_id: Pk, groupes: Vec<Groupe>) {
    if let Ok(mut cache) = PERMISSIONS_CACHE.write() {
        cache.insert(user_id, Arc::new(CachedPermissions { groupes }));
    }
}

/// Retourne les permissions cachées pour un utilisateur.
pub(crate) fn get_permissions(user_id: Pk) -> Option<Arc<CachedPermissions>> {
    PERMISSIONS_CACHE.read().ok()?.get(&user_id).cloned()
}

/// Supprime les permissions d'un utilisateur du cache (logout).
pub(crate) fn evict_permissions(user_id: Pk) {
    if let Ok(mut cache) = PERMISSIONS_CACHE.write() {
        cache.remove(&user_id);
    }
}

/// Vide entièrement le cache (redémarrage, maintenance).
pub(crate) fn clear_cache() {
    if let Ok(mut cache) = PERMISSIONS_CACHE.write() {
        cache.clear();
    }
}
