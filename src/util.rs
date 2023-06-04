use chrono::offset::*;
use chrono::*;
use tui::layout::*;

// get utc minute stamp
pub fn utc_now() -> i64 {
    Utc::now().timestamp() as i64
}

// offset from -12 to +12
pub fn conv_utc_loc(time: i64, tz: i64) -> i64 {
    return time + tz * 60 * 60;
}

// offset from -12 to +12
pub fn conv_loc_utc(time: i64, tz: i64) -> i64 {
    return time - tz * 60 * 60;
}

// parse year month data to timestamp
pub fn parse_year_month_date(y: i64, m: i64, d: i64, h: i64, min: i64) -> i64 {
    let datetime = chrono::NaiveDate::from_ymd_opt(y as i32, m as u32, d as u32).unwrap().and_hms_opt(h as u32, min as u32, 0).unwrap();
    datetime.timestamp()
}

// get days of a month
pub fn get_days_from_month(year: i32, month: u32) -> i64 {
    NaiveDate::from_ymd_opt(
        match month {
            12 => year + 1,
            _ => year,
        },
        match month {
            12 => 1,
            _ => month + 1,
        },
        1,
    ).unwrap()
    .signed_duration_since(NaiveDate::from_ymd_opt(year, month, 1).unwrap())
    .num_days()
}


pub fn perfect_split(rect: Rect, grid: (u16, u16)) -> Vec<Vec<Rect>> {
    let (width, height) = (rect.width, rect.height);
    let width_rest_left = (width % grid.0) / 2;
    let width_rest_right = (width % grid.1 + 1) / 2;
    let height_rest_top = (height % grid.1) / 2;
    let height_rest_bottom = (height % grid.1 + 1) / 2;
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
pub fn log(x: String) {
    use std::io::Write;
    let mut logger = std::fs::OpenOptions::new().create(true)
        .append(true).open("logger").unwrap();
    writeln!(&mut logger, "{x}").unwrap();
}