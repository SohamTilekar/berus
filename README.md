# Berus Browser

Berus is a simple web browser built with Rust using the `eframe` and `egui` libraries for the graphical user interface. This project aims to render basic HTML content and apply some CSS styling.

## Project Structure

- `src/main.rs`: Entry point of the application. Initializes the `eframe` and `BrowserApp`.
- `src/browser.rs`: Contains the main `BrowserApp` struct, handling tab management, URL loading, network requests, and the core rendering loop using `egui`. It processes the parsed HTML tree and applies styles during rendering.
- `src/html_parser.rs`: Implements a basic HTML parser to convert raw HTML text into a tree structure (`HtmlNode`). It handles element tags, text nodes, attributes, and performs some cleanup to ensure a standard `<html><body><head>...</head><body>...</body></html>` structure. It also extracts `<style>` tag content.
- `src/css_parser.rs`: Implements a simple CSS parser to parse CSS rules (`CssRule`) from `<style>` tag content. It supports basic selectors (Universal, Class, Id, Type) and property parsing for lengths, colors, and keywords.
- `src/layout.rs`: Defines the data structures used for representing the parsed HTML tree (`HtmlNode`, `NodeType`, `HtmlTag`), CSS rules (`CssRule`, `Selector`, `StyleProperty`), lengths (`Length`), and colors (`Color`). It also includes logic for applying CSS rules to the HTML node tree based on selectors and specificity.
- `src/network.rs`: Handles basic network requests (`http` and `https`) using `reqwest` to fetch content from URLs.
- `src/audio_player.rs`: Implements a simple audio player component using `rodio` to handle playback of audio files linked in HTML `<audio>` tags.

## Supported Features

### Supported HTML Tags

The browser's parser and renderer currently support the following HTML tags:

- **Structural:** `div`, `span`, `p`, `body`, `head`, `html`, `table`, `thead`, `tbody`, `tfoot`, `tr`, `th`, `td`, `caption`
- **Text Formatting:** `h1`, `h2`, `h3`, `h4`, `h5`, `h6`, `strong`, `em`, `small`, `big`, `b`, `w`, `u`, `i`, `s`, `br`, `hr`, `a`, `abbr`, `title`
- **Media:** `img`, `audio` (with basic controls)
- **Scripting/Styling:** `script` (content is parsed as raw text but not executed), `style` (content is parsed and applied as CSS)

### Supported CSS Properties

The CSS parser and rendering logic currently interpret and apply the following CSS properties:

- **Color:**
    - `text-color`, `color`: Applies color to text.
    - `background-color`: Applies background color to elements.
    - Supported color formats: Hex (`#rgb`, `#rrggbb`, `#rgba`, `#rrggbbaa`), `rgb()`, `rgba()`, `hsl()`, `hsla()`, and basic named colors (e.g., `red`, `blue`).
- **Box Model (Basic):**
    - `padding`: Applies padding equally to all sides.
    - `padding-top`, `padding-bottom`, `padding-left`, `padding-right`: Applies padding to specific sides.
    - `margin`: Applies margin equally to all sides.
    - `margin-top`, `margin-bottom`, `margin-left`, `margin-right`: Applies margin to specific sides.
    - Supported length units for padding and margin: `px`, `em`, `rem`, `%`.
- **Borders:**
    - `border-width`: Sets the width of the border.
    - `border-color`: Sets the color of the border.
    - `border-radius`: Sets the corner radius for all corners.
    - `border-radius-ne`, `border-radius-nw`, `border-radius-se`, `border-radius-sw`: Sets corner radius for specific corners.
    - Supported length units for border properties: `px`, `em`, `rem`, `%`.
- **Text Decoration:**
    - `text-decoration`: Supports `underline` and `strikethrough`.
- **Font Styles (Basic):**
    - `font-weight`: Supports `bold`, `bolder`, `lighter`.
    - `font-style`: Supports `normal`, `italic`, `bold`, `underline`, `strikethrough`.

## How to Run

(Assuming you have Rust and Cargo installed)

1. Navigate to the project root directory.
2. Run the application: `cargo run <optional-url>`
   - Replace `<optional-url>` with a URL you want to load initially (e.g., `cargo run https://example.com`).

## Limitations

This browser is currently very basic. Many standard HTML features, CSS properties, layout modes (like Flexbox or Grid), JavaScript execution, and advanced web standards are not yet supported. The rendering is a simplified interpretation based on `egui`'s capabilities.
