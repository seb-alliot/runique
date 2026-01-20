use crate::runique_body::runique_struct::RuniqueApp;


#[derive(Debug, Clone)]
pub struct GardeFou {
    pub use_csrf: bool,
    pub use_compression: bool,
    pub use_logging: bool,
    pub timeout_seconds: u64,
}


impl Default for GardeFou {
    fn default() -> Self {
        Self {
            use_logging: true,
            use_csrf: true,
            use_compression: false,
            timeout_seconds: 30,
        }
    }
}
// Dans RuniqueApp, on l'utilise pour configurer le router
impl RuniqueApp {
    pub fn apply_security(self) -> Self {
        if self.engine.config.security.csrf_enabled {
            // self.router = self.router.layer(CsrfLayer...);
        }
        self
    }
}