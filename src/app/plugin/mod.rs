//! command line plugins
use serde::*;
mod pj_editor;
mod ev_editor;

pub trait Plugin {
    // get name
    fn name(&self) -> String;
    // extend prompts with this plugin
    fn ext_prompts(&self, db: &crate::DataBase, prompts: &mut String);
    // try to execute a command, if this command is matched by this plugin, return true
    fn try_execute(&mut self, db: &mut crate::DataBase, command: &Vec<&str>) -> Result<bool, String>;
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PluginOpt {
    Null,
}

impl Plugin for PluginOpt {
    // get name
    fn name(&self) -> String {
        use PluginOpt::*;
        match self {
            Null => format!("null"),
        }
    }
    // extend prompts
    fn ext_prompts(&self, db: &crate::DataBase, prompts: &mut String) {
        use PluginOpt::*;
        match self {
            Null => {},
        }
    }
    // try execute a command
    fn try_execute(&mut self, db: &mut crate::DataBase, command: &Vec<&str>) -> Result<bool, String> {
        use PluginOpt::*;
        match self {
            Null => Ok(false),
        }
    }
}