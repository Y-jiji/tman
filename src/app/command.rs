use serde::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    command: String,
    xcursor: usize,
}

impl Command {
    pub fn new() -> Self {
        Command { command: String::new(), xcursor: 0 }
    }
    pub fn get(&self) -> &str {
        &self.command
    }
    pub fn xcursor(&self) -> u16 {
        self.xcursor as u16
    }
    pub fn clear(&mut self) {
        self.command.clear();
        self.xcursor = 0;
    }
    pub fn l(&mut self) {
        if self.xcursor == 0 { return }
        self.xcursor = self.command.floor_char_boundary(self.xcursor - 1);
    }
    pub fn r(&mut self) {
        if self.xcursor == self.command.len() { return }
        self.xcursor = self.command.ceil_char_boundary(self.xcursor + 1);
    }
    pub fn put(&mut self, c: char) {
        self.command.insert(self.xcursor, c);
        self.xcursor += c.len_utf8();
    }
    pub fn del(&mut self) {
        if self.xcursor == self.command.len() { return }
        self.command.remove(self.xcursor);
    }
    pub fn bks(&mut self) {
        if self.xcursor == 0 { return }
        self.l(); self.del();
    }
}