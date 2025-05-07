use android_activity::AndroidApp;
use android_logger::{init_once, Config};
use log::{info, LevelFilter};
use ndk::trace;

#[unsafe(no_mangle)]
fn android_main(_app: AndroidApp) {
    init_once(Config::default().with_max_level(LevelFilter::Info));

    let _trace;
    if trace::is_trace_enabled() {
        _trace = trace::Section::new("ndk-rs example main").unwrap();
    }

    info!("hello world");
    println!("hello world");
}
