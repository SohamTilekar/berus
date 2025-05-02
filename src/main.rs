// main.rs
mod browser;
mod network;
mod parser;

use browser::BrowserApp;
use eframe::egui;
use std::env;

fn main() -> Result<(), eframe::Error> {
    // Basic command-line argument handling for initial URL
    let args: Vec<String> = env::args().collect();
    let initial_url = args.get(1).cloned(); // Get the first argument as Option<String>

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0]) // Initial window size
            .with_title("Basic Rust Browser"), // Window title
        ..Default::default()
    };

    // Run the eframe application
    eframe::run_native(
        "Basic Browser", // App name used by OS
        options,
        Box::new(move |cc| {
            // Create the BrowserApp instance, passing the initial URL
            Box::new(BrowserApp::new(cc, initial_url))
        }),
    )
}
