use serde::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    string: String,
    cursor: usize,
}

impl Command {
    pub fn new() -> Self {
        Command { string: String::new(), cursor: 0 }
    }
    pub fn get(&self) -> &str {
        &self.string
    }
    pub fn clear(&mut self) {
        self.string.clear();
        self.cursor = 0;
    }
    pub fn l(&mut self) {
        if self.cursor == 0 { return }
        self.cursor = self.string.floor_char_boundary(self.cursor - 1);
    }
    pub fn r(&mut self) {
        if self.cursor == self.string.len() { return }
        self.cursor = self.string.ceil_char_boundary(self.cursor + 1);
    }
    pub fn put(&mut self, c: char) {
        self.string.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }
    pub fn del(&mut self) {
        if self.cursor == self.string.len() { return }
        self.string.remove(self.cursor);
    }
    pub fn bks(&mut self) {
        if self.cursor == 0 { return }
        self.l(); self.del();
    }
}