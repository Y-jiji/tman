use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Project {
    // project id
    pub(super) id: usize,
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
    // color
    pub color: (u8, u8, u8),
    // children ids
    pub(super) children: HashSet<usize>,
    // dependencies reversed
    pub(super) dependencies_reverse: HashSet<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogItem {
    ProjectMajorUpdate(Project, Project),
    ProjectMinorUpdate(Project, Project),
    ProjectInsert(Project),
    EventItemInsert(Event),
    EventItemUpdate(Event, Event),
}

impl Project {
    pub fn need_quota(&self) -> usize {
        self.quota.1 - self.quota.0
    }
    pub fn new(name: String) -> Project {
        Project {
            name,
            id: usize::MAX,
            limit: usize::MAX,
            ..Default::default()
        }
    }
    pub fn id(&self) -> usize {
        self.id
    }
}

impl Data {
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
    pub fn get_project_by_time_range(&self, time_range: (i64, i64)) -> Vec<usize> {
        self.project_date_map.range(time_range.0..time_range.1)
        .flat_map(|x| x.1.iter().map(|x| *x))
        .collect()
    }
    // insert or update a new project
    pub fn upsert_project(&mut self, mut new_project: Project) -> Result<(), DataError> {
        let mut id = new_project.id;
        if new_project.parent >= self.projects.len() {
            Err(DataError::InvalidId)?
        }
        if id >= self.projects.len() {
            id = self.projects.len();
            new_project.id = id;
            if new_project.state != State::Todo {
                Err(DataError::InvalidNewProject(
                    "creating a project which is not \"todo\"",
                ))?
            }
            self.log.push(LogItem::ProjectInsert(new_project.clone()));
            if self.project_name_map.contains_key(&new_project.name) {
                Err(DataError::IndexError("project name is not unique"))?
            }
            self.project_name_map.insert(new_project.name.clone(), id);
            if let Some(ddl) = new_project.deadline {
                self.project_date_map.entry(ddl)
                    .and_modify(|v| { v.insert(new_project.id()); })
                    .or_insert(HashSet::from([new_project.id()]));
            }
            new_project.children.clear();
            new_project.dependencies_reverse.clear();
            let parent = &mut self.projects[new_project.parent];
            new_project
                .dependencies
                .drain_filter(|x| !parent.children.contains(&x));
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
            if self.projects[id].state != State::Todo
                && new_project.state != self.projects[id].state
            {
                Err(DataError::InvalidStateChange)?
            }
            if self.projects[id].quota.0 > new_project.quota.0 {
                Err(DataError::InvalidQuotaChange(
                    "allocated quota should only increase",
                ))?
            }
            self.log.push(LogItem::ProjectMajorUpdate(
                self.projects[id].clone(),
                new_project.clone(),
            ));
            // update state recursively
            for c in self.recursive_children(id) {
                if matches!(self.projects[c].state, State::Todo) {
                    let old = self.projects[c].clone();
                    self.projects[c].state = new_project.state.clone();
                    self.log
                        .push(LogItem::ProjectMinorUpdate(old, self.projects[c].clone()));
                    self.project_name_map.remove(&self.projects[c].name);
                }
            }
            // get old project
            let old_project = self.projects[id].clone();
            // update project name index
            self.project_name_map.remove(&old_project.name);
            self.project_name_map.insert(new_project.name.clone(), id);
            // update project deadline index
            if let Some(ddl) = old_project.deadline {
                self.project_date_map.get_mut(&ddl).unwrap().remove(&id);
            }
            if let Some(ddl) = new_project.deadline {
                self.project_date_map.entry(ddl)
                    .and_modify(|v| { v.insert(new_project.id()); })
                    .or_insert(HashSet::from([new_project.id()]));
            }
            // remove old dependencies and quota
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
            new_project
                .dependencies
                .drain_filter(|x| !new_parent.children.contains(&x));
            for d in &new_project.dependencies {
                self.projects[*d].dependencies_reverse.remove(&d);
            }
            self.projects[id] = new_project;
            Ok(())
        }
    }
}
