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
                _ => None,
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
        } else if let Some(v) = s.strip_suffix("em") {
            v.parse().ok().map(Length::Em)
        } else if let Some(v) = s.strip_suffix("rem") {
            v.parse().ok().map(Length::Rem)
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
                if let (Ok(h), Ok(sat), Ok(light)) =
                    (parts[0].parse(), parts[1].parse(), parts[2].parse())
                {
                    return Some(Color::Hsl(h, sat, light));
                }
            }
        }
        if let Some(inner) = s.strip_prefix("hsla(").and_then(|s| s.strip_suffix(')')) {
            let parts: Vec<_> = inner.split(',').map(str::trim).collect();
            if parts.len() == 4 {
                if let (Ok(h), Ok(sat), Ok(light), Ok(a)) = (
                    parts[0].parse(),
                    parts[1].parse(),
                    parts[2].parse(),
                    parts[3].parse(),
                ) {
                    return Some(Color::Hsla(h, sat, light, a));
                }
            }
        }
        None
    }
}

// simple function delegate
pub fn parse_css(input: &str) -> Vec<CssRule> {
    let mut parser = CssParser::new(input);
    parser.parse_rules()
}
