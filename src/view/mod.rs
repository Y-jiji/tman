mod edit;
pub use edit::EditView;
mod calendar;
pub use calendar::CalendarMonthView;
mod plan;
pub use plan::AutoScheduleView;
mod command;
pub use command::Command;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    event,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    error::Error,
    io::{self, Stdout},
};
use tui::{backend::CrosstermBackend, Terminal, Frame};

#[derive(Debug, Clone)]
pub enum Switch {
    Edit{name: String},
    Plan,
    List,
    Exit,
    Calendar,
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
        if app.quit().is_some() { break; }
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