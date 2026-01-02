//! Macros pour simplifier l'envoi de messages flash
//!
//! # Utilisation
//!
//! Ces macros prennent en premier paramètre la variable `Message`
//! et ensuite le ou les messages à envoyer.
//!
//! # Exemples
//!
//! ```rust,ignore
//! use rusti::prelude::*;
//!
//! async fn create_user(mut message: Message) -> Response {
//!     // Un seul message
//!     success!(message, "Utilisateur créé avec succès");
//!
//!     // Plusieurs messages
//!     success!(message, "Utilisateur créé", "Email envoyé", "Bienvenue !");
//!
//!     // Messages mixtes
//!     success!(message, "Opération réussie");
//!     info!(message, "Vérifiez votre email");
//!
//!     Redirect::to("/users").into_response()
//! }
//! ```

/// Envoie un ou plusieurs messages de succès
///
/// # Syntaxe
/// ```ignore
/// success!(message, "Un message");
/// success!(message, "Message 1", "Message 2", "Message 3");
/// ```
#[macro_export]
macro_rules! success {
    ($msg:expr, $content:expr) => {
        $msg.success($content).await.unwrap()
    };
    ($msg:expr, $first:expr, $($rest:expr),+ $(,)?) => {
        $msg.success($first).await.unwrap();
        $(
            $msg.success($rest).await.unwrap();
        )+
    };
}

/// Envoie un ou plusieurs messages d'erreur
///
/// # Syntaxe
/// ```ignore
/// error!(message, "Un message");
/// error!(message, "Message 1", "Message 2");
/// ```
#[macro_export]
macro_rules! error {
    ($msg:expr, $content:expr) => {
        $msg.error($content).await.unwrap()
    };
    ($msg:expr, $first:expr, $($rest:expr),+ $(,)?) => {
        $msg.error($first).await.unwrap();
        $(
            $msg.error($rest).await.unwrap();
        )+
    };
}

/// Envoie un ou plusieurs messages d'information
///
/// # Syntaxe
/// ```ignore
/// info!(message, "Un message");
/// info!(message, "Message 1", "Message 2");
/// ```
#[macro_export]
macro_rules! info {
    ($msg:expr, $content:expr) => {
        $msg.info($content).await.unwrap()
    };
    ($msg:expr, $first:expr, $($rest:expr),+ $(,)?) => {
        $msg.info($first).await.unwrap();
        $(
            $msg.info($rest).await.unwrap();
        )+
    };
}

/// Envoie un ou plusieurs messages d'avertissement
///
/// # Syntaxe
/// ```ignore
/// warning!(message, "Un message");
/// warning!(message, "Message 1", "Message 2");
/// ```
#[macro_export]
macro_rules! warning {
    ($msg:expr, $content:expr) => {
        $msg.warning($content).await.unwrap()
    };
    ($msg:expr, $first:expr, $($rest:expr),+ $(,)?) => {
        $msg.warning($first).await.unwrap();
        $(
            $msg.warning($rest).await.unwrap();
        )+
    };
}
