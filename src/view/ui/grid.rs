use tui::layout::*;
use tui::text::*;
use tui::widgets::*;
use tui::style::*;

type Frame<'a> = tui::Frame<'a, tui::backend::CrosstermBackend<std::io::Stdout>>;

pub struct GridView(Vec<Rect>);

impl GridView {
	// creating grid view
	pub fn new(rect: Rect, rows: usize, cols: usize) -> GridView {
		use Direction::{Vertical as V, Horizontal as H};
		fn fold(dir: Direction, total: u16, split: u16) -> Layout {
			let mut v = vec![Constraint::Length((total % split) / 2)];
			v.extend(vec![Constraint::Length(total / split); split as usize]);
			v.push(Constraint::Length(((total + 1) % split) / 2));
			Layout::default().direction(dir).constraints(v)
		}
		let (w, h) = (rect.width, rect.height);
        let fold_rows = |rect| fold(V, w, rows as u16).split(rect).into_iter().skip(1).take(rows);
        let fold_cols = |rect| fold(H, h, cols as u16).split(rect).into_iter().skip(1).take(cols);
		GridView(
			fold_rows(rect)
                .flat_map(fold_cols)
                .collect::<Vec<_>>()
        )
	}
	// render contents into grid view
	pub fn render(&self, f: &mut Frame, conts: Vec<(Span, Color, Text)>) {
		let conts_iter = conts.into_iter().enumerate(); 
		for (i, (title, color, text)) in conts_iter {
			let paragraph = Paragraph::new(text).alignment(Alignment::Center);
			let block = Block::default().title(title)
                .borders(Borders::all())
				.title_alignment(Alignment::Center)
				.style(Style::default().fg(color));
			f.render_widget(paragraph.block(block), self.0[i]);
		}
	}
}
