mod day;
mod edit;
mod list;
mod month;
mod year;

pub enum SwitchView {
    None,
}

pub struct View<'a> {
    handle: &'a mut crate::data::Data,
    keycode: crossterm::event::KeyCode,
    command: String,
    current_view: SwitchView,
}

impl<'a> View<'a> {
    pub fn load(data: &'a mut crate::data::Data) -> Self {
        Self {
            handle: data,
            keycode: crossterm::event::KeyCode::Enter,
            command: String::new(),
            current_view: SwitchView::None,
        }
    }
    // run application
    pub fn runapp(&mut self) {
    }
    // render one frame
    pub fn render(&mut self) {
    }
}
