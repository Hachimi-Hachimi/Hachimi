use std::{fmt::Display, path::PathBuf};

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
    China,
    Global
}

impl Display for Region {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Region::Unknown => "Unknown",
            Region::Japan => "Japan",
            Region::Taiwan => "Taiwan",
            Region::Korea => "Korea",
            Region::China => "China",
            Region::Global => "Global"
        })
    }
}

impl Game {
    pub fn init() -> Game {
        let package_name = game_impl::get_package_name();
        let region = game_impl::get_region(&package_name);
        let data_dir = game_impl::get_data_dir(&package_name);

        Game {
            package_name,
            region,
            data_dir
        }
    }
}