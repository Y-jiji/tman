//! viewer plugins
use serde::*;

type Frame<'a> = tui::Frame<'a, tui::backend::CrosstermBackend<std::io::Stdout>>;

pub trait Viewer {
    fn name(&self) -> String;
    fn render(&self, f: &mut Frame, rect: tui::layout::Rect);
    fn refresh(&mut self, db: &crate::DataBase);
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ViewerOpt {
    Null,
}

impl Viewer for ViewerOpt {
    fn name(&self) -> String {
        use ViewerOpt::*;
        match self {
            Null => format!("null"),
        }
    }
    fn render(&self, f: &mut Frame, rect: tui::layout::Rect) {
        use ViewerOpt::*;
        match self {
            Null => {f.render_widget(tui::widgets::Block::default().borders(tui::widgets::Borders::ALL), rect);}
        }
    }
    fn refresh(&mut self, db: &crate::DataBase) {
        use ViewerOpt::*;
        match self {
            Null => {}
        }
    }
}