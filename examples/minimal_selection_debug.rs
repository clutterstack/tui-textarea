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
            .title("Minimal Debug - Testing wrapped selection"),
    );
    
    // Set VERY visible selection styling
    textarea.set_selection_style(Style::default().bg(Color::Red).fg(Color::Yellow));
    
    // Explicitly disable wrapping
    textarea.set_wrap(true);
    
    // Add text that will wrap to test selection highlighting
    textarea.insert_str("This is a very long line of text that should definitely wrap when the terminal is not wide enough to display it all on one line");
    textarea.move_cursor(tui_textarea::CursorMove::Top);

    let mut debug_log = Vec::<String>::new();

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3), Constraint::Length(15)])
                .split(f.area());

            let text_area = chunks[0];
            f.render_widget(&textarea, text_area);

            // Debug info
            let cursor_pos = textarea.cursor();
            let selection_range = textarea.selection_range();
            
            let selection_info = if let Some(range) = selection_range {
                format!("SELECTION ACTIVE: start={:?} end={:?}", range.0, range.1)
            } else {
                "NO SELECTION".to_string()
            };

            let mut debug_text = format!(
                "Minimal Debug - Click and drag 'Hello'\n\
                 X: Manual selection, C: Cancel, Q: Quit\n\
                 \n\
                 Text: {:?}\n\
                 Cursor: ({}, {})\n\
                 {}\n\
                 \n\
                 Recent events:",
                textarea.lines().get(0).unwrap_or(&"".to_string()),
                cursor_pos.0,
                cursor_pos.1,
                selection_info
            );

            // Show last 8 debug events
            for event in debug_log.iter().rev().take(8).rev() {
                debug_text.push_str("\n");
                debug_text.push_str(event);
            }

            let info_paragraph = Paragraph::new(debug_text)
                .style(Style::default().fg(Color::Gray))
                .block(Block::default().borders(Borders::ALL).title("Debug Log"));
            f.render_widget(info_paragraph, chunks[1]);
        })?;

        let event = event::read()?;
        
        match event {
            Event::Key(key) => {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => break,
                    KeyCode::Char('x') | KeyCode::Char('X') => {
                        // Manual selection of "Hello" (positions 0-5)
                        textarea.move_cursor(tui_textarea::CursorMove::Jump(0, 0));
                        textarea.start_selection();
                        textarea.move_cursor(tui_textarea::CursorMove::Jump(0, 5));
                        debug_log.push("MANUAL: Selected 'Hello' (0,0) to (0,5)".to_string());
                    }
                    KeyCode::Char('c') | KeyCode::Char('C') => {
                        textarea.cancel_selection();
                        debug_log.push("CANCELLED selection".to_string());
                    }
                    _ => {
                        let old_cursor = textarea.cursor();
                        textarea.input(key);
                        let new_cursor = textarea.cursor();
                        debug_log.push(format!("KEY {:?}: cursor {:?} -> {:?}", key.code, old_cursor, new_cursor));
                    }
                }
            }
            Event::Mouse(mouse) => {
                let mouse_key = match mouse.kind {
                    MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                        debug_log.push(format!("MOUSE DOWN at ({}, {})", mouse.column, mouse.row));
                        Some(Key::MouseClick(mouse.column, mouse.row))
                    }
                    MouseEventKind::Drag(crossterm::event::MouseButton::Left) => {
                        debug_log.push(format!("MOUSE DRAG to ({}, {})", mouse.column, mouse.row));
                        Some(Key::MouseDrag(mouse.column, mouse.row))
                    }
                    MouseEventKind::Up(crossterm::event::MouseButton::Left) => {
                        debug_log.push(format!("MOUSE UP at ({}, {})", mouse.column, mouse.row));
                        Some(Key::MouseUp(mouse.column, mouse.row))
                    }
                    _ => {
                        textarea.input(mouse);
                        None
                    }
                };

                if let Some(key) = mouse_key {
                    let old_cursor = textarea.cursor();
                    let old_selection = textarea.selection_range();
                    
                    let size = terminal.size()?;
                    let full_area = ratatui::layout::Rect::new(0, 0, size.width, size.height);
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Min(3), Constraint::Length(15)])
                        .split(full_area);
                    let widget_area = chunks[0];
                    
                    let handled = textarea.handle_mouse_event(key, widget_area);
                    let new_cursor = textarea.cursor();
                    let new_selection = textarea.selection_range();
                    
                    debug_log.push(format!(
                        "  -> handled={}, cursor {:?}->{:?}, sel {:?}->{:?}",
                        handled, old_cursor, new_cursor, old_selection, new_selection
                    ));
                    
                    // Keep debug log manageable
                    if debug_log.len() > 20 {
                        debug_log.remove(0);
                    }
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