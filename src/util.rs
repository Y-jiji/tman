use chrono::offset::*;

pub fn utc_now() -> i64 {
    Utc::now().timestamp()
}