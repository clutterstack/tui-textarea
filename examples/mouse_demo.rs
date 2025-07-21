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
use std::io::{self, Write};
use std::fs::OpenOptions;
use tui_textarea::TextArea;

fn main() -> io::Result<()> {
    // Create log file
    let mut log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("mouse_debug.log")?;
    
    writeln!(log_file, "Mouse demo starting...")?;
    
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut textarea = TextArea::default();
    textarea.set_block(
        Block::default()
            .borders(Borders::ALL)
            .title("Mouse Click Demo - Click anywhere to position cursor"),
    );
    
    // Add some sample text to demonstrate clicking
    textarea.insert_str("Click anywhere in this text to position the cursor!\n");
    textarea.insert_str("This line is longer and demonstrates how mouse clicking works with longer text that might extend beyond the visible area.\n");
    textarea.insert_str("Try clicking:\n");
    textarea.insert_str("- At the beginning of lines\n");
    textarea.insert_str("- At the end of lines\n");
    textarea.insert_str("- In the middle of words\n");
    textarea.insert_str("- On empty lines\n");
    textarea.insert_str("\n");
    textarea.insert_str("Text wrapping demo (enable with 'w' key):\n");
    textarea.insert_str("This is a very long line of text that will demonstrate how mouse clicking works with text wrapping when it is enabled. You can toggle wrapping with the 'w' key.");
    
    // Start with cursor at the beginning
    textarea.move_cursor(tui_textarea::CursorMove::Top);

    let mut last_click_info = String::new();

    loop {
        let _ = log_file.flush();
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3), Constraint::Length(5)])
                .split(f.area());

            // Get the text area for mouse coordinate translation
            let text_area = chunks[0];
            
            f.render_widget(&textarea, text_area);

            // Show information panel
            let wrap_status = {
                #[cfg(feature = "wrap")]
                {
                    if textarea.wrap_enabled() { "ON" } else { "OFF" }
                }
                #[cfg(not(feature = "wrap"))]
                {
                    "OFF (feature disabled)"
                }
            };
            
            let info_text = format!(
                "Controls:\n\
                 • Click anywhere in the text area to position cursor\n\
                 • 'w' - Toggle text wrapping (current: {})\n\
                 • 'l' - Toggle line numbers\n\
                 • 'Ctrl+q' - Quit\n\
                 \n\
                 Cursor: ({}, {})\n\
                 {}",
                wrap_status,
                textarea.cursor().0,
                textarea.cursor().1,
                last_click_info
            );

            let info_paragraph = Paragraph::new(info_text)
                .style(Style::default().fg(Color::Gray))
                .block(Block::default().borders(Borders::ALL).title("Info"));
            f.render_widget(info_paragraph, chunks[1]);
        })?;

        let event = event::read()?;
        writeln!(log_file, "Event received: {:?}", event)?;
        
        if let Event::Key(key) = event {
            match (key.code,  {
                (KeyCode::Char('q'), KeyModifiers::CONTROL) => {
                    break;
                }
                KeyCode::Char('w') => {
                    #[cfg(feature = "wrap")]
                    {
                        textarea.set_wrap(!textarea.wrap_enabled());
                        last_click_info = format!("Toggled wrapping to: {}", textarea.wrap_enabled());
                    }
                    #[cfg(not(feature = "wrap"))]
                    {
                        last_click_info = "Wrapping feature not enabled".to_string();
                    }
                }
                KeyCode::Char('l') => {
                    if textarea.line_number_style().is_some() {
                        textarea.remove_line_number();
                        last_click_info = "Line numbers disabled".to_string();
                    } else {
                        textarea.set_line_number_style(Style::default().fg(Color::DarkGray));
                        last_click_info = "Line numbers enabled".to_string();
                    }
                }
                _ => {
                    textarea.input(key);
                }
            }
        } else if let Event::Mouse(mouse) = event {
            writeln!(log_file, "Mouse event: {:?}", mouse)?;
            match mouse.kind {
                MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                    writeln!(log_file, "Left mouse button down at ({}, {})", mouse.column, mouse.row)?;
                    
                    // Get the widget area that matches what we use in rendering
                    let size = terminal.size()?;
                    let full_area = ratatui::layout::Rect::new(0, 0, size.width, size.height);
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Min(3), Constraint::Length(5)])
                        .split(full_area);
                    let widget_area = chunks[0]; // This is the full widget area including borders
                    
                    writeln!(log_file, "Widget area: {:?}", widget_area)?;
                    writeln!(log_file, "Terminal size: {:?}", size)?;
                    
                    #[cfg(feature = "mouse")]
                    {
                        writeln!(log_file, "Calling handle_mouse_click...")?;
                        let cursor_before = textarea.cursor();
                        let result = textarea.handle_mouse_click(mouse.column, mouse.row, widget_area);
                        let cursor_after = textarea.cursor();
                        writeln!(log_file, "handle_mouse_click returned: {}", result)?;
                        writeln!(log_file, "Cursor before: {:?}, after: {:?}", cursor_before, cursor_after)?;
                        
                        if result {
                            last_click_info = format!(
                                "Clicked at screen ({}, {}) -> cursor ({}, {})",
                                mouse.column, mouse.row, cursor_after.0, cursor_after.1
                            );
                        } else {
                            last_click_info = format!(
                                "Click at ({}, {}) was outside text area",
                                mouse.column, mouse.row
                            );
                        }
                    }
                    #[cfg(not(feature = "mouse"))]
                    {
                        writeln!(log_file, "Mouse feature not enabled")?;
                        last_click_info = "Mouse feature not enabled - compile with --features mouse".to_string();
                    }
                }
                _ => {
                    // Handle other mouse events normally (like scrolling)
                    textarea.input(mouse);
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}