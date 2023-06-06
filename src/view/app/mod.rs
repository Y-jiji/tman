mod command;
mod editor;
pub use command::*;
pub use editor::*;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use tui::{backend::*, Terminal, Frame, layout::Rect};

#[derive(Debug, Clone)]
pub enum Redirect {
	Exit,
	EditorApp{name: String, mode: editor::Mode},
	CalendarApp{ym: Option<(i32, u32)>},
	PlannerApp{ymd: Option<(i32, u32, u32)>},
    HelpApp,
}

pub trait App {
	fn quit(&self) -> Option<Redirect>;
	fn render(&self, f: &mut Frame<impl Backend>, rect: Rect);
	fn notify(&mut self, signal: Event, db: &mut crate::data::DataBase) -> bool;
}

pub fn run_app(mut app: impl App, db: &mut crate::data::DataBase) -> Result<Redirect, Box<dyn std::error::Error>> {
	// setup terminal
	enable_raw_mode()?;
	let mut stdout = std::io::stdout();
	execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
	let backend = CrosstermBackend::new(stdout);
	let mut terminal = Terminal::new(backend)?;

	// create app and run it
	loop {
		terminal.draw(|f| app.render(f, f.size()))?;
		app.notify(event::read()?, db);
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
