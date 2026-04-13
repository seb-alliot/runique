//! Macro to build routes with GET/POST/PUT/DELETE/PATCH in a single line

#[macro_export]
macro_rules! view {
    // Single handler managing all methods
    ($handler:expr) => {
        $crate::axum::routing::get($handler)
            .post($handler)
            .put($handler)
            .delete($handler)
            .patch($handler)
    };
}
