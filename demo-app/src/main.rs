#[macro_use]
extern crate runique;
mod prelude;
use prelude::*;
mod admin;
mod admins;
mod entities;
mod form_test;
mod formulaire;
mod url;
mod views;

use runique::app::builder::RuniqueAppBuilder as builder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();
    password_init(PasswordConfig::Manual(Manual::Argon2));
    set_lang(Lang::Fr);

    let config: RuniqueConfig = RuniqueConfig::from_env();

    let db_config = DatabaseConfig::from_env()?.min_connections(2).build();
    let db: DatabaseConnection = db_config.connect().await?;

    builder::new(config)
        .routes(url::routes())
        .with_database(db)
        .statics()
        .middleware(|m| {
            m.with_session_memory_limit(5 * 1024 * 1024, 10 * 1024 * 1024)
                .with_session_cleanup_interval(5)
                .with_allowed_hosts(|h| {
                    h.enabled(!is_debug())
                        .host("localhost:3000")
                        .host("127.0.0.1:3000")
                })
                .with_csp(|c| {
                    c.policy(SecurityPolicy::strict())
                        .with_header_security(true)
                        .with_upgrade_insecure(!is_debug())
                        .images(vec!["'self'", "data:"])
                })
        })
        .with_admin(|a| {
            a.site_title("Administration")
                .auth(RuniqueAdminAuth::new())
                .routes(admins::routes("/admin"))
                .templates(|t| t.with_dashboard("admin/test_dashboard.html"))
                .with_state(admins::admin_state())
        })
        .build()
        .await
        .map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) })?
        .run()
        .await?;

    Ok(())
}
