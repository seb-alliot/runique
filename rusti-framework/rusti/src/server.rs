//! Server running utilities

use std::net::SocketAddr;
use axum::{Router, Extension};
use axum::routing::get;
use tokio::net::TcpListener;
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::{
    trace::TraceLayer,
    timeout::TimeoutLayer,
    services::ServeDir,
};
use tower_sessions::{Expiry, MemoryStore, SessionManagerLayer};
use std::time::Duration;

use crate::app::RustiApp;
use crate::error::RustiResult;
use crate::middleware_folder::error_handler;

/// Run the server with the app's configuration
pub async fn run_server(app: RustiApp) -> RustiResult<()> {
    let addr = app.config().server_addr();
    run_server_on(app, &addr).await
}

/// Run the server on a specific address
pub async fn run_server_on(app: RustiApp, addr: &str) -> RustiResult<()> {
    // Initialize tracing
    init_tracing(&app);

    // Setup session layer
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(!app.config().debug)
        .with_expiry(Expiry::OnInactivity(Duration::from_secs(86400)));

    // Setup static file serving
    let static_service = ServeDir::new(&app.config().static_root);
    let media_service = ServeDir::new(&app.config().media_root);

    // Build the router
    let router = if let Some(custom_router) = app.router.clone() {
        // Use custom router if provided
        custom_router
            .nest_service(&app.config().static_url, static_service)
            .nest_service(&app.config().media_url, media_service)
    } else {
        // Use default router
        Router::new()
            .nest_service(&app.config().static_url, static_service)
            .nest_service(&app.config().media_url, media_service)
            .route("/", get(default_index))
    }
    // Add extensions
    .layer(Extension(app.tera.clone()))
    .layer(Extension(app.config.clone()));

    // Conditionally add DB extension if available
    #[cfg(feature = "orm")]
    let router = if let Some(db) = app.db.clone() {
        router.layer(Extension(db))
    } else {
        router
    };

    let router = router
        // Add middleware
        .layer(axum::middleware::from_fn(error_handler::handle_errors))
        // Add tower layers
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(TimeoutLayer::new(Duration::from_secs(30)))
        )
        // Add session layer
        .layer(session_layer);

    // Parse address
    let socket_addr: SocketAddr = addr.parse()
        .map_err(|e| crate::error::RustiError::Server(format!("Invalid address: {}", e)))?;

    // Start server
    tracing::info!("🦀 Rusti server starting on http://{}", socket_addr);
    tracing::info!("Debug mode: {}", app.config().debug);

    let listener = TcpListener::bind(&socket_addr).await?;

    // Graceful shutdown
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| crate::error::RustiError::Server(e.to_string()))?;

    tracing::info!("Server shut down gracefully");
    Ok(())
}

/// Initialize tracing/logging
fn init_tracing(app: &RustiApp) {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    let env_filter = if app.config().debug {
        "debug,tower_http=debug,axum=trace"
    } else {
        "info,tower_http=info"
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| env_filter.into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

/// Graceful shutdown signal handler
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C signal");
        },
        _ = terminate => {
            tracing::info!("Received terminate signal");
        },
    }
}

/// Default index page
async fn default_index(
    Extension(tera): Extension<std::sync::Arc<tera::Tera>>,
) -> axum::response::Response {
    let context = tera::Context::new();

    // Try to render index.html if it exists
    if let Ok(html) = tera.render("index.html", &context) {
        return (axum::http::StatusCode::OK, axum::response::Html(html)).into_response();
    }

    // Otherwise show default welcome page
    let html =
    r#"<!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Rusti Framework</title>
            <style>
                body {
                    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                    max-width: 800px;
                    margin: 50px auto;
                    padding: 20px;
                    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                    min-height: 100vh;
                }
                .container {
                    background: white;
                    border-radius: 12px;
                    padding: 40px;
                    box-shadow: 0 20px 60px rgba(0,0,0,0.3);
                }
                h1 { color: #333; margin-bottom: 20px; }
                .crab { font-size: 60px; margin-bottom: 20px; }
                p { color: #666; line-height: 1.6; }
                code {
                    background: #f4f4f4;
                    padding: 2px 6px;
                    border-radius: 3px;
                    font-family: 'Courier New', monospace;
                }
            </style>
        </head>
        <body>
            <div class="container">
                <div class="crab">🦀</div>
                <h1>Welcome to Rusti!</h1>
                <p>Your Rusti web application is running successfully.</p>
                <p>To customize this page, create a template at <code>src/templates/index.html</code></p>
                <p>Happy coding! 🚀</p>
            </div>
        </body>
        </html>"#;

    (axum::http::StatusCode::OK, axum::response::Html(html)).into_response()
}
