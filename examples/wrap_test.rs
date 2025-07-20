use crossterm::{
    event::{self, Event, KeyCode},
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
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut textarea = TextArea::default();
    textarea.set_block(
        Block::default()
            .borders(Borders::ALL)
            .title("Text Area with Wrapping - Toggle with Ctrl+W"),
    );
    
    // Add some long text to test wrapping
    textarea.insert_str("This is a very long line of text that should be wrapped when wrapping is enabled. It contains many words and should demonstrate the wrapping functionality clearly.");
    textarea.insert_newline();
    textarea.insert_str("Another long line that will also be wrapped. You can toggle wrapping on and off with Ctrl+W to see the difference in behavior.");
    
    #[cfg(feature = "wrap")]
    {
        textarea.set_wrap(true);
        textarea.set_wrap_width(Some(60)); // Wrap at 60 characters
    }

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3), Constraint::Length(3)])
                .split(f.area());

            f.render_widget(&textarea, chunks[0]);

            let info = if cfg!(feature = "wrap") {
                #[cfg(feature = "wrap")]
                {
                    format!(
                        "Wrapping: {} | Width: {:?} | Ctrl+W: toggle wrap | Ctrl+Q: quit",
                        if textarea.wrap_enabled() { "ON" } else { "OFF" },
                        textarea.wrap_width()
                    )
                }
                #[cfg(not(feature = "wrap"))]
                {
                    "Wrap feature not enabled | Ctrl+Q: quit".to_string()
                }
            } else {
                "Wrap feature not enabled | Ctrl+Q: quit".to_string()
            };

            let info_paragraph = Paragraph::new(info)
                .style(Style::default().fg(Color::Gray))
                .block(Block::default().borders(Borders::ALL).title("Info"));
            f.render_widget(info_paragraph, chunks[1]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                    break;
                }
                #[cfg(feature = "wrap")]
                KeyCode::Char('w') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                    textarea.set_wrap(!textarea.wrap_enabled());
                }
                _ => {
                    textarea.input(key);
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}