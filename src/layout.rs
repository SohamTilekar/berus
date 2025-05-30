use eframe::egui::Color32;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(1);

pub fn get_next_id() -> usize {
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

#[derive(Debug, Clone)]
pub enum HtmlTag {
    Div,
    Span,
    P, // Paragraph
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
    Strong,
    Em,
    Small,
    Big,
    B,    // bold
    W,    // week
    U,    // Underline
    I,    // italic
    S,    // strike through
    Br,   // breakline
    Hr,   // horizontal ruler
    A,    // Anchor tag
    Abbr, // Abbreviation tag
    Audio,
    Img,
    Table, // Table tag
    Thead,
    Tbody,
    Tfoot,
    Th,      // Table header cell tag
    Td,      // Table data cell tag
    Tr,      // Table row tag
    Caption, // Table caption tag
    Body,
    Head,
    Title,
    Html,
    Script,
    Style,
    Custom(String), // for arbitrary tags
}

#[derive(Debug, Clone)]
pub enum NodeType {
    Element(HtmlTag),
    Text(String),
}

#[derive(Debug, Clone)]
pub enum Length {
    Px(f32),
    Em(f32),
    Rem(f32),
    Percent(f32),
}

impl Length {
    /// Converts the length to an egui value (f32).
    /// Requires `base_font_size` for Em/Rem and `parent_size` for Percent.
    pub fn to_egui_value(&self, base_font_size: f32, parent_size: f32) -> f32 {
        match self {
            Length::Px(px) => *px,
            Length::Em(em) => em * base_font_size,
            Length::Rem(rem) => rem * base_font_size,
            Length::Percent(percent) => percent / 100.0 * parent_size,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Color {
    Rgb(u8, u8, u8),
    Rgba(u8, u8, u8, f32),
    Hsl(u8, u8, u8),
    Hsla(u8, u8, u8, f32),
    Hex(String),
}

impl Color {
    pub fn to_ecolor(self) -> Color32 {
        match self {
            Color::Rgb(r, g, b) => Color32::from_rgb(r, g, b),
            Color::Rgba(r, g, b, a) => {
                let alpha = (a.clamp(0.0, 1.0) * 255.0).round() as u8;
                Color32::from_rgba_premultiplied(r, g, b, alpha)
            }
            Color::Hsl(h, s, l) => {
                let (r, g, b) = hsl_to_rgb(h, s, l);
                Color32::from_rgb(r, g, b)
            }
            Color::Hsla(h, s, l, a) => {
                let (r, g, b) = hsl_to_rgb(h, s, l);
                let alpha = (a.clamp(0.0, 1.0) * 255.0).round() as u8;
                Color32::from_rgba_premultiplied(r, g, b, alpha)
            }
            Color::Hex(s) => parse_hex_color(&s),
        }
    }
}

/// Converts HSL (0–255 each) to RGB (0–255 each).
fn hsl_to_rgb(h: u8, s: u8, l: u8) -> (u8, u8, u8) {
    let h = (h as f32) / 255.0 * 360.0;
    let s = (s as f32) / 255.0;
    let l = (l as f32) / 255.0;

    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r1, g1, b1) = match h {
        h if h < 60.0 => (c, x, 0.0),
        h if h < 120.0 => (x, c, 0.0),
        h if h < 180.0 => (0.0, c, x),
        h if h < 240.0 => (0.0, x, c),
        h if h < 300.0 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    let r = ((r1 + m) * 255.0).round() as u8;
    let g = ((g1 + m) * 255.0).round() as u8;
    let b = ((b1 + m) * 255.0).round() as u8;

    (r, g, b)
}

fn parse_hex_color(hex: &str) -> Color32 {
    let hex = hex.trim_start_matches('#');
    let hex = hex.trim(); // Remove whitespace just in case

    // Helper to parse a component or default to 255
    fn parse_component(s: &str) -> u8 {
        u8::from_str_radix(s, 16).unwrap_or(255)
    }

    let (r, g, b, a) = match hex.len() {
        1 => {
            let c = hex.repeat(2);
            let v = parse_component(&c);
            (v, v, v, 255)
        }
        2 => {
            let v = parse_component(&hex);
            (v, v, v, 255)
        }
        3 => (
            parse_component(&hex[0..1].repeat(2)),
            parse_component(&hex[1..2].repeat(2)),
            parse_component(&hex[2..3].repeat(2)),
            255,
        ),
        4 => (
            parse_component(&hex[0..1].repeat(2)),
            parse_component(&hex[1..2].repeat(2)),
            parse_component(&hex[2..3].repeat(2)),
            parse_component(&hex[3..4].repeat(2)),
        ),
        6 => (
            parse_component(&hex[0..2]),
            parse_component(&hex[2..4]),
            parse_component(&hex[4..6]),
            255,
        ),
        8 => (
            parse_component(&hex[0..2]),
            parse_component(&hex[2..4]),
            parse_component(&hex[4..6]),
            parse_component(&hex[6..8]),
        ),
        _ => {
            // Try to use as much as possible
            let mut digits = hex
                .chars()
                .filter(|c| c.is_ascii_hexdigit())
                .collect::<Vec<_>>();

            while digits.len() < 8 {
                digits.push('F');
            }

            let hex = digits.into_iter().collect::<String>();
            (
                parse_component(&hex[0..2]),
                parse_component(&hex[2..4]),
                parse_component(&hex[4..6]),
                parse_component(&hex[6..8]),
            )
        }
    };

    Color32::from_rgba_premultiplied(r, g, b, a)
}

#[derive(Debug, Clone)]
pub enum StyleProperty {
    Keyword(String),
    Length(Length),
    Color(Color),
}

#[derive(Debug, Clone)]
pub enum Selector {
    Universal,     // * // All
    Class(String), // #message, ...
    Id(String),    // #message-box, ...
    Type(String),  // H1, P, ...
}

#[derive(Debug, Clone)]
pub struct CssRule {
    pub selectors: Vec<Selector>,
    pub properties: HashMap<String, StyleProperty>,
}

#[derive(Debug, Clone)]
pub struct HtmlNode {
    pub node_type: NodeType,
    pub attributes: HashMap<String, String>,
    pub style: HashMap<String, StyleProperty>, // property_name: value // curently not suported
    pub children: Vec<HtmlNode>,
}

impl HtmlNode {
    // Helper constructor for elements
    pub fn new_element(
        tag: HtmlTag,
        attributes: HashMap<String, String>,
        children: Vec<HtmlNode>,
    ) -> Self {
        HtmlNode {
            node_type: NodeType::Element(tag),
            style: HashMap::new(),
            attributes,
            children,
        }
    }

    // Helper constructor for text
    pub fn new_text(text: String) -> Self {
        HtmlNode {
            node_type: NodeType::Text(text),
            style: HashMap::new(),
            attributes: HashMap::new(),
            children: Vec::new(),
        }
    }

    /// apply rules and inheritance
    pub fn stylize(&mut self, rules: &Vec<CssRule>) {
        self.stylize_recursive(rules);
    }

    /// Recursive worker: apply rules with specificity and inherit from parent
    fn stylize_recursive(&mut self, rules: &Vec<CssRule>) {
        // Only element nodes get rules
        if let NodeType::Element(_) = self.node_type {
            // temp map: property -> (specificity, value)
            let mut computed: HashMap<String, (u8, StyleProperty)> = HashMap::new();

            // apply each rule in order
            for rule in rules {
                // find highest specificity among selectors that match
                let mut rule_spec: Option<u8> = None;
                for sel in &rule.selectors {
                    if self.matches_selector(sel) {
                        let spec = match sel {
                            Selector::Universal => 0,
                            Selector::Type(_) => 1,
                            Selector::Class(_) => 2,
                            Selector::Id(_) => 3,
                        };
                        rule_spec = Some(rule_spec.map_or(spec, |old| old.max(spec)));
                    }
                }
                if let Some(spec) = rule_spec {
                    // rule applies: integrate its properties
                    for (key, value) in &rule.properties {
                        // override if higher or equal specificity (later wins)
                        if computed
                            .get(key)
                            .map_or(true, |(old_spec, _)| spec >= *old_spec)
                        {
                            computed.insert(key.clone(), (spec, value.clone()));
                        }
                    }
                }
            }

            // write computed values into node.style
            self.style = computed.into_iter().map(|(k, (_spec, v))| (k, v)).collect();

            // recurse for all children, passing this node's style as parent
            for child in &mut self.children {
                child.stylize_recursive(rules);
            }
        }
    }

    fn matches_selector(&self, selector: &Selector) -> bool {
        match selector {
            Selector::Universal => true,
            Selector::Class(name) => self
                .attributes
                .get("class")
                .map_or(false, |cls| cls.split_whitespace().any(|c| c == name)),
            Selector::Id(id) => self.attributes.get("id").map_or(false, |v| v == id),
            Selector::Type(s) => {
                let s_lower = s.to_lowercase();
                // Borrow node_type to avoid moving out
                if let NodeType::Element(ref html_tag) = self.node_type {
                    match html_tag {
                        HtmlTag::Div => s_lower == "div",
                        HtmlTag::Span => s_lower == "span",
                        HtmlTag::P => s_lower == "p",
                        HtmlTag::H1 => s_lower == "h1",
                        HtmlTag::H2 => s_lower == "h2",
                        HtmlTag::H3 => s_lower == "h3",
                        HtmlTag::H4 => s_lower == "h4",
                        HtmlTag::H5 => s_lower == "h5",
                        HtmlTag::H6 => s_lower == "h6",
                        HtmlTag::Strong => s_lower == "strong",
                        HtmlTag::Small => s_lower == "small",
                        HtmlTag::Big => s_lower == "big",
                        HtmlTag::B => s_lower == "b",
                        HtmlTag::W => s_lower == "w",
                        HtmlTag::I => s_lower == "i",
                        HtmlTag::U => s_lower == "u",
                        HtmlTag::S => s_lower == "s",
                        HtmlTag::Br => s_lower == "br",
                        HtmlTag::Hr => s_lower == "hr",
                        HtmlTag::A => s_lower == "a",
                        HtmlTag::Em => s_lower == "em",
                        HtmlTag::Abbr => s_lower == "abbr",
                        HtmlTag::Img => s_lower == "img",
                        HtmlTag::Audio => s_lower == "audio",
                        HtmlTag::Table => s_lower == "table",
                        HtmlTag::Thead => s_lower == "thead",
                        HtmlTag::Tbody => s_lower == "tbody",
                        HtmlTag::Tfoot => s_lower == "tfoot",
                        HtmlTag::Th => s_lower == "th",
                        HtmlTag::Td => s_lower == "td",
                        HtmlTag::Tr => s_lower == "tr",
                        HtmlTag::Caption => s_lower == "caption",
                        HtmlTag::Body => s_lower == "body",
                        HtmlTag::Head => s_lower == "head",
                        HtmlTag::Title => s_lower == "title",
                        HtmlTag::Html => s_lower == "html",
                        HtmlTag::Script => s_lower == "script",
                        HtmlTag::Style => s_lower == "style",
                        HtmlTag::Custom(t) => t.to_lowercase() == s_lower,
                    }
                } else {
                    false
                }
            }
        }
    }
}
