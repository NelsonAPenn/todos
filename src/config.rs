extern crate toml;
extern crate serde;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Config {
    pub goal_color: String,
    pub condition_color: String,
    pub task_color: String
}

impl Default for Config
{
    fn default() -> Config
    {
        Config
        {
            goal_color: String::from("01;94"),
            condition_color: String::from("01;33"),
            task_color: String::from("0;39")
        }
    }
}

pub fn read_config_file(path: PathBuf) -> Config {
    let config_opt = fs::read_to_string(&path).ok().map(|content| {
        let config: Option<Config> = toml::from_str(&content).ok();
        config
    }).flatten();

    if let Some(config) = config_opt
    {
        config
    }
    else
    {
        println!("\x1B[1;42m Invalid config file, providing defaults.\x1B[00m");
        let config = Config::default();
        config
    }
}
