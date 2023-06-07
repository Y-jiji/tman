use serde::*;
use tui::layout::Rect;
use self::{viewer::Viewer, plugin::Plugin, execute::TryExecute};
use crossterm::event::KeyCode;
use regex::internal::Program;

mod grid;
mod viewer;
mod plugin;
mod execute;
mod command;

type F<'a> = tui::Frame<'a, tui::backend::CrosstermBackend<std::io::Stdout>>;

// application state can also be saved
#[derive(Serialize, Deserialize)]
pub struct App {
    // the input command line
    command: command::Command,
    // command line plugins that generates command prompts
    plugins: Vec<Vec<plugin::PluginOpt>>,
    // command line error
    exeinfo: String,
    // previous generated prompts, stored as newline-seperated strings
    prompts: String,
    // history
    history: Vec<String>,
    // selected prompt, correspondent xcursor is encapsuled in command
    ycursor: Option<usize>,
    // view port generators, recomputed only on command triggering
    viewers: Vec<Vec<(viewer::ViewerOpt, Option<(u16, u16)>)>>,
    // grid layout
    layouts: Vec<(u16, u16)>,
    // the current page of viewers
    current: usize,
    // if application will exit
    // exit signal is set to false on loading
    #[serde(default)]
    #[serde(skip_serializing)]
    sigexit: bool,
}

lazy_static::lazy_static!{
    // internal command implementations
    static ref EXES: Vec<execute::CommandExecution<App>> = execute::x_decl! {
        (w "^exit$", |this, _args, _db| {
            // modify exit state to true
            this.sigexit = true; Ok(())
        })
        (w "^page|pg$", w r"\+|new|create", |this, _args, db| {
            // insert a page after current page
            // this is potentially slow
            this.viewers.insert(this.current, vec![]);
            this.layouts.insert(this.current, (1, 1));
            this.plugins.insert(this.current, vec![]);
            this.history.insert(this.current, String::new());
            this.current += 1;
            this.refresh(db);
            Ok(())
        })
        (w "^page|pg$", w r"\-|del|delete", |this, _args, db| {
            // remove a current page
            // this is potentially slow
            if this.viewers.len() == 1 {
                Err(String::from("cannt delete a page when there is only one page"))?
            }
            this.viewers.remove(this.current);
            this.layouts.remove(this.current);
            this.plugins.remove(this.current);
            this.history.insert(this.current, String::new());
            this.current = if this.current == 0 { 0 } else { this.current - 1 };
            this.refresh(db);
            Ok(())
        })
        (w "^page|pg$", w r"[1-9]\d*", |this, args, db| {
            // switch page by a page number
            this.current = args[0].parse::<usize>()
                .unwrap().min(this.plugins.len()-1);
            this.refresh(db);
            Ok(())
        })
        (w "^hist|history$", w r"clean|clear", |this, _args, _db| {
            // clean history of current page
            this.history[this.current].clear();
            Ok(())
        })
        (w "^ed|edit$", w "^pj|proj|project$", v "^.*$", |this, args, db| {
            // add an editor plugin
            todo!("implement project editor plugin")
        })
        (w "^ed|edit$", w "^ev|event$", v "^.*$", |this, args, db| {
            todo!("implement event editor plugin")
        })
        (w "^list$", v r"^\d{2}:\d{2}$", v "^.*$", |this, args, db| {
            // add a list viewer
            todo!("implement project list viewer")
        })
        (w "^plan|planner$", v r"\d{2}:\d{2}", v r"\d{4}/\d{2}/\d{2}", |this, args, db| {
            // put planner on a given viewport
            todo!("implement auto planner")
        })
    };
}

impl App {
    pub fn new() -> Self {
        let command = command::Command::new();
        let plugins = vec![];
        let prompts = String::new();
        let exeinfo = String::new();
        let viewers = vec![vec![]];
        let layouts = vec![(1, 1)];
        Self { command, plugins, prompts, layouts, viewers, current: 0, ycursor: None, exeinfo, history: vec![String::new()], sigexit: false }
    }
    pub fn load_yaml(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(serde_yaml::from_reader::<_, Self>(std::fs::File::open(path)?)?)
    }
    pub fn save_yaml(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        Ok(serde_yaml::to_writer(std::fs::File::create(path)?, self)?)
    }
    fn key(&mut self, key: KeyCode, db: &mut crate::DataBase) {
        let to_num = |x: Option<usize>| x.map(|x| x + 1).unwrap_or(0);
        let to_opt = |x: usize| (x != 0).then_some(x - 1);
        match key {
            // edit command line
            KeyCode::Char(c) => {
                self.command.put(c);
                self.ycursor = None;
            },
            KeyCode::Backspace => {
                self.command.bks();
                self.ycursor = None;
            },
            KeyCode::Delete => {
                self.command.del();
                self.ycursor = None;
            },
            KeyCode::Tab => {
                let prompts = self.select_prompts();
                let prompts = prompts.get(self.ycursor.unwrap_or(0));
                if prompts.is_none() { return }
                let prompts = prompts.unwrap();
                let capture = self.command.get_regex().find(prompts);
                if capture.is_none() { return }
                let capture = capture.unwrap();
                let command = self.command.get()[..capture.end()].to_string();
                self.command.set(command);
                self.ycursor = None;
            },
            // move cursor
            KeyCode::Up => self.ycursor = to_opt((to_num(self.ycursor) + self.prompts.lines().count()) % (self.prompts.lines().count() + 1)),
            KeyCode::Down => self.ycursor = to_opt((to_num(self.ycursor) + 1) % (self.prompts.lines().count() + 1)),
            KeyCode::Left => self.command.l(),
            KeyCode::Right => self.command.r(),
            // trigger command
            KeyCode::Enter => {
                // set command to prompt of some prompt is selected
                let command = self.ycursor.map(|p| self.select_prompts()[p]);
                if let Some(cmd) = command {
                    self.command.set(cmd.to_string());
                }
                self.execute(db);
                self.refresh(db);
            },
            _ => {}
        };
    }
    // refresh prompts command and viewer states
    fn refresh(&mut self, db: &crate::DataBase) {
        // clear prompts and command
        self.command.clear();
        // iterate over plugins and get new prompts
        self.prompts = self.int_prompts(&db);
        for plugin in self.plugins[self.current].iter() {
            plugin.ext_prompts(db, &mut self.prompts);
        }
        // refresh viewer cache with new database state
        for (viewer, _) in self.viewers[self.current].iter_mut() {
            viewer.refresh(db);
        }
    }
    // execute command for each plugin
    // output execution result to result
    fn execute(&mut self, db: &mut crate::DataBase) {
        // decompose command to space-free arguments
        let args = self.command.get().to_string();
        let args = args.split_whitespace().collect::<Vec<&str>>();
        // try execute internal commands
        let res = EXES.try_execute(&args, self, db);
        // decompose down the attributes of self
        let Self { exeinfo, history, current, command, plugins, .. } = self;
        // capture the modified attributes
        let mut handle_result  = |res| match res {
            Ok(false) => {return false}
            Err(e) => {*exeinfo = e; return true}
            Ok(true) => {
                *exeinfo = String::from("succeed: ") + command.get(); 
                history[*current].push_str(command.get());
                history[*current].push('\n');
                return true
            }
        };
        // handle internal execution results
        handle_result(res);
        // try execute external commands
        for plugin in plugins[*current].iter_mut() {
            let res = plugin.try_execute(db, &args);
            if handle_result(res) { break }
        }
    }
    // generate internal prompts
    fn int_prompts(&self, db: &crate::DataBase) -> String {
        todo!("add internal prompts after implementing basic command executions")
    }
    // list standby (not rendered, but already pulled) plugins and views
    // also render hidden tablets
    fn render_standby(&self, f: &mut F, rect: Rect) {
        todo!("may be a mini-map?")
    }
    // put viewers in their place
    fn render_viewers(&self, f: &mut F, rect: Rect) {
        // get the grid rows and columns
        let (rows, cols) = self.layouts[self.current];
        // divide the page into rows and columns using grid layout
        let grids = grid::GridLayout::new(rect, rows, cols);
        // render the viewers w.r.t. selected corners
        for (view, grid) in self.viewers[self.current].iter() {
            if grid.is_none() { continue }
            let (lu, rd) = grid.unwrap();
            let rect = grids.select(lu, rd);
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
        use unicode_width::UnicodeWidthStr;
        fn count_matches<'a>(x: &'a str, regx: &Regex) -> Vec<(&'a str, usize)> {
            let mut output = x.lines().filter_map(
                |s: &str| -> Option<(&str, usize)> {
                    let caps = regx.find_iter(s).fold(0, |a, b| a + b.len());
                    (caps != 0).then_some((s, caps))
                }
            ).collect::<Vec<_>>();
            output.sort_by(|a, b| (b.1*a.0.width()).cmp(&(a.1*b.0.width())));
            return output;
        }
        let regex = self.command.get_regex();
        let match_hist = count_matches(&self.history[self.current], &regex);
        let match_prom = count_matches(&self.prompts, &regex);
        match_hist.into_iter().map(|(s, _)| s).chain(match_prom.into_iter().map(|(s, _)| s)).collect()
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