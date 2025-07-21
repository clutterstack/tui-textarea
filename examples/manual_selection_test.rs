use crossterm::{
    event::{self, Event, KeyCode, EnableMouseCapture, DisableMouseCapture},
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
use tui_textarea::TextArea;

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
            .title("Manual Selection Test - Press S to create selection"),
    );
    
    // Set VERY visible selection styling
    textarea.set_selection_style(Style::default().bg(Color::Red).fg(Color::White));
    
    // Disable wrapping
    textarea.set_wrap(false);
    
    // Add test text
    textarea.insert_str("This is a test string for selection");
    textarea.move_cursor(tui_textarea::CursorMove::Top);

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3), Constraint::Length(8)])
                .split(f.area());

            let text_area = chunks[0];
            f.render_widget(&textarea, text_area);

            // Debug info
            let cursor_pos = textarea.cursor();
            let selection_range = textarea.selection_range();
            
            let selection_info = if let Some(range) = selection_range {
                format!("Selection: {:?} -> {:?}", range.0, range.1)
            } else {
                "No selection".to_string()
            };

            let debug_text = format!(
                "Manual Selection Test\n\
                 S: Create manual selection from pos 5-12\n\
                 C: Cancel selection\n\
                 Ctrl+Q: Quit\n\
                 \n\
                 Cursor: ({}, {})\n\
                 {}",
                cursor_pos.0,
                cursor_pos.1,
                selection_info
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
                    KeyCode::Char('s') | KeyCode::Char('S') => {
                        // Manually create a selection from character 5 to 12 ("is a test")
                        textarea.move_cursor(tui_textarea::CursorMove::Jump(0, 5));  // Move to "i" in "is"
                        textarea.start_selection();
                        textarea.move_cursor(tui_textarea::CursorMove::Jump(0, 13)); // Move to " " after "test"
                        println!("Created manual selection from (0,5) to (0,13)");
                    }
                    KeyCode::Char('c') | KeyCode::Char('C') => {
                        textarea.cancel_selection();
                        println!("Cancelled selection");
                    }
                    _ => {
                        textarea.input(key);
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