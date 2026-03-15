use tracing_subscriber::{EnvFilter, fmt::format::FmtSpan};

pub fn init_logging() {
    dotenvy::dotenv().ok();

    let level = match std::env::var("DEBUG").as_deref() {
        Ok("true") | Ok("1") => "debug",
        _ => "warn",
    };

    let filter = std::env::var("RUST_LOG")
        .map(EnvFilter::new)
        .unwrap_or_else(|_| EnvFilter::new(level));

    if tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_span_events(FmtSpan::CLOSE)
        .try_init()
        .is_err()
    {
        eprintln!("Logger already initialized");
    }
}
