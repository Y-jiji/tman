#![feature(pattern)]
#![feature(hash_drain_filter)]
#![feature(round_char_boundary)]

mod data;
mod util;
mod view;

#[cfg(not(profile="release"))]
const PATH: &str = "./tmp";
#[cfg(profile="release")]
const PATH: &str = "C:/ProgramData/tman";
use view::*;

fn main() {
    let mut data = {
        let stringified_data = std::fs::read_to_string(format!("{PATH}/latest.json")).unwrap_or(String::new());
        let stamp = crate::util::utc_now();
        std::fs::copy(format!("{PATH}/latest.json"), format!("{PATH}/{stamp}.json")).unwrap_or(0);
        serde_json::from_str(&stringified_data).unwrap_or(crate::data::Data::new(0))
    };
    let args = std::env::args().collect::<Vec<_>>();
    let mut switch = match args.get(1).as_deref() {
        Some(x) if x == "edit" => {
            let name = args.get(2).cloned().unwrap_or(String::new());
            Switch::Edit { name }
        }
        Some(x) if x == "plan" => {
            Switch::Plan
        }
        Some(x) if x == "calendar" || x == "cal" => {
            Switch::Calendar
        }
        _ => panic!("You should provide a subcommand like [edit] or [plan]")
    };
    while !matches!(switch, Switch::Exit) {
        switch = match switch {
            Switch::Edit { name } => {
                run_app(EditView::new(name, &mut data)).unwrap()
            }
            Switch::Calendar => {
                run_app(CalendarMonthView::new(&mut data)).unwrap()
            }
            Switch::Plan => {
                Switch::Exit
            }
            Switch::List => {
                Switch::Exit
            }
            Switch::Exit => unreachable!()
        };
    }
    std::fs::write(format!("{PATH}/latest.json"), serde_json::to_string(&data).unwrap()).unwrap();
}