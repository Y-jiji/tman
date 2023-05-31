use crate::data::{Data, Project};

pub struct Schedule {
    data: Data,
    now: usize,
    quota_available: usize,
}

pub enum ProjectOrTimedItem {
    P(crate::data::Project),
    T(crate::data::Timed),
}

impl Schedule {
    // return quota count and project
    pub fn next(&mut self, now: usize) -> Vec<(ProjectOrTimedItem, usize)> {
    }
}