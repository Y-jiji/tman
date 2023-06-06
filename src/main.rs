#![feature(round_char_boundary)]

mod view;
mod data;

#[cfg(debug_assertions)]
const PATH: &str = "./tmp"; 
#[cfg(not(debug_assertions))]
const PATH: &str = "C:/ProgramData/tman";

#[cfg(test)]
pub fn log(x: String) {
    use std::io::Write;
    let mut logger = std::fs::OpenOptions::new().create(true)
        .append(true).open("tmp/log").unwrap();
    writeln!(&mut logger, "{x}").unwrap();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut db = {
        let db = data::DataBase::load_yaml(&format!("{PATH}/data.yaml"));
        if let Ok(db) = db { db }
        else {
            let stdin = std::io::stdin();
            let mut line = String::new();
            println!("input a timezone to get started: ");
            stdin.read_line(&mut line)?;
            let tz = line.trim().parse::<i32>()?;
            data::DataBase::new(tz)
        }
    };
    let app = view::Editor::new(view::Mode::Pj, String::from("root"), &db);
    view::run_app(app, &mut db)?;
    db.save_yaml(&format!("{PATH}/data.yaml"))?;
    Ok(())
}