use tui::layout::Rect;

pub struct GridLayout{
    grid: Vec<Rect>, 
    rows: u16, 
    cols: u16,
}

type F<'a> = tui::Frame<'a, tui::backend::CrosstermBackend<std::io::Stdout>>;

impl GridLayout {
    pub fn new(rect: Rect, rows: u16, cols: u16) -> Self {
        // try to make a perfect split
        todo!()
    }
    pub fn select(&self, corner_lu: u16, corner_rd: u16) -> Rect {
        let lu = corner_lu as usize;
        let rd = corner_rd as usize;
        Rect {
            x: self.grid[lu].x,
            y: self.grid[lu].y,
            width: self.grid[rd].x + self.grid[rd].width - self.grid[lu].x,
            height: self.grid[rd].y + self.grid[rd].height - self.grid[lu].y,
        }
    }
}
