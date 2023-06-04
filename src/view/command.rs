use super::Switch;
use crossterm::event::KeyCode;
use tui::layout::*;
use tui::backend::CrosstermBackend;
use tui::Frame;
use std::io::Stdout;
use tui::widgets::*;
use tui::text::*;
use unicode_width::UnicodeWidthStr;


pub struct Command {
    cursor: usize,
    command: String,
}

impl Command {
    pub fn new() -> Self {
        Command { cursor: 0, command: String::new() }
    }
    // clear command
    pub fn clear(&mut self) {
        self.cursor = 0;
        self.command.clear();
    }
    // get command
    pub fn get_command(&mut self) -> String {
        self.cursor = 0;
        std::mem::replace(&mut self.command, String::new())
    }
    // return true of command is triggered
    pub fn on_key_code(&mut self, key_code: KeyCode) -> bool {
        use crossterm::event::KeyCode::*;
        match key_code {
            Char(c) => {
                self.command.insert(self.cursor, c);
                self.cursor = self
                    .command
                    .ceil_char_boundary(usize::min(self.cursor + 1, self.command.len()));
                false
            }
            Enter => true,
            Backspace if self.cursor != 0 => {
                self.command.remove(
                    self.command
                        .floor_char_boundary((self.cursor - 1).min(self.command.len() - 1)),
                );
                self.cursor = self.command
                    .floor_char_boundary(self.cursor.checked_sub(1).unwrap_or(0));
                false
            }
            Left => {
                self.cursor = self.command
                    .floor_char_boundary(self.cursor.checked_sub(1).unwrap_or(0));
                false
            }
            Right => {
                self.cursor = self.command
                    .ceil_char_boundary(usize::min(self.cursor + 1, self.command.len()));
                false
            }
            Esc => { self.command = "exit".to_string(); true},
            _ => false,
        }
    }
    // try parse command to switch
    pub fn try_switch(&mut self) -> Option<Switch> {
        let args_string = self.command.clone();
        let args = args_string.trim().split_whitespace().collect::<Vec<_>>();
        match args.get(0).map(|x| x as &str) {
            Some("exit") => {self.clear(); Some(Switch::Exit)},
            Some("edit") if args.get(1).is_some() => {self.clear(); Some(Switch::Edit {name: args[1].to_string()})}
            Some("cal") | Some("calendar") => {self.clear(); Some(Switch::Calendar)},
            Some("plan") => {self.clear(); Some(Switch::Plan)}
            _ => None
        }
    }
    // draw frame in rectangle
    pub fn draw(&self, f: &mut Frame<CrosstermBackend<Stdout>>, rect: Rect) {
        let command_widget = Paragraph::new(Text::raw(self.command.clone()))
            .block(Block::default().borders(Borders::all()).title(" Command "));
        f.render_widget(command_widget, rect);
        let cursor_x = rect.x + self.command.get(..self.cursor).unwrap().width() as u16 + 1;
        let cursor_y = rect.y + 1;
        f.set_cursor(cursor_x, cursor_y);
    }
}
