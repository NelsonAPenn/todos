extern crate dirs;
use std::path::PathBuf;

pub fn get_home_directory() -> PathBuf
{
    dirs::home_dir().unwrap()

}