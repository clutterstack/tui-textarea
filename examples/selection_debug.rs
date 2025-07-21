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
            .title("Selection Debug - Click and Drag"),
    );
    
    // Set visible selection styling
    textarea.set_selection_style(Style::default().bg(Color::LightBlue).fg(Color::Black));
    
    // Disable wrapping for now
    textarea.set_wrap(false);
    
    // Add simple test text
    textarea.insert_str("Hello world this is a test");
    textarea.move_cursor(tui_textarea::CursorMove::Top);

    let mut last_event_info = String::new();

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3), Constraint::Length(10)])
                .split(f.area());

            let text_area = chunks[0];
            f.render_widget(&textarea, text_area);

            // Debug info
            let cursor_pos = textarea.cursor();
            let selection_range = textarea.selection_range();
            let has_selection = selection_range.is_some();
            
            let selection_info = if let Some(range) = selection_range {
                format!("Selection: {:?} -> {:?}", range.0, range.1)
            } else {
                "No selection".to_string()
            };

            let debug_text = format!(
                "Selection Debug\n\
                 Instructions: Click and drag to select\n\
                 Ctrl+Q to quit\n\
                 \n\
                 Cursor: ({}, {})\n\
                 Has selection: {}\n\
                 {}\n\
                 {}\n\
                 Text: {:?}",
                cursor_pos.0,
                cursor_pos.1,
                has_selection,
                selection_info,
                last_event_info,
                textarea.lines()[0]
            );

            let info_paragraph = Paragraph::new(debug_text)
                .style(Style::default().fg(Color::Gray))
                .block(Block::default().borders(Borders::ALL).title("Debug Info"));
            f.render_widget(info_paragraph, chunks[1]);
        })?;

        let event = event::read()?;
        
        match event {
            Event::Key(key) => {
                match key.code {
                    KeyCode::Char('q') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                        break;
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
                        .constraints([Constraint::Min(3), Constraint::Length(10)])
                        .split(full_area);
                    let widget_area = chunks[0];
                    
                    let handled = textarea.handle_mouse_event(key, widget_area);
                    last_event_info.push_str(&format!(" -> handled: {}, cursor now: {:?}", handled, textarea.cursor()));
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