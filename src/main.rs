use eframe::egui;

mod config;
mod ripgrep;
mod gui;

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let config = config::Config::load().unwrap_or_else(|_| config::Config::default_config());

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Ripgrep GUI - Fast Search Tool",
        options,
        Box::new(|cc| Box::new(gui::RipgrepApp::new(cc, config))),
    )
}
