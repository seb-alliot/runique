pub mod router_struct;
pub mod security_struct;
pub mod server_struct;
pub mod settings_struct;
pub mod static_struct;

// Ré-exports pratiques
pub use router_struct::RuniqueRouter;
pub use security_struct::SecurityConfig;
pub use server_struct::ServerConfig;
pub use settings_struct::AppSettings;
pub use static_struct::StaticConfig;
