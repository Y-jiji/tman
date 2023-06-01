mod edit;
pub use edit::EditView;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    event,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    error::Error,
    io::{self, Stdout},
    time::{Duration, Instant},
};
use tui::{backend::CrosstermBackend, Terminal, Frame};

pub enum Switch {
    Edit{name: String},
    Plan,
    List,
    Exit,
}

pub trait App {
    fn draw(&self, f: &mut Frame<CrosstermBackend<Stdout>>);
    fn quit(&self) -> Option<Switch>;
    fn on_key_code(&mut self, key_code: KeyCode) -> ();
}

pub fn run_app(
    mut app: impl App
) -> Result<Switch, Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    loop {
        terminal.draw(|f| app.draw(f))?;
        if let Event::Key(key) = event::read()? {
            app.on_key_code(key.code);
        }
        if let Some(switch) = app.quit() { break; }
    }

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(app.quit().unwrap())
}