use std::{path::PathBuf, process};

use crate::core::game::Region;

use super::utils;

pub fn get_package_name() -> String {
    "".to_owned()
}

pub fn get_region(_package_name: &str) -> Region {
    Region::Japan
}

pub fn get_data_dir(_package_name: &str) -> PathBuf {
    if let Some(game_dir) = utils::get_game_dir() {
        game_dir.join("hachimi")
    }
    else {
        error!("FATAL: Failed to get the directory of the current executable");
        process::exit(1);
    }
}