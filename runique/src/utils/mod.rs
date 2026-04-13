//! Cross-cutting framework utilities — type aliases, constants, i18n, password, CSRF, mailer, CLI…
pub mod aliases;
pub mod cli;
pub mod config;
pub mod constante;
pub mod forms;
pub mod init_error;
pub mod mailer;
pub mod middleware;
pub mod password;
pub mod reset_token;
pub mod resolve_ogimage;

pub mod trad;

pub use aliases::*;
pub use cli::create_new_project;
pub use config::*;
pub use constante::*;
pub use env::is_debug;
pub use forms::*;
pub use init_error::init_logging;
pub use mailer::{Email, MailerConfig, mailer_configured, mailer_init, mailer_init_from_env};
pub use middleware::*;
pub use password::*;
pub use pk::Pk;
pub use resolve_ogimage::resolve_og_image;
