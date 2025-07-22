// Simple demo showing horizontal scrolling functionality
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::io::{self, Stdout};
use tui_textarea::{Input, Key, TextArea, Scrolling};

type Term = Terminal<CrosstermBackend<Stdout>>;

fn run(term: &mut Term) -> io::Result<()> {
    // Create textarea with some long lines
    let mut textarea = TextArea::from([
        "This is a very long line that will demonstrate horizontal scrolling when the viewport is too narrow to display the entire line at once.",
        "Another long line with different content to show how horizontal scrolling works across multiple lines in the textarea widget.",
        "Short line",
        "🦀 Unicode characters and tabs\t\t<- tabs here work correctly with horizontal scrolling as well! 🎉",
        "Line with more text that extends beyond the normal viewport width to test horizontal navigation.",
    ]);

    // Style the textarea
    textarea.set_block(Block::default().borders(Borders::ALL).title("Horizontal Scroll Demo"));
    textarea.set_line_number_style(Style::default().fg(Color::DarkGray));

    loop {
        term.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(3)])
                .split(f.area());

            // Render the textarea
            f.render_widget(&textarea, chunks[0]);

            // Show instructions
            let instructions = Paragraph::new(
                "Use ← → arrows to scroll horizontally, ↑ ↓ arrows to move vertically\n\
                 Ctrl+← → to scroll by larger steps, q to quit, type to add text"
            )
            .block(Block::default().borders(Borders::ALL).title("Instructions"));
            f.render_widget(instructions, chunks[1]);
        })?;

        match crossterm::event::read()?.into() {
            Input { key: Key::Char('q'), .. } => break,
            
            // Manual horizontal scrolling with Ctrl+Left/Right
            Input { key: Key::Left, ctrl: true, .. } => {
                textarea.scroll(Scrolling::Delta { rows: 0, cols: -5 });
            }
            Input { key: Key::Right, ctrl: true, .. } => {
                textarea.scroll(Scrolling::Delta { rows: 0, cols: 5 });
            }
            
            // Regular input handling (includes automatic horizontal scrolling)
            input => {
                textarea.input(input);
            }
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(io::stderr(), crossterm::terminal::EnterAlternateScreen)?;
    let mut term = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let result = run(&mut term);

    crossterm::execute!(io::stderr(), crossterm::terminal::LeaveAlternateScreen)?;
    crossterm::terminal::disable_raw_mode()?;
    
    result
}