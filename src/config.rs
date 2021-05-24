extern crate toml;
extern crate serde;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Config {
    #[serde(skip_serializing, skip_deserializing)]
    pub config_path: PathBuf,

    pub goal_color: String,
    pub condition_color: String,
    pub task_color: String
}

impl Config
{
    pub fn save(&self)
    {
        use std::io::Write;

        let mut f = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(&self.config_path)
        .unwrap();

        write!(f, "{}", toml::to_string_pretty(&self).unwrap()).unwrap();
    }
}

impl Default for Config
{
    fn default() -> Config
    {
        Config
        {
            // TODO: is there a way to fix this?
            config_path: PathBuf::new(),

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

    if let Some(mut config) = config_opt
    {
        config.config_path = path;
        config
    }
    else
    {
        println!("\x1B[1;42m Invalid config file, providing defaults.\x1B[00m");
        let mut config = Config::default();
        config.config_path = path;
        config
    }
}
