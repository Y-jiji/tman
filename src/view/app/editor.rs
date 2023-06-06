use super::command::*;
use tui::{text::*, layout::*, style::*, widgets::*};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mode { Ev, Pj }

pub enum Item { Ev(crate::data::Ev), Pj(crate::data::Pj) }

pub struct Editor {
    command_line: CommandLine,
    inner: EditorInner,
}

impl Editor {
    pub fn new(mode: Mode, name: String, db: &crate::data::DataBase) -> Editor {
        let inner = EditorInner {
            item: match mode {
                Mode::Ev => db.ev_get_by_name(&name).map(Item::Ev),
                Mode::Pj => db.pj_get_by_name(&name).map(Item::Pj),
            },
            information: String::new()
        };
        Editor { command_line: CommandLine::new(), inner }
    }
}

pub struct EditorInner {
    // selected item
    item: Option<Item>,
    // aux information
    information: String,
}

pub fn justify(left: &str, right: &str, width: usize, least_sep: usize) -> String {
    assert!(left.width() + least_sep <= width);
    let margin = if right.width() + left.width() <= width {
        least_sep.max(width - left.width() - right.width())   
    } else { least_sep };
    format!("{left}{}{right}", " ".repeat(margin))
}

impl EditorInner {
    pub fn render_side(&self, f: &mut tui::Frame<impl tui::backend::Backend>, rect: tui::layout::Rect) {
        let text = Text::raw(&self.information);
        let paragraph = Paragraph::new(text).block(Block::default().borders(Borders::TOP));
        f.render_widget(paragraph, rect);
    }
    pub fn render_main(&self, f: &mut tui::Frame<impl tui::backend::Backend>, rect: tui::layout::Rect) {
        let text = {
            let mut text: Vec<Spans> = vec![];
            match &self.item {
                None => {
                    let span = Span::styled("no item selected", Style::default().fg(Color::Red)).into();
                    text.push(span);
                },
                Some(Item::Pj(pj)) => {
                    text.push("project\n".into());
                    text.push(justify("name", &pj.name(), rect.width as usize - 1, 4).into());
                },
                Some(Item::Ev(ev)) => { }
            }
            Text::from(text)
        };
        let paragraph = Paragraph::new(text).block(Block::default().borders(Borders::TOP | Borders::RIGHT));
        f.render_widget(paragraph, rect);
    }
    pub fn render(&self, f: &mut tui::Frame<impl tui::backend::Backend>, rect: tui::layout::Rect) {
        let divide = Layout::default().direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rect);
        self.render_main(f, divide[0]);
        self.render_side(f, divide[1]);
    }
}

use ArgItem::*;
use unicode_width::UnicodeWidthStr;
lazy_static::lazy_static!{
    static ref EXES: [Command<EditorInner>; 26] = [
        Command(vec![W("^n|name$"), V(r"^\w.*$")], |this, args, db| {
            this.item = match &this.item {
                None => { this.information = "no item selected".to_string(); None }
                Some(Item::Pj(ref pj)) => {
                    let res = db.pj_set_name(pj.id(), args[0].to_string());
                    this.information = format!("{res:?}");
                    Some(Item::Pj(db.pj_get_by_id(pj.id()).unwrap())) 
                }
                Some(Item::Ev(ref ev)) => { 
                    let res = db.ev_set_name(ev.id(), args[0].to_string()); 
                    this.information = format!("{res:?}");
                    Some(Item::Ev(db.ev_get_by_id(ev.id()).unwrap()))
                }
            };
            true
        }),
        Command(vec![W("^delete$")], |this, args, db| {
            // delete current project / event
            todo!()
        }),
        Command(vec![W("^delete$"), W("^recursive$")], |this, args, db| {
            // delete current project / event, recursively delete children and depedencies
            todo!()
        }),
        Command(vec![W("^u|undo$")], |this, args, db| {
            // undo previous operation
            todo!()
        }),
        Command(vec![W("^c|col|color$"), V(r"^[a-f0-9]{6}$")], |this, args, db| {
            // set color of current project
            todo!()
        }),
        Command(vec![W("^c|col|color$"), V(r"^[a-f0-9]{3}$")], |this, args, db| {
            // set color of current project
            todo!()
        }),
        Command(vec![W("^c|col|color$"), V(r"^darker|lighter$")], |this, args, db| {
            // make color lighter or darker
            todo!()
        }),
        Command(vec![W("^sel|select$"), W("^e|ev|event$"), V(r"^\w.*$")], |this, args, db| {
            // select an existing event by name
            todo!()
        }),
        Command(vec![W("^sel|select$"), W("^p|pj|project$"), V(r"^\w.*$")], |this, args, db| {
            // select an existing event by name
            todo!()
        }),
        Command(vec![W("^sel|select$"), V(r"^\w.*$")], |this, args, db| {
            // select an existing project or event, don't change current mode
            todo!()
        }),
        Command(vec![W("^time|due|deadline|ddl$"), V(r"\d{4}/\d{2}/\d{2}")], |this, args, db| {
            // set due time of the project
            todo!()
        }),
        Command(vec![W("^time|due|deadline|ddl$"), W(r"x|-")], |this, args, db| {
            // set due time of the project
            todo!()
        }),
        Command(vec![W("^p|parent$"), V(r"^\w.*")], |this, args, db| {
            // change parent
            todo!()
        }),
        Command(vec![W("^sp|spawn$"), W("e|ev|event"), V(r"^\w.*$")], |this, args, db| {
            // make a children of current project
            todo!()
        }),
        Command(vec![W("^sp|spawn$"), W("p|pj|project"), V(r"^\w.*$")], |this, args, db| {
            // make a children of current project
            todo!()
        }),
        Command(vec![W("^link|l$"), W("^aft|after$"), V(r"^\+\w.*$")], |this, args, db| {
            // link a dependency from this project
            todo!()
        }),
        Command(vec![W("^link|l$"), W("^aft|after$"), V(r"^-\w.*$")], |this, args, db| {
            // remove a link of this project
            todo!()
        }),
        Command(vec![W("^link|l$"), W("^bef|before$"), V(r"^\+\w.*$")], |this, args, db| {
            todo!()
        }),
        Command(vec![W("^link|l$"), W("^bef|before$"), V(r"^-\w.*$")], |this, args, db| {
            todo!()
        }),
        Command(vec![W("^w|wei|weight$"), V("^flex|perm$"), V(r"^\d+$")], |this, args, db| {
            todo!()
        }),
        Command(vec![W("^w|wei|weight$"), V(r"^\d+$")], |this, args, db| {
            todo!()
        }),
        Command(vec![W("^w|wei|weight$"), V(r"^flex|perm$")], |this, args, db| {
            // change weight type
            todo!()
        }),
        Command(vec![W("^q|quo|quota$"), V(r"^\+\d+$")], |this, args, db| {
            // add project quota
            todo!()
        }),
        Command(vec![W("^q|quo|quota$"), V(r"^-\d+$")], |this, args, db| {
            // subtract project quota
            todo!()
        }),
        Command(vec![W("^q|quo|quota$"), V(r"^\d+$")], |this, args, db| {
            // directly set project quota
           todo!()
        }),
        Command(vec![W("^f|finish"), V(r"^\d+$")], |this, args, db| {
            // increase finished quota
            todo!()
        })
    ];
}

impl EditorInner {
    pub fn new(db: &crate::data::DataBase, mode: Mode, name: String) -> Self {
        let item = match mode {
            Mode::Pj => db.pj_get_by_name(&name).map(Item::Pj),
            Mode::Ev => db.ev_get_by_name(&name).map(Item::Ev),
        };
        Self { item, information: String::new() }
    }
}

impl super::App for Editor {
    fn notify(&mut self, signal: crossterm::event::Event, db: &mut crate::data::DataBase) -> bool {
        self.command_line.notify(signal, db) || {
            let Editor { command_line, inner } = self;
            command_line.execute_from_app(inner, db, EXES.as_slice())
        }
    }
    fn quit(&self) -> Option<super::Redirect> {
        self.command_line.quit()
    }
    fn render(&self, f: &mut tui::Frame<impl tui::backend::Backend>, rect: tui::layout::Rect) {
        let upper = (rect.height / 4).min(8).max(4);
        let lower = rect.height - upper;
        let divide = Layout::default().direction(Direction::Vertical)
            .constraints([Constraint::Length(upper), Constraint::Length(lower-1)])
            .split(rect);
        let (upper, lower) = (divide[0], divide[1]);
        self.command_line.render(f, upper);
        self.inner.render(f, lower);
    }
}