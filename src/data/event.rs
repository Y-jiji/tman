use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub(super) id: usize,
    pub name: String,
    pub time: i64,
    pub state: State,
    pub quota: usize,
}

impl Data {
    // get tasks in time range
    pub fn get_event_by_range(&self, utc_range: (i64, i64)) -> Vec<Event> {
        self.event_time_map
            .range(utc_range.0..utc_range.1)
            .into_iter()
            .flat_map(|x| x.1.iter())
            .map(|x| self.event[*x].clone())
            .collect()
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
            self.event_time_map
                .entry(event.time)
                .and_modify(|x| {
                    x.insert(id);
                })
                .or_insert(HashSet::from([id]));
            self.event.push(event);
        } else {
            if self.event[id].name != event.name && self.event_name_map.contains_key(&event.name) {
                Err(DataError::IndexError("event item name is not unique"))?
            }
            self.log.push(LogItem::EventItemUpdate(
                self.event[id].clone(),
                event.clone(),
            ));
            self.event_name_map.remove(&self.event[id].name);
            self.event_time_map
                .get_mut(&self.event[id].time)
                .unwrap()
                .remove(&id);
            self.event_name_map.insert(event.name.clone(), event.id);
            self.event_time_map
                .entry(event.time)
                .and_modify(|x| {
                    x.insert(id);
                })
                .or_insert(HashSet::from([id]));
            self.event[id] = event;
        }
        Ok(())
    }
}
