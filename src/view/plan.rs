use super::Command;
use std::collections::VecDeque;
use tui::text::*;
use tui::style::*;
use tui::backend::CrosstermBackend;
use tui::Frame;
use std::io::Stdout;
use unicode_width::UnicodeWidthStr;
use super::App;
use tui::layout::*;
use tui::widgets::*;

pub struct AutoScheduleView<'a> {
    // the view of auto scheduler
    data: &'a mut crate::data::Data,
    // schedule
    schedule: (i64, VecDeque<Vec<(usize, usize)>>),
    // range displayed in year and week
    year: i32,
    week: u32,
    // exit state
    quit: Option<super::Switch>,
    // command line
    command: super::Command,
}

impl<'a> AutoScheduleView<'a> {
    pub fn new(data: &'a mut crate::data::Data) -> Self {
        // compute the utc timestamp of today's start
        let today = crate::util::utc_now();
        let today = crate::util::conv_utc_loc(today, data.tz);
        let today = today / (24*60*60) * (24*60*60);
        // compute the year and week of today
        let (year, week) = {
            use chrono::Datelike;
            let today = chrono::NaiveDateTime::from_timestamp_opt(today, 0).unwrap();
            (today.year(), today.iso_week().week())
        };
        let today = crate::util::conv_loc_utc(today, data.tz);
        // compute the schedule start from today
        let schedule = data.auto_schedule(today);
        Self { data, schedule, year, week, command: Command::new(), quit: None }
    }
    pub fn trigger_command(&mut self) {
        use chrono::*;
        let args_string = self.command.get_command();
        let args = args_string.trim().split_whitespace().collect::<Vec<_>>();
        match args.get(0).map(|x| x as &str) {
            Some("next") | Some("n") => {
                let day = NaiveDate::from_isoywd_opt(self.year, self.week, Weekday::Sun)
                    .unwrap().succ_opt().unwrap();
                (self.year, self.week) = (day.year(), day.iso_week().week())
            }
            Some("last") | Some("l") => {
                let day = NaiveDate::from_isoywd_opt(self.year, self.week, Weekday::Mon)
                    .unwrap().pred_opt().unwrap();
                (self.year, self.week) = (day.year(), day.iso_week().week())
            }
            _ => {}
        }
    }
    fn get_week_schedule(&self, year: i32, week: u32, width: usize) -> [Text; 7] {
        use chrono::*;
        // get text for each week
        let width = width - 4;
        let start = NaiveDate::from_isoywd_opt(year, week, Weekday::Mon)
            .unwrap().pred_opt()
            .unwrap().and_hms_opt(0, 0, 0)
            .unwrap().timestamp();
        let start = crate::util::conv_loc_utc(start, self.data.tz);
        [0, 1, 2, 3, 4, 5, 6].map(|i| {
            let i = (start + i * 24*60*60 - self.schedule.0) / (24*60*60);
            let content = if i < 0 || i >= self.schedule.1.len() as i64 { 
                vec![Spans::from(String::new())] 
            } else {
                let events = self.data
                    .get_event_by_range((start + i*24*60*60, start + (i+1)*24*60*60))
                    .into_iter().map(|event| {
                        let amount = format!("{}", event.quota);
                        let mut name = event.name.clone();
                        let mut width_0 = name.width();
                        let width_1 = amount.width();
                        while width_0 + width_1 + 1 > width {
                            name.pop();
                            width_0 = name.width();
                        }
                        let content = format!("{name}{}{amount}", " ".repeat(width - width_0 - width_1));
                        let color = event.color;
                        let light = if color.0/3 + color.1/3 + color.2/3 > 85 
                            { 0 } else { 255 };
                        let style = Style::default()
                            .bg(Color::Rgb(color.0, color.1, color.2))
                            .fg(Color::Rgb(light, light, light));
                        Span::styled(content, style)
                    })
                    .map(Spans::from);
                self.schedule.1[i as usize]
                    .iter()
                    .map(|&(i, amount)| {
                        let project = self.data.get_project_by_id(i).unwrap();
                        let amount = format!("{}", amount);
                        let mut name = project.name.clone();
                        let mut width_0 = name.width();
                        let width_1 = amount.width();
                        while width_0 + width_1 + 1 > width {
                            name.pop();
                            width_0 = name.width();
                        }
                        let content = format!("{name}{}{amount}", " ".repeat(width - width_0 - width_1));
                        let color = project.color;
                        let light = if color.0/3 + color.1/3 + color.2/3 > 85 
                            { 0 } else { 255 };
                        let style = Style::default()
                            .bg(Color::Rgb(color.0, color.1, color.2))
                            .fg(Color::Rgb(light, light, light));
                        Span::styled(content, style)
                    })
                    .map(Spans::from).chain(events)
                    .collect::<Vec<_>>()
            };
            Text::from(content)
        })
    }
}
impl<'a> App for AutoScheduleView<'a> {
    fn draw(&self, f: &mut Frame<CrosstermBackend<Stdout>>) {
        // split frame to upper and lower half
        let _tmp = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(4), Constraint::Min(4)])
            .split(f.size());
        let grid = (7 as u16, 1 as u16);
        let grid = crate::util::perfect_split(_tmp[1], grid);
        let week = self.get_week_schedule(self.year, self.week, grid[0][0].width as usize);
        for (i, grid) in grid.into_iter().flatten().enumerate() {
            let title = {
                let weekday = match i {
                    0 => "Sun", 1 => "Mon", 2 => "Tue", 3 => "Wed", 
                    4 => "Thu", 5 => "Fri", 6 => "Sat", _ => "", 
                };
                Block::default()
                    .borders(Borders::all())
                    .title(format!("{:04}|{:02}|{weekday}", self.year, self.week))
                    .title_alignment(Alignment::Center)
            };
            let block = Paragraph::new(week[i].clone())
                .block(title)
                .alignment(Alignment::Center);
            f.render_widget(block, grid);
        }
        self.command.draw(f, _tmp[0]);
    }
    fn on_key_code(&mut self, key_code: crossterm::event::KeyCode) -> () {
        let trigger = self.command.on_key_code(key_code);
        if !trigger { return }
        match self.command.try_switch() {
            Some(q) => self.quit = Some(q),
            None => self.trigger_command()
        }
    }
    fn quit(&self) -> Option<super::Switch> {
        self.quit.clone()
    }
}