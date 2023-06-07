use serde::*;
use tui::layout::Rect;
use self::{viewer::Viewer, plugin::Plugin};
mod viewer;
mod plugin;
mod command;
mod grids;

type F<'a> = tui::Frame<'a, tui::backend::CrosstermBackend<std::io::Stdout>>;

// application state can also be saved
#[derive(Serialize, Deserialize)]
pub struct App {
    // the input command line
    command: command::Command,
    // command line plugins that generates command prompts
    plugins: Vec<Vec<plugin::PluginOpt>>,
    // command line error
    errinfo: String,
    // previous generated prompts, stored as newline-seperated strings
    prompts: String,
    // selected prompt, correspondent xcursor is encapsuled in command
    ycursor: Option<usize>,
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
        let errinfo = String::new();
        let viewers = vec![vec![]];
        let layouts = vec![(1, 1)];
        Self { command, plugins, prompts, layouts, viewers, current: 0, ycursor: None, errinfo }
    }
    // refresh prompts command and viewer states
    pub fn refresh(&mut self, db: &crate::DataBase) {
        // clear prompts and command
        self.command.clear();
        // iterate over plugins and get new prompts
        self.prompts = self.internal_prompts(&db);
        for plugin in self.plugins[self.current].iter() {
            plugin.ext_prompts(db, &mut self.prompts);
        }
        // refresh viewer cache with a data base state
        for (viewer, _) in self.viewers[self.current].iter_mut() {
            viewer.refresh(db);
        }
    }
    // execute command for each plugin
    pub fn execute(&mut self, db: &mut crate::DataBase) {
        // try execute internal commands
        let res = self.internal_execute(db);
        match res {
            Err(e) => {self.errinfo = e; return}
            Ok(true) => return,
            Ok(false) => {},
        }
        // try execute external commands
        let command = self.command.get();
        for plugin in self.plugins[self.current].iter_mut() {
            let res = plugin.try_execute(db, command);
            match res {
                Err(e) => {self.errinfo = e; return}
                Ok(true) => return,
                Ok(false) => continue,
            }
        }
    }
    /// internal prompts
    fn internal_prompts(&self, db: &crate::DataBase) -> String {
        todo!()
    }
    // internally execute command
    fn internal_execute(&mut self, db: &mut crate::DataBase) -> Result<bool, String> {
        Ok(false)
    }
    // render miscellaneous information like viewer and plugins
    fn render_miscell(&self, f: &mut F, rect: Rect) {
        todo!("may be a mini-map?")
    }
    // put viewers in their place
    fn render_viewers(&self, f: &mut F, rect: Rect) {
        // get the grid rows and columns
        let (rows, cols) = self.layouts[self.current];
        // divide the page into rows and columns using grid layout
        let grids = grids::GridLayout::new(rect, rows, cols);
        // render the viewers w.r.t. selected corners
        for (view, (lu, rd)) in self.viewers[self.current].iter() {
            let rect = grids.select(*lu, *rd);
            view.render(f, rect);
        }
    }
    // put the command line and prompts into their place
    fn render_command(&self, f: &mut F, rect: Rect) {
        use tui::text::*;
        use tui::widgets::Paragraph;
        let text = Text::from(self.window_content(rect.height as usize));
        f.render_widget(Paragraph::new(text), rect);
        f.set_cursor(rect.x + self.command.xcursor(), rect.y);
    }
    // select prompts
    fn select_prompts(&self) -> Vec<&str> {
        use regex::*;
        let regex = Regex::new(&escape(self.command.get()).replace(" ", ".*")).unwrap();
        self.prompts.lines().filter(|s| regex.is_match(&s)).collect()
    }
    // compute text highlighting from cursor position and given prompts
    fn window_content(&self, height: usize) -> Vec<tui::text::Spans> {
        use tui::text::*;
        use tui::style::*;
        // selected prompts and the original command
        let prompts = self.select_prompts();
        let command = self.command.get();
        // strong and faint colors
        let strong = Style::default().add_modifier(Modifier::BOLD);
        let normal = Style::default().fg(Color::Rgb(180, 180, 180));
        if let Some(cursor) = self.ycursor {
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
                    .map(|(i, s)| Span::styled(s, if height - 2 - i == prompts.len() - cursor { strong } else { normal }))
                    .map(Spans::from).collect()
            }
        } else {
            // if cursor is on command
            let window = prompts[..height-1].iter().copied();
            [Span::styled(command, strong)].into_iter()
                .chain(window.map(|s| Span::styled(s, normal))).map(Spans::from).collect()
        }
    }
}