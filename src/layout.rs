use std::collections::HashMap;

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
    Small,
    Big,
    B,  // bold
    I,  // italic
    Br, // breakline
    Hr, // horizontal ruler
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

#[derive(Debug, Clone)]
pub enum Color {
    Rgb(u8, u8, u8),
    Rgba(u8, u8, u8, f32),
    Hsl(u8, u8, u8),
    Hsla(u8, u8, u8, f32),
    Hex(String),
}

#[derive(Debug, Clone)]
pub enum StyleProperty {
    Keyword(String),
    Length(Length),
    Color(Color),
}

#[derive(Debug, Clone)]
pub enum Selector {
    Universal,
    Class(String),
    Id(String),
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

    /// Public entry: apply rules and inheritance
    pub fn stylize(&mut self, rules: &[CssRule]) {
        self.stylize_recursive(rules, None);
    }

    /// Recursive worker: apply rules with specificity and inherit from parent
    fn stylize_recursive(
        &mut self,
        rules: &[CssRule],
        parent_style: Option<&HashMap<String, StyleProperty>>,
    ) {
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
                            Selector::Class(_) => 1,
                            Selector::Id(_) => 2,
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

            // inherit properties from parent if missing
            const INHERITED: &[&str] = &[
                "color",
                "font-size",
                "font-family",
                "font-weight",
                "line-height",
                "visibility",
            ];
            if let Some(parent_map) = parent_style {
                for &prop in INHERITED {
                    if !self.style.contains_key(prop) {
                        if let Some(val) = parent_map.get(prop) {
                            self.style.insert(prop.to_string(), val.clone());
                        }
                    }
                }
            }
        }

        // recurse for all children, passing this node's style as parent
        for child in &mut self.children {
            child.stylize_recursive(rules, Some(&self.style));
        }
    }

    /// Does this node match a given selector?
    fn matches_selector(&self, selector: &Selector) -> bool {
        match selector {
            Selector::Universal => true,
            Selector::Class(name) => self
                .attributes
                .get("class")
                .map_or(false, |cls| cls.split_whitespace().any(|c| c == name)),
            Selector::Id(id) => self.attributes.get("id").map_or(false, |v| v == id),
        }
    }
}
