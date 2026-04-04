//! Type de la clé primaire utilisateur — `i32` par défaut, `i64` avec la feature `big-pk`.

/// Type de la clé primaire utilisateur.
///
/// Par défaut `i32`. Activer la feature `big-pk` pour passer à `i64` :
///
/// ```toml
/// runique = { version = "...", features = ["big-pk"] }
/// ```
#[cfg(feature = "big-pk")]
pub type UserId = i64;

#[cfg(not(feature = "big-pk"))]
pub type UserId = i32;
