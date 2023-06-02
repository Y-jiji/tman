use chrono::{Datelike, Months};

pub struct CalendarMonthView<'a> {
    command: String,
    year_and_month: (u32, u32),
    days: Vec<[String; 7]>,
    data: &'a mut crate::data::Data,
}

impl<'a> CalendarMonthView<'a> {
    pub fn new(data: &'a mut crate::data::Data) -> CalendarMonthView<'a> {
        // get current year and month
        let now = crate::util::conv_utc_loc(crate::util::utc_now(), data.tz);
        let now = chrono::NaiveDateTime::from_timestamp_opt(now, 0).unwrap();
        let (year, month) = (now.year(), now.month());
        // get the beginning and end days of the month, rounded by week
        let mut days_begin = chrono::NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let mut days_end = chrono::NaiveDate::from_ymd_opt(year + if month+1 > 12 { 1 } else { 0 }, month % 12 + 1, 1).unwrap();
        while days_begin.weekday().number_from_sunday() != 0 {
            days_begin = days_begin.pred_opt().unwrap();
        }
        while days_end.weekday().number_from_sunday() != 0 {
            days_end = days_end.succ_opt().unwrap();
        }
        // get the deadlines and events in this month 
        todo!()
    }
    pub fn trigger_command(&mut self) {
    }
}