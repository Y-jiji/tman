use crate::data::{Data, Timed, State};
use std::collections::{HashMap, HashSet};

pub struct Schedule {
    data: Data,
    this_day: i64,
    quota_each_day: usize,
}

impl Schedule {
    // return quota count
    pub fn step_day(&mut self) -> (Vec<Timed>, Vec<(usize, usize)>) {
        // query the timed items on that day
        let timed = self.data.get_timed_by_range((self.this_day, self.this_day+24*60*60));
        // compute consumed projects
        let consumed = timed.iter().map(|x| x.quota).fold(0, |x, y| x + y);
        let available = self.quota_each_day.max(consumed) - consumed;
        // schedule projects on that day, collect items
        let mut project_quota_count = HashMap::<usize, usize>::new();
        let mut ban = HashSet::<usize>::new(); // projects that reached the day limit
        for _ in 0..available {
            // select most urgent active project by aggregated urgency
            let active_projects = self.data.active_projects(&ban);
            if active_projects.is_empty() { break; }
            let aggregated_quota = self.data.aggregate_quota();
            let selected_project_id = active_projects.into_iter()
                .map(|i| {
                    // compute urgency for active projects
                    let u = self.data.recursive_reverse_dependencies_exclusive_and_same_for_parent(i)
                        .into_iter().fold(0f32, |u, i| {
                            let quota = aggregated_quota[i] as f32;
                            let ddl = match self.data.get_project_by_id(i).unwrap().deadline {
                                Some(ddl) => (ddl - self.this_day) as f32,
                                None => return u
                            };
                            u.max(quota / ddl)
                        });
                    (i, u)
                })
                .fold((0, -1.0), |(ix, ux), (iy, uy)| if ux < uy { (iy, uy) } else { (ix, ux) }).0;
            // add project quota to today's arrangement
            project_quota_count.entry(selected_project_id)
                .and_modify(|x| *x += 1)
                .or_insert(1);
            let mut selected_project = self.data.get_project_by_id(selected_project_id).unwrap();
            // FIXME: add limit to parent project, ban if parent project is banned
            if project_quota_count[&selected_project_id] == selected_project.limit {
                ban.insert(selected_project_id);
            }
            // update project quota
            selected_project.quota.0 += 1;
            if selected_project.quota.0 == selected_project.quota.1 {
                selected_project.state = State::Done;
            }
            self.data.upsert_project(selected_project).unwrap();
        }
        self.this_day += 24*60*60;
        (timed, project_quota_count.into_iter().collect())
    }
    // return the final schedule
    pub fn compute(mut self) -> Vec<(Vec<Timed>, Vec<(usize, usize)>)> {
        let mut out = vec![];
        loop {
            let next = self.step_day();
            if next.0.is_empty() && next.1.is_empty() { break out }
            else { out.push(next); }
        }
    }
}