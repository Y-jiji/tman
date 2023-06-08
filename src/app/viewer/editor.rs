use unicode_width::UnicodeWidthStr;
use crate::data::*;
use super::Viewer;
use serde::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Item {
    Pj(Pj),
    Ev(Ev),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorView {
    item: Item,
    parent: String,
    children: Vec<String>,
    deps: Vec<String>,
    deps_rvs: Vec<String>,
    tz: i64,
}

impl EditorView {
    pub fn new_pj(pj: Pj, db: &DataBase) -> Self {
        let mut this = Self {
            item: Item::Pj(pj), 
            parent: String::new(), 
            children: vec![], 
            deps: vec![], tz: 0, 
            deps_rvs: vec![], 
        };
        this.refresh(db);
        return this;
    }
    pub fn new_ev(ev: Ev, db: &DataBase) -> Self {
        let mut this = Self {
            item: Item::Ev(ev), 
            parent: String::new(), 
            children: vec![], 
            deps: vec![], tz: 0, 
            deps_rvs: vec![], 
        };
        this.refresh(db);
        return this;
    }
    fn color(&self) -> tui::style::Color {
        match &self.item {
            Item::Ev(ev) => ev.color_rgb(),
            Item::Pj(pj) => pj.color_rgb(),
        }
    }
    fn print_one(w: usize, mut pair: (&str, &str)) -> String {
        assert!(pair.1.width() <= w);
        let total = pair.0.width() + pair.1.width() + 4;
        *(&mut pair.1) = pair.1.get(
            ..total.min(w).max(pair.0.width() + 4) - pair.0.width() - 4).unwrap();
        let total = pair.0.width() + pair.1.width();
        format!("{}{}{}", pair.0, " ".repeat(w - total), pair.1)
    }
    fn parse_due(&self, due: i64) -> String {
        use chrono::NaiveDateTime;
        NaiveDateTime::from_timestamp_opt(
            due + self.tz * 60 * 60, 0
        ).unwrap().format("%Y/%m/%d %H:%M").to_string()
    }
    fn print_all(&self, w: usize, h: usize) -> Vec<String> {
        match &self.item {
            Item::Pj(pj) => [
                ("type",    format!("project")),
                ("name" ,   pj.name().to_string()),
                ("id"   ,   format!("p{}", pj.id())),
                ("color",   format!("#{:06x}", pj.color_usize())),
                ("done/esti quota", format!("{}/{}", pj.quota_done(), pj.quota_esti())),
                ("weight",  format!("{}", pj.weight())),
                ("", format!("{}", match pj.weight_type() {
                    WeightType::Flexible => "flexible",
                    WeightType::Reserved => "reserved",
                })),
                ("due", match pj.deadline() {
                    None => "N/A".to_string(),
                    Some(due) => self.parse_due(due),
                }),
                ("parent",  format!("{}", self.parent)),
            ].into_iter()
                .chain(self.children.iter().enumerate().map(|(i, ch)| {
                    if i == 0 { ("children", format!("{ch}")) }
                    else { ("", format!("{ch}")) }
                }))
                .chain(self.deps.iter().enumerate().map(|(i, d)| {
                    if i == 0 { ("after", format!("d")) }
                    else { ("", format!("{d}")) }
                }))
                .chain(self.deps_rvs.iter().enumerate().map(|(i, d)| {
                    if i == 0 { ("before", format!("d")) }
                    else { ("", format!("{d}")) }
                }))
                .enumerate()
                .filter_map(|(i, (name, content))| {
                    (i < h).then(|| Self::print_one(w, (name, &content)))
                })
                .collect::<Vec<_>>(),
            Item::Ev(ev) => [
                ("type",    format!("event")),
                ("name",    format!("{}", ev.name())),
                ("id",      format!("e{}", ev.id())),
                ("esti quota", format!("{}", ev.quota_esti())),
                ("time",    format!("{}", self.parse_due(ev.time()))),
                ("color",   format!("#{:06x}", ev.color_usize())),
                ("parent",  format!("{}", self.parent)),
            ].into_iter()
                .enumerate()
                .filter_map(|(i, (name, content))| {
                    (i < h).then(|| Self::print_one(w, (name, &content)))
                })
                .collect::<Vec<_>>()
        }
    }
}

impl super::Viewer for EditorView {
    fn name(&self) -> String {
        "editor".to_string()
    }
    fn refresh(&mut self, db: &DataBase) {
        self.item = match &self.item {
            Item::Ev(ev) => {
                self.deps.clear();
                self.deps_rvs.clear();
                self.children.clear();
                self.tz = db.tz() as i64;
                self.parent = db.pj_get_by_id(ev.pp()).unwrap().name().to_string();
                Item::Ev(db.ev_get_by_id(ev.id()).unwrap())
            },
            Item::Pj(pj) => {
                self.deps = pj.iter_deps()
                    .map(|id| 
                        db.pj_get_by_id(id).unwrap()
                        .name().to_string())
                    .collect::<Vec<_>>();
                self.deps_rvs = pj.iter_deps_rvs()
                    .map(|id|
                        db.pj_get_by_id(id).unwrap()
                        .name().to_string())
                    .collect::<Vec<_>>();
                self.children = 
                    pj.iter_chev().map(|id| 
                        db.ev_get_by_id(id).unwrap()
                        .name().to_string()
                    ).chain(
                        pj.iter_chpj()
                        .map(|id| db.pj_get_by_id(id).unwrap().name().to_string())
                    ).collect::<Vec<_>>();
                self.tz = db.tz() as i64;
                self.parent = db.pj_get_by_id(pj.pp()).unwrap().name().to_string();
                Item::Pj(db.pj_get_by_id(pj.id()).unwrap())
            },
        }
    }
    fn render(&self, f: &mut super::Frame, rect: tui::layout::Rect) {
        // render project or event to given area
        use tui::widgets::*;
        use tui::style::*;
        use tui::text::*;
        // print current item to an area of given size
        let print = self.print_all(rect.width as usize - 2, rect.height as usize - 2);
        let print = print.into_iter().map(|x| Spans::from(Span::raw(x)));
        let print = Text::from(print.collect::<Vec<_>>());
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style { fg: Some(self.color()), ..Default::default() });
        f.render_widget(Paragraph::new(print).block(block), rect);
    }
}