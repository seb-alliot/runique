//! Macro pour construire des routes avec GET/POST/PUT/DELETE/PATCH en une seule ligne

#[macro_export]
macro_rules! view {
    // Handler unique qui gère toutes les méthodes
    ($handler:expr) => {
        $crate::axum::routing::get($handler)
            .post($handler)
            .put($handler)
            .delete($handler)
            .patch($handler)
    };
}
