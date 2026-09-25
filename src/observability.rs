use crate::cli::LogFormat;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub fn init(format: LogFormat, level: Option<&str>) {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(level.unwrap_or("info")))
        .unwrap_or_else(|_| EnvFilter::new("info"));
    match format {
        LogFormat::Json => tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().json())
            .init(),
        LogFormat::Pretty => tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer())
            .init(),
    }
}
