use tui::layout::*;
use tui::widgets::*;
use tui::text::*;
use tui::style::*;
use unicode_width::UnicodeWidthStr;
use super::Switch;

pub struct EditView<'a> {
    quit: Option<Switch>,
    command: super::Command,
    project: crate::data::Project,
    information_window: String,
    data: &'a mut crate::data::Data,
}

fn event_to_string(event: &crate::data::Event, data: &crate::data::Data)
-> String {
    todo!()
}

fn project_to_string(project: &crate::data::Project, data: &crate::data::Data) -> String {
    let dependencies = project.dependencies.iter().map(|x| data.get_project_by_id(*x).unwrap().name)
        .fold(String::new(), |x, y| x + &y + " ");
    let deadline = project.deadline.map(|x| chrono::NaiveDateTime::from_timestamp_opt(crate::util::conv_utc_loc(x, data.tz), 0).unwrap().format("%Y/%m/%d %H:%M").to_string()).unwrap_or("None".to_string());
format!(
"name            {}
quota           {} {}
weight          {}
deadline        {}
dependencies    {}
parent          {}
state           {:?}
color           {:06x}"
, project.name, project.quota.0, project.quota.1, project.weight, deadline, dependencies, data.get_project_by_id(project.parent).unwrap().name, project.state, 256*(256*project.color.0 as u32+project.color.1 as u32)+project.color.2 as u32)
}

impl<'a> EditView<'a> {
    pub fn new(name: String, data: &'a mut crate::data::Data) -> Self {
        let project = data.get_project_by_name(&name);
        let information_window = if project.is_ok() { String::new() } else { format!("create new project {name}") };
        let project = project.unwrap_or(crate::data::Project::new(name.to_string()));
        Self {quit: None, project, command: super::Command::new(), information_window, data}
    }
    fn trigger_command(&mut self) {
        let args_string = self.command.get_command();
        let args = args_string.trim().split_whitespace().collect::<Vec<_>>();
        self.information_window.clear();
        match args.get(0).map(|x| x as &str) {
            Some("save") | Some("s") => {
                self.information_window.clear();
                match self.data.upsert_project(&self.project) {
                Ok(project) => {
                    self.project = project;
                    self.information_window.push_str("save ok");
                }
                Err(e) => {
                    self.information_window.push_str("save failed\n");
                    self.information_window.push_str(&format!("{e:?}"));
                }}
            }
            Some("color") | Some("c") if args.get(1).is_some() => {
                if let Ok(color) = u32::from_str_radix(args[1], 16) {
                    self.project.color = (
                        (color / (256*256) % 256) as u8, 
                        (color / 256 % 256) as u8, 
                        (color % 256) as u8
                    );
                }
            }
            Some("name") | Some("n") if args.get(1).is_some() => {
                if self.data.get_project_by_name(args[1]).is_err() {
                    self.project.name = args[1].to_string();
                } else {
                    self.information_window = format!("Project {} exists", args[1]);
                }
            }
            Some("weight") | Some("wei") | Some("w") 
                if args.get(1).is_some() && args[1].parse::<usize>().is_ok() 
            => {
                self.project.weight = args[1].parse().unwrap();
            }
            Some("parent") | Some("p") if args.get(1).is_some() 
            => {
                let parent = self.data.get_project_by_name(args[1]);
                if let Ok(parent) = parent {
                    self.project.parent = parent.id();
                }
            }
            Some("dependencies") | Some("dep") | Some("d") 
                if args.get(1).is_some() && args[1].starts_with("+") 
            => {
                let dependency = self.data.get_project_by_name(
                    args[1].strip_prefix("+").unwrap());
                if let Ok(dependency) = dependency {
                    self.project.dependencies.insert(dependency.id());
                }
            }
            Some("dependencies") | Some("dep") | Some("d") 
                if args.get(1).is_some() && args[1].starts_with("-") 
            => {
                let dependency = self.data.get_project_by_name(
                    args[1].strip_prefix("-").unwrap());
                if let Ok(dependency) = dependency {
                    self.project.dependencies.remove(&dependency.id());
                }
            }
            Some("quota") | Some("q") if args.get(1).is_some() && args[1].starts_with("+") 
            => {
                let delta = args[1].strip_prefix("+").unwrap().parse::<usize>();
                if let Ok(quota)= delta {
                    self.project.quota.1 += quota;
                }
            }
            Some("quota") | Some("q") if args.get(1).is_some() && args[1].starts_with("-") 
            => {
                let delta = args[1].strip_prefix("-").unwrap().parse::<usize>();
                if let Ok(quota) = delta {
                    self.project.quota.1 = 
                        self.project.quota.1.checked_sub(quota).unwrap_or(0);
                }
            }
            Some("quota") | Some("q") if args.get(1).is_some() => {
                let delta = args[1].parse::<usize>();
                if let Ok(quota) = delta {
                    self.project.quota.1 = quota;
                }
            }
            Some("abort") if args.get(1).is_some() && args[1] == self.project.name => {
                self.project.state = crate::data::State::Abort;
            },
            Some("finish") | Some("f") if args.get(1).is_some() => {
                self.project.quota.0 += args[1].parse::<usize>().unwrap_or(0);
                self.project.quota.1 = self.project.quota.1.max(self.project.quota.0);
            }
            Some("finish") | Some("f") if args.get(1).is_none() => {
                self.project.state = crate::data::State::Done;
            }
            Some("ddl") | Some("deadline") | Some("due") if matches!(args.get(1), Some(&"x") | Some(&"-")) => {
                self.project.deadline = None;
            }
            Some("ddl") | Some("deadline") | Some("due") if args.get(1).is_some() => {
                let mut date = args[1].split("/").filter_map(|x| x.parse::<i64>().ok());
                let (y, m, d) = (date.next().unwrap_or(0), date.next().unwrap_or(0), date.next().unwrap_or(0));
                let ts = if let Some(time) = args.get(2) {
                    let mut time = time.split(":").filter_map(|x| x.parse::<i64>().ok());
                    let (h, min) = (time.next().unwrap_or(0), date.next().unwrap_or(0));
                    crate::util::parse_year_month_date(y, m, d, h, min)
                } else {
                    crate::util::parse_year_month_date(y, m, d, 0, 0)
                };
                let ts = crate::util::conv_loc_utc(ts, self.data.tz);
                if ts > crate::util::utc_now() { self.project.deadline = Some(ts); }
                else { self.information_window.push_str("deadline before now") }
            }
            Some("i") | Some("info") if args.get(1).is_some() => {
                self.information_window = match self.data.get_project_by_name(args[1]) {
                    Ok(project) => project_to_string(&project, &self.data),
                    Err(e) => format!("{e:?}"),
                };
            }
            Some(any) => {
                self.information_window = any.to_string();
            },
            None => {
                self.information_window = "no such command\n".to_string() + &args_string;
            }
        }
    }
}

impl super::App for EditView<'_> {
    fn draw(&self, f: &mut tui::Frame<tui::backend::CrosstermBackend<std::io::Stdout>>) {
        let area = f.size();
        let _tmp = Layout::default().direction(Direction::Vertical)
            .constraints([Constraint::Length(4), Constraint::Min(4)]).split(area);
        let _tmp = Layout::default().direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(40)]).margin(1).split(_tmp[1]);
        let area_main = _tmp[0];
        let color = self.project.color.clone();
        let area_info = _tmp[1];
        let information_widget = Paragraph::new(Text::raw(self.information_window.clone())).style(Style::default()).block(Block::default().borders(Borders::all()).title(" Information "));
        let main_editor_content = project_to_string(&self.project, &self.data);
        let main_editor_widget = Paragraph::new(Text::from(
            main_editor_content.split('\n').map(Spans::from)
            .chain([Spans::from(vec![Span::styled(" ".repeat(22), Style::default().bg(Color::Rgb(color.0, color.1, color.2)))])])
            .collect::<Vec<_>>()
        )).block(Block::default().borders(Borders::all()).title(" Editing "));
        f.render_widget(information_widget, area_info);
        f.render_widget(main_editor_widget, area_main);
        self.command.draw(f, _tmp[0]);
    }
    fn on_key_code(&mut self, key_code: crossterm::event::KeyCode) {
        let trigger = self.command.on_key_code(key_code);
        if !trigger { return }
        match self.command.try_switch() {
            Some(q) => self.quit = Some(q),
            None => self.trigger_command()
        }
    }
    fn quit(&self) -> Option<super::Switch> {
        self.quit.clone() 
    }
}