// parser.rs
#[derive(Debug, Clone)]
pub enum Token {
    Text(String),
    Tag(String),
}

pub fn tokenize_html(body: &String) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut current_text = String::new();
    let mut in_tag = false;

    for c in body.chars() {
        match c {
            '<' => {
                // Finish previous text run, if any
                if !current_text.is_empty() {
                    tokens.push(Token::Text(current_text));
                    current_text = String::new();
                }
                in_tag = true;
            }
            '>' => {
                // Finish the tag
                if in_tag {
                    // Normalize tag name (lowercase, trim) for easier matching later
                    let tag_name = current_text.trim().to_lowercase();
                    if !tag_name.is_empty() {
                        tokens.push(Token::Tag(tag_name));
                    } // else: ignore empty tags like <>
                    current_text = String::new();
                    in_tag = false;
                } else {
                    // '>' character outside a tag, treat as text
                    current_text.push(c);
                }
            }
            _ => {
                current_text.push(c);
            }
        }
    }

    // Add any remaining text after the last tag
    if !in_tag && !current_text.is_empty() {
        tokens.push(Token::Text(current_text));
    }
    // for token in tokens.clone() {
    //     match token {
    //         Token::Tag(tag) => {
    //             println!("Tag: {}", tag);
    //         }
    //         Token::Text(txt) => {
    //             println!("Text: `{}`", txt);
    //         }
    //     }
    // }
    tokens
}
