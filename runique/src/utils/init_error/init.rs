use tracing_subscriber::fmt::format::FmtSpan;

pub fn init_logging() {
    if tracing_subscriber::fmt()
        .with_max_level(tracing::Level::ERROR)
        .with_span_events(FmtSpan::CLOSE)
        .try_init()
        .is_err()
    {
        eprintln!("Logger déjà initialisé");
    }
}
