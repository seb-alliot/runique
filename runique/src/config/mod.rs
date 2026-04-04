//! Configuration de l'application — serveur, sécurité, fichiers statiques, routeur.
pub mod app;
pub mod router;
pub mod security;
pub mod server;
pub mod static_files;

pub use app::*;
pub use router::*;
pub use security::*;
pub use server::*;
pub use static_files::*;
