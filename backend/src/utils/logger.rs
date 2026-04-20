use std::{io::IsTerminal, io::Write, panic, str::FromStr, sync::Arc};

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
    let ts = now.format("%H:%M:%S%.3fZ");

    write!(w, "{} {:<5}| {}", ts, record.level(), record.args())
}

#[cfg(not(feature = "tracing"))]
fn iso_format(
    w: &mut dyn Write,
    now: &mut DeferredNow,
    record: &log::Record,
) -> std::io::Result<()> {
    let ts = now.format("%Y-%m-%dT%H:%M:%S%.3fZ");

    write!(w, "{} {:<5}| {}", ts, record.level(), record.args())
}

#[cfg(not(feature = "tracing"))]
pub fn init() {
    let is_tty = std::io::stdout().is_terminal();
    let fmt = if is_tty { custom_format } else { iso_format };

    let level = match var("RUST_LOG").map(|x| LevelFilter::from_str(&x.to_lowercase())) {
        Ok(Ok(level)) => level,
        _ => LevelFilter::Info,
    };

    let spec = LogSpecification::builder()
        .default(LevelFilter::Off)
        .module("backend", level)
        .build();

    Logger::with(spec)
        .format(fmt)
        .use_utc()
        .start()
        .expect("Fail to setup logger");

    log::info!("using log level: {}", level);

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
                "Panic location: {}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            );
        }

        if let Some(thread) = std::thread::current().name() {
            log::error!("Panic thread: {}", thread);
        }
    });
    panic::set_hook(Box::new(move |info| hook(info)));
}
