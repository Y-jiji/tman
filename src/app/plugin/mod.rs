//! command line plugins
use serde::*;

pub trait Plugin {
    // add name
    fn name(&self) -> String;
    // extend prompts with this plugin
    fn ext_prompts(&self, db: &crate::DataBase, prompts: &mut String);
    // try to execute a command, if this command is matched by this plugin, return true
    fn try_execute(&mut self, db: &mut crate::DataBase, command: &str) -> bool;
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PluginOpt {

}