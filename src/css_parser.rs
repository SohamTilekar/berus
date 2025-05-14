use crate::layout::{Color, CssRule, Length, Selector, StyleProperty};
use std::collections::HashMap;

/// A CSS parser implemented in an object-oriented style using a Parser struct
pub struct CssParser<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> CssParser<'a> {
    /// Create a new parser for the given CSS input
    pub fn new(input: &'a str) -> Self {
        Self { input, position: 0 }
    }

    /// Parse all rules from the stylesheet
    pub fn parse_rules(&mut self) -> Vec<CssRule> {
        let mut rules = Vec::new();
        let len = self.input.len();
        while self.position < len {
            self.skip_whitespace();
            if let Some(rule) = self.parse_rule() {
                rules.push(rule);
            } else {
                break;
            }
        }
        rules
    }

    /// Parse a single rule: selectors { properties }
    fn parse_rule(&mut self) -> Option<CssRule> {
        let selector_text = self.consume_until('{')?;
        self.position += 1; // skip '{'
        let body_text = self.consume_until('}')?;
        self.position += 1; // skip '}'

        let selectors = self.parse_selectors(&selector_text);
        let properties = self.parse_properties(&body_text);

        Some(CssRule {
            selectors,
            properties,
        })
    }

    /// Split and parse selectors
    fn parse_selectors(&self, text: &str) -> Vec<Selector> {
        text.split(',')
            .map(|s| s.trim())
            .filter_map(|s| match s {
                "*" => Some(Selector::Universal),
                _ if s.starts_with('.') => Some(Selector::Class(s[1..].to_string())),
                _ if s.starts_with('#') => Some(Selector::Id(s[1..].to_string())),
                _ => Some(Selector::Type(s.to_string())),
            })
            .collect()
    }

    /// Split and parse property declarations
    fn parse_properties(&self, text: &str) -> HashMap<String, StyleProperty> {
        let mut map = HashMap::new();
        for decl in text.split(';') {
            let decl = decl.trim();
            if decl.is_empty() {
                continue;
            }
            if let Some((name, value)) = decl.split_once(':') {
                if let Some(prop) = self.parse_value(value.trim()) {
                    map.insert(name.trim().to_string(), prop);
                }
            }
        }
        map
    }

    /// Parse a CSS value into a StyleProperty, supporting lengths, colors, keywords
    fn parse_value(&self, s: &str) -> Option<StyleProperty> {
        // try lengths
        if let Some(length) = LengthParser::parse(s) {
            return Some(StyleProperty::Length(length));
        }
        // try colors
        if let Some(color) = ColorParser::parse(s) {
            return Some(StyleProperty::Color(color));
        }
        // fallback to keyword
        Some(StyleProperty::Keyword(s.to_string()))
    }

    /// Advance until a given delimiter, returning the consumed slice (without the delimiter)
    fn consume_until(&mut self, delim: char) -> Option<String> {
        if let Some(idx) = self.input[self.position..].find(delim) {
            let result = &self.input[self.position..self.position + idx];
            self.position += idx;
            Some(result.trim().to_string())
        } else {
            None
        }
    }

    /// Skip whitespace characters
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.input[self.position..].chars().next() {
            if ch.is_whitespace() {
                self.position += ch.len_utf8();
            } else {
                break;
            }
        }
    }
}

/// Helper struct for parsing length units
struct LengthParser;
impl LengthParser {
    fn parse(s: &str) -> Option<Length> {
        let s = s.trim();
        if let Some(v) = s.strip_suffix("px") {
            v.parse().ok().map(Length::Px)
        } else if let Some(v) = s.strip_suffix("rem") {
            // Put rem before em cz em will still come after r
            v.parse().ok().map(Length::Rem)
        } else if let Some(v) = s.strip_suffix("em") {
            v.parse().ok().map(Length::Em)
        } else if let Some(v) = s.strip_suffix('%') {
            v.parse().ok().map(Length::Percent)
        } else {
            None
        }
    }
}

/// Helper struct for parsing various color formats
struct ColorParser;
impl ColorParser {
    fn parse(s: &str) -> Option<Color> {
        let s = s.trim();
        if let Some(hex) = s.strip_prefix('#') {
            return Some(Color::Hex(hex.to_string()));
        }
        if let Some(inner) = s.strip_prefix("rgb(").and_then(|s| s.strip_suffix(')')) {
            let parts: Vec<_> = inner.split(',').map(str::trim).collect();
            if parts.len() == 3 {
                if let (Ok(r), Ok(g), Ok(b)) =
                    (parts[0].parse(), parts[1].parse(), parts[2].parse())
                {
                    return Some(Color::Rgb(r, g, b));
                }
            }
        }
        if let Some(inner) = s.strip_prefix("rgba(").and_then(|s| s.strip_suffix(')')) {
            let parts: Vec<_> = inner.split(',').map(str::trim).collect();
            if parts.len() == 4 {
                if let (Ok(r), Ok(g), Ok(b), Ok(a)) = (
                    parts[0].parse(),
                    parts[1].parse(),
                    parts[2].parse(),
                    parts[3].parse(),
                ) {
                    return Some(Color::Rgba(r, g, b, a));
                }
            }
        }
        if let Some(inner) = s.strip_prefix("hsl(").and_then(|s| s.strip_suffix(')')) {
            let parts: Vec<_> = inner.split(',').map(str::trim).collect();
            if parts.len() == 3 {
                if let (Some(h), Some(sat), Some(light)) = (
                    parts[0].parse().ok(),
                    parts[1].strip_suffix('%').and_then(|s| s.parse().ok()),
                    parts[2].strip_suffix('%').and_then(|s| s.parse().ok()),
                ) {
                    return Some(Color::Hsl(h, sat, light));
                }
            }
        }
        if let Some(inner) = s.strip_prefix("hsla(").and_then(|s| s.strip_suffix(')')) {
            let parts: Vec<_> = inner.split(',').map(str::trim).collect();
            if parts.len() == 4 {
                if let (Some(h), Some(sat), Some(light), Some(a)) = (
                    parts[0].parse().ok(),
                    parts[1].strip_suffix('%').and_then(|s| s.parse().ok()),
                    parts[2].strip_suffix('%').and_then(|s| s.parse().ok()),
                    parts[3].parse().ok(),
                ) {
                    return Some(Color::Hsla(h, sat, light, a));
                }
            }
        }

        match s {
            "aliceblue" => Some(Color::Rgb(240, 248, 255)),
            "antiquewhite" => Some(Color::Rgb(250, 235, 215)),
            "aqua" => Some(Color::Rgb(0, 255, 255)),
            "aquamarine" => Some(Color::Rgb(127, 255, 212)),
            "azure" => Some(Color::Rgb(240, 255, 255)),
            "beige" => Some(Color::Rgb(245, 245, 220)),
            "bisque" => Some(Color::Rgb(255, 228, 196)),
            "black" => Some(Color::Rgb(0, 0, 0)),
            "blanchedalmond" => Some(Color::Rgb(255, 235, 205)),
            "blue" => Some(Color::Rgb(0, 0, 255)),
            "blueviolet" => Some(Color::Rgb(138, 43, 226)),
            "brown" => Some(Color::Rgb(165, 42, 42)),
            "burlywood" => Some(Color::Rgb(222, 184, 135)),
            "cadetblue" => Some(Color::Rgb(95, 158, 160)),
            "chartreuse" => Some(Color::Rgb(127, 255, 0)),
            "chocolate" => Some(Color::Rgb(210, 105, 30)),
            "coral" => Some(Color::Rgb(255, 127, 80)),
            "cornflowerblue" => Some(Color::Rgb(100, 149, 237)),
            "cornsilk" => Some(Color::Rgb(255, 248, 220)),
            "crimson" => Some(Color::Rgb(220, 20, 60)),
            "cyan" => Some(Color::Rgb(0, 255, 255)),
            "darkblue" => Some(Color::Rgb(0, 0, 139)),
            "darkcyan" => Some(Color::Rgb(0, 139, 139)),
            "darkgoldenrod" => Some(Color::Rgb(184, 134, 11)),
            "darkgray" => Some(Color::Rgb(169, 169, 169)),
            "darkgrey" => Some(Color::Rgb(169, 169, 169)),
            "darkgreen" => Some(Color::Rgb(0, 100, 0)),
            "darkkhaki" => Some(Color::Rgb(189, 183, 107)),
            "darkmagenta" => Some(Color::Rgb(139, 0, 139)),
            "darkolivegreen" => Some(Color::Rgb(85, 107, 47)),
            "darkorange" => Some(Color::Rgb(255, 140, 0)),
            "darkorchid" => Some(Color::Rgb(153, 50, 204)),
            "darkred" => Some(Color::Rgb(139, 0, 0)),
            "darksalmon" => Some(Color::Rgb(233, 150, 122)),
            "darkseagreen" => Some(Color::Rgb(143, 188, 143)),
            "darkslateblue" => Some(Color::Rgb(72, 61, 139)),
            "darkslategray" => Some(Color::Rgb(47, 79, 79)),
            "darkslategrey" => Some(Color::Rgb(47, 79, 79)),
            "darkturquoise" => Some(Color::Rgb(0, 206, 209)),
            "darkviolet" => Some(Color::Rgb(148, 0, 211)),
            "deeppink" => Some(Color::Rgb(255, 20, 147)),
            "deepskyblue" => Some(Color::Rgb(0, 191, 255)),
            "dimgray" => Some(Color::Rgb(105, 105, 105)),
            "dimgrey" => Some(Color::Rgb(105, 105, 105)),
            "dodgerblue" => Some(Color::Rgb(30, 144, 255)),
            "firebrick" => Some(Color::Rgb(178, 34, 34)),
            "floralwhite" => Some(Color::Rgb(255, 250, 240)),
            "forestgreen" => Some(Color::Rgb(34, 139, 34)),
            "fuchsia" => Some(Color::Rgb(255, 0, 255)),
            "gainsboro" => Some(Color::Rgb(220, 220, 220)),
            "ghostwhite" => Some(Color::Rgb(248, 248, 255)),
            "gold" => Some(Color::Rgb(255, 215, 0)),
            "goldenrod" => Some(Color::Rgb(218, 165, 32)),
            "gray" => Some(Color::Rgb(128, 128, 128)),
            "grey" => Some(Color::Rgb(128, 128, 128)),
            "green" => Some(Color::Rgb(0, 128, 0)),
            "greenyellow" => Some(Color::Rgb(173, 255, 47)),
            "honeydew" => Some(Color::Rgb(240, 255, 240)),
            "hotpink" => Some(Color::Rgb(255, 105, 180)),
            "indianred" => Some(Color::Rgb(205, 92, 92)),
            "indigo" => Some(Color::Rgb(75, 0, 130)),
            "ivory" => Some(Color::Rgb(255, 255, 240)),
            "khaki" => Some(Color::Rgb(240, 230, 140)),
            "lavender" => Some(Color::Rgb(230, 230, 250)),
            "lavenderblush" => Some(Color::Rgb(255, 240, 245)),
            "lawngreen" => Some(Color::Rgb(124, 252, 0)),
            "lemonchiffon" => Some(Color::Rgb(255, 250, 205)),
            "lightblue" => Some(Color::Rgb(173, 216, 230)),
            "lightcoral" => Some(Color::Rgb(240, 128, 128)),
            "lightcyan" => Some(Color::Rgb(224, 255, 255)),
            "lightgoldenrodyellow" => Some(Color::Rgb(250, 250, 210)),
            "lightgray" => Some(Color::Rgb(211, 211, 211)),
            "lightgrey" => Some(Color::Rgb(211, 211, 211)),
            "lightgreen" => Some(Color::Rgb(144, 238, 144)),
            "lightpink" => Some(Color::Rgb(255, 182, 193)),
            "lightsalmon" => Some(Color::Rgb(255, 160, 122)),
            "lightseagreen" => Some(Color::Rgb(32, 178, 170)),
            "lightskyblue" => Some(Color::Rgb(135, 206, 250)),
            "lightslategray" => Some(Color::Rgb(119, 136, 153)),
            "lightslategrey" => Some(Color::Rgb(119, 136, 153)),
            "lightsteelblue" => Some(Color::Rgb(176, 196, 222)),
            "lightyellow" => Some(Color::Rgb(255, 255, 224)),
            "lime" => Some(Color::Rgb(0, 255, 0)),
            "limegreen" => Some(Color::Rgb(50, 205, 50)),
            "linen" => Some(Color::Rgb(250, 240, 230)),
            "magenta" => Some(Color::Rgb(255, 0, 255)),
            "maroon" => Some(Color::Rgb(128, 0, 0)),
            "mediumaquamarine" => Some(Color::Rgb(102, 205, 170)),
            "mediumblue" => Some(Color::Rgb(0, 0, 205)),
            "mediumorchid" => Some(Color::Rgb(186, 85, 211)),
            "mediumpurple" => Some(Color::Rgb(147, 112, 219)),
            "mediumseagreen" => Some(Color::Rgb(60, 179, 113)),
            "mediumslateblue" => Some(Color::Rgb(123, 104, 238)),
            "mediumspringgreen" => Some(Color::Rgb(0, 250, 154)),
            "mediumturquoise" => Some(Color::Rgb(72, 209, 204)),
            "mediumvioletred" => Some(Color::Rgb(199, 21, 133)),
            "midnightblue" => Some(Color::Rgb(25, 25, 112)),
            "mintcream" => Some(Color::Rgb(245, 255, 250)),
            "mistyrose" => Some(Color::Rgb(255, 228, 225)),
            "moccasin" => Some(Color::Rgb(255, 228, 181)),
            "navajowhite" => Some(Color::Rgb(255, 222, 173)),
            "navy" => Some(Color::Rgb(0, 0, 128)),
            "oldlace" => Some(Color::Rgb(253, 245, 230)),
            "olive" => Some(Color::Rgb(128, 128, 0)),
            "olivedrab" => Some(Color::Rgb(107, 142, 35)),
            "orange" => Some(Color::Rgb(255, 165, 0)),
            "orangered" => Some(Color::Rgb(255, 69, 0)),
            "orchid" => Some(Color::Rgb(218, 112, 214)),
            "palegoldenrod" => Some(Color::Rgb(238, 232, 170)),
            "palegreen" => Some(Color::Rgb(152, 251, 152)),
            "paleturquoise" => Some(Color::Rgb(175, 238, 238)),
            "palevioletred" => Some(Color::Rgb(219, 112, 147)),
            "papayawhip" => Some(Color::Rgb(255, 239, 213)),
            "peachpuff" => Some(Color::Rgb(255, 218, 185)),
            "peru" => Some(Color::Rgb(205, 133, 63)),
            "pink" => Some(Color::Rgb(255, 192, 203)),
            "plum" => Some(Color::Rgb(221, 160, 221)),
            "powderblue" => Some(Color::Rgb(176, 224, 230)),
            "purple" => Some(Color::Rgb(128, 0, 128)),
            "rebeccapurple" => Some(Color::Rgb(102, 51, 153)),
            "red" => Some(Color::Rgb(255, 0, 0)),
            "rosybrown" => Some(Color::Rgb(188, 143, 143)),
            "royalblue" => Some(Color::Rgb(65, 105, 225)),
            "saddlebrown" => Some(Color::Rgb(139, 69, 19)),
            "salmon" => Some(Color::Rgb(250, 128, 114)),
            "sandybrown" => Some(Color::Rgb(244, 164, 96)),
            "seagreen" => Some(Color::Rgb(46, 139, 87)),
            "seashell" => Some(Color::Rgb(255, 245, 238)),
            "sienna" => Some(Color::Rgb(160, 82, 45)),
            "silver" => Some(Color::Rgb(192, 192, 192)),
            "skyblue" => Some(Color::Rgb(135, 206, 235)),
            "slateblue" => Some(Color::Rgb(106, 90, 205)),
            "slategray" => Some(Color::Rgb(112, 128, 144)),
            "slategrey" => Some(Color::Rgb(112, 128, 144)),
            "snow" => Some(Color::Rgb(255, 250, 250)),
            "springgreen" => Some(Color::Rgb(0, 255, 127)),
            "steelblue" => Some(Color::Rgb(70, 130, 180)),
            "tan" => Some(Color::Rgb(210, 180, 140)),
            "teal" => Some(Color::Rgb(0, 128, 128)),
            "thistle" => Some(Color::Rgb(216, 191, 216)),
            "tomato" => Some(Color::Rgb(255, 99, 71)),
            "turquoise" => Some(Color::Rgb(64, 224, 208)),
            "violet" => Some(Color::Rgb(238, 130, 238)),
            "wheat" => Some(Color::Rgb(245, 222, 179)),
            "white" => Some(Color::Rgb(255, 255, 255)),
            "whitesmoke" => Some(Color::Rgb(245, 245, 245)),
            "yellow" => Some(Color::Rgb(255, 255, 0)),
            "yellowgreen" => Some(Color::Rgb(154, 205, 50)),
            _ => None,
        }
    }
}

// simple function delegate
pub fn parse_css(input: &str) -> Vec<CssRule> {
    let mut parser = CssParser::new(input);
    parser.parse_rules()
}
