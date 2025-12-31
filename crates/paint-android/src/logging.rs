use std::sync::Once;

use tracing_logcat::{LogcatMakeWriter, LogcatTag};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::format::Format;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;

pub mod ffi {
    use jni_fn::jni_fn;

    use super::*;

    #[unsafe(no_mangle)]
    #[jni_fn("site.nyaalex.paint.rust.Logging")]
    pub fn init() {
        static ONCE: Once = Once::new();
        ONCE.call_once(init_once);
    }
}

fn init_once() {
    let tag = LogcatTag::Fixed("Rust".to_string());
    let writer = LogcatMakeWriter::new(tag).expect("Failed to initialize logcat writer");

    let filter_layer = EnvFilter::new("info,paint=trace");

    let fmt_layer = tracing_subscriber::fmt::layer()
        .event_format(Format::default().with_level(false).without_time())
        .with_writer(writer)
        .with_ansi(false);

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();

    std::panic::set_hook(Box::new(tracing_panic::panic_hook));

    tracing::info!("Logging configured");
}
