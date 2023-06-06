use tui::layout::*;
use tui::text::*;
use tui::widgets::*;
use tui::style::*;

type Frame<'a> = tui::Frame<'a, tui::backend::CrosstermBackend<std::io::Stdout>>;

pub struct Prompt(Rect);

impl Prompt {
    fn new(rect: Rect) -> Prompt { Prompt(rect) }
    fn render(&self, f: &mut Frame, cursor: (usize, usize), command: (&str, Color), prompt: (&Vec<String>, Color)) {
        let (cursor, selection) = cursor;
        
    }
}