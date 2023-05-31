use chrono::offset::*;

// get utc minute stamp
pub fn utc_now() -> i64 {
    Utc::now().timestamp() as i64 / 60i64
}

// offset from -12 to +12
pub fn conv_utc_loc(time: i64, offset: i64) -> i64 {
    return time + offset * 60;
}

// offset from -12 to +12
pub fn conv_loc_utc(time: i64, offset: i64) -> i64 {
    return time - offset * 60;
}