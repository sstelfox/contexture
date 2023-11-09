use std::path::PathBuf;

use pico_args::Arguments;
use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

use contexture::Contexture;

#[tokio::main]
async fn main() {
    let (non_blocking_writer, _guard) = tracing_appender::non_blocking(std::io::stdout());

    let env_filter = EnvFilter::builder()
        .with_default_directive(Level::INFO.into())
        .from_env_lossy();

    let mut log_builder = tracing_subscriber::fmt::layer()
        .compact()
        .with_ansi(false)
        .with_writer(non_blocking_writer);

    if cfg!(debug_assertions) {
        log_builder = log_builder
            .with_file(true)
            .with_ansi(true)
            .with_line_number(true)
            .with_span_events(FmtSpan::ACTIVE);
    }

    let stdout_layer = log_builder.with_filter(env_filter);
    tracing_subscriber::registry().with(stdout_layer).init();

    tracing::info!("starting up dotfile fuse system");

    let mut cli_args = Arguments::from_env();
    let mount_point: PathBuf = cli_args
        .value_from_str("--mount")
        .unwrap_or(PathBuf::from("./mount"));

    if !mount_point.is_dir() {
        tracing::error!("mountpoint must be a directory");
        std::process::exit(1);
    }

    let contexture_fs = Contexture::new(mount_point, None);
    contexture_fs.run().await.expect("to succeed");
}
