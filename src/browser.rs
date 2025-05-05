// browser.rs
use crate::html_parser;
use crate::layout::{HtmlNode, HtmlTag, NodeType}; // Import layout definitions
use crate::network;
use eframe::egui;
use std::sync::mpsc;
use std::thread;

// --- Constants for styling and layout ---
const BASE_SIZE: f32 = 16.0; // Default font size
const SMALL_DECREMENT: f32 = 3.0; // How much smaller <small> makes text
const BIG_INCREMENT: f32 = 4.0; // How much larger <big> makes text

// --- NEW: Tab State ---
#[derive(Clone, Debug)]
enum ContentState {
    Idle,
    Loading(String), // URL being loaded
    Error(String),   // Error message
    Loaded {
        url: String,
        root_node: HtmlNode, // Store the parsed HTML tree
    },
}

#[derive(Clone, Debug)]
struct TabState {
    id: usize, // Unique identifier for the tab
    title: String,
    url_input: String, // URL currently in the address bar for this tab
    content_state: ContentState,
}

impl TabState {
    fn new(id: usize) -> Self {
        TabState {
            id,
            title: "New Tab".to_string(),
            url_input: "".to_string(),
            content_state: ContentState::Idle,
        }
    }

    // Helper to get the display title
    fn display_title(&self) -> &str {
        match &self.content_state {
            ContentState::Loading(_) => "Loading...",
            ContentState::Loaded { root_node, url, .. } => {
                // Try to find title in head
                if let Some(head) = root_node.children.get(0) {
                    // Assuming cleanup_tree puts head first
                    if matches!(head.node_type, NodeType::Element(HtmlTag::Head)) {
                        for node in &head.children {
                            if let NodeType::Element(HtmlTag::Title) = node.node_type {
                                if let Some(text_node) = node.children.get(0) {
                                    if let NodeType::Text(text) = &text_node.node_type {
                                        let trimmed = text.trim();
                                        if !trimmed.is_empty() {
                                            // TODO: Cache this title?
                                            // For now, we just return a reference, which is tricky
                                            // Let's update the TabState title instead when loaded
                                            // return trimmed; // Cannot return local reference
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                // Fallback title
                if !self.title.is_empty() && self.title != "New Tab" {
                    &self.title
                } else if !url.is_empty() {
                    url // Fallback to URL if title not found/set
                } else {
                    &self.title // Default "New Tab"
                }
            }
            _ => &self.title,
        }
    }

    // Update title from HTML <title> tag
    fn update_title_from_node(&mut self, root_node: &HtmlNode) {
        if let Some(head) = root_node.children.get(0) {
            // Assuming cleanup_tree puts head first
            if matches!(head.node_type, NodeType::Element(HtmlTag::Head)) {
                for node in &head.children {
                    if let NodeType::Element(HtmlTag::Title) = node.node_type {
                        if let Some(text_node) = node.children.get(0) {
                            if let NodeType::Text(text) = &text_node.node_type {
                                let trimmed = text.trim();
                                if !trimmed.is_empty() {
                                    self.title = trimmed.to_string();
                                    return; // Found title
                                }
                            }
                        }
                    }
                }
            }
        }
        // If no title found, maybe use URL? Or keep existing?
        if let ContentState::Loaded { url, .. } = &self.content_state {
            if !url.is_empty() {
                self.title = url.clone();
            }
        }
    }
}

pub struct BrowserApp {
    tabs: Vec<TabState>,
    active_tab_index: usize,
    next_tab_id: usize,
    // Channel now sends Result<(tab_id, url, raw_body, root_node), (tab_id, url, error_msg)>
    network_receiver:
        mpsc::Receiver<Result<(usize, String, String, HtmlNode), (usize, String, String)>>,
    network_sender:
        mpsc::Sender<Result<(usize, String, String, HtmlNode), (usize, String, String)>>,
}

impl BrowserApp {
    pub fn new(cc: &eframe::CreationContext<'_>, initial_url: Option<String>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::light());

        let (sender, receiver) = mpsc::channel();

        let initial_tab_id = 0;
        let mut initial_tab = TabState::new(initial_tab_id);
        let next_tab_id = 1; // Start next ID from 1

        if let Some(url) = initial_url {
            if !url.is_empty() {
                initial_tab.url_input = url;
                // Loading will be triggered in the first update if url_input is set
            }
        }

        let mut app = Self {
            tabs: vec![initial_tab],
            active_tab_index: 0,
            next_tab_id,
            network_receiver: receiver,
            network_sender: sender,
        };

        // Trigger initial load if URL was provided
        if !app.tabs[0].url_input.is_empty() {
            app.start_loading(0, app.tabs[0].url_input.clone());
        }

        app
    }

    fn get_active_tab_mut(&mut self) -> Option<&mut TabState> {
        self.tabs.get_mut(self.active_tab_index)
    }

    fn start_loading(&mut self, tab_index: usize, url_str: String) {
        if let Some(tab) = self.tabs.get_mut(tab_index) {
            if !url_str.starts_with("http://") && !url_str.starts_with("https://") {
                // Basic check, URL::new does more validation
                if !url_str.starts_with("file://") {
                    // Allow file URLs if needed later
                    tab.content_state =
                        ContentState::Error(format!("URL must start with http:// or https://"));
                    tab.url_input = url_str; // Update input even on error
                    return;
                }
            }

            tab.content_state = ContentState::Loading(url_str.clone());
            tab.url_input = url_str.clone(); // Update input when loading starts
            tab.title = url_str.chars().take(20).collect(); // Temporary title

            let sender = self.network_sender.clone();
            let url_to_load = url_str;
            let tab_id = tab.id; // Send tab ID, not index

            thread::spawn(move || {
                match network::load_url(&url_to_load) {
                    // --- MODIFIED: Parse HTML and send root node ---
                    Ok(body) => {
                        // Use the robust parser
                        let root_node = html_parser::parse_html(&body);
                        // Optionally print the tree for debugging
                        html_parser::print_tree(&root_node);
                        sender
                            .send(Ok((tab_id, url_to_load, body, root_node))) // Send tab_id, url, body, node
                            .unwrap_or_else(|e| eprintln!("Failed to send success result: {}", e));
                    }
                    Err(e) => {
                        sender
                            .send(Err((tab_id, url_to_load, e.to_string())))
                            .unwrap_or_else(|e| eprintln!("Failed to send error result: {}", e));
                    }
                }
            });
        } else {
            eprintln!("Attempted to load URL for invalid tab index: {}", tab_index);
        }
    }

    fn add_new_tab(&mut self) {
        let new_tab_id = self.next_tab_id;
        self.next_tab_id += 1;
        self.tabs.push(TabState::new(new_tab_id));
        self.active_tab_index = self.tabs.len() - 1; // Activate the new tab
    }

    // Find tab index by tab ID
    fn find_tab_index_by_id(&self, tab_id: usize) -> Option<usize> {
        self.tabs.iter().position(|tab| tab.id == tab_id)
    }
}

impl eframe::App for BrowserApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // --- Receive Network Results ---
        match self.network_receiver.try_recv() {
            Ok(Ok((tab_id, loaded_url, _, root_node))) => {
                if let Some(index) = self.find_tab_index_by_id(tab_id) {
                    if let Some(tab) = self.tabs.get_mut(index) {
                        tab.content_state = ContentState::Loaded {
                            url: loaded_url,
                            root_node: root_node.clone(), // Clone the node into the state
                        };
                        // Update tab title from <title> tag
                        tab.update_title_from_node(&root_node);
                    }
                } else {
                    eprintln!("Received network result for unknown tab id: {}", tab_id);
                }
            }
            Ok(Err((tab_id, failed_url, error_msg))) => {
                if let Some(index) = self.find_tab_index_by_id(tab_id) {
                    if let Some(tab) = self.tabs.get_mut(index) {
                        tab.content_state = ContentState::Error(format!(
                            "Failed to load {}: {}",
                            failed_url, error_msg
                        ));
                        tab.title = "Error".to_string();
                    }
                } else {
                    eprintln!("Received network error for unknown tab id: {}", tab_id);
                }
            }
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => {
                eprintln!("Network channel disconnected!");
                // Optionally show an error in the active tab?
                if let Some(tab) = self.get_active_tab_mut() {
                    tab.content_state =
                        ContentState::Error("Internal communication error.".to_string());
                }
            }
        }

        // --- Top Panel: Tab Bar and URL Bar ---
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // Tab Bar
            ui.horizontal(|ui| {
                for (index, tab) in self.tabs.iter().enumerate() {
                    // Use a shorter, potentially truncated title for the tab button itself
                    let tab_display_title = tab.display_title();
                    let truncated_title = if tab_display_title.len() > 20 {
                        format!("{}...", &tab_display_title[..17])
                    } else {
                        tab_display_title.to_string()
                    };

                    if ui
                        .selectable_label(self.active_tab_index == index, truncated_title)
                        .clicked()
                    {
                        self.active_tab_index = index;
                    }
                    // TODO: Add close button per tab
                }
                if ui.button("+").clicked() {
                    self.add_new_tab();
                }
            });

            ui.separator();

            // URL Bar - Operates on the active tab
            if let Some(active_tab) = self.tabs.get_mut(self.active_tab_index) {
                // We’ll store a clone of the URL here if the user hits Enter or clicks Go:
                let mut url_to_load: Option<String> = None;

                ui.horizontal(|ui| {
                    ui.label("URL:");
                    let input = ui.add(
                        egui::TextEdit::singleline(&mut active_tab.url_input)
                            .hint_text("Enter URL (e.g., https://example.com)")
                            .id(egui::Id::new(format!("url_input_{}", active_tab.id)))
                            .desired_width(f32::INFINITY),
                    );

                    // If they pressed Enter:
                    if input.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        // clone just once, into our temp
                        url_to_load = Some(active_tab.url_input.clone());
                    }
                    // If they clicked “Go”:
                    if ui.button("Go").clicked() {
                        url_to_load = Some(active_tab.url_input.clone());
                    }
                });

                // Now that the closure (and its borrows) are done, actually start loading:
                if let Some(url) = url_to_load {
                    self.start_loading(self.active_tab_index, url);
                }
            } else {
                ui.label("No active tab selected."); // Shouldn’t happen if tabs exist
            }
        });

        // --- Central Panel: Content Display for Active Tab ---
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(active_tab) = self.tabs.get(self.active_tab_index) {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    match &active_tab.content_state {
                        ContentState::Idle => {
                            ui.label("Enter a URL above and click 'Go' or press Enter.");
                        }
                        ContentState::Loading(url) => {
                            ui.label(format!("Loading {}...", url));
                            ui.spinner();
                        }
                        ContentState::Error(err_msg) => {
                            ui.colored_label(egui::Color32::RED, err_msg);
                        }
                        // --- MODIFIED: Render from HtmlNode tree ---
                        ContentState::Loaded { root_node, .. } => {
                            // Start rendering the parsed HTML tree
                            let initial_context = RenderContext::default();
                            render_node(ui, root_node, &initial_context);
                        }
                    }
                    // Ensure the scroll area takes up available space
                    ui.allocate_space(ui.available_size());
                });
            } else {
                ui.label("No tabs open."); // Handle case where all tabs might be closed (if closing is added)
            }
        });

        // Request repaint if any tab is loading
        if self
            .tabs
            .iter()
            .any(|tab| matches!(tab.content_state, ContentState::Loading(_)))
        {
            ctx.request_repaint();
        }
    }
}

// --- NEW: Rendering Logic ---

#[derive(Clone, Debug)]
struct RenderContext {
    font_size: f32,
    is_bold: bool,
    is_italic: bool,
    // Add other style attributes as needed (color, text-align, etc.)
}

impl Default for RenderContext {
    fn default() -> Self {
        RenderContext {
            font_size: BASE_SIZE,
            is_bold: false,
            is_italic: false,
        }
    }
}

/// Recursively renders an HtmlNode and its children
fn render_node(ui: &mut egui::Ui, node: &HtmlNode, context: &RenderContext) {
    match &node.node_type {
        NodeType::Text(text) => {
            let trimmed_text = text.trim_matches(|c: char| c.is_whitespace() && c != '\n'); // Keep internal newlines maybe? Trim ends.
            if !trimmed_text.is_empty() {
                let mut rich_text = egui::RichText::new(trimmed_text).size(context.font_size);
                if context.is_bold {
                    rich_text = rich_text.strong();
                }
                if context.is_italic {
                    rich_text = rich_text.italics();
                }
                // Use label for inline behavior within the current layout
                ui.label(rich_text);
            }
        }
        NodeType::Element(tag) => {
            // Create a new context potentially modified by this tag
            let mut child_context = context.clone();
            let mut block_element = false; // Does this element cause line breaks?

            // --- Apply Tag-Specific Styling/Layout ---
            match tag {
                // --- Block Level Elements ---
                HtmlTag::H1
                | HtmlTag::H2
                | HtmlTag::H3
                | HtmlTag::H4
                | HtmlTag::H5
                | HtmlTag::H6 => {
                    child_context.font_size = match tag {
                        HtmlTag::H1 => BASE_SIZE * 2.0,
                        HtmlTag::H2 => BASE_SIZE * 1.5,
                        HtmlTag::H3 => BASE_SIZE * 1.17,
                        HtmlTag::H4 => BASE_SIZE * 1.0,
                        HtmlTag::H5 => BASE_SIZE * 0.83,
                        HtmlTag::H6 => BASE_SIZE * 0.67,
                        _ => context.font_size, // Should not happen
                    };
                    child_context.is_bold = true;
                    block_element = true;
                }
                HtmlTag::P => {
                    block_element = true;
                }
                HtmlTag::Div => {
                    block_element = true;
                    // Div doesn't add default spacing like P or H*
                }
                HtmlTag::Br => {
                    ui.end_row(); // Force a line break immediately
                    return; // No children or further processing needed for <br>
                }
                HtmlTag::Body | HtmlTag::Html => {
                    // These are containers, don't add specific style here
                    // but allow children to render. Body might imply block context.
                    block_element = true;
                }
                HtmlTag::Head | HtmlTag::Title => {
                    // Don't render children of head or title directly
                    return;
                }

                // --- Inline Elements ---
                HtmlTag::Strong | HtmlTag::B => {
                    child_context.is_bold = true;
                }
                HtmlTag::I => {
                    // Assuming 'I' is for italic
                    child_context.is_italic = true;
                }
                HtmlTag::Small => {
                    child_context.font_size = (context.font_size - SMALL_DECREMENT).max(1.0);
                }
                HtmlTag::Big => {
                    child_context.font_size += BIG_INCREMENT;
                }
                HtmlTag::Span => {
                    // Span is neutral, inherits style. Handled by default context passing.
                }
                HtmlTag::Hr => {
                    ui.separator();
                }
                HtmlTag::Custom(_) => {
                    // Handle unknown tags - maybe render as inline? Or block?
                    // Default to inline for now.
                    // eprintln!("Rendering unknown tag: <{}>", name);
                }
                _ => {}
            }

            // --- Render Children ---
            if !node.children.is_empty() {
                // Use a wrapping horizontal layout for children by default
                // Block elements will add ui.end_row() after their content.
                let layout = egui::Layout::left_to_right(egui::Align::TOP).with_main_wrap(true);

                // If it's a block element, render its children vertically or ensure line breaks
                if block_element {
                    // Option 1: Use a vertical layout for children of block elements
                    // ui.vertical(|ui| {
                    //    for child in &node.children {
                    //        render_node(ui, child, &child_context);
                    //    }
                    // });

                    // Option 2: Use the wrapping layout but add end_row after content
                    ui.with_layout(layout, |ui| {
                        for child in &node.children {
                            render_node(ui, child, &child_context);
                        }
                    });
                    // Add line break *after* the block element's content
                    ui.end_row();
                } else {
                    // Inline elements continue in the current layout flow
                    // Render children within the parent's layout context
                    // (The parent might be using the wrapping layout already)
                    for child in &node.children {
                        render_node(ui, child, &child_context);
                    }
                }
            } else if block_element {
                // Handle empty block elements like <p></p> - still add spacing and line break
                ui.end_row();
            }
        }
    }
}
