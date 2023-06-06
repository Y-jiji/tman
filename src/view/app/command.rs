use crossterm::event::*;
use super::Redirect;

#[derive(Debug, Clone, Default)]
pub struct CommandLine {
    command: String,
    trigger: bool,
    active: bool,
    cursor: usize,
    prompt: (usize, Vec<String>),
    quit: Option<Redirect>,
}

pub enum ArgItem {
    // Variable
    V(&'static str),
    // Word
    W(&'static str),
}

pub struct Command<A> (
    pub Vec<ArgItem>, pub fn(&mut A, args: Vec<&str>, db: &mut crate::data::DataBase) -> bool);

impl<A> Command<A> {
    fn try_execute(&self, app: &mut A, args: &str, db: &mut crate::data::DataBase) -> bool {
        let args = args.split_whitespace().collect::<Vec<_>>();
        let is_match = self.0.len() == args.len();
        let is_match = is_match && (0..self.0.len()).all(|i| match self.0[i] {
            ArgItem::V(re) => regex::Regex::new(re).unwrap().is_match(args[i]),
            ArgItem::W(re) => regex::Regex::new(re).unwrap().is_match(args[i]),
        });
        if !is_match { return false }
        let args = args.into_iter().enumerate().filter_map(|(i, v)| matches!(self.0[i], ArgItem::V(_)).then(|| v)).collect::<Vec<_>>();
        (self.1)(app, args, db)
    }
}

fn possible_prompt(db: &crate::data::DataBase) -> impl Iterator<Item=String> {
    use chrono::*;
    let pj_list = db.pj_list().unwrap();
    let ev_list = db.ev_list().unwrap();
    let today = db.datetime_loc().unwrap().date();
    [
        format!("help"),
        format!("exit"),
        format!("cal"),
        format!("cal {:04}/{:02}", today.year(), today.month()),
        format!("calendar"),
        format!("calendar {:04}/{:02}", today.year(), today.month()),
        format!("plan"),
        format!("planner"),
        format!("plan {:04}/{:02}/{:02}", today.year(), today.month(), today.day()),
        format!("planner {:04}/{:02}/{:02}", today.year(), today.month(), today.day()),
        format!("edit event"),
        format!("edit e"),
        format!("edit ev"),
        format!("edit project"),
        format!("edit p"),
        format!("edit pj"),
    ].into_iter()
    .chain(
        (0..=12).flat_map(|tz| [
            format!("tz UTC+{tz}"),
            format!("tz UTC-{tz}")
        ])
    ).chain(
        (ev_list).into_iter().flat_map(|ev| [
            format!("edit event {ev}"),
            format!("edit ev {ev}"),
            format!("edit e {ev}"),
        ])
    ).chain(
        (pj_list).into_iter().flat_map(|pj| [
            format!("edit project {pj}"),
            format!("edit p {pj}"),
            format!("edit pj {pj}")
        ])
    )
}

use ArgItem::*;
use tui::widgets::Paragraph;
use unicode_width::UnicodeWidthStr;

lazy_static::lazy_static! {
    static ref EXES: [Command<CommandLine>; 9] = [
        Command(vec![W(r"tz"), V(r"^UTC\+|-\d{2}$")], |_, args, db| {
            let tz = args[0].trim_start_matches("UTC").trim_start_matches("+").parse::<i32>();
            if let Ok(tz) = tz {
                if tz >= -12 && tz <= 12 {
                    db.set_tz(tz); return true
                }
            }
            false
        }),
        Command(vec![W(r"edit"), W(r"e|ev|event"), V(r"^.*$")], |this, args, _| {
            this.quit = Some(Redirect::EditorApp{name: args[0].to_string(), mode: crate::view::app::editor::Mode::Ev});
            true
        }),
        Command(vec![W(r"edit"), W(r"p|pj|project"), V(r"^.*$")], |this, args, _| {
            this.quit = Some(Redirect::EditorApp{name: args[0].to_string(), mode: crate::view::app::editor::Mode::Pj});
            true
        }),
        Command(vec![W(r"cal|calendar")], |this, _, _| {
            this.quit = Some(Redirect::CalendarApp{ym: None});
            true
        }),
        Command(vec![W(r"cal|calendar"), V(r"^\d{4}\/\d{2}$")], |this, args, _| {
            let mut args = args[0].split("/");
            let y = args.next().unwrap().trim_start_matches('0').parse::<i32>().unwrap();
            let m = args.next().unwrap().trim_start_matches('0').parse::<u32>().unwrap();
            this.quit = Some(Redirect::CalendarApp{ym: Some((y, m))});
            true
        }),
        Command(vec![W(r"plan|planner")], |this, _, _| {
            this.quit = Some(Redirect::PlannerApp{ymd: None});
            true
        }),
        Command(vec![W(r"plan|planner"), V(r"^\d{4}\/\d{2}/\d{2}$")], |this, args, _| {
            let mut args = args[0].split("/");
            let y = args.next().unwrap().trim_start_matches('0').parse::<i32>().unwrap();
            let m = args.next().unwrap().trim_start_matches('0').parse::<u32>().unwrap();
            let d = args.next().unwrap().trim_start_matches('0').parse::<u32>().unwrap();
            this.quit = Some(Redirect::PlannerApp{ymd: Some((y, m, d))});
            true
        }),
        Command(vec![W(r"help")], |this, _, _| {
            this.quit = Some(Redirect::HelpApp);
            true
        }),
        Command(vec![W(r"exit")], |this, _, _| {
            this.quit = Some(Redirect::Exit);
            true
        })
    ];
}

impl CommandLine {
    pub fn new() -> Self {
        Self { active: true, ..Default::default() }
    }
    pub fn execute_from_app<A>(&mut self, app: &mut A, db: &mut crate::data::DataBase, command_pack: &[Command<A>]) -> bool {
        if !self.trigger { return false }
        self.trigger = false;
        let command = self.command.clone();
        for e in command_pack.iter() {
            let is_match = e.try_execute(app, &command, db);
            if is_match { self.command.clear(); self.cursor = 0; return true }
        }
        return false;
    }
    fn on_key_event(&mut self, key: KeyEvent, db: &mut crate::data::DataBase) -> bool {
        use KeyCode::*;
        if key.code == KeyCode::Esc {
            self.active ^= true;
            self.trigger = false;
            return true;
        }
        if !self.active { return false };
        match key.code {
            // add a char after cursor
            Char(c) => {
                self.trigger = false;
                self.command.insert(self.cursor, c);
                self.cursor = self.command
                    .ceil_char_boundary(usize::min(self.cursor + 1, self.command.len()));
                self.refresh_prompt(db);
            }
            // remove char under cursor
            Backspace if self.cursor != 0 => {
                self.command.remove(
                    self.command
                        .floor_char_boundary((self.cursor - 1).min(self.command.len() - 1)),
                );
                self.cursor = self.command
                    .floor_char_boundary(self.cursor.checked_sub(1).unwrap_or(0));
                self.refresh_prompt(db);
            }
            // move cursor
            Left => {
                self.cursor = self.command.floor_char_boundary(self.cursor.checked_sub(1).unwrap_or(0));
            }
            Right => {
                self.cursor = self.command.ceil_char_boundary(usize::min(self.cursor + 1, self.command.len()));
            }
            // select prompt
            Down => {
                self.prompt.0 = (self.prompt.0 + 1) % (1+self.prompt.1.len());
            }
            Up => {
                self.prompt.0 = (self.prompt.0 + self.prompt.1.len()) % (1+self.prompt.1.len());
            }
            // execute command
            Enter if self.prompt.0 == 0 => { 
                if self.execute_internal(db) { 
                    self.prompt.0 = 0;
                    self.prompt.1 = vec![];
                } else { 
                    self.trigger = true; 
                    return false
                }
            }
            // execute command
            Enter if self.prompt.0 != 0 => { 
                self.command = self.prompt.1[self.prompt.0 - 1].clone();
                self.cursor = self.command.len(); 
                self.prompt.0 = 0;
                self.prompt.1 = vec![];
            }
            _ => { return false }
        }
        return true;
    }
    fn refresh_prompt(&mut self, db: &crate::data::DataBase) {
        let regex = regex::Regex::new(&regex::escape(&self.command).replace(" ", ".*")).unwrap();
        let mut prompt = possible_prompt(db).into_iter()
            .filter_map(|prompt| {
                let count = regex.find_iter(&prompt).count();
                let lengt = prompt.len();
                (count != 0).then_some((prompt, count, lengt))
            }).collect::<Vec<_>>();
        prompt.sort_by(|a, b| (b.1 * a.2).cmp(&(a.1 * b.2)));
        self.prompt.0 = 0;
        self.prompt.1 = prompt.into_iter().map(|(x, _, _)| x).collect();
    }
    fn execute_internal(&mut self, db: &mut crate::data::DataBase) -> bool {
        let command = self.command.clone();
        for e in EXES.iter() {
            let is_match = e.try_execute(self, &command, db);
            if is_match { self.command.clear(); self.cursor = 0; return true }
        }
        return false;
    }
}

impl super::App for CommandLine {
    fn notify(&mut self, signal: crossterm::event::Event, db: &mut crate::data::DataBase) -> bool {
        use Event::*;
        match signal {
            Key(event) => self.on_key_event(event, db),
            _ => false
        }
    }
    fn quit(&self) -> Option<super::Redirect> {
        self.quit.clone()
    }
    fn render(&self, f: &mut tui::Frame<impl tui::backend::Backend>, rect: tui::layout::Rect) {
        use tui::text::*;
        use tui::style::*;
        let h = rect.height as usize;
        let h = h.min(self.prompt.1.len() + 1);
        let text = (0..h).map(|i| {
            let i = 
                if i == 0 { 0 }
                else if self.prompt.0 == 0 { i }
                else if self.prompt.0 + h < self.prompt.1.len() + 2 { i + self.prompt.0 - 1 }
                else { i + self.prompt.1.len() - h + 1 };
            let style = 
                if i == self.prompt.0 { Style::default().add_modifier(Modifier::BOLD) }
                else { Style::default().fg(Color::Rgb(0xaa, 0xaa, 0xaa)) };
            Spans::from(
                if i == 0 { Span::styled(&self.command, style) }
                else { Span::styled(&self.prompt.1[i-1], style) }
            )
        });
        let widget = Paragraph::new(Text::from(text.collect::<Vec<_>>()));
        f.render_widget(widget, rect);
        f.set_cursor(rect.x + self.command[..self.cursor].width() as u16, rect.y);
    }
}