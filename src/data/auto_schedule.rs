use std::collections::VecDeque;
use super::State;

#[derive(Debug, Clone)]
struct TreeNode {
    id: usize,
    ch: Vec<TreeNode>,
    rest_quota: usize,
    weight: usize,
}

impl TreeNode {
    // allocate quota to everyday project
    fn allocate(&self, mut today_quota: usize) -> Vec<(usize, usize)> {
        assert!(today_quota <= self.rest_quota);
        if self.ch.is_empty() { return vec![(self.id, today_quota)] }
        let mut weight_sum = self.ch.iter().fold(0, |x, y| x + y.weight);
        let mut output = vec![];
        for ch in self.ch.iter() {
            if ch.rest_quota == 0 || ch.weight == 0 { continue }
            let allocated = if ch.rest_quota * weight_sum <= ch.weight * today_quota {
                ch.rest_quota
            } else {
                ch.weight * today_quota / weight_sum
            };
            output.extend(ch.allocate(allocated));
            weight_sum -= ch.weight;
            today_quota -= allocated;
        }
        assert!(today_quota == 0 && weight_sum == 0);
        return output;
    }
}

impl super::Data {
    fn create_activity_tree(
        &self, id: usize, count: &Vec<usize>, 
        rest_quota: &Vec<usize>, now: i64
    ) -> TreeNode {
        let mut ch = self.projects[id].children.iter()
            .filter(|&&id| count[id] == 0 && 
                self.projects[id].deadline.map(|d| d >= now).unwrap_or(true))
            .map(|&id| self.create_activity_tree(id, &count, &rest_quota, now))
            .filter(|tree| tree.rest_quota != 0 && tree.weight != 0)
            .collect::<Vec<_>>();
        // sort children nodes so that once today_quota is not adequate
        ch.sort_by(|b, a| (a.weight * b.rest_quota).cmp(&(b.weight * a.rest_quota)));
        let rest_quota = ch.iter().fold(rest_quota[id], |x, y| x + y.rest_quota);
        let weight = self.projects[id].weight;
        TreeNode { id, rest_quota, ch, weight }
    }
    pub fn auto_schedule(&self, day_sta: i64) -> (i64, VecDeque<Vec<(usize, usize)>>) {
        // quota each day
        let quota_each_day = self.projects[0].weight;
        // get the day before deadline
        let day_end = match self.project_date_map.last_key_value() {
            Some((due, _)) => (*due - day_sta)/(24*60*60)*(24*60*60) + day_sta,
            None => return (day_sta, VecDeque::new()),
        };
        // count project reverse dependencies
        let mut count = self.projects.iter().map(
            |p| p.dependencies_reverse.len() + 
            if p.parent != 0 { 1 } else { 0 }
        ).collect::<Vec<_>>();
        // iterate and collect project dependencies
        let deps = self.projects.iter().map(|p| {
            let mut dep = p.children.clone();
            dep.extend(p.dependencies.iter().map(|x| *x));
            return dep;
        }).collect::<Vec<_>>();
        // iterate and collect rest quota of each project
        let mut rest_quota = self.projects.iter().map(|p| 
            if p.state == State::Todo {
                p.quota.1 - p.quota.0
            } else { 0 }
        ).collect::<Vec<_>>();
        // select project with zero reverse dependency count
        // create an active project tree
        let mut day_now = day_end;
        let mut activity_tree = self.create_activity_tree(0, &count, &rest_quota, day_now);
        // the final schedule
        let mut output = VecDeque::new();
        while day_now >= day_sta {
            // allocate time along current tree
            let mut today_quota = quota_each_day - self.get_event_by_range((day_now, day_now+24*60*60)).into_iter().fold(0, |x, y| x + y.quota).min(quota_each_day);
            let mut today_schedule = vec![];
            // allocate time along current tree
            while today_quota != 0 && activity_tree.rest_quota != 0 {
                let new_schedule = if today_quota <= activity_tree.rest_quota {
                    activity_tree.allocate(std::mem::replace(&mut today_quota, 0))
                } else {
                    today_quota -= activity_tree.rest_quota;
                    activity_tree.allocate(activity_tree.rest_quota)
                };
                for &(id, quota) in new_schedule.iter() {
                    rest_quota[id] -= quota;
                    if rest_quota[id] == 0 {
                        for &d in deps[id].iter() { count[d] -= 1; }
                    }
                }
                today_schedule.extend(new_schedule);
                activity_tree = self.create_activity_tree(0, &count, &rest_quota, day_now);
            }
            // compute today schedule
            output.push_front(today_schedule);
            day_now -= 24*60*60;
            // build new activity tree from new data
            activity_tree = self.create_activity_tree(0, &count, &rest_quota, day_now); 
        }
        return (day_now.min(day_sta), output);
    }
}