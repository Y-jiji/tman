use std::collections::BTreeMap;
use btreemultimap::BTreeMultiMap;

const PATH: &str = "C:/ProgramData/TMan/";

#[derive(Debug, serde::Deserialize, serde::Serialize, Default)]
pub struct Data {
    count: usize,
    tasks: BTreeMap<usize, Task>,
    childrens: BTreeMultiMap<usize, usize>,
    naming_map: BTreeMap<String, usize>,
    ddl_sorted: BTreeMultiMap<i64, usize>,
    dead: Vec<usize>,
    categories: BTreeMap<usize, Category>,
}

impl Data {
    pub fn load() -> Self {
        let s = std::fs::read_to_string(format!("{PATH}/data-latest.json")).unwrap_or(String::new());
        serde_json::from_str(&s).unwrap_or(Self::new())
    }
    pub fn save(&self) {
        let s = serde_json::to_string(&self).unwrap();
        let now = crate::util::utc_now();
        std::fs::copy(format!("{PATH}/data-latest.json"), format!("{PATH}/date-{now}.json")).unwrap_or(0);
        std::fs::write(format!("{PATH}/data-latest.json"), s).unwrap();
    }
    pub fn prune(&mut self, now: i64) {
        let mut dead = self.ddl_sorted.range(..now).map(|x| *x.1).chain(self.dead.drain(..)).collect::<Vec<_>>();
        loop {
            if dead.is_empty() { break; }
            let mut new_dead = vec![];
            for tid in dead.drain(..) {
                let task = match self.tasks.remove(&tid) { Some(x) => x, None => continue };
                new_dead.extend(self.childrens.get_vec(&tid).map(|x| x.clone()).unwrap_or(vec![]));
                self.ddl_sorted.get_vec_mut(&task.time.1).map(|x| x.drain_filter(|x| *x == tid));
                self.naming_map.remove(&task.name);
            }
            dead = new_dead;
        }
    }
    pub fn add_task(&mut self, task: Task) {
        self.childrens.insert(task.pid, task.tid);
        self.ddl_sorted.insert(task.time.1, task.tid);
        self.naming_map.insert(task.name.clone(), task.tid);
        self.tasks.insert(task.tid, task);
    }
    pub fn add_category(&mut self, category: Category) {
        self.categories.insert(category.cid, category);
    }
    pub fn remove_category(&mut self, cid: usize) {
        self.categories.remove(&cid);
        let mut dead = self.tasks.drain_filter(|_, v| v.cid == cid).map(|x| x.1.tid).collect::<Vec<_>>();
        loop {
            if dead.is_empty() { break; }
            let mut new_dead = vec![];
            for tid in dead.drain(..) {
                let task = match self.tasks.remove(&tid) { Some(x) => x, None => continue };
                new_dead.extend(self.childrens.get_vec(&tid).map(|x| x.clone()).unwrap_or(vec![]));
                self.ddl_sorted.get_vec_mut(&task.time.1).map(|x| x.drain_filter(|x| *x == tid));
                self.naming_map.remove(&task.name);
            }
            dead = new_dead;
        }
    }
    pub fn finish_task(&mut self, tid: usize) {
        self.tasks.get_mut(&tid).unwrap().done = true;
        self.dead.push(tid);
    }
    pub fn finish_category(&mut self, cid: usize) {
    }
    fn new() -> Self {
        Self { count: 1, ..Default::default() }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Task {
    pub tid: usize,
    pub pid: usize,
    pub cid: usize,
    pub time: (i64, i64),
    pub cost: i64,
    pub name: String,
    pub note: String,
    pub done: bool,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Category {
    pub cid: usize,
    pub gen: (i64, i64, i64, i64), // (start, period, quota, expiration)
    pub name: String,
    pub note: String,
}

impl Category {
    pub fn quota(&self, now: i64) -> i64 {
        let (sta, per, quo, exp) = self.gen;
        if exp == 0 {
            (quo * ((now - sta) / per)).max(0)
        } else {
            (quo * ((now - sta) / per)).max(0) - 
            (quo * ((now - sta - exp) / per)).max(0)
        }
    }
}
