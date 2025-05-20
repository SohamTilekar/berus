// browser.rs
use crate::html_parser::{self};
use crate::layout::{self, HtmlNode, HtmlTag, NodeType}; // Import layout definitions
use crate::network;
use eframe::egui;
use std::sync::mpsc;
use std::thread;

// --- Constants for styling and layout ---
const BASE_SIZE: f32 = 16.0; // Default font size

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
        // ctx.set_debug_on_hover(true);
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
            let mut tab_to_close_index: Option<usize> = None; // Keep track of which tab to close

            // Handle keyboard shortcuts for tab management
            ctx.input_mut(|i| {
                // Ctrl/Cmd + T: New Tab
                if i.consume_key(egui::Modifiers::COMMAND, egui::Key::T)
                    || i.consume_key(egui::Modifiers::CTRL, egui::Key::T)
                {
                    self.add_new_tab();
                }
                // Ctrl/Cmd + W: Close Active Tab
                else if (i.consume_key(egui::Modifiers::COMMAND, egui::Key::W)
                    || i.consume_key(egui::Modifiers::CTRL, egui::Key::W))
                    && !self.tabs.is_empty()
                {
                    tab_to_close_index = Some(self.active_tab_index);
                }
                // Ctrl/Cmd + Q: Close Browser
                else if i.consume_key(egui::Modifiers::COMMAND, egui::Key::Q)
                    || i.consume_key(egui::Modifiers::CTRL, egui::Key::Q)
                {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });

            ui.horizontal(|ui| {
                // Tabs and New Tab button
                for (index, tab) in self.tabs.iter().enumerate() {
                    // Use a shorter, potentially truncated title for the tab button itself
                    let tab_display_title = tab.display_title();
                    let truncated_title = if tab_display_title.len() > 20 {
                        format!("{}...", &tab_display_title[..17])
                    } else {
                        tab_display_title.to_string()
                    };

                    // Use a nested horizontal layout for the tab title and close button
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            if ui
                                .selectable_label(self.active_tab_index == index, truncated_title)
                                .clicked()
                            {
                                self.active_tab_index = index;
                            }

                            // Add close button
                            if ui.small_button("x").clicked() {
                                // Mark this tab for closing after the loop
                                tab_to_close_index = Some(index);
                            }
                        }); // End of nested horizontal layout for tab
                    });
                }

                // Handle tab closing outside the iteration
                if let Some(index_to_close) = tab_to_close_index {
                    self.tabs.remove(index_to_close);

                    // Adjust active_tab_index if the closed tab was active
                    if self.active_tab_index == index_to_close {
                        // If the last tab was closed, select the new last tab (or index 0 if no tabs left)
                        if !self.tabs.is_empty() {
                            self.active_tab_index = self.tabs.len() - 1;
                        } else {
                            // If no tabs are left, index 0 is fine (UI handles empty state)
                            self.active_tab_index = 0;
                        }
                    } else if self.active_tab_index > index_to_close {
                        // If a tab before the active one was closed, decrement the active index
                        self.active_tab_index -= 1;
                    }
                }

                // Add New Tab button
                if ui.button("+").clicked() {
                    self.add_new_tab();
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Close button
                    if ui.button("x").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    // Minimize button
                    if ui.button("-").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                    }
                }); // End of ui.with_layout for window controls
            }); // End of ui.horizontal for tabs and + button and Minimize, Maximize, & Close button

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
            // pull out a reference to the tab once…
            if let Some(tab) = self.tabs.get(self.active_tab_index) {
                match &tab.content_state {
                    ContentState::Idle => {
                        ui.label("Enter a URL above and click 'Go' or press Enter.");
                        return; // <— drop the borrow of `tab` immediately
                    }
                    ContentState::Loading(url) => {
                        ui.label(format!("Loading {}...", url));
                        ui.spinner();
                        return; // <— borrow ends here
                    }
                    ContentState::Error(err) => {
                        ui.colored_label(egui::Color32::RED, err);
                        return; // <— borrow ends here
                    }
                    ContentState::Loaded { root_node, .. } => {
                        // we still have an immutable borrow on `tab` until the end of this block…
                        let body = match root_node.get_body() {
                            Some(b) => b.clone(), // clone it out
                            None => return,
                        };
                        let mut initial_context = RenderContext::default();
                        // now do the scroll area; we only capture `body` (owned) and `self`
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            render_node(self, ui, &body, &mut initial_context);
                            ui.allocate_space(ui.available_size());
                        });
                    }
                }
            } else {
                ui.label("No tabs open.");
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
    text_color: Option<layout::Color>,
    font_size: f32,
    bold: bool,
    week: bool,
    italic: bool,
    strikethrough: bool,
    underline: bool,
    text_style: Option<egui::TextStyle>,
    font_family: Option<egui::FontFamily>,
    href: Option<String>,
    abbr: Option<String>,
}

impl Default for RenderContext {
    fn default() -> Self {
        RenderContext {
            text_color: None,
            font_size: BASE_SIZE,
            bold: false,
            week: false,
            italic: false,
            strikethrough: false,
            underline: false,
            text_style: None,
            font_family: None,
            href: None,
            abbr: None,
        }
    }
}

/// Decide which tags count as “inline” (i.e. should live
/// in a horizontal buffer).  Here we treat raw text
/// and formatting tags (B, I, U, S, W, etc.) as inline;
/// everything else (e.g. DIV, P, custom blocks) is block.
fn is_inline(node: &HtmlNode) -> bool {
    match &node.node_type {
        NodeType::Text(_) => true,
        NodeType::Element(tag) => {
            for (property_name, properties) in node.style.clone() {
                if property_name == "display" {
                    if let layout::StyleProperty::Keyword(display) = properties {
                        if display == "block" {
                            return false;
                        } else if display == "inline" {
                            return true;
                        }
                    }
                }
            }
            matches!(
                tag,
                HtmlTag::B
                    | HtmlTag::I
                    | HtmlTag::U
                    | HtmlTag::S
                    | HtmlTag::W
                    | HtmlTag::A
                    | HtmlTag::Br
                    | HtmlTag::Span
                    | HtmlTag::Strong
                    | HtmlTag::Em
                    | HtmlTag::Abbr
                    | HtmlTag::Small
                    | HtmlTag::Big
                    | HtmlTag::Img
            )
        }
    }
}

fn set_node(
    browser: &mut BrowserApp,
    ui: &mut egui::Ui,
    node: &HtmlNode,
    context: &mut RenderContext,
) -> egui::Frame {
    match &node.node_type {
        NodeType::Text(text) => {
            let mut rich = egui::RichText::new(text).size(context.font_size);
            if context.bold {
                rich = rich.strong();
            }
            if context.week {
                rich = rich.weak();
            }
            if context.italic {
                rich = rich.italics();
            }
            if context.underline {
                rich = rich.underline();
            }
            if context.strikethrough {
                rich = rich.strikethrough();
            }
            if let Some(ts) = &context.text_style {
                rich = rich.text_style(ts.clone());
            }
            if let Some(ff) = &context.font_family {
                rich = rich.family(ff.clone());
            }
            if let Some(c) = &context.text_color {
                rich = rich.color(c.clone().to_ecolor());
            }
            let mut label = egui::Label::new(rich);
            if let Some(_) = &context.href {
                label = label.sense(egui::Sense::click());
            }
            let mut response = ui.add(label);
            if let Some(href) = &context.href {
                response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
                if response.clicked() {
                    browser.add_new_tab();
                    browser.start_loading(browser.active_tab_index, href.clone());
                }
            }
            if let Some(title) = &context.abbr {
                response.on_hover_text(title);
            }
        }
        NodeType::Element(HtmlTag::Br) => ui.end_row(),
        NodeType::Element(HtmlTag::Hr) => {
            ui.separator();
        }
        NodeType::Element(HtmlTag::Big) => {
            context.font_size *= 1.2;
        }
        NodeType::Element(HtmlTag::Small) => {
            context.font_size *= 0.8;
        }
        NodeType::Element(HtmlTag::W) => context.week = true,
        NodeType::Element(HtmlTag::Strong | HtmlTag::B) => context.bold = true,
        NodeType::Element(HtmlTag::Em | HtmlTag::I) => context.italic = true,
        NodeType::Element(HtmlTag::S) => context.strikethrough = true,
        NodeType::Element(HtmlTag::U) => context.underline = true,
        NodeType::Element(HtmlTag::A) => {
            if let Some(href) = node.attributes.get("href") {
                context.text_color = Some(layout::Color::Rgb(127, 127, 255));
                context.underline = true;
                context.href = Some(href.clone());
            }
        }
        NodeType::Element(HtmlTag::Abbr) => {
            if let Some(title) = node.attributes.get("title") {
                context.abbr = Some(title.clone());
            }
        }
        NodeType::Element(HtmlTag::Img) => {
            if let Some(src) = node.attributes.get("src") {
                // Get the image from the network
                let mut image =
                    egui::Image::new(egui::ImageSource::Uri(std::borrow::Cow::Owned(src.clone())));

                // Try parsing width and height from attributes
                let width = node
                    .attributes
                    .get("width")
                    .and_then(|w| w.parse::<f32>().ok());
                let height = node
                    .attributes
                    .get("height")
                    .and_then(|h| h.parse::<f32>().ok());

                // Get the original size to compute aspect ratio if needed
                if let Some(original_size) = image.size() {
                    image = image.fit_to_exact_size(match (width, height) {
                        (Some(w), Some(h)) => egui::Vec2::new(w, h),
                        (Some(w), None) => {
                            let h = w * original_size.y / original_size.x;
                            egui::Vec2::new(w, h)
                        }
                        (None, Some(h)) => {
                            let w = h * original_size.x / original_size.y;
                            egui::Vec2::new(w, h)
                        }
                        (None, None) => original_size,
                    });
                } else {
                    image = image.fit_to_original_size(1.);
                }

                // Apply hover sense if there’s an alt or href attribute
                let has_title =
                    node.attributes.contains_key("alt") || node.attributes.contains_key("title");
                let is_clickable = context.href.is_some();

                if is_clickable {
                    image = image.sense(egui::Sense::click()); // ::click() senses both click & hover
                } else if has_title {
                    image = image.sense(egui::Sense::hover());
                }

                let mut response = ui.add(image);

                if let Some(title) = node.attributes.get("title") {
                    response = response.on_hover_text(egui::RichText::new(title));
                } else if let Some(alt) = node.attributes.get("alt") {
                    response = response.on_hover_text(egui::RichText::new(alt));
                }

                // Handle clicking the image like an anchor
                if let Some(href) = &context.href {
                    response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
                    if response.clicked() {
                        browser.add_new_tab();
                        browser.start_loading(browser.active_tab_index, href.clone());
                    }
                }
            }
        }
        NodeType::Element(tag) => {
            let scale = match tag {
                HtmlTag::H1 => Some(2.0),
                HtmlTag::H2 => Some(1.8),
                HtmlTag::H3 => Some(1.6),
                HtmlTag::H4 => Some(1.4),
                HtmlTag::H5 => Some(1.2),
                HtmlTag::H6 => Some(1.1),
                _ => None,
            };

            if let Some(scale) = scale {
                context.text_style = Some(egui::TextStyle::Heading);
                context.font_size *= scale;
            }
        }
    }
    // Initialize mutable frame properties
    let mut inner_margin = egui::Margin::default();
    let mut outer_margin = egui::Margin::default();
    let mut stroke = egui::Stroke::NONE;
    let mut rounding = egui::CornerRadius::ZERO;
    let mut fill = egui::Color32::TRANSPARENT;

    // Process styles before matching node type
    for (property_name, properties) in node.style.clone() {
        match property_name.as_str() {
            "text-color" | "color" => {
                if let layout::StyleProperty::Color(color) = properties {
                    context.text_color = Some(color.clone());
                }
            }
            "padding" => {
                if let layout::StyleProperty::Length(len) = properties {
                    let value = len.to_egui_value(context.font_size, ui.available_size().x);
                    inner_margin = egui::Margin::same(value as i8);
                }
            }
            "padding-top" => {
                if let layout::StyleProperty::Length(len) = properties {
                    inner_margin.top =
                        len.to_egui_value(context.font_size, ui.available_size().y) as i8;
                }
            }
            "padding-bottom" => {
                if let layout::StyleProperty::Length(len) = properties {
                    inner_margin.bottom =
                        len.to_egui_value(context.font_size, ui.available_size().y) as i8;
                }
            }
            "padding-left" => {
                if let layout::StyleProperty::Length(len) = properties {
                    inner_margin.left =
                        len.to_egui_value(context.font_size, ui.available_size().x) as i8;
                }
            }
            "padding-right" => {
                if let layout::StyleProperty::Length(len) = properties {
                    inner_margin.right =
                        len.to_egui_value(context.font_size, ui.available_size().x) as i8;
                }
            }
            "margin" => {
                if let layout::StyleProperty::Length(len) = properties {
                    let value = len.to_egui_value(context.font_size, ui.available_size().x);
                    outer_margin = egui::Margin::same(value as i8);
                }
            }
            "margin-top" => {
                if let layout::StyleProperty::Length(len) = properties {
                    outer_margin.top =
                        len.to_egui_value(context.font_size, ui.available_size().y) as i8;
                }
            }
            "margin-bottom" => {
                if let layout::StyleProperty::Length(len) = properties {
                    outer_margin.bottom =
                        len.to_egui_value(context.font_size, ui.available_size().y) as i8;
                }
            }
            "margin-left" => {
                if let layout::StyleProperty::Length(len) = properties {
                    outer_margin.left =
                        len.to_egui_value(context.font_size, ui.available_size().x) as i8;
                }
            }
            "margin-right" => {
                if let layout::StyleProperty::Length(len) = properties {
                    outer_margin.right =
                        len.to_egui_value(context.font_size, ui.available_size().x) as i8;
                }
            }
            "border-width" => {
                if let layout::StyleProperty::Length(len) = properties {
                    stroke.width = len.to_egui_value(context.font_size, ui.available_size().x);
                }
            }
            "border-color" => {
                if let layout::StyleProperty::Color(color) = properties {
                    stroke.color = color.to_ecolor();
                }
            }
            "border-radius" => {
                if let layout::StyleProperty::Length(len) = properties {
                    let radius = len.to_egui_value(context.font_size, ui.available_size().x);
                    rounding = egui::CornerRadius::same(radius as u8);
                }
            }
            "border-radius-ne" => {
                if let layout::StyleProperty::Length(len) = properties {
                    let radius = len.to_egui_value(context.font_size, ui.available_size().x);
                    rounding.ne = radius as u8;
                }
            }
            "border-radius-nw" => {
                if let layout::StyleProperty::Length(len) = properties {
                    let radius = len.to_egui_value(context.font_size, ui.available_size().x);
                    rounding.nw = radius as u8;
                }
            }
            "border-radius-se" => {
                if let layout::StyleProperty::Length(len) = properties {
                    let radius = len.to_egui_value(context.font_size, ui.available_size().x);
                    rounding.se = radius as u8;
                }
            }
            "border-radius-sw" => {
                if let layout::StyleProperty::Length(len) = properties {
                    let radius = len.to_egui_value(context.font_size, ui.available_size().x);
                    rounding.sw = radius as u8;
                }
            }
            "background-color" => {
                if let layout::StyleProperty::Color(color) = properties {
                    fill = color.to_ecolor();
                }
            }
            "text-decoration" => {
                if let layout::StyleProperty::Keyword(keyword) = properties {
                    if keyword == "underline" {
                        context.underline = true;
                    } else if keyword == "nounderline" {
                        context.underline = false;
                    } else if keyword == "strikethrough" {
                        context.strikethrough = true;
                    } else if keyword == "nostrikethrough" {
                        context.strikethrough = false;
                    }
                }
            }
            "font-weight" => {
                if let layout::StyleProperty::Keyword(keyword) = properties {
                    if keyword == "bold" {
                        context.bold = true;
                    } else if keyword == "bolder" {
                        context.font_size *= 1.2;
                    } else if keyword == "lighter" {
                        context.font_size *= 0.8;
                    }
                }
            }
            "font-style" => {
                if let layout::StyleProperty::Keyword(keyword) = properties {
                    if keyword == "normal" {
                        context.bold = false;
                        context.italic = false;
                        context.underline = false;
                        context.strikethrough = false;
                    } else if keyword == "italic" {
                        context.italic = true;
                    } else if keyword == "bold" {
                        context.bold = true;
                    } else if keyword == "underline" {
                        context.underline = true;
                    } else if keyword == "strikethrough" {
                        context.strikethrough = true;
                    }
                }
            }
            _ => {
                // Unhandled property
            }
        }
    }

    // Re-create the frame with calculated properties
    let frame = egui::Frame::default()
        .inner_margin(inner_margin)
        .outer_margin(outer_margin)
        .stroke(stroke)
        .corner_radius(rounding)
        .fill(fill);

    frame
}

fn render_node(
    browser: &mut BrowserApp,
    ui: &mut egui::Ui,
    node: &HtmlNode,
    context: &mut RenderContext,
) {
    let frame = set_node(browser, ui, node, context);

    if frame != egui::Frame::default() {
        frame.show(ui, |ui| {
            ui.vertical(|ui| {
                let mut i = 0;
                while i < node.children.len() {
                    if is_inline(&node.children[i]) {
                        let start = i;
                        while i < node.children.len() && is_inline(&node.children[i]) {
                            i += 1;
                        }
                        let old_item_spacing = ui.style().spacing.item_spacing;
                        ui.style_mut().spacing.item_spacing.x = 7.;
                        ui.horizontal_wrapped(|ui| {
                            for child in &node.children[start..i] {
                                let mut ctx = context.clone();
                                render_inline(browser, ui, child, &mut ctx);
                            }
                        });
                        ui.style_mut().spacing.item_spacing = old_item_spacing;
                    } else {
                        let mut ctx = context.clone();
                        render_node(browser, ui, &node.children[i], &mut ctx);
                        i += 1;
                    }
                }
            });
        });
    } else {
        let mut i = 0;
        while i < node.children.len() {
            if is_inline(&node.children[i]) {
                let start = i;
                while i < node.children.len() && is_inline(&node.children[i]) {
                    i += 1;
                }
                let old_item_spacing = ui.style().spacing.item_spacing;
                ui.style_mut().spacing.item_spacing.x = 7.;
                ui.horizontal_wrapped(|ui| {
                    for child in &node.children[start..i] {
                        let mut ctx = context.clone();
                        render_inline(browser, ui, child, &mut ctx);
                    }
                });
                ui.style_mut().spacing.item_spacing = old_item_spacing;
            } else {
                let mut ctx = context.clone();
                render_node(browser, ui, &node.children[i], &mut ctx);
                i += 1;
            }
        }
    }
}

fn render_inline(
    browser: &mut BrowserApp,
    ui: &mut egui::Ui,
    node: &HtmlNode,
    context: &mut RenderContext,
) {
    let frame = set_node(browser, ui, node, context);

    if frame != egui::Frame::default() {
        frame.show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                let mut i = 0;
                while i < node.children.len() {
                    if is_inline(&node.children[i]) {
                        let mut ctx = context.clone();
                        render_inline(browser, ui, &node.children[i], &mut ctx);
                        i += 1;
                    } else {
                        let start = i;
                        while i < node.children.len() && !is_inline(&node.children[i]) {
                            i += 1;
                        }
                        ui.vertical(|ui| {
                            for child in &node.children[start..i] {
                                let mut ctx = context.clone();
                                render_node(browser, ui, child, &mut ctx);
                            }
                        });
                    }
                }
            });
        });
    } else {
        let mut i = 0;
        while i < node.children.len() {
            if is_inline(&node.children[i]) {
                let mut ctx = context.clone();
                render_inline(browser, ui, &node.children[i], &mut ctx);
                i += 1;
            } else {
                let start = i;
                while i < node.children.len() && !is_inline(&node.children[i]) {
                    i += 1;
                }
                ui.vertical(|ui| {
                    for child in &node.children[start..i] {
                        let mut ctx = context.clone();
                        render_node(browser, ui, child, &mut ctx);
                    }
                });
            }
        }
    }
}
