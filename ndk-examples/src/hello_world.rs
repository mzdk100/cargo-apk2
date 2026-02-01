use {
    android_logger::{Config, init_once},
    log::{LevelFilter, info},
    ndk::trace,
    winit::platform::android::activity::AndroidApp,
};

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
