use crate::error::ErrorContext;
use crate::settings::Settings;
use axum::{
    extract::{Extension, Request},
    http::StatusCode,
    middleware::Next,
    response::{Html, IntoResponse, Response},
};
use std::sync::Arc;
use tera::{Context, Tera};

pub async fn error_handler_middleware(
    Extension(tera): Extension<Arc<Tera>>,
    Extension(config): Extension<Arc<Settings>>,
    request: Request,
    next: Next,
) -> Response {
    let response = next.run(request).await;

    if response.status() == StatusCode::INTERNAL_SERVER_ERROR {
        tracing::error!("Middleware intercepted 500 error");
        if !config.debug {
            return render_500(&tera, &config);
        }
    }
    if response.status() == StatusCode::NOT_FOUND {
        tracing::warn!("Middleware intercepted 404 error");
        if !config.debug {
            return render_404(&tera, &config);
        }
    }
    response
}

pub fn render_template(
    tera: &Tera,
    template: &str,
    context: &Context,
    status: StatusCode,
    config: &Settings,
) -> Response {
    match tera.render(template, context) {
        Ok(html) => (status, Html(html)).into_response(),
        Err(e) => {
            tracing::error!("Template rendering error for '{}': {}", template, e);
            if template == "errors/debug_error.html" {
                tracing::error!("Error template itself failed to render");
                return critical_error_html(&e.to_string(), tera, context, config);
            }
            if config.debug {
                render_debug_error(tera, template, &e, config)
            } else {
                render_production_error(tera, &e, config)
            }
        }
    }
}

fn render_production_error(tera: &Tera, error: &tera::Error, config: &Settings) -> Response {
    let is_not_found = matches!(&error.kind, tera::ErrorKind::TemplateNotFound(_));

    if is_not_found {
        tracing::warn!("Template not found in production mode");
        render_404(tera, config)
    } else {
        tracing::error!("Template rendering error in production mode");
        render_500(tera, config)
    }
}

pub fn render_404(tera: &Tera, config: &Settings) -> Response {
    let mut context = Context::new();
    context.insert("static_rusti", &config.static_rusti_url);

    match tera.render("404", &context) {
        Ok(html) => (StatusCode::NOT_FOUND, Html(html)).into_response(),
        Err(e) => {
            tracing::error!("Failed to render 404 template: {}", e);
            fallback_404_html()
        }
    }
}

pub fn render_500(tera: &Tera, config: &Settings) -> Response {
    let mut context = Context::new();
    context.insert("static_rusti", &config.static_rusti_url);

    match tera.render("500", &context) {
        Ok(html) => (StatusCode::INTERNAL_SERVER_ERROR, Html(html)).into_response(),
        Err(e) => {
            tracing::error!("Failed to render 500 template: {}", e);
            fallback_500_html()
        }
    }
}

fn render_debug_error(
    tera: &Tera,
    template_name: &str,
    error: &tera::Error,
    config: &Settings,
) -> Response {
    let error_ctx = ErrorContext::from_tera_error(error, template_name, tera);

    let mut template_context = match Context::from_serialize(&error_ctx) {
        Ok(ctx) => ctx,
        Err(e) => {
            tracing::error!("Failed to serialize error context: {}", e);
            return critical_error_html(&e.to_string(), tera, &Context::new(), config);
        }
    };

    template_context.insert("static_rusti", &config.static_rusti_url);

    match tera.render("errors/debug_error.html", &template_context) {
        Ok(html) => (StatusCode::INTERNAL_SERVER_ERROR, Html(html)).into_response(),
        Err(fallback_err) => {
            tracing::error!("Fallback template rendering failed: {}", fallback_err);
            critical_error_html(&fallback_err.to_string(), tera, &template_context, config)
        }
    }
}

pub fn render_index(tera: &Tera, context: &Context, config: &Settings) -> Response {
    let mut context = context.clone();
    context.insert("static_rusti", &config.static_rusti_url);

    if let Ok(html) = tera.render("index.html", &context) {
        return (StatusCode::OK, Html(html)).into_response();
    }

    if let Ok(html) = tera.render("base_index", &context) {
        return (StatusCode::OK, Html(html)).into_response();
    }

    fallback_index_html()
}

// --- Fonctions Fallback HTML (Rétablies avec ton contenu original) ---

fn fallback_404_html() -> Response {
    let html = r#"<!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>404 - Page Not Found</title>
            <style>
                * { margin: 0; padding: 0; box-sizing: border-box; }
                body {
                    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                    min-height: 100vh;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    padding: 20px;
                }
                .container {
                    background: white;
                    border-radius: 16px;
                    padding: 60px 40px;
                    max-width: 600px;
                    text-align: center;
                    box-shadow: 0 20px 60px rgba(0,0,0,0.3);
                }
                .icon { font-size: 80px; margin-bottom: 20px; }
                h1 { color: #333; font-size: 32px; margin-bottom: 16px; }
                p { color: #666; font-size: 18px; margin-bottom: 30px; }
            </style>
        </head>
        <body>
            <div class="container">
                <div class="icon">🔍</div>
                <h1>Page Not Found</h1>
                <p>The page you're looking for doesn't exist.</p>
            </div>
        </body>
        </html>"#;

    (StatusCode::NOT_FOUND, Html(html)).into_response()
}

fn fallback_500_html() -> Response {
    let html = r#"<!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>500 - Internal Server Error</title>
            <style>
                * { margin: 0; padding: 0; box-sizing: border-box; }
                body {
                    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                    background: linear-gradient(135deg, #e74c3c 0%, #c0392b 100%);
                    min-height: 100vh;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    padding: 20px;
                }
                .container {
                    background: white;
                    border-radius: 16px;
                    padding: 60px 40px;
                    max-width: 600px;
                    text-align: center;
                    box-shadow: 0 20px 60px rgba(0,0,0,0.3);
                }
                .icon { font-size: 80px; margin-bottom: 20px; }
                h1 { color: #333; font-size: 32px; margin-bottom: 16px; }
                p { color: #666; font-size: 18px; margin-bottom: 30px; }
            </style>
        </head>
        <body>
            <div class="container">
                <div class="icon">😕</div>
                <h1>Something Went Wrong</h1>
                <p>We're sorry, but something unexpected happened.</p>
            </div>
        </body>
        </html>"#;

    (StatusCode::INTERNAL_SERVER_ERROR, Html(html)).into_response()
}

fn fallback_index_html() -> Response {
    let html = r#"<!DOCTYPE html>
        <html lang="fr">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Welcome to Rusti</title>
            <style>
                * { margin: 0; padding: 0; box-sizing: border-box; }
                body {
                    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
                    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                    min-height: 100vh;
                    padding: 20px;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                }
                .container {
                    max-width: 900px;
                    width: 100%;
                    background: white;
                    border-radius: 20px;
                    padding: 50px;
                    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
                    animation: fadeIn 0.6s ease-out;
                }
                @keyframes fadeIn {
                    from { opacity: 0; transform: translateY(20px); }
                    to { opacity: 1; transform: translateY(0); }
                }
                header {
                    text-align: center;
                    margin-bottom: 50px;
                    padding-bottom: 30px;
                    border-bottom: 2px solid #f0f0f0;
                }
                h1 {
                    font-size: 56px;
                    color: #333;
                    margin-bottom: 15px;
                    font-weight: 700;
                    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                    -webkit-background-clip: text;
                    -webkit-text-fill-color: transparent;
                    background-clip: text;
                }
                .subtitle { font-size: 20px; color: #666; line-height: 1.6; }
                main { margin-bottom: 40px; }
                section { margin-bottom: 40px; }
                h2 { font-size: 32px; color: #667eea; margin-bottom: 20px; font-weight: 600; }
                .features ul { list-style: none; padding: 0; }
                .features li {
                    font-size: 18px;
                    color: #555;
                    padding: 15px 20px;
                    margin-bottom: 10px;
                    background: linear-gradient(90deg, #f8f9fa 0%, #ffffff 100%);
                    border-left: 4px solid #667eea;
                    border-radius: 8px;
                    transition: all 0.3s ease;
                }
                .features li:hover {
                    transform: translateX(10px);
                    box-shadow: 0 5px 15px rgba(102, 126, 234, 0.2);
                }
                .info pre {
                    background: #1e1e1e;
                    color: #d4d4d4;
                    padding: 25px;
                    border-radius: 12px;
                    overflow-x: auto;
                    box-shadow: inset 0 2px 10px rgba(0, 0, 0, 0.2);
                }
                .info code {
                    font-family: 'Courier New', Consolas, Monaco, monospace;
                    font-size: 16px;
                    line-height: 1.8;
                    display: block;
                    white-space: pre;
                }
                footer {
                    text-align: center;
                    padding-top: 30px;
                    border-top: 2px solid #f0f0f0;
                    color: #999;
                    font-size: 16px;
                }
                footer p { animation: pulse 2s ease-in-out infinite; }
                @keyframes pulse {
                    0%, 100% { opacity: 0.8; }
                    50% { opacity: 1; }
                }
                @media (max-width: 768px) {
                    .container { padding: 30px; }
                    h1 { font-size: 40px; }
                    h2 { font-size: 24px; }
                    .features li { font-size: 16px; padding: 12px 15px; }
                }
            </style>
        </head>
        <body>
            <div class="container">
                <header>
                    <h1>🦀 Welcome to Rusti</h1>
                    <p class="subtitle">A Rust web framework inspired by Django and built on Axum.</p>
                </header>
                <main>
                    <section class="features">
                        <h2>Caractéristiques</h2>
                        <ul>
                            <li>✨ Inspiré de Django pour une expérience familière</li>
                            <li>⚡ Construit sur Axum pour des performances optimales</li>
                            <li>🛡️ Gestion d'erreur sophistiquée avec pages de debug</li>
                            <li>📝 Support de Tera pour le templating</li>
                            <li>🗄️ Intégration SeaORM optionnelle</li>
                            <li>🔧 Configuration flexible et intuitive</li>
                        </ul>
                    </section>
                    <section class="info">
                        <h2>Commencer</h2>
                        <pre><code>cargo new my-app
cd my-app
cargo add rusti</code></pre>
                    </section>
                </main>
                <footer>
                    <p>Construit avec ❤️ en Rust</p>
                </footer>
            </div>
        </body>
        </html>"#;

    (StatusCode::OK, Html(html)).into_response()
}

fn critical_error_html(error: &str, tera: &Tera, context: &Context, config: &Settings) -> Response {
    let mut context = context.clone();
    context.insert("static_rusti", &config.static_rusti_url);

    if let Ok(html) = tera.render("debug", &context) {
        return (StatusCode::INTERNAL_SERVER_ERROR, Html(html)).into_response();
    }

    let escaped_error = html_escape(error);
    let html = format!(
        r#"<!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Critical Error</title>
            <style>
                body {{
                    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                    background: #f8d7da;
                    color: #721c24;
                    padding: 40px;
                }}
                .container {{
                    background: white;
                    border: 2px solid #f5c6cb;
                    border-radius: 12px;
                    padding: 30px;
                    box-shadow: 0 10px 30px rgba(0,0,0,0.1);
                }}
                h1 {{ margin-bottom: 20px; }}
                pre {{
                    background: #f1f1f1;
                    padding: 20px;
                    border-radius: 8px;
                    overflow-x: auto;
                }}
            </style>
        </head>
        <body>
            <div class="container">
                <h1>Critical Error Occurred</h1>
                <p>An unrecoverable error occurred while processing your request:</p>
                <pre>{}</pre>
            </div>
        </body>
        </html>"#,
        escaped_error
    );
    (StatusCode::INTERNAL_SERVER_ERROR, Html(html)).into_response()
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}
