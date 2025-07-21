# tui-textarea Reference Guide for Ratatui Text Editors

A comprehensive reference for integrating `tui-textarea` into ratatui-based terminal applications.

## Table of Contents

- [Overview](#overview)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Core API Reference](#core-api-reference)
- [Configuration Options](#configuration-options)
- [Text Operations](#text-operations)
- [Cursor Movement](#cursor-movement)
- [Text Selection and Clipboard](#text-selection-and-clipboard)
- [Search Functionality](#search-functionality)
- [Text Wrapping](#text-wrapping)
- [Styling and Theming](#styling-and-theming)
- [Advanced Usage Patterns](#advanced-usage-patterns)
- [Integration Examples](#integration-examples)
- [Best Practices](#best-practices)

## Overview

`tui-textarea` is a powerful multi-line text editor widget for ratatui applications. It provides:

- **Multi-line editing** with automatic scrolling
- **Text wrapping** support (optional feature)
- **Emacs-like key bindings** with full customization
- **Undo/Redo** functionality with configurable history
- **Text search** with regular expressions (optional feature)
- **Text selection** and yank/paste operations
- **Mouse support** for scrolling
- **Multiple textarea instances** in the same application
- **Backend agnostic** (crossterm, termion, termwiz)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ratatui = "0.29"
tui-textarea = "0.7"
crossterm = "0.28"  # or termion/termwiz
```

### Optional Features

```toml
[dependencies]
tui-textarea = { version = "0.7", features = ["search", "wrap", "serde"] }
```

- `search`: Adds regex-based text search functionality
- `wrap`: Enables text wrapping support
- `serde`: Adds serialization/deserialization support

## Quick Start

### Basic Setup

```rust
use tui_textarea::{TextArea, Input, Key};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
    widgets::{Block, Borders},
};
use crossterm::event::{Event, read};

// Create textarea
let mut textarea = TextArea::default();
textarea.set_block(Block::default().borders(Borders::ALL).title("Editor"));

// Event loop
loop {
    term.draw(|f| {
        f.render_widget(&textarea, f.area());
    })?;

    if let Event::Key(key) = read()? {
        match key.code {
            KeyCode::Esc => break,
            _ => { textarea.input(key); }
        }
    }
}

// Get content
let content = textarea.lines();
```

### Creating TextArea with Content

```rust
// From string slice
let textarea = TextArea::from([
    "Line 1",
    "Line 2", 
    "Line 3"
]);

// From file
use std::io::BufReader;
let file = std::fs::File::open("file.txt")?;
let textarea: TextArea = BufReader::new(file)
    .lines()
    .collect::<io::Result<_>>()?;

// From Vec<String>
let lines = vec!["Line 1".to_string(), "Line 2".to_string()];
let textarea = TextArea::new(lines);
```

## Core API Reference

### TextArea Creation

| Method | Description |
|--------|-------------|
| `TextArea::default()` | Create empty textarea |
| `TextArea::new(lines: Vec<String>)` | Create with initial content |
| `TextArea::from<I>(iter: I)` | Create from iterator of strings |

### Input Handling

| Method | Description |
|--------|-------------|
| `textarea.input(input)` | Handle key input with default bindings |
| `textarea.input_without_shortcuts(input)` | Handle input with minimal bindings |

### Content Access

| Method | Description |
|--------|-------------|
| `textarea.lines()` | Get content as `&[String]` |
| `textarea.into_lines()` | Move content out as `Vec<String>` |

### Widget Rendering

```rust
// TextArea implements Widget trait
f.render_widget(&textarea, rect);

// Or get widget explicitly
let widget = textarea.widget();
f.render_widget(widget, rect);
```

## Configuration Options

### Basic Configuration

```rust
let mut textarea = TextArea::default();

// Set tab width (default: 4)
textarea.set_tab_length(2);

// Set undo history size (default: 50, 0 disables)
textarea.set_max_histories(100);

// Set placeholder text
textarea.set_placeholder_text("Enter your text here...");
```

### Block and Borders

```rust
use ratatui::widgets::{Block, Borders};

textarea.set_block(
    Block::default()
        .borders(Borders::ALL)
        .title("My Editor")
);
```

## Text Operations

### Basic Text Manipulation

| Method | Description |
|--------|-------------|
| `insert_char(c: char)` | Insert character at cursor |
| `insert_str(s: &str)` | Insert string at cursor |
| `insert_tab()` | Insert tab (or spaces) |
| `insert_newline()` | Insert new line |
| `delete_char()` | Delete character before cursor |
| `delete_next_char()` | Delete character after cursor |
| `delete_str(n: usize)` | Delete n characters before cursor |

### Line Operations

| Method | Description |
|--------|-------------|
| `delete_line_by_end()` | Delete from cursor to end of line |
| `delete_line_by_head()` | Delete from cursor to start of line |
| `delete_newline()` | Delete newline at cursor |

### Word Operations

| Method | Description |
|--------|-------------|
| `delete_word()` | Delete word before cursor |
| `delete_next_word()` | Delete word after cursor |

### History Operations

| Method | Description |
|--------|-------------|
| `undo()` | Undo last edit |
| `redo()` | Redo last undone edit |

## Cursor Movement

### Basic Movement

```rust
use tui_textarea::CursorMove;

// Character movement
textarea.move_cursor(CursorMove::Forward);   // →
textarea.move_cursor(CursorMove::Back);      // ←
textarea.move_cursor(CursorMove::Up);        // ↑
textarea.move_cursor(CursorMove::Down);      // ↓

// Line movement
textarea.move_cursor(CursorMove::Head);      // Start of line
textarea.move_cursor(CursorMove::End);       // End of line

// Document movement
textarea.move_cursor(CursorMove::Top);       // Start of document
textarea.move_cursor(CursorMove::Bottom);    // End of document

// Word movement
textarea.move_cursor(CursorMove::WordForward);
textarea.move_cursor(CursorMove::WordBack);
textarea.move_cursor(CursorMove::WordEnd);

// Paragraph movement
textarea.move_cursor(CursorMove::ParagraphForward);
textarea.move_cursor(CursorMove::ParagraphBack);

// Jump to specific position
textarea.move_cursor(CursorMove::Jump(row, col));

// Keep cursor in viewport
textarea.move_cursor(CursorMove::InViewport);
```

### Scrolling

```rust
use tui_textarea::Scrolling;

textarea.scroll(Scrolling::PageDown);
textarea.scroll(Scrolling::PageUp);
textarea.scroll(Scrolling::HalfPageDown);
textarea.scroll(Scrolling::HalfPageUp);

// Scroll to specific position
textarea.scroll((row, col));
```

## Text Selection and Clipboard

### Selection Operations

```rust
// Start text selection
textarea.start_selection();

// Cancel selection
textarea.cancel_selection();

// Select all text
textarea.select_all();

// Check if selecting
if textarea.is_selecting() {
    // Selection is active
}
```

### Clipboard Operations

```rust
// Copy selected text to yank buffer
textarea.copy();

// Cut selected text to yank buffer
let was_cut = textarea.cut();

// Paste from yank buffer
let was_pasted = textarea.paste();
```

## Search Functionality

**Requires `search` feature**

```rust
// Set search pattern (regex)
match textarea.set_search_pattern(r"hello\s+world") {
    Ok(_) => {
        // Pattern is valid, matches are highlighted
    }
    Err(err) => {
        // Invalid regex pattern
        eprintln!("Search error: {}", err);
    }
}

// Navigate search results
textarea.search_forward(false);  // Next match
textarea.search_back(false);     // Previous match

// Clear search
textarea.set_search_pattern("").unwrap();
```

### Search Styling

```rust
use ratatui::style::{Color, Style};

// Customize search highlight style
textarea.set_search_style(
    Style::default()
        .bg(Color::Yellow)
        .fg(Color::Black)
);
```

## Text Wrapping

**Requires `wrap` feature**

```rust
// Enable text wrapping
textarea.set_wrap(true);

// Set custom wrap width (None = use textarea width)
textarea.set_wrap_width(Some(80));

// Check wrapping status
if textarea.wrap_enabled() {
    println!("Wrap width: {:?}", textarea.wrap_width());
}

// Disable wrapping
textarea.set_wrap(false);
```

**Note:** Text wrapping is incompatible with horizontal scrolling. When enabled, the editor focuses on vertical navigation only.

## Styling and Theming

### Text Styling

```rust
use ratatui::style::{Color, Modifier, Style};

// Set base text style
textarea.set_style(
    Style::default()
        .fg(Color::White)
        .bg(Color::Black)
);

// Set cursor line style (default: underlined)
textarea.set_cursor_line_style(
    Style::default()
        .bg(Color::DarkGray)
        .add_modifier(Modifier::BOLD)
);

// Disable cursor line highlighting
textarea.set_cursor_line_style(Style::default());
```

### Line Numbers

```rust
// Enable line numbers with styling
textarea.set_line_number_style(
    Style::default()
        .fg(Color::DarkGray)
        .bg(Color::Black)
);

// Disable line numbers (default)
// Don't set line number style or set to None
```

### Selection Styling

```rust
// Customize selection appearance
textarea.set_selection_style(
    Style::default()
        .bg(Color::Blue)
        .fg(Color::White)
);

// Get current selection style
let style = textarea.selection_style();
```

### Cursor Styling

```rust
// Set cursor style (affects cursor character display)
textarea.set_cursor_style(
    Style::default()
        .bg(Color::White)
        .fg(Color::Black)
        .add_modifier(Modifier::REVERSED)
);
```

## Advanced Usage Patterns

### Single-Line Input

```rust
use tui_textarea::{Input, Key};

// Create single-line input
let mut input = TextArea::new(vec!["".to_string()]);

// Custom input handling to prevent newlines
match input_event {
    Input { key: Key::Enter, .. } 
    | Input { key: Key::Char('m'), ctrl: true, .. } => {
        // Ignore newline inputs
        continue;
    }
    input => {
        input.input(input);
    }
}

// Get single line result
let text = input.into_lines().remove(0);
```

### Multiple TextAreas

```rust
let mut editors = [
    TextArea::default(),
    TextArea::default(),
];
let mut focused = 0;

// In event loop
match input {
    Input { key: Key::Tab, .. } => {
        // Switch focus
        focused = (focused + 1) % editors.len();
    }
    input => {
        // Send input to focused editor
        editors[focused].input(input);
    }
}

// Render with different styles for focus
for (i, editor) in editors.iter().enumerate() {
    let block = if i == focused {
        Block::default().borders(Borders::ALL).title("Focused")
    } else {
        Block::default().borders(Borders::ALL).title("Unfocused")
    };
    editor.set_block(block);
    f.render_widget(editor, rects[i]);
}
```

### Custom Key Bindings

```rust
// Disable default shortcuts and implement custom ones
match input {
    Input { key: Key::Char('s'), ctrl: true, .. } => {
        // Custom save operation
        save_file(&textarea)?;
    }
    Input { key: Key::Char('q'), ctrl: true, .. } => {
        // Custom quit
        break;
    }
    input => {
        // Use minimal input handling
        textarea.input_without_shortcuts(input);
    }
}
```

### Vim-like Modal Editing

```rust
#[derive(PartialEq)]
enum Mode { Normal, Insert, Visual }

let mut mode = Mode::Normal;

match (mode, input) {
    (Mode::Normal, Input { key: Key::Char('i'), .. }) => {
        mode = Mode::Insert;
    }
    (Mode::Insert, Input { key: Key::Esc, .. }) => {
        mode = Mode::Normal;
    }
    (Mode::Normal, Input { key: Key::Char('v'), .. }) => {
        mode = Mode::Visual;
        textarea.start_selection();
    }
    (Mode::Insert, input) => {
        textarea.input(input);
    }
    (Mode::Normal, Input { key: Key::Char('h'), .. }) => {
        textarea.move_cursor(CursorMove::Back);
    }
    // Add more vim bindings...
    _ => {}
}
```

## Integration Examples

### File Editor with Search

```rust
struct Editor {
    textarea: TextArea<'static>,
    search_box: TextArea<'static>,
    search_mode: bool,
    file_path: Option<PathBuf>,
}

impl Editor {
    fn new() -> Self {
        let mut textarea = TextArea::default();
        textarea.set_block(Block::default().borders(Borders::ALL).title("Editor"));
        
        let mut search_box = TextArea::default();
        search_box.set_block(Block::default().borders(Borders::ALL).title("Search"));
        
        Self {
            textarea,
            search_box,
            search_mode: false,
            file_path: None,
        }
    }
    
    fn handle_input(&mut self, input: Input) -> io::Result<bool> {
        match input {
            Input { key: Key::Char('s'), ctrl: true, .. } if !self.search_mode => {
                // Start search
                self.search_mode = true;
                self.search_box.delete_line_by_head(); // Clear previous search
                Ok(true)
            }
            Input { key: Key::Esc, .. } if self.search_mode => {
                // Exit search
                self.search_mode = false;
                self.textarea.set_search_pattern("")?;
                Ok(true)
            }
            Input { key: Key::Enter, .. } if self.search_mode => {
                // Execute search
                let pattern = &self.search_box.lines()[0];
                match self.textarea.set_search_pattern(pattern) {
                    Ok(_) => self.textarea.search_forward(false),
                    Err(err) => eprintln!("Search error: {}", err),
                }
                Ok(true)
            }
            input if self.search_mode => {
                self.search_box.input(input);
                Ok(true)
            }
            Input { key: Key::Char('n'), ctrl: true, .. } => {
                // Next search result
                self.textarea.search_forward(false);
                Ok(true)
            }
            Input { key: Key::Char('p'), ctrl: true, .. } => {
                // Previous search result
                self.textarea.search_back(false);
                Ok(true)
            }
            input => {
                self.textarea.input(input);
                Ok(true)
            }
        }
    }
    
    fn render(&self, f: &mut Frame, area: Rect) {
        if self.search_mode {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(3)])
                .split(area);
            
            f.render_widget(&self.textarea, chunks[0]);
            f.render_widget(&self.search_box, chunks[1]);
        } else {
            f.render_widget(&self.textarea, area);
        }
    }
}
```

### Password Input Field

```rust
fn create_password_field() -> TextArea<'static> {
    let mut textarea = TextArea::default();
    
    // Set mask character for password input
    textarea.set_mask_char('●');
    
    // Disable cursor line highlighting for cleaner look
    textarea.set_cursor_line_style(Style::default());
    
    // Set placeholder
    textarea.set_placeholder_text("Enter password...");
    
    // Set styling
    textarea.set_block(
        Block::default()
            .borders(Borders::ALL)
            .title("Password")
            .border_style(Color::Blue)
    );
    
    textarea
}
```

## Best Practices

### Performance

1. **Minimize redraws**: Only call `term.draw()` when necessary
2. **Use appropriate history size**: Balance undo functionality with memory usage
3. **Consider text wrapping**: Enable for better performance with long lines

### User Experience

1. **Provide visual feedback**: Use styling to indicate modes and states
2. **Handle edge cases**: Empty content, large files, etc.
3. **Consistent key bindings**: Follow established conventions (Emacs, Vim, etc.)

### Error Handling

```rust
// Handle regex errors in search
match textarea.set_search_pattern(pattern) {
    Ok(_) => { /* Pattern valid */ }
    Err(regex::Error::Syntax(msg)) => {
        show_error(format!("Invalid regex: {}", msg));
    }
    Err(err) => {
        show_error(format!("Search error: {}", err));
    }
}

// Validate content before saving
fn validate_content(textarea: &TextArea) -> Result<(), String> {
    let content = textarea.lines().join("\n");
    if content.trim().is_empty() {
        return Err("Content cannot be empty".to_string());
    }
    // Additional validation...
    Ok(())
}
```

### Memory Management

```rust
// For large files, consider loading incrementally
fn load_large_file(path: &Path) -> io::Result<TextArea> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    
    // Could implement progressive loading here
    let textarea = reader.lines().collect::<io::Result<TextArea>>()?;
    
    Ok(textarea)
}

// Clean up resources
impl Drop for MyEditor {
    fn drop(&mut self) {
        // Save unsaved changes, clean up temp files, etc.
    }
}
```

### Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_editing() {
        let mut textarea = TextArea::default();
        
        textarea.insert_str("Hello");
        assert_eq!(textarea.lines(), ["Hello"]);
        
        textarea.insert_char(' ');
        textarea.insert_str("World");
        assert_eq!(textarea.lines(), ["Hello World"]);
        
        // Test undo
        textarea.undo();
        assert_eq!(textarea.lines(), ["Hello "]);
    }
    
    #[test]
    fn test_search_functionality() {
        let mut textarea = TextArea::from(["Hello World", "Hello Universe"]);
        
        textarea.set_search_pattern("Hello").unwrap();
        textarea.search_forward(false);
        
        // Verify cursor position, etc.
    }
}
```

This reference guide covers the essential aspects of using `tui-textarea` in ratatui applications. The library provides a robust foundation for building text editors with minimal setup while offering extensive customization options for advanced use cases.