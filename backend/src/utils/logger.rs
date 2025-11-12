use std::{io::Write, panic, str::FromStr, sync::Arc};

use dotenv::var;
use flexi_logger::{DeferredNow, LogSpecification, Logger};
use log::LevelFilter;

#[cfg(feature = "tracing")]
pub fn init() {
    use console_subscriber::ConsoleLayer;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stderr)
        .with_thread_ids(true)
        .with_target(true);

    let console_layer = ConsoleLayer::builder().spawn();

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(console_layer)
        .init();

    log::info!("tracing + console_subscriber initialized");
}

#[cfg(not(feature = "tracing"))]
fn custom_format(
    w: &mut dyn Write,
    now: &mut DeferredNow,
    record: &log::Record,
) -> std::io::Result<()> {
    // TODO: add options to use ISO 8601 format
    let ts = now.format("%H:%M:%S%.3fZ");

    write!(w, "{} {:<5}| {}", ts, record.level(), record.args())
}

#[cfg(not(feature = "tracing"))]
pub fn init() {
    // Only run flexi_logger if tracing is NOT enabled
    let level = match var("RUST_LOG").map(|x| LevelFilter::from_str(&x.to_lowercase())) {
        Ok(Ok(level)) => level,
        _ => LevelFilter::Info,
    };

    let spec = LogSpecification::builder()
        .default(LevelFilter::Off)
        .module("backend", level)
        .build();

    Logger::with(spec)
        .format(custom_format)
        .use_utc()
        .start()
        .unwrap();

    let hook = Arc::new(|info: &panic::PanicHookInfo| {
        if let Some(s) = info.payload().downcast_ref::<&str>() {
            log::error!("Panic occurred: {}", s);
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            log::error!("Panic occurred: {}", s);
        } else {
            log::error!("Panic occurred with unknown payload");
        }

        if let Some(location) = info.location() {
            log::error!(
                "Location: {}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            );
        }
    });
    panic::set_hook(Box::new(move |info| hook(info)));
}
