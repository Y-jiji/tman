use tui::layout::*;
use tui::widgets::*;
use tui::text::*;
use unicode_width::UnicodeWidthStr;
use super::Switch;
use chrono::Datelike;
use tui::style::*;
use chrono::Weekday;
use std::str::pattern::Pattern;

pub struct CalendarMonthView<'a> {
    cursor: usize,
    command: String,
    year_and_month: (i32, u32),
    // month, date, name, color
    days: Vec<((u32, u32), Vec<(String, (u8, u8, u8))>)>,
    data: &'a mut crate::data::Data,
    quit: Option<Switch>,
}

impl<'a> CalendarMonthView<'a> {
    pub fn new(data: &'a mut crate::data::Data) -> CalendarMonthView<'a> {
        // get current year and month
        let now = crate::util::conv_utc_loc(crate::util::utc_now(), data.tz);
        let now = chrono::NaiveDateTime::from_timestamp_opt(now, 0).unwrap();
        let (year, month) = (now.year(), now.month());
        let mut out = Self { cursor: 0, command: String::new(), year_and_month: (year, month), days: vec![], data, quit: None };
        out.refresh_items();
        return out;
    }
    pub fn refresh_items(&mut self) {
        // get year and month
        let (year, month) = self.year_and_month;
        // get the beginning and end days of the month, rounded by week
        let mut days_begin = chrono::NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let mut days_end = chrono::NaiveDate::from_ymd_opt(year + if month+1 > 12 { 1 } else { 0 }, month % 12 + 1, 1).unwrap();
        while days_begin.weekday() != Weekday::Sun {
            days_begin = days_begin.pred_opt().unwrap();
        }
        while days_end.weekday() != Weekday::Sun {
            days_end = days_end.succ_opt().unwrap();
        }
        // iterate over days, put project name and colour into month schedule
        let mut current_day = days_begin.clone();
        let mut days = vec![];
        while current_day != days_end {
            let current_day_start = crate::util::conv_loc_utc(current_day.and_hms_opt(0, 0, 0).unwrap().timestamp(), self.data.tz);
            let current_day_end = crate::util::conv_loc_utc(current_day.succ_opt().unwrap().and_hms_opt(0, 0, 0).unwrap().timestamp(), self.data.tz);
            let current_day_project_ddls = self.data.get_project_by_time_range((current_day_start, current_day_end));
            let current_day_events = self.data.get_event_by_range((current_day_start, current_day_end));
            let today_items = 
                current_day_project_ddls.into_iter().map(|i| {
                    let project = self.data.get_project_by_id(i).unwrap();
                    (
                        project.name.clone(), project.color
                    )
                }).chain(current_day_events.into_iter().map(|e| {
                    (
                        e.name.clone(), e.color
                    )
                })).collect::<Vec<_>>();
            // use std::io::Write;
            // let mut logger = std::fs::OpenOptions::new().create(true).append(true).open("logger").unwrap();
            // writeln!(&mut logger, "{current_day_start}:{current_day_end}\n\t{today_items:?}").unwrap();
            days.push(((current_day.month(), current_day.day()), today_items));
            current_day = current_day.succ_opt().unwrap();
        }
        self.days = days;
    }
    pub fn trigger_command(&mut self) {
        let args_string = self.command.clone();
        self.command = String::new();
        self.cursor = 0;
        let args = args_string.trim().split_whitespace().collect::<Vec<_>>();
        match args.get(0).map(|x| x as &str) {
            Some("exit") => {
                self.quit = Some(Switch::Exit);
            }
            Some("edit") if args.get(1).is_some() => {
                self.quit = Some(Switch::Edit { name: args[1].to_string() });
            }
            Some("cal") | Some("calendar") => {
                self.quit = Some(Switch::Calendar);
            }
            Some("go") if args.get(1).is_some() => {
                let mut arg1 = args[1].trim().split("/");
                let (year, month) = (arg1.next(), arg1.next().map(|x| x.chars().skip_while(|x| *x == '0').collect()));
                if let (Some(Ok(year)), Some(Ok(month))) = (year.map(|x| x.parse::<i32>()), month.map(|x: String| x.parse::<u32>())) {
                    self.year_and_month = (year, month);
                    self.refresh_items();
                }
            }
            Some("next") | Some("n") => {
                let ym = self.year_and_month.0 * 12 + self.year_and_month.1 as i32;
                self.year_and_month = (ym / 12, (ym % 12 + 1) as u32);
                self.refresh_items();
            }
            Some("last") | Some("l") => {
                let ym = self.year_and_month.0 * 12 + self.year_and_month.1 as i32 - 2;
                self.year_and_month = (ym / 12, (ym % 12 + 1) as u32);
                self.refresh_items();
            }
            _ => {}
        }
    }
}

fn perfect_split(rect: Rect, grid: (u16, u16)) -> Vec<Vec<Rect>> {
    let (width, height) = (rect.width, rect.height);
    let width_rest_left = (width % grid.0) / 2;
    let width_rest_right = (width % grid.1 + 1) / 2;
    let height_rest_top = (height % grid.1 + 1) / 2;
    let height_rest_bottom = (height % grid.1) / 2;
    let mut width_split = vec![Constraint::Length(width_rest_left)];
    width_split.extend(vec![Constraint::Length(width / grid.0); grid.0 as usize].into_iter());
    width_split.push(Constraint::Length(width_rest_right));
    let mut height_split = vec![Constraint::Length(height_rest_top)];
    height_split.extend(vec![Constraint::Length(height / grid.1); grid.0 as usize].into_iter());
    height_split.push(Constraint::Length(height_rest_bottom));
    let mut out = vec![];
    for chunk in Layout::default().direction(tui::layout::Direction::Vertical).constraints(height_split).split(rect).into_iter().skip(1).take(grid.1 as usize) {
        let mut row = vec![];
        for chunk in Layout::default().direction(tui::layout::Direction::Horizontal).constraints(width_split.clone()).split(chunk).into_iter().skip(1).take(grid.0 as usize) {
            row.push(chunk);
        }
        out.push(row);
    }
    return out;
}

impl<'a> super::App for CalendarMonthView<'a> {
    fn draw(&self, f: &mut tui::Frame<tui::backend::CrosstermBackend<std::io::Stdout>>) {
        let area = f.size();
        let _tmp = Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(4), Constraint::Min(4)]).split(area);
        let area_command = _tmp[0];
        let command_widget = Paragraph::new(Text::raw(self.command.clone())).block(Block::default().borders(Borders::all()).title(" Command "));
        f.render_widget(command_widget, area_command);
        let grid = (7 as u16, (self.days.len()/7) as u16);
        for (i, block) in perfect_split(_tmp[1], grid).into_iter().flat_map(|x| x).enumerate() {
            let date_as_header = Spans::from(Span::raw(format!("{}/{}", self.days[i].0.0, self.days[i].0.1)));
            let day_items = self.days[i].1.iter().map(|(name, color)| Spans::from(Span::styled(format!("{name:<16}"), Style::default().bg(Color::Rgb(color.0, color.1, color.2)).fg(Color::Rgb(!color.0, !color.1, !color.2)))));
            let text = Text::from([date_as_header].into_iter().chain(day_items).collect::<Vec<_>>());
            f.render_widget(Paragraph::new(text).alignment(Alignment::Center).block(Block::default().borders(Borders::all())), block);
        }
        f.set_cursor(area_command.x + self.command.get(..self.cursor).unwrap().width() as u16 + 1, area_command.y + 1);
    }
    fn on_key_code(&mut self, key_code: crossterm::event::KeyCode) -> () {
        use crossterm::event::KeyCode::*;
        match key_code {
            Char(c) => { self.command.insert(self.cursor, c); self.cursor = self.command.ceil_char_boundary(usize::min(self.cursor + 1, self.command.len())); }
            Enter => { self.trigger_command(); }
            Backspace if self.cursor != 0 => { self.command.remove(self.command.floor_char_boundary((self.cursor - 1).min(self.command.len()-1))); self.cursor = self.command.floor_char_boundary(self.cursor.checked_sub(1).unwrap_or(0)); }
            Left => { self.cursor = self.command.floor_char_boundary(self.cursor.checked_sub(1).unwrap_or(0)); }
            Right => { self.cursor = self.command.ceil_char_boundary(usize::min(self.cursor + 1, self.command.len())); }
            Esc => { self.quit = Some(Switch::Exit) }
            _ => {},
        }
    }
    fn quit(&self) -> Option<Switch> {
        self.quit.clone()
    }
}