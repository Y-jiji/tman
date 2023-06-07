//! viewer plugins
use serde::*;

type Frame<'a> = tui::Frame<'a, tui::backend::CrosstermBackend<std::io::Stdout>>;

pub trait Viewer {
    fn name(&self) -> String;
    fn render(&self, frame: &mut Frame, rect: tui::layout::Rect);
    fn update(&mut self, db: &crate::DataBase);
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ViewerOpt {
}