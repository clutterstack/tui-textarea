use crossterm::{
    event::{self, Event, KeyCode, MouseEventKind, EnableMouseCapture, DisableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::io;
use tui_textarea::{TextArea, Key};

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut textarea = TextArea::default();
    textarea.set_block(
        Block::default()
            .borders(Borders::ALL)
            .title("Debug Selection - Enable/Disable Wrapping with 'W'"),
    );
    
    // Set visible selection styling
    textarea.set_selection_style(Style::default().bg(Color::LightBlue).fg(Color::Black));
    
    // Add test text that will wrap
    textarea.insert_str("This is a very long line that should wrap when the width is small enough to trigger text wrapping behavior in the textarea widget.");
    
    // Start with cursor at the beginning
    textarea.move_cursor(tui_textarea::CursorMove::Top);

    let mut last_event_info = String::new();
    let mut wrap_enabled = false;

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3), Constraint::Length(12)])
                .split(f.area());

            let text_area = chunks[0];
            
            f.render_widget(&textarea, text_area);

            let selection_info = if let Some(range) = textarea.selection_range() {
                format!("Selection: {:?} -> {:?}", range.0, range.1)
            } else {
                "No selection".to_string()
            };

            let cursor_pos = textarea.cursor();
            let wrap_status = if wrap_enabled { "ON" } else { "OFF" };
            let wrap_width = if wrap_enabled {
                format!("width: {:?}", textarea.wrap_width())
            } else {
                "disabled".to_string()
            };

            let info_text = format!(
                "Debug Selection Test\n\
                 Instructions:\n\
                 • W: Toggle text wrapping (current: {}) - {}\n\
                 • Click and drag to select text\n\
                 • Ctrl+C to copy, Ctrl+X to cut, Ctrl+V to paste\n\
                 • Ctrl+Q to quit\n\
                 \n\
                 Cursor: ({}, {})\n\
                 {}\n\
                 {}",
                wrap_status,
                wrap_width,
                cursor_pos.0,
                cursor_pos.1,
                selection_info,
                last_event_info
            );

            let info_paragraph = Paragraph::new(info_text)
                .style(Style::default().fg(Color::Gray))
                .block(Block::default().borders(Borders::ALL).title("Info"));
            f.render_widget(info_paragraph, chunks[1]);
        })?;

        let event = event::read()?;
        
        match event {
            Event::Key(key) => {
                match key.code {
                    KeyCode::Char('q') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                        break;
                    }
                    KeyCode::Char('w') | KeyCode::Char('W') => {
                        wrap_enabled = !wrap_enabled;
                        if wrap_enabled {
                            textarea.set_wrap(true);
                            textarea.set_wrap_width(Some(40)); // Force wrapping
                        } else {
                            textarea.set_wrap(false);
                            textarea.set_wrap_width(None);
                        }
                        last_event_info = format!("Text wrapping: {}", if wrap_enabled { "ON" } else { "OFF" });
                    }
                    _ => {
                        textarea.input(key);
                    }
                }
            }
            Event::Mouse(mouse) => {
                let mouse_key = match mouse.kind {
                    MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                        last_event_info = format!("Mouse Down at ({}, {})", mouse.column, mouse.row);
                        Some(Key::MouseClick(mouse.column, mouse.row))
                    }
                    MouseEventKind::Drag(crossterm::event::MouseButton::Left) => {
                        last_event_info = format!("Mouse Drag to ({}, {})", mouse.column, mouse.row);
                        Some(Key::MouseDrag(mouse.column, mouse.row))
                    }
                    MouseEventKind::Up(crossterm::event::MouseButton::Left) => {
                        last_event_info = format!("Mouse Up at ({}, {})", mouse.column, mouse.row);
                        Some(Key::MouseUp(mouse.column, mouse.row))
                    }
                    _ => {
                        textarea.input(mouse);
                        None
                    }
                };

                if let Some(key) = mouse_key {
                    let size = terminal.size()?;
                    let full_area = ratatui::layout::Rect::new(0, 0, size.width, size.height);
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Min(3), Constraint::Length(12)])
                        .split(full_area);
                    let widget_area = chunks[0];
                    
                    let handled = textarea.handle_mouse_event(key, widget_area);
                    last_event_info.push_str(&format!(" -> handled: {}", handled));
                }
            }
            _ => {}
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}