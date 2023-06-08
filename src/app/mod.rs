use serde::*;
use tui::layout::Rect;
use crossterm::event::KeyCode;

mod grid;
mod viewer;
mod plugin;
mod execute;
mod command;

use grid::*;
use viewer::*;
use plugin::*;
use execute::*;
use command::*;

type F<'a> = tui::Frame<'a, tui::backend::CrosstermBackend<std::io::Stdout>>;

// application state can also be saved
#[derive(Serialize, Deserialize)]
pub struct App {
    // the input command line
    command: Command,
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
        // exit application
        (w "^exit$", |this, _args, _db| {
            // modify exit state to true
            this.sigexit = true; Ok(())
        })
        // set grid for viewer layout
        (w "^grid$", v r"^[1-4][1-4]$", |this, args, _db| {
            // cols <= 4 and rows <= 4
            let grid = args[0].parse::<u16>().unwrap();
            this.layouts[this.current] = (grid / 10, grid % 10);
            let viewers = this.viewers[this.current].iter_mut();
            for (_viewer, grid) in viewers { *grid = None }
            Ok(())
        })
        // clear current page
        (w "^clear$", |this, _args, _db| {
            this.viewers[this.current].clear();
            this.plugins[this.current].clear();
            Ok(())
        })
        // create a page
        (w "^page|pg$", w r"\+|new|create", |this, _args, db| {
            // insert a page after current page
            // this is potentially slow
            if this.viewers.len() >= 10 {
                Err(String::from("cannot add a page because 10 pages are maximal"))?
            }
            this.current += 1;
            this.viewers.insert(this.current, vec![]);
            this.layouts.insert(this.current, (1, 1));
            this.plugins.insert(this.current, vec![]);
            this.history.insert(this.current, String::new());
            this.refresh(db);
            Ok(())
        })
        // delete current page
        (w "^page|pg$", w r"-|del|delete", |this, _args, db| {
            // remove a current page
            if this.viewers.len() == 1 {
                Err(String::from("cannot delete a page when there is only one page"))?
            }
            this.viewers.remove(this.current);
            this.layouts.remove(this.current);
            this.plugins.remove(this.current);
            this.history.insert(this.current, String::new());
            this.current = if this.current == 0 { 0 } else { this.current - 1 };
            this.refresh(db);
            Ok(())
        })
        // switch page
        (w "^page|pg$", v r"[0-9]*", |this, args, db| {
            // switch page by a page number
            this.current = args[0].parse::<usize>()
                .unwrap().min(this.plugins.len()-1);
            this.refresh(db);
            Ok(())
        })
        // clean history of current page
        (w "^hist|history$", w r"clean|clear", |this, _args, _db| {
            this.history[this.current].clear();
            Ok(())
        })
        // color block for testing
        (w "^color$", w "^block$", v r"^[0-3]{2}:[0-3]{2}", v r"^[a-f0-9]{6}$", |this, args, _db| {
            let cb = usize::from_str_radix(args[1].trim_start_matches("0"), 16).unwrap_or(0);
            let cb = ColorBlock::new((cb / (256 * 256)) as u8, (cb / 256 % 256) as u8, (cb % 256) as u8);
            let (rows, cols) = this.layouts[this.current];
            this.viewers[this.current].push(
                (cb.into(), GridLayout::corner_from(args[0], rows, cols)));
            Ok(())
        })
        // display project list
        (w "^list$", v r"^[0-3]{2}:[0-3]{2}$", v "^.*$", |this, args, db| {
            // add a list viewer
            todo!("implement project list viewer")
        })
        // display auto planner with year-month-date
        (w "^plan|planner$", v r"[0-3]{2}:[0-3]{2}", v r"\d{4}/\d{2}/\d{2}", |this, args, db| {
            // put planner on a given viewport for a given date
            todo!("implement auto planner, add planner commands")
        })
        // display auto planner this week
        (w "^plan|planner$", v r"[0-3]{2}:[0-3]{2}", v r"\d{4}/\d{2}/\d{2}", |this, args, db| {
            // put planner on a given viewport for a given date
            todo!("implement auto planner, add planner commands")
        })
        // display month calendar
        (w "^cal|calender$", v r"[0-3]{2}:[0-3]{2}", v r"\d{4}/\d{2}", |this, args, db| {
            todo!("remove current calendar and put a new one")
        })
        // display year calendar
        (w "^cal|calender$", v r"[0-3]{2}:[0-3]{2}", v r"\d{4}", |this, args, db| {
            todo!("remove current calendar and put a new one")
        })
        // project editor plugin and project viewer
        (w "^ed|edit|editor$", v r"[0-3]{2}:[0-3]{2}", w "^pj|proj|project$", v "^.*$", |this, args, db| {
            let name = args[1];
            let pj = db.pj_get_or_create_by_name(&name);
            // TODO: remove current editor plugins or viewers if there is any
            // TODO: add an editor plugin
            // add editor viewer
            let ed = EditorView::new_pj(pj, db);
            let (rows, cols) = this.layouts[this.current];
            this.viewers[this.current].push(
                (ed.into(), GridLayout::corner_from(&args[0], rows, cols))
            );
            Ok(())
        })
        // event editor plugin and event viewer
        (w "^ed|edit|editor$", v r"[0-3]{2}:[0-3]{2}", w "^ev|event$", v "^.*$", |this, args, db| {
            // remove current editor plugins or viewers if there is any
            todo!("implement event editor plugin")
        })
        // remove editor if there is any, else display an error message
        (w "^ed|edit|editor", v r"-|stop|quit|exit|remove", |this, args, db| {
            todo!("remove editor plugin and viewer")
        })
    };
}

impl App {
    pub fn new() -> Self {
        let command = Command::new();
        let prompts = String::new();
        let exeinfo = String::new();
        let plugins = vec![vec![]];
        let viewers = vec![vec![]];
        let layouts = vec![(1, 1)];
        Self { command, plugins, prompts, layouts, viewers, current: 0, ycursor: None, exeinfo, history: vec![String::new()], sigexit: false }
    }
    pub fn load_yaml(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(serde_json::from_reader::<_, Self>(std::fs::File::open(path)?)?)
    }
    pub fn save_yaml(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        Ok(serde_json::to_writer(std::fs::File::create(path)?, self)?)
    }
    pub fn run(&mut self, db: &mut crate::DataBase) 
    -> Result<(), Box<dyn std::error::Error>> {
        use crossterm::{
            event::{self, DisableMouseCapture, EnableMouseCapture, Event},
            execute,
            terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        };
        use std::{io, time::{Duration, Instant}};
        use tui::{backend::CrosstermBackend, Terminal};
        // setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        let mut last_tick = Instant::now();
        let tick_rate = Duration::from_secs_f32(0.01);
        // run application
        while !self.sigexit {
            terminal.draw(|f| self.render(f))?;
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));
            if crossterm::event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    self.key(key.code, db);
                }
            }
            if last_tick.elapsed() >= tick_rate {
                last_tick = Instant::now();
            }
        }
        // restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;
        Ok(())
    }
    fn render(&self, f: &mut F) {
        let mut rect = f.size();
        let (_rows, cols) = self.layouts[self.current];
        rect.x = rect.x + (rect.width % cols + 3) / 2;
        rect.width = ((rect.width - 2) / cols) * cols;
        rect.y = 1;
        rect.height -= 1;
        let _h = rect.height / 5;
        let _h = _h.max(4).min(16);
        let _w = rect.width / 2;
        self.render_command(f, Rect {
            x: rect.x, y: rect.y, width: _w, height: _h });
        self.render_standby(f, Rect {
            x: rect.x + _w, y: rect.y, width: rect.width - _w, height: _h });
        self.render_viewers(f, Rect {
            x: rect.x, y: rect.y + _h, width: rect.width, height: rect.height - _h });
    }
    fn key(&mut self, key: KeyCode, db: &mut crate::DataBase) {
        let to_num = |x: Option<usize>| x.map(|x| x + 1).unwrap_or(0);
        let to_opt = |x: usize| (x != 0).then(|| x - 1);
        let prompts = self.select_prompts();
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
                let prompts = prompts.get(self.ycursor.unwrap_or(0));
                if prompts.is_none() { return }
                let prompts = prompts.unwrap();
                let capture = self.command.get_regex().find(prompts);
                if capture.is_none() { return }
                let capture = capture.unwrap();
                let command = prompts[..capture.end()].to_string();
                self.command.set(command);
                self.ycursor = None;
            },
            // move cursor
            KeyCode::Up => self.ycursor = to_opt((to_num(self.ycursor) + prompts.len()) % (prompts.len() + 1)),
            KeyCode::Down => self.ycursor = to_opt((to_num(self.ycursor) + 1) % (prompts.len() + 1)),
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
                self.ycursor = None;
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
        // clear execution information
        exeinfo.clear();
        // capture the modified attributes
        let mut handle_result  = |res| match res {
            Ok(false) => {return false}
            Err(e) => {*exeinfo = e; return true}
            Ok(true) => {
                // this might be filled by command
                if exeinfo.is_empty() {
                    *exeinfo = String::from("succeed: ") + command.get(); 
                }
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
        // FIXME: output meaningful prompts
        let rands = "12344567890\nqwertyuiopasdfghjjklzxcvbnm ".chars().collect::<Vec<char>>();
        String::from_iter((0..10000).map(|_| -> char { rands[rand::random::<usize>() % rands.len()] }))
    }
    // list standby (not rendered, but already pulled) plugins and views
    // also render hidden tablets
    fn render_standby(&self, f: &mut F, rect: Rect) {
        use tui::widgets::*;
        use tui::style::*;
        // TODO: page: current / total
        // TODO: list plugins and viewers, their position
        let block = Block::default().style(Style::default().bg(Color::Rgb(0, 0, 0)));
        f.render_widget(block, rect);
    }
    // put viewers in their place
    fn render_viewers(&self, f: &mut F, rect: Rect) {
        // get the grid rows and columns
        let (rows, cols) = self.layouts[self.current];
        // divide the page into rows and columns using grid layout
        let mut grids = grid::GridLayout::new(rect, rows, cols);
        // render the viewers w.r.t. selected corners
        for (view, grid) in self.viewers[self.current].iter().rev() {
            if grid.is_none() { continue }
            let (lu, rd) = grid.unwrap();
            if let Some(rect) = grids.corner_grid(lu, rd) {
                view.render(f, rect);
            }
        }
        grids.render_placeholder(f);
    }
    // put the command line and prompts into their place
    fn render_command(&self, f: &mut F, rect: Rect) {
        use tui::text::*;
        use tui::widgets::*;
        let text = Text::from(self.window_content(rect.height as usize));
        f.render_widget(Paragraph::new(text).wrap(Wrap { trim: true }), rect);
        f.set_cursor(rect.x + self.command.xcursor() % rect.width, rect.y + self.command.xcursor() / rect.width);
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
        let mut matches = match_hist.into_iter().map(|(s, _)| s).chain(match_prom.into_iter().map(|(s, _)| s)).collect::<Vec<_>>();
        matches.dedup();
        return matches;
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
                let window = prompts[prompts.len().max(height-1) + 1-height .. prompts.len()].iter().copied();
                [command].into_iter()
                    .chain(window).enumerate()
                    .map(|(i, s)| Span::styled(s, if height.min(prompts.len()+1) - i == prompts.len() - cursor { strong } else { normal }))
                    .map(Spans::from).collect()
            }
        } else {
            // if cursor is on command
            let window = prompts[..(height-1).min(prompts.len())].iter().copied();
            [Span::styled(command, strong)].into_iter()
                .chain(window.map(|s| Span::styled(s, normal))).map(Spans::from).collect()
        }
    }
}
