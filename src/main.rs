// Logging
use chrono::Local;
use tracing::level_filters::LevelFilter;
use tracing::warn;
use tracing_appender::rolling;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;
use tracing_subscriber::fmt::{time, format};

struct LocalTime;

impl time::FormatTime for LocalTime {
    fn format_time(&self, w: &mut format::Writer<'_>) -> std::fmt::Result {
        let now = Local::now();
        write!(w, "{}", now.format("%Y-%m-%d %H:%M:%S%.3f"))
    }
}

#[tokio::main]
async fn main() {
    // .env
    dotenv::dotenv().ok();

    // Logger
    let timern = Local::now().format("%Y-%m-%d %H-%M-%S");
    let log_file: String = timern.to_string()+".log";
    let file_appender = rolling::never("logs", &log_file);
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let log_file_layer = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_target(false)
        .with_span_events(FmtSpan::CLOSE)
        .event_format(tracing_subscriber::fmt::format().with_timer(LocalTime).compact())
        .with_filter(LevelFilter::WARN);

    let terminal_layer = tracing_subscriber::fmt::layer()
        .with_ansi(true)
        .with_target(false)
        .with_span_events(FmtSpan::CLOSE)
        .event_format(tracing_subscriber::fmt::format().with_timer(LocalTime).compact())
        .with_filter(LevelFilter::WARN);

    tracing_subscriber::registry()
        .with(log_file_layer)
        .with(terminal_layer)
        .init();

    warn!("Hello, world!");
}
