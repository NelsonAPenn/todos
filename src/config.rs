extern crate toml;
extern crate serde;

use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub goal_color: String,
    pub condition_color: String,
    pub task_color: String
}

impl Config
{
    fn color_is_valid(color: &String) -> bool
    {
        true
    }

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
    let config_opt = fs::read_to_string(path).ok().map(|content| {
        let config: Config = toml::from_str(&content).unwrap();
        config
    });

    get_validated_config(config_opt)
}

fn get_validated_config(config_opt: Option<Config>) -> Config
{
    if let Some(config) = config_opt
    {
        if
            !Config::color_is_valid(&config.goal_color) ||
            !Config::color_is_valid(&config.condition_color) ||
            !Config::color_is_valid(&config.task_color)
        {
            Config::default()
        }
        else
        {
            config
        }
    }
    else
    {
        Config::default()
    }

}
