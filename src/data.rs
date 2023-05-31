use serde::*;
use std::collections::{HashMap, HashSet, BTreeMap};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    // project id
    id: usize,
    // project name
    name: String,
    // quota limit for each day
    limit: usize,
    // allocated and estimated quota for this project
    quota: (usize, usize),
    // deadline, if there is any
    deadline: Option<i64>,
    // parent id
    parent: usize,
    // dependencies id
    dependencies: HashSet<usize>,
    // state
    state: State,
    // children ids
    children: HashSet<usize>,
    // dependencies reversed
    dependencies_reverse: HashSet<usize>,
}

impl Project {
    pub fn urgency(&self, utc_now: i64, tz: i64) -> f32 {
        if self.deadline.is_none() { return 0.0; }
        // compute the day before deadline
        let utc_ddl = self.deadline.unwrap();
        let loc_ddl_day = (utc_ddl + tz) / (24*60*60*60);
        let loc_now_day = (utc_now + tz + 24*60*60*60-1) / (24*60*60*60);
        let days = loc_ddl_day - loc_now_day;
        (self.quota.1 - self.quota.0) as f32 / (self.limit * days as usize) as f32
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timed {
    id: usize,
    name: String,
    time: i64,
    repeat: Option<i64>,
    state: State,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum State {
    Done,
    Todo,
    Abort,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Data {
    projects: Vec<Project>,
    project_name_map: HashMap<String, usize>,
    timed: Vec<Timed>,
    timed_name_map: HashMap<String, usize>,
    timed_time_map: BTreeMap<i64, HashSet<usize>>,
    log: Vec<LogItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogItem {
    ProjectMajorUpdate(Project, Project),
    ProjectMinorUpdate(Project, Project),
    ProjectInsert(Project),
    TimedItemInsert(Timed),
    TimedItemUpdate(Timed, Timed),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataError {
    IndexError(&'static str),
    InvalidId,
    InvalidStateChange,
    InvalidQuotaChange(&'static str),
    NoSuchProject,
}

impl Data {
    pub fn compact(&self) -> Data {
        todo!()
    }
    pub fn recursive_children(&self, id: usize) -> Vec<usize> {
        let rc = self.projects[id].children.iter();
        let rc = rc.flat_map(|x| self.recursive_children(*x));
        rc.chain([id]).collect::<Vec<_>>()
    }
    pub fn get_project_by_name(&self, name: &str) -> Result<Project, DataError> {
        let id = self.project_name_map.get(name).ok_or(DataError::NoSuchProject)?;
        Ok(self.projects[*id].clone())
    }
    pub fn get_project_by_id(&self, id: usize) -> Result<Project, DataError> {
        self.projects.get(id).ok_or(DataError::NoSuchProject).cloned()
    }
    // insert or update a new task
    pub fn upsert_timed(&mut self, mut timed: Timed) -> Result<(), DataError> {
        let mut id = timed.id;
        if id >= self.timed.len() {
            id = self.timed.len();
            timed.id = id;
            if self.timed_name_map.contains_key(&timed.name) {
                Err(DataError::IndexError("timed item name is not unique"))?
            }
            self.log.push(LogItem::TimedItemInsert(timed.clone()));
        } else {
            if self.timed[id].name != timed.name && self.timed_name_map.contains_key(&timed.name) {
                Err(DataError::IndexError("timed item name is not unique"))?
            }
            self.log.push(LogItem::TimedItemUpdate(self.timed[id], timed.clone()));
            self.timed_name_map.remove(&self.timed[id].name);
            self.timed_time_map[&self.timed[id].time].remove(&id);
        }
        self.timed_name_map.insert(timed.name.clone(), timed.id);
        self.timed_time_map.entry(timed.time)
            .and_modify(|x| {x.insert(id); })
            .or_insert(HashSet::from([id]));
        self.timed.push(timed);
        Ok(())
    }
    // insert or update a new project
    pub fn upsert_project(&mut self, mut new_project: Project) -> Result<(), DataError> {
        let mut id = new_project.id;
        if new_project.parent >= self.projects.len() { Err(DataError::InvalidId)? }
        if id >= self.projects.len() {
            id = self.projects.len();
            new_project.id = id;
            self.log.push(LogItem::ProjectInsert(new_project.clone()));
            if self.project_name_map.contains_key(&new_project.name) { Err(DataError::IndexError("project name is not unique"))? }
            self.project_name_map.insert(new_project.name.clone(), id);
            new_project.children.clear();
            new_project.dependencies_reverse.clear();
            let parent = &mut self.projects[new_project.parent];
            new_project.dependencies.drain_filter(|x| !parent.children.contains(&x));
            parent.children.insert(id);
            parent.quota.0 += new_project.quota.1;
            parent.quota.1 = parent.quota.1.max(parent.quota.0);
            for d in &new_project.dependencies {
                self.projects[*d].dependencies_reverse.insert(id);
            }
            self.projects.push(new_project);
            Ok(())
        } else {
            if self.projects[id].name != new_project.name {
                if self.project_name_map.contains_key(&new_project.name) {
                    Err(DataError::IndexError("project name is not unique"))?
                }
            }
            if self.projects[id].state != State::Todo && 
                new_project.state != self.projects[id].state {
                Err(DataError::InvalidStateChange)?
            }
            if self.projects[id].quota.0 > new_project.quota.0 {
                Err(DataError::InvalidQuotaChange("allocated quota should only increase"))?
            }
            self.log.push(LogItem::ProjectMajorUpdate(self.projects[id].clone(), new_project.clone()));
            self.project_name_map.insert(new_project.name.clone(), id);
            for c in self.recursive_children(id) {
                if matches!(self.projects[c].state, State::Todo) {
                    let old = self.projects[c].clone();
                    self.projects[c].state = new_project.state.clone();
                    self.log.push(LogItem::ProjectMinorUpdate(old, self.projects[c].clone()));
                    self.project_name_map.remove(&self.projects[c].name);
                }
            }
            // remove old dependencies and quota
            let old_project = self.projects[id].clone();
            let old_parent = &mut self.projects[old_project.parent];
            old_parent.children.remove(&id);
            old_parent.quota.0 -= old_project.quota.1;
            for d in &old_project.dependencies {
                self.projects[*d].dependencies_reverse.remove(&d);
            }
            // add new dependencies and quota
            let new_parent = &mut self.projects[new_project.parent];
            new_parent.children.insert(id);
            new_parent.quota.0 += new_project.quota.1;
            new_parent.quota.1 = new_parent.quota.1.max(new_parent.quota.0);
            new_project.dependencies.drain_filter(|x| !new_parent.children.contains(&x));
            for d in &new_project.dependencies {
                self.projects[*d].dependencies_reverse.remove(&d);
            }
            self.projects[id] = new_project;
            Ok(())
        }
    }
}