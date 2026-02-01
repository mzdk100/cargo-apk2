#![cfg(target_os = "android")]

use {
    android_logger::{Config, init_once},
    eframe::{
        App, NativeOptions,
        egui::{CentralPanel, Context},
        run_native,
    },
    log::LevelFilter,
    winit::platform::android::activity::AndroidApp,
};

pub struct MyApp;

impl App for MyApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Android App");
            ui.label("Hello world!");
        });
    }
}

//noinspection SpellCheckingInspection
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    // Initialize the logger so you can see output in `adb logcat`
    init_once(Config::default().with_max_level(LevelFilter::Info));

    let options = NativeOptions {
        android_app: Some(app),
        ..Default::default()
    };

    run_native("My egui App", options, Box::new(|_cc| Ok(Box::new(MyApp)))).unwrap();
}
