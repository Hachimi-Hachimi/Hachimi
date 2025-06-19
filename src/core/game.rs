use std::path::PathBuf;

use crate::game_impl;

pub struct Game {
    pub package_name: String,
    pub region: Region,
    pub data_dir: PathBuf
}

#[derive(PartialEq, Eq)]
pub enum Region {
    Unknown,
    Japan,
    Taiwan,
    Korea,
    China
}

impl Game {
    pub fn init() -> Game {
        let package_name = game_impl::get_package_name();
        let region = game_impl::get_region(&package_name);
        let data_dir = game_impl::get_data_dir(&package_name);

        if region == Region::Unknown {
            warn!("Failed to detect game region")
        }

        Game {
            package_name,
            region,
            data_dir
        }
    }
}