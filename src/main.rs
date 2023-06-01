#![feature(hash_drain_filter)]

use view::Switch;
mod data;
mod util;
mod view;
mod algo;

const PATH: &str = "C:/ProgramData/tman";

fn main() {
    let mut data = {
        let stringified_data = std::fs::read_to_string(format!("{PATH}/latest.json")).unwrap_or(String::new());
        serde_json::from_str(&stringified_data).unwrap_or(crate::data::Data::new(0))
    };
    let stamp = crate::util::utc_now();
    let args = std::env::args().collect::<Vec<_>>();
    let mut switch = match args.get(1).as_deref() {
        Some(x) if x == "edit" => {
            let name = args.get(2).cloned().unwrap_or(String::new());
            Switch::Edit { name }
        },
        Some(x) if x == "plan" => {
            Switch::Plan
        },
        _ => panic!("You should provide a subcommand like [edit] or [plan]")
    };
    while !matches!(switch, Switch::Exit) {
        switch = match switch {
            Switch::Edit { name } => {
                let project = data.get_project_by_name(&name).unwrap_or(crate::data::Project::new(name.to_string()));
                let app = crate::view::EditView::new(project, &mut data);
                crate::view::run_app(app).unwrap()
            },
            Switch::Plan => {
                crate::view::Switch::Exit
            },
            Switch::List => {
                crate::view::Switch::List
            }
            Switch::Exit => unreachable!()
        };
    }
    std::fs::copy(format!("{PATH}/latest.json"), format!("{PATH}/{stamp}.json")).unwrap_or(0);
    std::fs::write(format!("{PATH}/latest.json"), serde_json::to_string(&data).unwrap()).unwrap();
}