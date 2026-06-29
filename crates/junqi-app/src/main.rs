mod app;
mod screens;
mod widgets;

use app::JunqiApp;

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 850.0])
            .with_min_inner_size([1000.0, 700.0])
            .with_title("军棋 陆战棋"),
        ..Default::default()
    };

    eframe::run_native(
        "军棋",
        options,
        Box::new(|_cc| Ok(Box::new(JunqiApp::default()))),
    )
}
