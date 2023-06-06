use std::collections::{HashSet, HashMap, BTreeMap};
use serde::*;

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Pj {
    // project id
    id: usize,
    // parent project
    pp: usize, 
    // color of this project
    color: (u8, u8, u8),
    // children projects
    chpj: HashSet<usize>,
    // children events
    chev: HashSet<usize>,
    // project name
    name: String,
    // dependencies
    deps: HashSet<usize>,
    // dependencies reversed
    deps_rvs: HashSet<usize>,
    // project deadline
    deadline: Option<i64>,
    // spended quota, estimated quota
    quota_done: usize,
    quota_esti: usize,
    // weight
    weight: usize,
    // weight type
    weight_type: WeightType,
}

impl Pj {
    pub fn new(name: String) -> Self {
        Pj { id: 0usize, pp: 0usize, name, ..Default::default() }
    }
    pub fn id(&self) -> usize {
        self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub enum WeightType {
    Flexible,
    #[default]
    Permanent,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PjStore {
    vect: Vec<Option<Pj>>,
    name: HashMap<String, usize>,
    time: BTreeMap<i64, HashSet<usize>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum PjLog {
    Create{id: usize},
    Delete{pj: Pj},
    QuotaEsti{id: usize, old: usize},
    QuotaDone{id: usize, old_done: usize, old_esti: usize},
    Deadline{id: usize, old: Option<i64>},
    DepsAdd{id: usize, add: usize},
    DepsRmv{id: usize, rmv: usize},
    Name{id: usize, old: String},
    Parent{id: usize, old: usize},
    Weight{id: usize, old: usize},
    WeightType{id: usize, old: WeightType},
    Compact{idmap: Vec<usize>, length: usize},    // id to original position
    AddEvent{id: usize, ev: usize},
    RmvEvent{id: usize, ev: usize},
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum PjErr {
    NameOfRootCannotBeModified,
    NameNotDistinct,
    DependencyNotInPeer,
    BannedByPeerDependency,
    BannedByChildren,
    NewProjectButPeerDependency,
    NewProjectButChildren,
    InvalidProjectId,
}

impl PjStore {
    pub fn new() -> Self {
        Self { 
            vect: vec![Some(Pj{id: 0, pp: 0, name: "root".to_string(), .. Default::default()})], 
            time: BTreeMap::new(),
            name: HashMap::from([("root".to_string(), 0)])
        }
    }
    pub fn create(&mut self, mut pj: Pj) -> Result<PjLog, PjErr> {
        if !pj.deps.is_empty() || !pj.deps_rvs.is_empty() {
            Err(PjErr::NewProjectButPeerDependency)?
        } else if !pj.chev.is_empty() || !pj.chpj.is_empty() {
            Err(PjErr::NewProjectButChildren)?
        } else if pj.pp >= self.vect.len() {
            Err(PjErr::InvalidProjectId)?
        } else {
            let id = self.vect.len();
            pj.id = id;
            let pp = self.vect[pj.pp].as_mut().unwrap();
            pp.chpj.insert(id);
            self.vect.push(Some(pj));
            Ok(PjLog::Create { id })
        }
    }
    pub fn check_exists(&self, id: usize) -> Result<(), PjErr> {
        if id >= self.vect.len() || self.vect[id].is_none() {
            Err(PjErr::InvalidProjectId)
        } else {
            Ok(())
        }
    }
    pub fn get_all_names(&self) -> Result<Vec<String>, PjErr> {
        Ok(self.name.iter().map(|(name, _)| name.clone()).collect())
    }
    pub fn get_by_name(&self, name: &String) -> Option<Pj> {
        self.name.get(name).map(|x| self.vect[*x].clone()).unwrap_or(None)
    }
    pub fn get_by_id(&self, id: usize) -> Option<Pj> {
        self.vect.get(id).map(|x| x.clone()).unwrap_or(None)
    }
    pub fn delete(&mut self, id: usize) -> Result<PjLog, PjErr> {
        self.check_exists(id)?;
        let pj = self.vect[id].as_ref().unwrap();
        if !pj.deps.is_empty() || !pj.deps_rvs.is_empty() {
            Err(PjErr::BannedByPeerDependency)
        } else if !pj.chev.is_empty() || !pj.chpj.is_empty() {
            Err(PjErr::BannedByChildren)
        } else {
            let pj = std::mem::replace(&mut self.vect[id], None).expect("already deleted");
            Ok(PjLog::Delete { pj })
        }
    }
    pub fn add_event(&mut self, id: usize, ev: usize) -> Result<PjLog, PjErr> {
        self.check_exists(id)?;
        self.vect[id].as_mut().unwrap().chev.insert(ev);
        Ok(PjLog::AddEvent { id, ev })
    }
    pub fn rmv_event(&mut self, id: usize, ev: usize) -> Result<PjLog, PjErr> {
        self.check_exists(id)?;
        self.vect[id].as_mut().unwrap().chev.remove(&ev);
        Ok(PjLog::RmvEvent { id, ev })
    }
    pub fn update_weight(&mut self, id: usize, weight: usize) -> Result<PjLog, PjErr> {
        self.check_exists(id)?;
        let old = self.vect[id].as_ref().unwrap().weight;
        self.vect[id].as_mut().unwrap().weight = weight;
        Ok(PjLog::Weight { id, old })
    }
    pub fn update_weight_type(&mut self, id: usize, weight_type: WeightType) -> Result<PjLog, PjErr> {
        self.check_exists(id)?;
        let old = self.vect[id].as_ref().unwrap().weight_type.clone();
        self.vect[id].as_mut().unwrap().weight_type = weight_type;
        Ok(PjLog::WeightType { id, old })
    }
    pub fn update_pp(&mut self, id: usize, pp: usize) -> Result<PjLog, PjErr> {
        self.check_exists(id)?;
        let old = self.vect[id].as_ref().unwrap();
        if old.pp != pp && (!old.deps.is_empty() || !old.deps_rvs.is_empty()) {
            Err(PjErr::BannedByPeerDependency)
        } else if pp >= self.vect.len() || self.vect[pp].is_none() {
            Err(PjErr::InvalidProjectId)
        } else {
            let old = std::mem::replace(&mut self.vect[id].as_mut().unwrap().pp, pp);
            self.vect[old].as_mut().unwrap().chpj.remove(&id);
            self.vect[pp].as_mut().unwrap().chpj.insert(id);
            Ok(PjLog::Parent { id, old })
        }
    }
    pub fn update_name(&mut self, id: usize, name: String) -> Result<PjLog, PjErr> {
        self.check_exists(id)?;
        if &self.vect[id].as_ref().unwrap().name != &name && 
            self.name.contains_key(&name) {
            Err(PjErr::NameNotDistinct)
        } else if id == 0 {
            Err(PjErr::NameOfRootCannotBeModified) 
        } else {
            let old = std::mem::replace(
                &mut self.vect[id].as_mut().unwrap().name, name.clone());
            self.name.remove(&old);
            self.name.insert(name, id);
            Ok(PjLog::Name { id, old })
        }
    }
    pub fn add_deps(&mut self, id: usize, dep: usize) -> Result<PjLog, PjErr> {
        // fix me: add loop detection
        self.check_exists(id)?;
        let pp = self.vect[id].as_ref().unwrap().pp;
        let peers = &self.vect[pp].as_ref().unwrap().chpj;
        if !peers.contains(&dep) {
            Err(PjErr::DependencyNotInPeer)
        } else {
            self.vect[id].as_mut().unwrap().deps.insert(dep);
            self.vect[dep].as_mut().unwrap().deps_rvs.insert(id);
            Ok(PjLog::DepsAdd { id, add: dep })
        }
    }
    pub fn rmv_deps(&mut self, id: usize, dep: usize) -> Result<PjLog, PjErr> {
        self.check_exists(id)?;
        self.check_exists(dep)?;
        self.vect[id].as_mut().unwrap().deps.remove(&dep);
        self.vect[dep].as_mut().unwrap().deps_rvs.remove(&id);
        Ok(PjLog::DepsRmv { id, rmv: dep })
    }
    pub fn update_deadline(&mut self, id: usize, new: Option<i64>)
    -> Result<PjLog, PjErr> {
        self.check_exists(id)?;
        let old = std::mem::replace(&mut self.vect[id].as_mut().unwrap().deadline, new);
        if let Some(deadline) = old {
            self.time.get_mut(&deadline)
                .unwrap().remove(&id);
        }
        if let Some(deadline) = new {
            self.time.entry(deadline)
                .and_modify(|ent| {ent.insert(id);})
                .or_insert(HashSet::from([id]));
        }
        Ok(PjLog::Deadline { id, old })
    }
    pub fn update_quota_esti(&mut self, id: usize, quota: usize)
    -> Result<PjLog, PjErr> {
        self.check_exists(id)?;
        let old = std::mem::replace(&mut self.vect[id].as_mut().unwrap().quota_esti, quota);
        Ok(PjLog::QuotaEsti { id, old })
    }
    pub fn update_quota_done(&mut self, id: usize, quota: usize) 
    -> Result<PjLog, PjErr> {
        self.check_exists(id)?;
        let old_done = std::mem::replace(&mut self.vect[id].as_mut().unwrap().quota_done, quota);
        let old_esti = self.vect[id].as_ref().unwrap().quota_esti;
        if quota > self.vect[id].as_ref().unwrap().quota_esti {
            self.vect[id].as_mut().unwrap().quota_esti = quota;
        }
        Ok(PjLog::QuotaDone { id, old_done, old_esti })
    }
    pub fn undo(&mut self, log: PjLog) {
        match log {
            PjLog::AddEvent { id, ev } => {
                self.vect[id].as_mut().unwrap().chev.remove(&ev);
            }
            PjLog::RmvEvent { id, ev } => {
                self.vect[id].as_mut().unwrap().chev.insert(ev);
            }
            PjLog::Compact{idmap, length} => {
                let mut pjs = vec![None::<Pj>; length];
                for i in 0..self.vect.len() {
                    pjs[idmap[i]] = self.vect[i].clone(); 
                }
                let name = HashMap::from_iter(pjs.iter().filter_map(|pj| match pj {
                    None => None,
                    Some(pj) => Some((pj.name.clone(), pj.id.clone()))
                }));
                let mut deadline = BTreeMap::<i64, HashSet<usize>>::new();
                for pj in pjs.iter() {
                    if let Some(pj) = pj {
                        if let Some(pj_deadline) = pj.deadline {
                            deadline.entry(pj_deadline)
                                .and_modify(|ent| {ent.insert(pj.id); })
                                .or_insert(HashSet::from([pj.id]));
                        }
                    }
                }
                self.vect = pjs;
                self.name = name;
                self.time = deadline;
            }
            PjLog::Weight { id, old } => {
                self.vect[id].as_mut().unwrap().weight = old;
            }
            PjLog::WeightType { id, old } => {
                self.vect[id].as_mut().unwrap().weight_type = old;
            }
            PjLog::Create { id } => {
                if id == self.vect.len() - 1 {
                    self.delete(id).unwrap();
                    self.vect.pop();
                } else { unreachable!() }
            }
            PjLog::Delete { pj } => {
                self.vect[pj.id] = Some(pj.clone());
                let pp = pj.pp;
                self.vect[pp].as_mut().unwrap().chpj.insert(pj.id);
                for dep in pj.deps {
                    self.vect[dep].as_mut().unwrap().deps_rvs.insert(pj.id);
                }
                for dep in pj.deps_rvs {
                    self.vect[dep].as_mut().unwrap().deps.insert(pj.id);
                }
            }
            PjLog::QuotaEsti { id, old } => {
                self.vect[id].as_mut().unwrap().quota_esti = old;
            }
            PjLog::QuotaDone { id, old_done, old_esti } => {
                self.vect[id].as_mut().unwrap().quota_done = old_done;
                self.vect[id].as_mut().unwrap().quota_esti = old_esti;
            }
            PjLog::Deadline { id, old } => {
                let new = self.vect[id].as_ref().unwrap().deadline;
                if let Some(new) = new {
                    self.time.get_mut(&new)
                        .unwrap().remove(&id);
                }
                if let Some(old) = old {
                    self.time.entry(old)
                        .and_modify(|ent| { ent.insert(id); })
                        .or_insert(HashSet::from([id]));
                }
            }
            PjLog::DepsAdd { id, add } => {
                self.vect[id].as_mut().unwrap().deps.remove(&add);
                self.vect[add].as_mut().unwrap().deps_rvs.remove(&id);
            }
            PjLog::DepsRmv { id, rmv } => {
                self.vect[id].as_mut().unwrap().deps.insert(rmv);
                self.vect[rmv].as_mut().unwrap().deps_rvs.insert(id);
            }
            PjLog::Parent { id, old } => {
                let new = self.vect[id].as_ref().unwrap().pp;
                self.vect[id].as_mut().unwrap().pp = old;
                self.vect[new].as_mut().unwrap().chpj.remove(&id);
                self.vect[old].as_mut().unwrap().chpj.insert(id);
            }
            PjLog::Name { id, old } => {
                let new = &self.vect[id].as_ref().unwrap().name;
                self.name.remove(new);
                self.name.insert(old.clone(), id);
                self.vect[id].as_mut().unwrap().name = old;
            }
        }
    }
}