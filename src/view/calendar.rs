use tui::layout::*;
use tui::widgets::*;
use tui::text::*;
use super::Switch;
use chrono::Datelike;
use tui::style::*;
use chrono::Weekday;

pub struct CalendarMonthView<'a> {
    command: super::Command,
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
        let mut out = Self { command: super::Command::new(), year_and_month: (year, month), days: vec![], data, quit: None };
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
            days.push(((current_day.month(), current_day.day()), today_items));
            current_day = current_day.succ_opt().unwrap();
        }
        self.days = days;
    }
    pub fn trigger_command(&mut self) {
        let args_string = self.command.get_command();
        let args = args_string.trim().split_whitespace().collect::<Vec<_>>();
        match args.get(0).map(|x| x as &str) {
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

impl<'a> super::App for CalendarMonthView<'a> {
    fn draw(&self, f: &mut tui::Frame<tui::backend::CrosstermBackend<std::io::Stdout>>) {
        let area = f.size();
        // split frame to upper and lower half
        let _tmp = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(4), Constraint::Min(4)])
            .split(area);
        let grid = (7 as u16, (self.days.len()/7) as u16);
        let grid = crate::util::perfect_split(_tmp[1], grid).into_iter().flat_map(|x| x).enumerate();
        // render days into grid
        for (i, grid) in grid {
            let width = (grid.width - 2) as usize;
            let today = self.days[i].1.iter()
                .map(|(name, color)| Span::styled(format!("{name:<width$}"), 
                    Style::default().bg(Color::Rgb(color.0, color.1, color.2))
                )).map(Spans::from);
            let today = Text::from(today.collect::<Vec<_>>());
            let title = {
                let weekday = match i {
                    0 => "Sun ", 1 => "Mon ", 2 => "Tue ", 3 => "Wed ", 
                    4 => "Thu ", 5 => "Fri ", 6 => "Sat ", _ => "", 
                };
                Block::default()
                .borders(Borders::all())
                .title(format!("{weekday}{}/{}", self.days[i].0.0, self.days[i].0.1))
                .title_alignment(Alignment::Center)
            };
            f.render_widget(Paragraph::new(today).block(title), grid);
        }
        self.command.draw(f, _tmp[0]);
    }
    fn on_key_code(&mut self, key_code: crossterm::event::KeyCode) {
        let trigger = self.command.on_key_code(key_code);
        if !trigger { return }
        match self.command.try_switch() {
            Some(q) => self.quit = Some(q),
            None => self.trigger_command()
        }
    }
    fn quit(&self) -> Option<Switch> {
        self.quit.clone()
    }
}