use serde::*;
use std::collections::{HashMap, HashSet, BTreeMap};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Project {
    // project id
    id: usize,
    // project name
    pub name: String,
    // quota limit for each day
    pub limit: usize,
    // allocated and estimated quota for this project
    pub quota: (usize, usize),
    // deadline, if there is any
    pub deadline: Option<i64>,
    // parent id
    pub parent: usize,
    // dependencies id
    pub dependencies: HashSet<usize>,
    // state
    pub state: State,
    // children ids
    children: HashSet<usize>,
    // dependencies reversed
    dependencies_reverse: HashSet<usize>,
}

impl Project {
    pub fn need_quota(&self) -> usize {
        self.quota.1 - self.quota.0
    }
    pub fn new(name: String) -> Project {
        Project {
            name, id: usize::MAX,
            limit: usize::MAX, 
            ..Default::default()
        }
    }
    pub fn id(&self) -> usize {
        self.id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    id: usize,
    pub name: String,
    pub time: i64,
    pub state: State,
    pub quota: usize,
}

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
pub enum LogItem {
    ProjectMajorUpdate(Project, Project),
    ProjectMinorUpdate(Project, Project),
    ProjectInsert(Project),
    EventItemInsert(Event),
    EventItemUpdate(Event, Event),
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
    pub fn new(tz: i64) -> Data {
        Data { tz, projects: vec![Project { name: "root".to_string(), id: 0, ..Default::default() }], ..Default::default() }
    }
    pub fn active_projects(&self, ban: &HashSet<usize>) -> Vec<usize> {
        (0..self.projects.len()).filter(
            |i| self.projects[*i].state == State::Todo && 
                self.projects[*i].dependencies.iter().all(|x| self.projects[*x].state != State::Todo) &&
                self.projects[*i].children.iter().all(|x| self.projects[*x].state != State::Todo) &&
                !ban.contains(&i)
            ).collect()
    }
    pub fn compact(&self) -> Data {
        todo!()
    }
    pub fn recursive_children(&self, id: usize) -> Vec<usize> {
        let rc = self.projects[id].children.iter();
        let rc = rc.flat_map(|x| self.recursive_children(*x));
        rc.chain([id]).collect::<Vec<_>>()
    }
    pub fn get_project_by_name(&self, name: &str) -> Result<Project, DataError> {
        if name == "root" { return Ok(self.projects[0].clone()); }
        let id = self.project_name_map.get(name).ok_or(DataError::NoSuchProject)?;
        Ok(self.projects[*id].clone())
    }
    pub fn get_project_by_id(&self, id: usize) -> Result<Project, DataError> {
        self.projects.get(id).ok_or(DataError::NoSuchProject).cloned()
    }
    pub fn recursive_dependencies(&self, id: usize) -> Vec<usize> {
        let rd = self.projects[id].dependencies.iter();
        let rd = rd.flat_map(|x| self.recursive_dependencies(*x));
        rd.chain([id]).collect::<Vec<_>>()
    }
    pub fn recursive_reverse_dependencies(&self, id: usize) -> Vec<usize> {
        let rrd = self.projects[id].dependencies_reverse.iter();
        let rrd = rrd.flat_map(|x| self.recursive_reverse_dependencies(*x));
        rrd.chain([id]).collect::<Vec<_>>()
    }
    pub fn recursive_reverse_dependencies_exclusive_and_same_for_parent(&self, id: usize) -> Vec<usize> {
        if id == 0 { return vec![] }
        let mut rrd = self.recursive_dependencies(id); rrd.pop();
        let rrdep = self.recursive_reverse_dependencies_exclusive_and_same_for_parent(self.projects[id].parent);
        rrdep.into_iter().chain(rrd).collect::<Vec<_>>()
    }
    // aggregate quota from children and dependencies
    pub fn aggregate_quota(&self) -> Vec<usize> {
        // 先把每棵子树的quota聚到根上作为子项目自己的quota
        let self_quota = (0..self.projects.len())
            .map(|x| if self.projects[x].state == State::Todo {
                    self.projects[x].quota.1 - self.projects[x].quota.0
                } else { 0 }
            ).collect::<Vec<_>>();
        let self_quota = (0..self.projects.len())
            .map(|x| self.recursive_children(x).into_iter().fold(0, |a, b| self_quota[a] + self_quota[b]))
            .collect::<Vec<_>>();
        // 累加兄弟节点中前驱的quota, 复杂度 O(NW)
        let mut deps_quota = (0..self.projects.len())
            .map(|x| self.recursive_dependencies(x).into_iter().filter(|i| *i != x)
                    .fold(0, |a, b| self_quota[a] + self_quota[b])
            ).collect::<Vec<_>>();
        // 将子树根的前驱累加到子树的每个元素的前驱上, 由于recursive children是后序的, 
        // 所以要倒过来才能保证先算了父节点的累加, 复杂度 O(N)
        for i in self.recursive_children(0).into_iter().rev() {
            for &c in self.projects[i].children.iter() {
                deps_quota[c] += deps_quota[i];
            }
        }
        // 最后将每个子项目自己的quota和前驱的quota加起来, 得到最后的结果
        // 总的时间复杂度O(NW), N是总的节点数, W是最大的树宽, 通常达不到上界
        return (0..self.projects.len()).map(|i| deps_quota[i] + self_quota[i]).collect::<Vec<_>>();
    }
    // get tasks in time range
    pub fn get_event_by_range(&self, utc_range: (i64, i64)) -> Vec<Event> {
        self.event_time_map.range(utc_range.0..utc_range.1).into_iter()
            .flat_map(|x| x.1.iter()).map(|x| self.event[*x].clone()).collect()
    }
    // insert or update a new task
    pub fn upsert_event(&mut self, mut event: Event) -> Result<(), DataError> {
        let mut id = event.id;
        if id >= self.event.len() {
            id = self.event.len();
            event.id = id;
            if self.event_name_map.contains_key(&event.name) {
                Err(DataError::IndexError("event item name is not unique"))?
            }
            self.log.push(LogItem::EventItemInsert(event.clone()));
            self.event_name_map.insert(event.name.clone(), event.id);
            self.event_time_map.entry(event.time)
                .and_modify(|x| {x.insert(id); })
                .or_insert(HashSet::from([id]));
            self.event.push(event);
        } else {
            if self.event[id].name != event.name && self.event_name_map.contains_key(&event.name) {
                Err(DataError::IndexError("event item name is not unique"))?
            }
            self.log.push(LogItem::EventItemUpdate(self.event[id].clone(), event.clone()));
            self.event_name_map.remove(&self.event[id].name);
            self.event_time_map.get_mut(&self.event[id].time).unwrap().remove(&id);
            self.event_name_map.insert(event.name.clone(), event.id);
            self.event_time_map.entry(event.time)
                .and_modify(|x| {x.insert(id); })
                .or_insert(HashSet::from([id]));
            self.event[id] = event;
        }
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