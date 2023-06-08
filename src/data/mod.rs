mod ev;
mod pj;
mod auto_schedule;

pub use ev::*;
pub use pj::*;
use serde::*;
use chrono::{NaiveDateTime, TimeZone};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DBLog {
    Ev(ev::EvLog),
    Pj(pj::PjLog),
}

impl From<ev::EvLog> for DBLog {
    fn from(value: ev::EvLog) -> Self { DBLog::Ev(value) }
}

impl From<pj::PjLog> for DBLog {
    fn from(value: pj::PjLog) -> Self { DBLog::Pj(value) }
}

#[derive(Debug, Clone)]
pub enum DBErr {
    Pj(pj::PjErr),
    Ev(ev::EvErr),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataBase {
    tz: i32,
    ev: ev::EvStore,
    pj: pj::PjStore,
    log: Vec<Vec<DBLog>>,
}

impl DataBase {
    pub fn new(tz: i32) -> Self {
        Self {
            ev: ev::EvStore::new(),
            pj: pj::PjStore::new(),
            tz, log: Vec::new(),
        }
    }
    pub fn load_yaml(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(serde_yaml::from_reader::<_, Self>(std::fs::File::open(path)?)?)
    }
    pub fn save_yaml(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        Ok(serde_yaml::to_writer(std::fs::File::create(path)?, self)?)
    }
    pub fn pj_list(&self) -> Result<Vec<String>, DBErr> {
        self.pj.get_all_names().map_err(DBErr::Pj)
    }
    pub fn ev_list(&self) -> Result<Vec<String>, DBErr> {
        self.ev.get_all_names().map_err(DBErr::Ev)
    }
    pub fn pj_get_by_name(&self, name: &str) -> Option<Pj> {
        self.pj.get_by_name(name)
    }
    pub fn pj_get_or_create_by_name(&mut self, name: &str) -> Pj {
        self.pj.get_by_name(name).unwrap_or_else(|| {
            let pj = pj::Pj::new(name.to_string());
            let res = self.pj.create(pj);
            self.log.push(vec![res.unwrap().into()]);
            self.pj.get_by_name(name).unwrap()
        })
    }
    pub fn ev_get_by_name(&self, name: &str) -> Option<Ev> {
        self.ev.get_by_name(name)
    }
    pub fn pj_get_by_id(&self, id: usize) -> Option<Pj> {
        self.pj.get_by_id(id)
    }
    pub fn ev_get_by_id(&self, id: usize) -> Option<Ev> {
        self.ev.get_by_id(id)
    }
    pub fn pj_set_name(&mut self, id: usize, name: String) -> Result<(), DBErr> {
        let log = self.pj.update_name(id, name).map_err(DBErr::Pj)?;
        self.log.push(vec![DBLog::Pj(log)]); Ok(())
    }
    pub fn ev_set_name(&mut self, id: usize, name: String) -> Result<(), DBErr> {
        let log = self.ev.update_name(id, name).map_err(DBErr::Ev)?;
        self.log.push(vec![DBLog::Ev(log)]); Ok(())
    }
    pub fn set_tz(&mut self, tz: i32) {
        assert!(tz >= -12 && tz <= 12);
        self.tz = tz;
    }
    pub fn tz(&self) -> i32 {
        self.tz
    }
    pub fn datetime_loc(&self) -> Result<chrono::NaiveDateTime, DBErr> {
        let naive_utc = chrono::offset::Utc::now().naive_utc();
        Ok(naive_utc.checked_add_signed(chrono::Duration::hours(self.tz as i64)).unwrap())
    }
    pub fn datetime_utc(&self) -> Result<chrono::NaiveDateTime, DBErr> {
        Ok(chrono::offset::Utc::now().naive_utc())
    }
}