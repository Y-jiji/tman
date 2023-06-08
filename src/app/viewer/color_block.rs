use super::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct ColorBlock {
    color: (u8, u8, u8),
}

impl ColorBlock {
    pub fn new(r: impl Into<u8>, g: impl Into<u8>, b: impl Into<u8>) -> Self {
        ColorBlock { color: (r.into(), g.into(), b.into()) }
    }
}

impl Viewer for ColorBlock {
    fn name(&self) -> String {
        return format!("color block");       
    }
    fn refresh(&mut self, _db: &crate::DataBase) {}
    fn render(&self, f: &mut Frame, rect: tui::layout::Rect) {
        use tui::widgets::*;
        use tui::style::*;
        let color = Color::Rgb(self.color.0, self.color.1, self.color.2);
        f.render_widget(Block::default().style(Style::default().bg(color)), rect);   
    }
}