# Supported CSS Properties and Values

This document details the CSS properties, values, and units that are currently recognized and applied by the Berus browser.

## Value Types

### Color Values

Supported color formats:
- **Hexadecimal:**
  - `#rgb` (e.g., `#f00` for red)
  - `#rrggbb` (e.g., `#ff0000` for red)
  - `#rgba` (e.g., `#f008` for semi-transparent red)
  - `#rrggbbaa` (e.g., `#ff000080` for semi-transparent red)
- **Functional:**
  - `rgb(r, g, b)` (e.g., `rgb(255, 0, 0)`) - `r`, `g`, `b` are integers 0-255.
  - `rgba(r, g, b, a)` (e.g., `rgba(255, 0, 0, 0.5)`) - `r`, `g`, `b` are integers 0-255, `a` is a float 0.0 to 1.0.
  - `hsl(h, s, l)` (e.g., `hsl(0, 100%, 50%)`) - `h`, `s`, `l` are integers 0-255 based on implementation, not standard CSS hue (0-360) and percentage (0-100%).
  - `hsla(h, s, l, a)` (e.g., `hsla(0, 100%, 50%, 0.5)`) - `h`, `s`, `l` as above, `a` is float 0.0 to 1.0.
- **Named Colors:** A wide range of standard CSS named colors (e.g., `red`, `blue`, `black`, `white`, `gray`, etc.) are supported as defined in `src/layout.rs`.

### Length Values

Supported length units for relevant properties:
- `px`: Pixels. Represents screen pixels directly.
- `em`: Relative to the font size of the element.
- `rem`: Relative to the font size of the root element (`<html>`). (Note: Currently based on the browser's base font size setting).
- `%`: Percentage. Relative to the size of the parent element (width for horizontal properties, height for vertical properties).

### Keyword Values

Specific string values used for certain properties (e.g., `bold`, `italic`, `underline`, `block`, `inline`).

## Supported Properties

### `color` / `text-color`

- **Value:** `color`
- Sets the foreground color of the text content within an element.

Example:
```css
p {
  color: blue;
}
span {
  text-color: #ff00ff; /* magenta */
}
```

### `background-color`

- **Value:** `color`
- Sets the background color of the content, padding, and border areas of an element.

Example:
```css
div {
  background-color: rgba(0, 0, 0, 0.1); /* light black with transparency */
}
```

### `padding`, `padding-top`, `padding-bottom`, `padding-left`, `padding-right`

- **Value:** `Length`
- Sets the space between an element's content and its border.
  - `padding`: Applies to all four sides.
  - `padding-top`, `padding-bottom`, `padding-left`, `padding-right`: Apply to specific sides.

Example:
```css
.box {
  padding: 10px; /* 10 pixels on all sides */
  padding-left: 2em; /* overrides padding on the left */
}
```

### `margin`, `margin-top`, `margin-bottom`, `margin-left`, `margin-right`

- **Value:** `Length`
- Sets the space outside an element's border.
  - `margin`: Applies to all four sides.
  - `margin-top`, `margin-bottom`, `margin-left`, `margin-right`: Apply to specific sides.

Example:
```css
h1 {
  margin-bottom: 1.5rem; /* Space below the heading */
}
```

### `border-width`

- **Value:** `Length`
- Sets the thickness of an element's border. (Note: Border style is currently solid by default).

Example:
```css
img {
  border-width: 2px;
  border-color: gray; /* Needs border-color to be visible */
}
```

### `border-color`

- **Value:** `color`
- Sets the color of an element's border.

Example:
```css
.highlight {
  border-width: 1px;
  border-color: yellow;
}
```

### `border-radius`, `border-radius-ne`, `border-radius-nw`, `border-radius-se`, `border-radius-sw`

- **Value:** `Length`
- Rounds the corners of an element's outer border edge.
  - `border-radius`: Applies equally to all four corners.
  - `border-radius-ne`: North-East (top-right)
  - `border-radius-nw`: North-West (top-left)
  - `border-radius-se`: South-East (bottom-right)
  - `border-radius-sw`: South-West (bottom-left)

Example:
```css
button {
  border-radius: 5px; /* slightly rounded corners */
}
.circle {
  border-radius: 50%; /* circular */
}
```

### `text-decoration`

- **Value:** `keyword` (`underline`, `strikethrough`)
- Applies text decoration lines to an element's text.

Example:
```css
.important {
  text-decoration: underline;
}
.old-price {
  text-decoration: strikethrough;
}
```

### `font-weight`

- **Value:** `keyword` (`bold`, `bolder`, `lighter`)
- Sets the weight (or boldness) of the font.
  - `bold`: Renders text as bold.
  - `bolder`: Makes the text bolder than its parent (basic implementation just increases font size slightly).
  - `lighter`: Makes the text lighter than its parent (basic implementation just decreases font size slightly).

Example:
```css
.title {
  font-weight: bold;
}
```

### `font-style`

- **Value:** `keyword` (`normal`, `italic`, `bold`, `underline`, `strikethrough`)
- Sets the style of the font. Can also reset styles with `normal`.

Example:
```css
.caption {
  font-style: italic;
}
.reset {
  font-style: normal; /* remove any inherited styles */
}
```

### `display`

- **Value:** `keyword` (`block`, `inline`)
- Sets whether an element is rendered as a block-level element (taking up the full width available and starting on a new line) or an inline element (taking only the space it needs and flowing with the text).

Example:
```css
.my-div {
  display: inline; /* makes a div behave like a span */
}
.my-span {
  display: block; /* makes a span behave like a div */
}
```