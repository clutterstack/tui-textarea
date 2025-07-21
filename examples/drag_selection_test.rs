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
            .title("Drag Selection Test - Try mouse drag to select text!"),
    );
    
    // Set a visible selection style for better visual feedback
    textarea.set_selection_style(Style::default().bg(Color::LightBlue).fg(Color::Black));
    
    // Add some sample text to demonstrate drag selection
    textarea.insert_str("This is line 1 with some text to select.\n");
    textarea.insert_str("This is line 2 with more content here.\n");
    textarea.insert_str("Line 3 has different words you can drag over.\n");
    textarea.insert_str("Try dragging your mouse to select text!\n");
    textarea.insert_str("The selection should work with mouse drag.");
    
    // Start with cursor at the beginning
    textarea.move_cursor(tui_textarea::CursorMove::Top);

    let mut last_event_info = String::new();

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3), Constraint::Length(7)])
                .split(f.area());

            // Get the text area for mouse coordinate translation
            let text_area = chunks[0];
            
            f.render_widget(&textarea, text_area);

            // Show information panel
            let selection_info = if let Some(range) = textarea.selection_range() {
                format!("Selection: {:?} -> {:?}", range.0, range.1)
            } else {
                "No selection".to_string()
            };

            let info_text = format!(
                "Drag Selection Test\n\
                 Instructions:\n\
                 • Click and drag to select text\n\
                 • Ctrl+C to copy selected text\n\
                 • Ctrl+X to cut selected text\n\
                 • Ctrl+V to paste\n\
                 • Ctrl+Q to quit\n\
                 \n\
                 Cursor: ({}, {})\n\
                 {}\n\
                 {}",
                textarea.cursor().0,
                textarea.cursor().1,
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
                    // Get the widget area that matches what we use in rendering
                    let size = terminal.size()?;
                    let full_area = ratatui::layout::Rect::new(0, 0, size.width, size.height);
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Min(3), Constraint::Length(7)])
                        .split(full_area);
                    let widget_area = chunks[0]; // This is the full widget area including borders
                    
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