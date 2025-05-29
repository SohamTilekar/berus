# Supported HTML Tags

This document outlines the HTML tags currently recognized and partially supported by the Berus browser.

## Structural Tags

- **`<html>`**: The root element of an HTML page.
- **`<head>`**: Contains machine-readable information (metadata) about the HTML document, like its title and links to stylesheets.
- **`<body>`**: Represents the content of an HTML document. There is only one `<body>` element in a document.
- **`<div>`**: A generic container element for flow content. Often used for layout purposes.
- **`<span>`**: A generic inline container for phrasing content, which does not inherently represent anything.
- **`<p>`**: Represents a paragraph.
- **`<table>`**: Represents tabular data. Basic rendering of table structure is supported.
- **`<thead>`**: Represents the block of rows that describe the column headers of a table.
- **`<tbody>`**: Represents the block of rows that comprise the primary table data.
- **`<tfoot>`**: Represents the block of rows that describe the column footers of a table.
- **`<tr>`**: Defines a row of cells in a table.
- **`<th>`**: Defines a cell as a header of a group of table cells.
- **`<td>`**: Defines a cell of a table that contains data.
- **`<caption>`**: Specifies the caption (or title) of a table.

## Text Formatting and Semantic Tags

- **`<h1>` to `<h6>`**: Heading elements, representing six levels of section headings. Basic font size scaling is applied.
- **`<strong>`**: Indicates that its contents have strong importance, seriousness, or urgency. Renders text in **bold**.
- **`<em>`**: Marks text that needs to be stressed or emphasized. Renders text in *italic*.
- **`<small>`**: Represents side comments and small print, like copyright and legal text. Renders text in a smaller font size.
- **`<big>`**: Renders text in a larger font size (deprecated in HTML5, but supported here).
- **`<b>`**: Renders text in **bold** (presentational).
- **`<w>`**: Custom tag used internally or for testing "weak" emphasis. Renders text in a weaker (lighter) font style.
- **`<u>`**: Represents a span of inline text rendered with a solid underline.
- **`<i>`**: Renders text in *italic* (presentational).
- **`<s>`**: Renders text with a strikethrough, indicating text that is no longer accurate or relevant.
- **`<br>`**: Produces a line break in text.
- **`<hr>`**: Represents a thematic break between paragraph-level elements. Renders as a horizontal line.
- **`<a>`**: Represents a hyperlink. Supports the `href` attribute for navigation (opens in a new tab). Renders with underline and blue color by default.
- **`<abbr>`**: Represents an abbreviation or acronym. Supports the `title` attribute to provide the full description on hover.
- **`<title>`**: Defines the title of the document, which appears in the browser tab or window title bar. (Handled internally to update tab titles).

## Media Tags

- **`<img>`**: Represents an image. Supports the `src`, `width`, and `height` attributes. Basic image loading is supported. `alt` and `title` attributes are used for hover text.
- **`<audio>`**: Used to embed audio content. Supports the `src`, `autoplay`, `loop`, and `controls` attributes. A basic audio player interface is rendered if `controls` is present.

## Scripting and Style Tags

- **`<script>`**: Used to embed or reference executable code (typically JavaScript). The content is parsed as raw text but *not executed*.
- **`<style>`**: Used to contain CSS style information for a document. The CSS content within this tag is parsed and applied to the HTML tree.

Any other tags encountered are treated as custom elements (`HtmlTag::Custom`) and will behave like a `div` or `span` unless specific CSS `display` properties are applied.