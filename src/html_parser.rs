// parser.rs
use crate::css_parser::parse_css;
use crate::layout::{CssRule, HtmlNode, HtmlTag, NodeType, Selector};
use std::collections::HashMap;

/// Parse an HTML string into a tree of HtmlNode, discarding comments and doctype.
pub fn parse_html(input: &str) -> HtmlNode {
    let mut parser = Parser::new(input);
    let nodes = parser.parse_nodes(None); // Start parsing top-level nodes

    // Filter out any potential non-element nodes at the top level (e.g., whitespace text)
    // before deciding if wrapping is needed.
    let mut top_level_elements: Vec<HtmlNode> = nodes
        .into_iter()
        .filter(|n| matches!(n.node_type, NodeType::Element(_)))
        .collect();

    let first_element_is_html = top_level_elements
        .first()
        .map_or(false, |n| match &n.node_type {
            NodeType::Element(HtmlTag::Html) => true,
            NodeType::Element(HtmlTag::Custom(tag)) if tag.eq_ignore_ascii_case("html") => true,
            _ => false,
        });

    if top_level_elements.is_empty() {
        // Handle empty input or input with only comments/doctype/whitespace
        cleanup_tree(HtmlNode::new_element(HtmlTag::Html, HashMap::new(), vec![]))
    } else if top_level_elements.len() == 1 && first_element_is_html {
        // Single top-level element is <html>, clean it up directly
        cleanup_tree(top_level_elements.remove(0))
    } else {
        // Multiple top-level elements, or the first one isn't <html>. Wrap them.
        let html_node = HtmlNode::new_element(HtmlTag::Html, HashMap::new(), top_level_elements);
        cleanup_tree(html_node)
    }
}

/// Prints the HTML tree, including attributes and CSS style properties.
pub fn print_tree(node: &HtmlNode) {
    fn rec(node: &HtmlNode, indent: usize) {
        let pad = "  ".repeat(indent);
        match &node.node_type {
            NodeType::Element(tag) => {
                // Print opening tag with attributes
                if node.attributes.is_empty() {
                    eprintln!("{}<{:?}>", pad, tag);
                } else {
                    eprintln!("{}<{:?} {:?}>", pad, tag, node.attributes);
                }

                // Print CSS style properties, if any
                if !node.style.is_empty() {
                    for (prop_name, prop_value) in &node.style {
                        eprintln!("{}  {}: {:?}", pad, prop_name, prop_value);
                    }
                }

                // Recurse into children
                for child in &node.children {
                    rec(child, indent + 1);
                }

                // Print closing tag for non-void elements
                if !is_void_element(tag) {
                    eprintln!("{}</{:?}>", pad, tag);
                }
            }
            NodeType::Text(text) => {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    eprintln!("{}TEXT: \"{}\"", pad, trimmed);
                }
            }
        }
    }
    rec(node, 0);
}

/// Check if a tag is a known void element
fn is_void_element(tag: &HtmlTag) -> bool {
    match tag {
        // Check specific enum variants first for performance
        HtmlTag::Br | HtmlTag::Hr => true,
        _ => false,
    }
}

/// Ensure tree has html root with head and body
pub fn cleanup_tree(mut root: HtmlNode) -> HtmlNode {
    // Ensure the root node itself is HtmlTag::Html
    let root_tag_name = match &root.node_type {
        NodeType::Element(HtmlTag::Html) => Some("html"),
        NodeType::Element(HtmlTag::Custom(name)) if name.eq_ignore_ascii_case("html") => {
            Some("html")
        }
        _ => None,
    };

    if root_tag_name.is_none() {
        let new_root = HtmlNode {
            node_type: NodeType::Element(HtmlTag::Html),
            style: HashMap::new(),
            attributes: HashMap::new(),
            children: vec![root],
        };
        return cleanup_tree(new_root);
    }

    root.node_type = NodeType::Element(HtmlTag::Html);

    let mut head_node: Option<HtmlNode> = None;
    let mut body_node: Option<HtmlNode> = None;
    let mut other_children: Vec<HtmlNode> = Vec::new();
    let mut style_contents: Vec<String> = Vec::new();
    let mut head_found = false;
    let mut body_found = false;

    for child in root.children.drain(..) {
        if let NodeType::Text(t) = &child.node_type {
            if t.trim().is_empty() {
                continue;
            }
        }

        if let NodeType::Element(tag) = &child.node_type {
            match tag {
                HtmlTag::Head if !head_found => {
                    head_node = Some(child);
                    head_found = true;
                }
                HtmlTag::Body if !body_found => {
                    body_node = Some(child);
                    body_found = true;
                }
                HtmlTag::Custom(name) if !head_found && name.eq_ignore_ascii_case("head") => {
                    head_node = Some(HtmlNode {
                        node_type: NodeType::Element(HtmlTag::Head),
                        ..child
                    });
                    head_found = true;
                }
                HtmlTag::Custom(name) if !body_found && name.eq_ignore_ascii_case("body") => {
                    body_node = Some(HtmlNode {
                        node_type: NodeType::Element(HtmlTag::Body),
                        ..child
                    });
                    body_found = true;
                }
                HtmlTag::Head | HtmlTag::Body => {}
                HtmlTag::Custom(name)
                    if name.eq_ignore_ascii_case("head") || name.eq_ignore_ascii_case("body") => {}
                HtmlTag::Style => {
                    if let Some(NodeType::Text(style_text)) =
                        child.children.get(0).map(|n| &n.node_type)
                    {
                        style_contents.push(style_text.clone());
                    }
                    // Don't push this style tag into children — it's being merged
                }
                _ => other_children.push(child),
            }
        } else {
            other_children.push(child);
        }
    }

    // Handle <style> tags inside head and body
    let mut collect_styles = |node: &mut Option<HtmlNode>| {
        if let Some(n) = node {
            let mut retained_children = Vec::new();
            for child in n.children.drain(..) {
                if let NodeType::Element(HtmlTag::Style) = &child.node_type {
                    if let Some(NodeType::Text(style_text)) =
                        child.children.get(0).map(|n| &n.node_type)
                    {
                        style_contents.push(style_text.clone());
                    }
                } else {
                    retained_children.push(child);
                }
            }
            n.children = retained_children;
        }
    };

    collect_styles(&mut head_node);
    collect_styles(&mut body_node);

    let final_head =
        head_node.unwrap_or_else(|| HtmlNode::new_element(HtmlTag::Head, HashMap::new(), vec![]));
    let mut final_body =
        body_node.unwrap_or_else(|| HtmlNode::new_element(HtmlTag::Body, HashMap::new(), vec![]));

    let mut body_children = other_children;
    body_children.append(&mut final_body.children);
    final_body.children = body_children;

    // Create merged <style> tag if there are styles to insert
    // let mut final_head = final_head;
    // if !style_contents.is_empty() {
    //     let mut childrens: Vec<HtmlNode> = vec![];
    //     for style_text in style_contents {
    //         childrens.push(HtmlNode {
    //             node_type: NodeType::Text(style_text),
    //             style: HashMap::new(),
    //             attributes: HashMap::new(),
    //             children: vec![],
    //         });
    //     }
    // let style_node = HtmlNode {
    //     node_type: NodeType::Element(HtmlTag::Style),
    //     style: HashMap::new(),
    //     attributes: HashMap::new(),
    //     children: childrens,
    // };
    // final_head.children.insert(0, style_node);
    // }
    root.children = vec![final_head, final_body];
    if !style_contents.is_empty() {
        let mut rules: Vec<CssRule> = vec![];
        for style_text in style_contents {
            rules.extend(parse_css(style_text.as_str()));
        }
        root.stylize(&rules);
        for rule in rules {
            println!();
            for selctor in rule.selectors {
                match selctor {
                    Selector::Universal => print!("* "),
                    Selector::Class(s) => print!(".{} ", s),
                    Selector::Id(s) => print!("#{} ", s),
                    Selector::Type(s) => print!("{} ", s),
                }
            }
            print!("( \n");
            for (name, property) in rule.properties {
                println!("{}: {:?}", name, property)
            }
            print!(")\n")
        }
    }
    root
}

// --- internal parser implementation ---
struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Parser { input, pos: 0 }
    }

    /// Parses nodes until it encounters a closing tag matching `end_tag` or EOF.
    /// If `end_tag` is None, parses until EOF (for top-level).
    /// Comments and Doctype are consumed but not added to the result vector.
    fn parse_nodes(&mut self, end_tag: Option<&str>) -> Vec<HtmlNode> {
        let mut nodes = Vec::new();
        loop {
            self.consume_whitespace(); // Consume leading whitespace before nodes
            if self.eof() || (end_tag.is_some() && self.starts_with("</")) {
                if let Some(tag_to_match) = end_tag {
                    if self.check_closing_tag(tag_to_match) {
                        break; // Found the correct closing tag
                    }
                    if self.eof() {
                        break;
                    }
                } else if self.eof() {
                    break;
                }
            }

            let start_pos = self.pos; // For loop termination check

            if self.starts_with("<!--") {
                self.parse_comment(); // Consume comment, returns ()
            } else if self.starts_with("<!DOCTYPE") || self.starts_with("<!doctype") {
                self.parse_doctype(); // Consume doctype, returns ()
            } else if self.starts_with("<") {
                if self
                    .input
                    .get(self.pos + 1..)
                    .map_or(false, |s| s.starts_with('/'))
                {
                    if end_tag.is_none() {
                        let _ = self.parse_closing_tag(); // Consume and discard
                        eprintln!("Warning: Found unexpected closing tag at top level. Skipping.");
                    } else {
                        break; // Let the parent element's parser handle unexpected closing tag
                    }
                } else {
                    // It's an opening tag, parse it and add to nodes
                    nodes.push(self.parse_element());
                }
            } else {
                // Parse text node, only add if not empty after trimming whitespace
                let text_node = self.parse_text();
                if let NodeType::Text(t) = &text_node.node_type {
                    if !t.trim().is_empty() {
                        nodes.push(text_node);
                    }
                    // else: discard whitespace-only text nodes between elements
                } else {
                    // Should not happen based on parse_text logic, but safe guard
                    nodes.push(text_node);
                }
            }

            // Prevent infinite loop if no progress is made
            if self.pos == start_pos && !self.eof() {
                eprintln!(
                    "Warning: Parser stuck at position {}. Attempting to advance.",
                    self.pos
                );
                if !self.eof() {
                    self.consume_char();
                } else {
                    break;
                }
            }
        }
        nodes
    }

    /// Parses and consumes `<!DOCTYPE ... >`. Returns nothing.
    fn parse_doctype(&mut self) {
        // Changed return type to ()
        assert!(self.starts_with("<!DOCTYPE") || self.starts_with("<!doctype"));
        // Consume until '>'
        while !self.eof() && self.current_char() != '>' {
            self.pos += 1;
        }
        if !self.eof() {
            self.pos += 1; // Consume '>'
        }
        // No node is created or returned
    }

    /// Parses and consumes `<!-- ... -->`. Returns nothing.
    fn parse_comment(&mut self) {
        // Changed return type to ()
        assert!(self.starts_with("<!--"));
        self.pos += 4; // Consume "<!--"
        while !self.eof() && !self.starts_with("-->") {
            self.pos += 1;
        }
        if self.starts_with("-->") {
            self.pos += 3; // Consume "-->"
        } else {
            eprintln!("Warning: Unterminated comment found.");
        }
        // No node is created or returned
    }

    /// Parses `<tag attr="value"> ... </tag>` or `<tag />`
    fn parse_element(&mut self) -> HtmlNode {
        assert!(self.consume_char() == '<');
        let tag_name = self.parse_tag_name();
        let attributes = self.parse_attributes();
        self.consume_whitespace();

        let is_self_closing_slash = self.starts_with("/>");
        if is_self_closing_slash {
            self.pos += 2; // Consume "/>"
        } else if self.starts_with(">") {
            self.pos += 1; // Consume ">"
        } else {
            eprintln!(
                "Warning: Malformed tag opening for <{}>. Expected '>' or '/>'. Found '{}'",
                tag_name,
                self.current_char()
            );
            if !self.eof() {
                self.pos += 1;
            }
        }

        let tag = Self::match_tag(&tag_name);

        if is_self_closing_slash || is_void_element(&tag) {
            return HtmlNode::new_element(tag, attributes, vec![]);
        }

        let children = if matches!(tag, HtmlTag::Script | HtmlTag::Style) {
            self.parse_raw_text_content(&tag_name)
        } else {
            self.parse_nodes(Some(&tag_name))
        };

        if self.starts_with("</") {
            if self.check_closing_tag(&tag_name) {
                self.parse_closing_tag();
            } else {
                eprintln!(
                    "Warning: Expected closing tag </{}> but found different closing tag or EOF. Auto-closing <{}>.",
                    tag_name, tag_name
                );
            }
        } else {
            eprintln!(
                "Warning: Missing closing tag for <{}>. Auto-closing.",
                tag_name
            );
        }

        HtmlNode::new_element(tag, attributes, children)
    }

    /// Parses the raw text content of elements like <script> or <style>
    fn parse_raw_text_content(&mut self, tag_name: &str) -> Vec<HtmlNode> {
        let start = self.pos;
        let end_tag = format!("</{}>", tag_name);
        let end_tag_lower = end_tag.to_lowercase();

        while !self.eof() {
            if self.input[self.pos..]
                .to_lowercase()
                .starts_with(&end_tag_lower)
            {
                break;
            }
            self.pos += 1;
        }

        let text = self.input[start..self.pos].to_string();
        // Don't trim raw text content here
        if text.is_empty() {
            vec![]
        } else {
            vec![HtmlNode::new_text(text)]
        }
    }

    /// Parses plain text content between tags
    fn parse_text(&mut self) -> HtmlNode {
        let start = self.pos;
        while !self.eof() && self.current_char() != '<' {
            self.pos += 1;
        }
        let text = self.input[start..self.pos].to_string();
        // Simple HTML entity decoding
        let decoded_text = text
            .replace("<", "<")
            .replace(">", ">")
            .replace("&", "&")
            .replace("\"", "\"")
            .replace("'", "'");
        // Return the node even if whitespace only, parse_nodes will filter if needed
        HtmlNode::new_text(decoded_text)
    }

    /// Parses a tag name (alphanumeric characters)
    fn parse_tag_name(&mut self) -> String {
        self.consume_whitespace();
        let start = self.pos;
        while !self.eof()
            && (self.current_char().is_ascii_alphanumeric() || self.current_char() == '_')
        {
            self.pos += 1;
        }
        self.input[start..self.pos].to_string()
    }

    /// Parses tag attributes `key="value" key=value key`
    fn parse_attributes(&mut self) -> HashMap<String, String> {
        let mut attrs = HashMap::new();
        loop {
            self.consume_whitespace();
            if self.eof() || self.starts_with(">") || self.starts_with("/>") {
                break;
            }

            if self.starts_with("/") {
                eprintln!("Warning: Unexpected '/' found while parsing attributes.");
                break;
            }

            let name = self.parse_attribute_name();
            if name.is_empty() {
                if !self.eof() && !self.starts_with(">") && !self.starts_with("/>") {
                    eprintln!(
                        "Warning: Encountered unexpected character '{}' while expecting attribute name. Skipping.",
                        self.current_char()
                    );
                    self.consume_char();
                }
                continue;
            }

            self.consume_whitespace();
            let value = if self.starts_with("=") {
                self.pos += 1; // Consume '='
                self.consume_whitespace();
                self.parse_attribute_value()
            } else {
                "".to_string() // Boolean attribute
            };
            attrs.insert(name.to_lowercase(), value);
        }
        attrs
    }

    /// Parses an attribute name
    fn parse_attribute_name(&mut self) -> String {
        let start = self.pos;
        while !self.eof() {
            let c = self.current_char();
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ':' {
                self.pos += 1;
            } else {
                break;
            }
        }
        self.input[start..self.pos].to_string()
    }

    /// Parses an attribute value (quoted or unquoted)
    fn parse_attribute_value(&mut self) -> String {
        let value = if self.starts_with("\"") || self.starts_with("'") {
            let quote = self.consume_char();
            let start = self.pos;
            while !self.eof() && self.current_char() != quote {
                self.pos += 1;
            }
            let val = self.input[start..self.pos].to_string();
            if !self.eof() {
                self.pos += 1; // Consume closing quote
            } else {
                eprintln!("Warning: Unterminated quoted attribute value found.");
            }
            val
        } else {
            let start = self.pos;
            while !self.eof() {
                let c = self.current_char();
                if c.is_whitespace() || c == '>' || c == '/' {
                    break;
                }
                self.pos += 1;
            }
            if self.pos == start && !self.eof() && (self.starts_with(">") || self.starts_with("/>"))
            {
                "".to_string()
            } else {
                self.input[start..self.pos].to_string()
            }
        };
        // Decode HTML entities in attribute values
        value
            .replace("<", "<")
            .replace(">", ">")
            .replace("&", "&")
            .replace("\"", "\"")
            .replace("'", "'")
    }

    /// Consumes `</tag>`
    fn parse_closing_tag(&mut self) -> String {
        assert!(self.starts_with("</"));
        self.pos += 2; // Consume '</'
        let tag_name = self.parse_tag_name();
        self.consume_whitespace();
        if self.starts_with(">") {
            self.pos += 1; // Consume '>'
        } else {
            eprintln!(
                "Warning: Malformed closing tag </{}>. Expected '>'. Found '{}'",
                tag_name,
                self.current_char()
            );
            if !self.eof() {
                self.pos += 1;
            }
        }
        tag_name
    }

    /// Checks if the upcoming closing tag matches `expected_tag_name` case-insensitively.
    fn check_closing_tag(&self, expected_tag_name: &str) -> bool {
        if !self.starts_with("</") {
            return false;
        }
        let start_pos = self.pos + 2;
        let mut temp_pos = start_pos;
        while temp_pos < self.input.len() && self.input.as_bytes()[temp_pos].is_ascii_alphanumeric()
        {
            temp_pos += 1;
        }
        let actual_tag_name = &self.input[start_pos..temp_pos];

        while temp_pos < self.input.len() && self.input.as_bytes()[temp_pos].is_ascii_whitespace() {
            temp_pos += 1;
        }

        if temp_pos < self.input.len() && self.input.as_bytes()[temp_pos] == b'>' {
            actual_tag_name.eq_ignore_ascii_case(expected_tag_name)
        } else {
            false
        }
    }

    /// Consumes whitespace characters.
    fn consume_whitespace(&mut self) {
        while !self.eof() && self.current_char().is_whitespace() {
            self.pos += 1;
        }
    }

    /// Checks if the input starts with `s` from the current position.
    fn starts_with(&self, s: &str) -> bool {
        self.input
            .get(self.pos..)
            .map_or(false, |slice| slice.starts_with(s))
    }

    /// Checks if the end of the input has been reached.
    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    /// Consumes and returns the next character.
    fn consume_char(&mut self) -> char {
        // Ensure we don't panic if called at eof, though logic should prevent this.
        if self.eof() {
            return '\0';
        }

        let mut iter = self.input[self.pos..].char_indices();
        let (_, cur_char) = iter.next().unwrap(); // Safe due to eof check
        // Calculate the byte length of the current char to advance pos correctly
        let char_len = cur_char.len_utf8();
        self.pos += char_len;
        cur_char
    }

    /// Returns the current character without consuming it.
    fn current_char(&self) -> char {
        self.input
            .get(self.pos..)
            .and_then(|s| s.chars().next())
            .unwrap_or('\0')
    }

    /// Matches a tag name string (case-insensitive) to the HtmlTag enum.
    fn match_tag(tag_name: &str) -> HtmlTag {
        match tag_name.to_lowercase().as_str() {
            "div" => HtmlTag::Div,
            "span" => HtmlTag::Span,
            "p" => HtmlTag::P,
            "h1" => HtmlTag::H1,
            "h2" => HtmlTag::H2,
            "h3" => HtmlTag::H3,
            "h4" => HtmlTag::H4,
            "h5" => HtmlTag::H5,
            "h6" => HtmlTag::H6,
            "strong" => HtmlTag::Strong,
            "small" => HtmlTag::Small,
            "big" => HtmlTag::Big,
            "br" => HtmlTag::Br,
            "body" => HtmlTag::Body,
            "head" => HtmlTag::Head,
            "html" => HtmlTag::Html,
            "b" => HtmlTag::B,
            "w" => HtmlTag::W,
            "i" => HtmlTag::I,
            "u" => HtmlTag::U,
            "s" => HtmlTag::S,
            "title" => HtmlTag::Title,
            "hr" => HtmlTag::Hr,
            "script" => HtmlTag::Script,
            "style" => HtmlTag::Style,
            _ => HtmlTag::Custom(tag_name.to_string()),
        }
    }
}
