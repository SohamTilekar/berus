// browser.rs
use crate::network;
use crate::parser::{self, Token};
use eframe::egui;
use std::sync::mpsc;
use std::thread;

// --- Constants for styling and layout ---
const BASE_SIZE: f32 = 16.0; // Default font size
const SMALL_DECREMENT: f32 = 3.0; // How much smaller <small> makes text
const BIG_INCREMENT: f32 = 4.0; // How much larger <big> makes text
const VSTEP: f32 = 10.0; // Vertical space after a paragraph </p>

// --- MODIFIED: ContentState ---
#[derive(Clone, Debug)]
enum ContentState {
    Idle,
    Loading(String),
    Error(String),
    Loaded {
        url: String,
        raw_body: String,
        tokens: Vec<Token>, // Store tokens instead of parsed_text
    },
}

pub struct BrowserApp {
    url_input: String,
    content_state: ContentState,
    // Channel now sends Result<(url, raw_body, tokens), (url, error_msg)>
    network_receiver: mpsc::Receiver<Result<(String, String, Vec<Token>), (String, String)>>,
    network_sender: mpsc::Sender<Result<(String, String, Vec<Token>), (String, String)>>,
}

impl BrowserApp {
    pub fn new(cc: &eframe::CreationContext<'_>, initial_url: Option<String>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::light());

        let (sender, receiver) = mpsc::channel();

        let mut app = Self {
            url_input: initial_url.clone().unwrap_or_default(),
            content_state: ContentState::Idle,
            network_receiver: receiver,
            network_sender: sender,
        };

        if let Some(url) = initial_url {
            if !url.is_empty() {
                app.start_loading(url);
            }
        }

        app
    }

    fn start_loading(&mut self, url_str: String) {
        if !url_str.starts_with("http://") && !url_str.starts_with("https://") {
            // Basic check, URL::new does more validation
            if !url_str.starts_with("file://") {
                // Allow file URLs if needed later
                self.content_state =
                    ContentState::Error(format!("URL must start with http:// or https://"));
                self.url_input = url_str;
                return;
            }
        }

        self.content_state = ContentState::Loading(url_str.clone());
        self.url_input = url_str.clone();
        let sender = self.network_sender.clone();
        let url_to_load = url_str;

        thread::spawn(move || {
            match network::load_url(&url_to_load) {
                // --- MODIFIED: Send tokens back ---
                Ok(body) => {
                    let tokens = parser::tokenize_html(&body);
                    sender
                        .send(Ok((url_to_load, body, tokens))) // Send url, body, tokens
                        .unwrap_or_else(|e| eprintln!("Failed to send success result: {}", e));
                }
                Err(e) => {
                    sender
                        .send(Err((url_to_load, e.to_string())))
                        .unwrap_or_else(|e| eprintln!("Failed to send error result: {}", e));
                }
            }
        });
    }
}

impl eframe::App for BrowserApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // --- MODIFIED: Receive tokens ---
        match self.network_receiver.try_recv() {
            Ok(Ok((loaded_url, body, tokens))) => {
                // Receive url, body, tokens
                self.content_state = ContentState::Loaded {
                    url: loaded_url,
                    raw_body: body,
                    tokens, // Store tokens
                };
            }
            Ok(Err((failed_url, error_msg))) => {
                self.content_state =
                    ContentState::Error(format!("Failed to load {}: {}", failed_url, error_msg));
            }
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => {
                eprintln!("Network channel disconnected!");
                self.content_state =
                    ContentState::Error("Internal communication error.".to_string());
            }
        }

        // --- Top Panel: URL Bar (unchanged) ---
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("URL:");
                let input_response = ui.add(
                    egui::TextEdit::singleline(&mut self.url_input)
                        .hint_text("Enter URL (e.g., https://example.com)")
                        .desired_width(f32::INFINITY),
                );

                if input_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.start_loading(self.url_input.clone());
                }

                if ui.button("Go").clicked() {
                    self.start_loading(self.url_input.clone());
                }
            });
        });

        // --- Central Panel: Content Display (MODIFIED) ---
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                match &self.content_state {
                    ContentState::Idle => {
                        ui.label("Enter a URL above and click 'Go'.");
                    }
                    ContentState::Loading(url) => {
                        ui.label(format!("Loading {}...", url));
                        ui.spinner();
                    }
                    ContentState::Error(err_msg) => {
                        ui.colored_label(egui::Color32::RED, err_msg);
                    }
                    // --- MODIFIED: Render from tokens ---
                    ContentState::Loaded { tokens, .. } => {
                        // Layout state
                        let mut current_size = BASE_SIZE;
                        let mut is_bold = false;
                        let mut is_italic = false;

                        // Heading sizes relative to base size
                        let h1_size = BASE_SIZE * 2.0;
                        let h2_size = BASE_SIZE * 1.5;
                        let h3_size = BASE_SIZE * 1.17;
                        let h4_size = BASE_SIZE * 1.0;
                        let h5_size = BASE_SIZE * 0.83;
                        let h6_size = BASE_SIZE * 0.67;

                        let layout =
                            egui::Layout::left_to_right(egui::Align::TOP).with_main_wrap(true);

                        // Process tokens and build the UI
                        for token in tokens {
                            match token {
                                Token::Tag(tag_name) => match tag_name.as_str() {
                                    "b" => is_bold = true,
                                    "/b" => is_bold = false,
                                    "i" => is_italic = true,
                                    "/i" => is_italic = false,
                                    "small" => {
                                        current_size = (current_size - SMALL_DECREMENT).max(1.0)
                                    }
                                    "/small" => current_size += SMALL_DECREMENT,
                                    "big" => current_size += BIG_INCREMENT,
                                    "/big" => current_size -= BIG_INCREMENT,
                                    "h1" => {
                                        ui.end_row();
                                        current_size = h1_size;
                                        is_bold = true;
                                    }
                                    "/h1" => {
                                        ui.end_row();
                                        ui.add_space(VSTEP * 2.0);
                                        current_size = BASE_SIZE;
                                        is_bold = false;
                                    }
                                    "h2" => {
                                        ui.end_row();
                                        current_size = h2_size;
                                        is_bold = true;
                                    }
                                    "/h2" => {
                                        ui.end_row();
                                        ui.add_space(VSTEP * 1.5);
                                        current_size = BASE_SIZE;
                                        is_bold = false;
                                    }
                                    "h3" => {
                                        ui.end_row();
                                        current_size = h3_size;
                                        is_bold = true;
                                    }
                                    "/h3" => {
                                        ui.end_row();
                                        ui.add_space(VSTEP * 1.25);
                                        current_size = BASE_SIZE;
                                        is_bold = false;
                                    }
                                    "h4" => {
                                        ui.end_row();
                                        current_size = h4_size;
                                        is_bold = true;
                                    }
                                    "/h4" => {
                                        ui.end_row();
                                        ui.add_space(VSTEP);
                                        current_size = BASE_SIZE;
                                        is_bold = false;
                                    }
                                    "h5" => {
                                        ui.end_row();
                                        current_size = h5_size;
                                        is_bold = true;
                                    }
                                    "/h5" => {
                                        ui.end_row();
                                        ui.add_space(VSTEP);
                                        current_size = BASE_SIZE;
                                        is_bold = false;
                                    }
                                    "h6" => {
                                        ui.end_row();
                                        current_size = h6_size;
                                        is_bold = true;
                                    }
                                    "/h6" => {
                                        ui.end_row();
                                        ui.add_space(VSTEP);
                                        current_size = BASE_SIZE;
                                        is_bold = false;
                                    }
                                    "br" | "br/" => {
                                        ui.end_row();
                                    }
                                    "p" => {
                                        ui.end_row();
                                    }
                                    "/p" => {
                                        ui.end_row();
                                        ui.add_space(VSTEP);
                                    }
                                    _ => {}
                                },
                                Token::Text(text) => {
                                    if text.trim().is_empty() {
                                        continue;
                                    }
                                    let mut rich_text =
                                        egui::RichText::new(text).size(current_size);
                                    if is_bold {
                                        rich_text = rich_text.strong();
                                    }
                                    if is_italic {
                                        rich_text = rich_text.italics();
                                    }

                                    ui.with_layout(layout, |ui| {
                                        ui.label(rich_text);
                                    });
                                    ui.end_row();
                                }
                            }
                        }
                    }
                }
                // Ensure the scroll area takes up available space
                ui.allocate_space(ui.available_size());
            });
        });

        if let ContentState::Loading(_) = self.content_state {
            ctx.request_repaint();
        }
    }
}
