mod event;
pub use event::*;
mod project;
pub use project::*;
mod auto_schedule;
pub use auto_schedule::*;

use serde::*;
use std::collections::{HashMap, HashSet, BTreeMap};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum State {
    #[default]
    Todo,
    Done,
    Abort,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Data {
    pub tz: i64,
    projects: Vec<Project>,
    project_name_map: HashMap<String, usize>,
    event: Vec<Event>,
    event_name_map: HashMap<String, usize>,
    event_time_map: BTreeMap<i64, HashSet<usize>>,
    log: Vec<LogItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataError {
    IndexError(&'static str),
    InvalidId,
    InvalidStateChange,
    InvalidQuotaChange(&'static str),
    InvalidNewProject(&'static str),
    NoSuchProject,
}

impl Data {
    pub fn new(tz: i64) -> Data {
        Data { tz, projects: vec![Project { name: "root".to_string(), id: 0, ..Default::default() }], ..Default::default() }
    }
    pub fn compact(&self) -> Data {
        todo!()
    }
}