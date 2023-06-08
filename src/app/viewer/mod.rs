//! viewer plugins
use serde::*;
mod color_block;
pub use color_block::*;

type Frame<'a> = tui::Frame<'a, tui::backend::CrosstermBackend<std::io::Stdout>>;

pub trait Viewer {
    fn name(&self) -> String;
    fn render(&self, f: &mut Frame, rect: tui::layout::Rect);
    fn refresh(&mut self, db: &crate::DataBase);
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ViewerOpt {
    Null,
    ColorBlock(ColorBlock),
}

impl Viewer for ViewerOpt {
    fn name(&self) -> String {
        use ViewerOpt::*;
        match self {
            Null => format!("null"),
            ColorBlock(cb) => cb.name(),
        }
    }
    fn render(&self, f: &mut Frame, rect: tui::layout::Rect) {
        use ViewerOpt::*;
        match self {
            Null => {f.render_widget(tui::widgets::Block::default().borders(tui::widgets::Borders::ALL), rect);}
            ColorBlock(cb) => cb.render(f, rect),
        }
    }
    fn refresh(&mut self, db: &crate::DataBase) {
        use ViewerOpt::*;
        match self {
            Null => {}
            ColorBlock(cb) => cb.refresh(db),
        }
    }
}