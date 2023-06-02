use crate::data::{Data, Event, State};
use std::collections::{HashMap, HashSet};

pub struct Schedule {
    pub data: Data,
    pub this_day: i64,
    pub quota_each_day: usize,
}

impl Schedule {
    // return quota count
    pub fn step_day(&mut self) -> (Vec<Event>, Vec<(usize, usize)>) {
        // query the timed items on that day
        let timed = self
            .data
            .get_event_by_range((self.this_day, self.this_day + 24 * 60 * 60));
        // compute consumed projects
        let consumed = timed.iter().map(|x| x.quota).fold(0, |x, y| x + y);
        let available = self.quota_each_day.max(consumed) - consumed;
        // schedule projects on that day, collect items
        let mut project_quota_count = HashMap::<usize, usize>::new();
        let mut ban = HashSet::<usize>::new(); // projects that reached the day limit
        for _ in 0..available {
            // select most urgent active project by aggregated urgency
            let active_projects = self.data.active_projects(&ban);
            if active_projects.is_empty() {
                break;
            }
            let aggregated_quota = self.data.aggregate_quota();
            let selected_project_id = active_projects
                .into_iter()
                .map(|i| {
                    // compute urgency for active projects
                    let u = self.data
                        .recursive_reverse_dependencies_exclusive_and_same_for_parent(i)
                        .into_iter()
                        .fold(0f32, |u, i| {
                            let quota = aggregated_quota[i] as f32;
                            let ddl = match self.data.get_project_by_id(i).unwrap().deadline {
                                Some(ddl) => (ddl - self.this_day) as f32,
                                None => return u,
                            };
                            u.max(quota / ddl)
                        });
                    (i, u)
                }).fold((0, -1.0), |(ix, ux), (iy, uy)| if ux < uy { (iy, uy) } else { (ix, ux) }).0;
            // add project quota to today's arrangement
            project_quota_count
                .entry(selected_project_id)
                .and_modify(|x| *x += 1)
                .or_insert(1);
            let mut selected_project = self.data.get_project_by_id(selected_project_id).unwrap();
            // FIXME: 将quota count也累加到父节点上, 当父节点的quota count超过限制时, 将整个子树ban掉
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
        self.this_day += 24 * 60 * 60;
        (timed, project_quota_count.into_iter().collect())
    }
    // return the final schedule
    pub fn compute(mut self) -> Vec<(Vec<Event>, Vec<(usize, usize)>)> {
        let mut out = vec![];
        loop {
            let next = self.step_day();
            if next.0.is_empty() && next.1.is_empty() {
                break out;
            } else {
                out.push(next);
            }
        }
    }
}

impl Data {
    // generate auto schedule
    pub fn generate_auto_schedule(&self) -> Vec<(Vec<Event>, Vec<(usize, usize)>)> {
        // 将数据复制一份, 自动生成日程的过程中需要修改project的状态
        let data = self.clone();
        let this_day = crate::util::utc_now() / (24*60*60) * (24*60*60);
        let quota_each_day = data.projects[0].limit;
        // 之后考虑用compact的方式复制而不是全复制
        return Schedule { data, this_day, quota_each_day }.compute()
    }
    // get active projects recursively
    fn active_projects(&self, ban: &HashSet<usize>) -> Vec<usize> {
        // 筛选当前正在活动的项目
        (0..self.projects.len()).filter(
            |i| self.projects[*i].state == State::Todo && 
                self.projects[*i].dependencies.iter().all(|x| self.projects[*x].state != State::Todo) &&
                self.projects[*i].children.iter().all(|x| self.projects[*x].state != State::Todo) &&
                !ban.contains(&i)
            ).collect()
    }
    // aggregate quota from children and dependencies
    fn aggregate_quota(&self) -> Vec<usize> {
        // 先把每棵子树的quota聚到根上作为子项目自己的quota
        let self_quota = (0..self.projects.len())
            .map(|x| {
                if self.projects[x].state == State::Todo {
                    self.projects[x].quota.1 - self.projects[x].quota.0
                } else {
                    0
                }
            })
            .collect::<Vec<_>>();
        let self_quota = (0..self.projects.len())
            .map(|x| {
                self.recursive_children(x)
                    .into_iter()
                    .fold(0, |a, b| self_quota[a] + self_quota[b])
            })
            .collect::<Vec<_>>();
        // 累加兄弟节点中前驱的quota, 复杂度 O(NW), 通常是前驱是非常稀疏的一个两个
        let mut deps_quota = (0..self.projects.len())
            .map(|x| {
                self.recursive_dependencies(x)
                    .into_iter()
                    .filter(|i| *i != x)
                    .fold(0, |a, b| self_quota[a] + self_quota[b])
            })
            .collect::<Vec<_>>();
        // 将子树根的前驱累加到子树的每个元素的前驱上, 由于recursive children是后序的,
        // 此处要计算树的前序和, 因此要倒过来才行, 复杂度 O(N)
        for i in self.recursive_children(0).into_iter().rev() {
            for &c in self.projects[i].children.iter() {
                deps_quota[c] += deps_quota[i];
            }
        }
        // 最后将每个子项目自己的quota和前驱的quota加起来, 得到最后的结果
        // 总的时间复杂度O(NW), N是总的节点数, W是最大的树宽, 通常达不到上界
        return (0..self.projects.len())
            .map(|i| deps_quota[i] + self_quota[i])
            .collect::<Vec<_>>();
    }
}
