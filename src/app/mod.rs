use serde::*;
use tui::layout::Rect;
mod viewer;
mod plugin;
mod command;

type F<'a> = tui::Frame<'a, tui::backend::CrosstermBackend<std::io::Stdout>>;

#[derive(Serialize, Deserialize)]
pub struct App {
    // the input command line
    command: command::Command,
    // command line plugins that generates command prompts
    plugins: Vec<Vec<plugin::PluginOpt>>,
    // previous generated prompts, stored as newline-seperated strings
    prompts: String,
    // selected prompt
    cursor: Option<usize>,
    // view port generators, recomputed only on command triggering
    viewers: Vec<Vec<(viewer::ViewerOpt, (u16, u16))>>,
    // grid layout
    layouts: Vec<(u16, u16)>,
    // the current page of viewers
    current: usize,
}

impl App {
    pub fn new() -> Self {
        let command = command::Command::new();
        let plugins = vec![];
        let prompts = String::new();
        let viewers = vec![vec![]];
        let layouts = vec![(1, 1)];
        Self { command, plugins, prompts, layouts, viewers, current: 0, cursor: None }
    }
    // put viewers in their place
    fn render_viewers(&self, f: &mut F, rect: Rect) {
        // get the grid rows and columns
        let (rows, cols) = self.layouts[self.current];
        // divide the page into rows and columns using blocks
        // render the viewers w.r.t. the rectangle angles
    }
    // select prompts
    fn select_prompts(&self) -> Vec<&str> {
        use regex::*;
        let regex = Regex::new(&escape(self.command.get()).replace(" ", ".*")).unwrap();
        self.prompts.lines().filter(|s| regex.is_match(&s)).collect()
    }
    // compute text from cursor position and given prompts
    fn window_content(&self, height: usize) -> Vec<tui::text::Spans> {
        use tui::text::*;
        use tui::style::*;
        // selected prompts and the original command
        let prompts = self.select_prompts();
        let command = self.command.get();
        // strong and faint colors
        let strong = Style::default().add_modifier(Modifier::BOLD);
        let normal = Style::default().fg(Color::Rgb(180, 180, 180));
        if let Some(cursor) = self.cursor {
            if cursor + height - 1 < prompts.len() {
                // if cursor is on prompts but not the bottom h - 1 ones
                let window = prompts[cursor..cursor + height - 1].iter().copied();
                [command].into_iter()
                    .chain(window).enumerate()
                    .map(|(i, s)| Span::styled(s, if i == 1 { strong } else { normal }))
                    .map(Spans::from).collect()
            } else {
                // if cursor is on the bottom h - 1 elements
                let window = prompts[prompts.len().max(height-1) - height + 1 .. prompts.len()].iter().copied();
                [command].into_iter()
                    .chain(window).enumerate()
                    .map(|(i, s)| Span::styled(s, if height - 1 - i == prompts.len() - cursor { strong } else { normal }))
                    .map(Spans::from).collect()
            }
        } else {
            // if cursor is on command
            let window = prompts[..height-1].iter().copied();
            [Span::styled(command, strong)].into_iter()
                .chain(window.map(|s| Span::styled(s, normal))).map(Spans::from).collect()
        }
    }
    // put the command line and prompts into their place
    fn render_command(&self, f: &mut F, rect: Rect) {
        use tui::text::*;
        use tui::widgets::Paragraph;
        let text = Text::from(self.window_content(rect.height as usize));
        f.render_widget(Paragraph::new(text), rect);
    }
}