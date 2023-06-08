use tui::layout::Rect;
use crossterm::style::Stylize;


pub struct GridLayout {
    grid: Vec<Rect>,
    free: Vec<bool>, 
    rows: u16, 
    cols: u16,
}

type F<'a> = tui::Frame<'a, tui::backend::CrosstermBackend<std::io::Stdout>>;

impl GridLayout {
    pub fn new(rect: Rect, rows: u16, cols: u16) -> Self {
        // make a split
        let (w, h) = (rect.width / cols, rect.height / rows);
        let mut grid = vec![];
        let free = vec![true; (rows*cols) as usize];
        for y in 0..rows {
            for x in 0..cols {
                if x == cols - 1 {
                    grid.push(Rect { x: x * w + rect.x, y: y * h + rect.y, width: w + rect.width % cols, height: h });
                } else {
                    grid.push(Rect { x: x * w + rect.x, y: y * h + rect.y, width: w, height: h });
                }
            }
        }
        Self { grid, free, rows, cols }
    }
    pub fn corner_grid(&mut self, corner_lu: u16, corner_rd: u16) -> Option<Rect> {
        let lu = corner_lu as usize;
        let rd = corner_rd as usize;
        let cols = self.cols as usize;
        for r in (lu/cols)..=(rd/cols) { 
            for c in (lu%cols)..=(rd%cols) {
                if !self.free[r*cols + c] { return None }
                self.free[r*cols + c] = false
            }
        }
        Some(Rect {
            x: self.grid[lu].x,
            y: self.grid[lu].y,
            width: self.grid[rd].x + self.grid[rd].width - self.grid[lu].x,
            height: self.grid[rd].y + self.grid[rd].height - self.grid[lu].y,
        })
    }
    pub fn corner_from(s: &str, rows: u16, cols: u16) -> Option<(u16, u16)> {
        let mut corner = s.split(":").map(|x| x.trim_start_matches("0").parse::<u16>().unwrap_or(0));
        let corner_lu = corner.next().unwrap();
        let corner_rd = corner.next().unwrap();
        let (row_0, row_1) = ((corner_lu / 10), (corner_rd / 10));
        let (col_0, col_1) = ((corner_lu % 10), (corner_rd % 10));
        (
            (row_1 < rows) && (row_0 <= row_1) &&
            (col_1 < cols) && (col_0 <= col_1)
        ).then_some((
            row_0 * cols + col_0,
            row_1 * cols + col_1,
        ))
    }
    pub fn render_placeholder(&self, f: &mut F) {
        use tui::widgets::*;
        use tui::text::*;
        use tui::style::*;
        for i in 0..self.grid.len() {
            if self.free[i] {
                let s = format!("grid[{}][{}]", i as u16 / self.cols, i as u16 % self.cols);
                let c = Some(Color::Rgb(127, 127, 127));
                f.render_widget(Paragraph::new(Span::styled(s, Style{fg: c, ..Default::default()})), self.grid[i]);
            }
        }
    }
}
