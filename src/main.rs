// main.rs
mod browser;
mod css_parser;
mod html_parser;
mod layout;
mod network;

use browser::BrowserApp;
use eframe::egui;
use std::env;

fn main() -> Result<(), eframe::Error> {
    // Basic command-line argument handling for initial URL
    let args: Vec<String> = env::args().collect();
    let initial_url = args.get(1).cloned(); // Get the first argument as Option<String>

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_maximized(true)
            .with_title("Berus Browser")
            .with_decorations(false)
            .with_titlebar_shown(false)
            .with_taskbar(false),
        ..Default::default()
    };

    // Run the eframe application
    eframe::run_native(
        "BerusBrowser", // App name used by OS
        options,
        Box::new(move |cc| {
            // Create the BrowserApp instance, passing the initial URL
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(BrowserApp::new(cc, initial_url)))
        }),
    )
}
