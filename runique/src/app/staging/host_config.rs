//! Configuration des hôtes autorisés passée via closure au builder.
//
// Utilisé exclusivement dans la closure de `with_allowed_hosts` :
//
//   .middleware(|m| {
//       m.with_allowed_hosts(|h| {
//           h.enabled(true)
//            .host("monsite.fr")
//            .host("www.monsite.fr")
//       })
//   })
//
// ACTIVATION :
//   .enabled(true)          → active la validation du header Host
//   .enabled(false)         → désactive (cas rare, escape hatch)
//
// HÔTES :
//   .host("monsite.fr")     → ajoute un hôte exact
//   .hosts(vec![...])       → ajoute plusieurs hôtes d'un coup
//
// WILDCARDS :
//   ".monsite.fr"           → monsite.fr + tous les sous-domaines
//   "*"                     → tout autoriser (désactive la validation de fait)
//
// ═══════════════════════════════════════════════════════════════

/// Configuration des hôtes autorisés, passée via closure à `.with_allowed_hosts(|h| { ... })`.
///
/// Désactivé par défaut — appeler `.enabled(true)` pour activer.
///
/// # Exemple
/// ```rust,ignore
/// .middleware(|m| {
///     m.with_allowed_hosts(|h| {
///         h.enabled(true)
///          .host("monsite.fr")
///          .host("www.monsite.fr")
///     })
/// })
/// ```
///
/// # Wildcard sous-domaines
/// ```rust,ignore
/// m.with_allowed_hosts(|h| {
///     h.enabled(true)
///      .host(".monsite.fr") // monsite.fr + sous-domaines
/// })
/// ```
///
/// # Désactiver — ne pas appeler `.with_allowed_hosts` du tout.
#[derive(Default)]
pub struct HostConfig {
    pub(crate) hosts: Vec<String>,
    pub(crate) enabled: bool,
}

impl HostConfig {
    /// Active ou désactive la validation du header Host.
    ///
    /// Sans appel à `.enabled(true)`, la validation est inactive
    /// même si des hôtes sont définis.
    pub fn enabled(mut self, enable: bool) -> Self {
        self.enabled = enable;
        self
    }

    /// Ajoute un hôte autorisé.
    ///
    /// Peut être chaîné plusieurs fois :
    /// ```rust,ignore
    /// h.host("monsite.fr").host("www.monsite.fr")
    /// ```
    ///
    /// Préfixer par `.` pour autoriser le domaine et tous ses sous-domaines :
    /// ```rust,ignore
    /// h.host(".monsite.fr") // monsite.fr + api.monsite.fr + ...
    /// ```
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.hosts.push(host.into());
        self
    }

    /// Ajoute plusieurs hôtes d'un coup.
    ///
    /// ```rust,ignore
    /// h.hosts(vec!["monsite.fr", "www.monsite.fr"])
    /// ```
    pub fn hosts(mut self, hosts: Vec<impl Into<String>>) -> Self {
        self.hosts.extend(hosts.into_iter().map(Into::into));
        self
    }
}
