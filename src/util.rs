use chrono::offset::*;

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