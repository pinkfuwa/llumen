use std::{io::Write, str::FromStr};

use dotenv::var;
use flexi_logger::{DeferredNow, LogSpecification, Logger};
use log::LevelFilter;

fn custom_format(
    w: &mut dyn Write,
    now: &mut DeferredNow,
    record: &log::Record,
) -> std::io::Result<()> {
    // TODO: add options to use ISO 8601 format
    let ts = now.format("%H:%M:%S%.3fZ");

    write!(w, "{} {:<5}| {}", ts, record.level(), record.args())
}

pub fn init() {
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
}
