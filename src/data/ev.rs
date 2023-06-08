use std::collections::{HashMap, BTreeMap};
use serde::*;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Ev {
    id: usize,
    pp: usize,
    name: String,
    time: i64,
    quota_esti: usize,
    color: (u8, u8, u8)
}

impl Ev {
    pub fn id(&self) -> usize {
        self.id
    }
    pub fn pp(&self) -> usize {
        self.pp
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn quota_esti(&self) -> usize {
        self.quota_esti
    }
    pub fn time(&self) -> i64 {
        self.time
    }
    pub fn color_usize(&self) -> usize {
        (self.color.0 as usize * 256 * 256) +
        (self.color.1 as usize * 256) +
        (self.color.2 as usize)
    }
    pub fn color_rgb(&self) -> tui::style::Color {
        tui::style::Color::Rgb(self.color.0, self.color.1, self.color.2)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EvStore {
    vect: Vec<Option<Ev>>,
    time: BTreeMap<i64, usize>,
    name: HashMap<String, usize>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum EvErr {
    TimeClash,
    NameClash,
    ParentNotExist,
    InvalidEventId,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum EvLog {
    Create{id: usize},
    Delete{old: Ev},
    Parent{id: usize, old: usize},
    Name{id: usize, old: String},
    Time{id: usize, old: i64},
    QuotaEsti{id: usize, old: usize},
}

impl EvStore {
    pub fn new() -> Self {
        Self { vect: Vec::new(), time: BTreeMap::new(), name: HashMap::new() }
    }
    pub fn create(&mut self, mut ev: Ev, pj_store: &mut super::pj::PjStore) 
    -> Result<(EvLog, super::pj::PjLog), EvErr> {
        if self.time.contains_key(&ev.time) {
            Err(EvErr::TimeClash)
        } else if self.name.contains_key(&ev.name) {
            Err(EvErr::NameClash)
        } else {
            ev.id = self.vect.len();
            match pj_store.add_event(ev.pp, ev.id) {
                Ok(log) => {
                    let id = ev.id;
                    self.vect.push(Some(ev));
                    Ok((EvLog::Create { id }, log))
                },
                Err(_) => {
                    Err(EvErr::ParentNotExist)
                }
            }
        }
    }
    pub fn get_by_name(&self, name: &str) -> Option<Ev> {
        self.name.get(name).map(|x| self.vect[*x].clone()).unwrap_or(None)
    }
    pub fn get_by_id(&self, id: usize) -> Option<Ev> {
        self.vect.get(id).map(|x| x.clone()).unwrap_or(None)
    }
    pub fn delete(&mut self, id: usize, pj_store: &mut super::pj::PjStore) -> Result<EvLog, EvErr> {
        self.check_exists(id)?;
        let old = &mut self.vect[id];
        pj_store.rmv_event(old.as_ref().unwrap().pp, id).unwrap();
        self.time.remove(&old.as_ref().unwrap().time);
        self.name.remove(&old.as_ref().unwrap().name);
        Ok(EvLog::Delete { old: std::mem::replace(old, None).unwrap() })
    }
    pub fn check_exists(&self, id: usize) -> Result<(), EvErr> {
        if id >= self.vect.len() || self.vect[id].is_none() {
            Err(EvErr::InvalidEventId)
        } else {
            Ok(())
        }
    }
    pub fn get_all_names(&self) -> Result<Vec<String>, EvErr> {
        Ok(self.name.iter().map(|(name, _)| name.clone()).collect())
    }
    pub fn update_quota_esti(&mut self, id: usize, quota: usize) 
    -> Result<EvLog, EvErr> {
        self.check_exists(id)?;
        let old = self.vect[id].as_ref().unwrap().quota_esti;
        self.vect[id].as_mut().unwrap().quota_esti = quota;
        Ok(EvLog::QuotaEsti { id, old })
    }
    pub fn update_pp(&mut self, id: usize, pp: usize, pj_store: &mut super::pj::PjStore)
    -> Result<EvLog, EvErr> {
        self.check_exists(id)?;
        let pp_new = pp;
        let pp_old = self.vect[id].as_ref().unwrap().pp;
        if let Err(_) = pj_store.add_event(pp_new, id) {
            Err(EvErr::ParentNotExist)?
        }
        pj_store.rmv_event(pp_old, id).unwrap();
        self.vect[id].as_mut().unwrap().pp = pp_new;
        Ok(EvLog::Parent { id, old: pp_old })
    }
    pub fn update_name(&mut self, id: usize, name: String)
    -> Result<EvLog, EvErr> {
        self.check_exists(id)?;
        if self.name.contains_key(&name) {
            Err(EvErr::NameClash)
        } else {
            let old = self.vect[id].as_ref().unwrap().name.clone();
            self.name.remove(&old);
            self.name.insert(name.clone(), id);
            self.vect[id].as_mut().unwrap().name = name.clone();
            Ok(EvLog::Name { id, old })
        }
    }
    pub fn update_time(&mut self, id: usize, time: i64)
    -> Result<EvLog, EvErr> {
        self.check_exists(id)?;
        if self.time.contains_key(&time) {
            Err(EvErr::TimeClash)
        } else {
            let old = self.vect[id].as_ref().unwrap().time;
            self.time.remove(&old);
            self.time.insert(time, id);
            self.vect[id].as_mut().unwrap().time = time;
            Ok(EvLog::Time { id, old })
        }
    }
}