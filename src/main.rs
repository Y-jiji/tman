#![feature(round_char_boundary)]

use data::DataBase;

mod app;
mod data;

#[cfg(debug_assertions)]
const PATH: &str = "./tmp"; 
#[cfg(not(debug_assertions))]
const PATH: &str = "C:/ProgramData/tman";

#[cfg(debug_assertions)]
pub fn log(x: String) {
    use std::io::Write;
    let mut logger = std::fs::OpenOptions::new().create(true)
        .append(true).open("./tmp/log").unwrap();
    writeln!(&mut logger, "{x}").unwrap();
}

fn db_load_or_new() -> Result<DataBase, Box<dyn std::error::Error>> {
    let db = data::DataBase::load_yaml(&format!("{PATH}/data.yaml"));
    if let Ok(db) = db { Ok(db) }
    else {
        let stdin = std::io::stdin();
        let mut line = String::new();
        println!("input a timezone to get started: ");
        stdin.read_line(&mut line)?;
        let tz = line.trim().parse::<i32>()?;
        Ok(data::DataBase::new(tz))
    }
}

fn app_load_or_new() -> Result<app::App, Box<dyn std::error::Error>> {
    let app = app::App::load_yaml(&format!("{PATH}/app.yaml"));
    if let Ok(app) = app { Ok(app) }
    else { Ok(app::App::new()) }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut db = db_load_or_new()?;
    let mut app = app_load_or_new()?;
    app.run(&mut db)?;
    app.save_yaml(&format!("{PATH}/app.yaml"))?;
    db.save_yaml(&format!("{PATH}/data.yaml"))?;
    Ok(())
}