use std::path::{Path, PathBuf};

use crate::core::game::Region;

use super::utils;

pub fn get_package_name() -> String {
    utils::get_exec_path()
        .to_str()
        .unwrap()
        .to_owned()
}

pub fn get_region(package_name: &str) -> Region {
    match Path::new(package_name)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_ascii_lowercase()
        .as_str()
    {
        "umamusume.exe" | "umamusumeprettyderby_jpn.exe" => Region::Japan,
        "umamusumeprettyderby.exe" => Region::Global,
        _ => Region::Unknown
    }
}

pub fn get_data_dir(package_name: &str) -> PathBuf {
    Path::new(package_name)
        .parent()
        .unwrap()
        .join("hachimi")
}

pub fn is_steam_release(package_name: &str) -> bool {
    let exec_path = Path::new(package_name);
    let exec_dir = exec_path.parent().unwrap();

    let mut data_dir_name = exec_path.file_stem().unwrap().to_owned();
    data_dir_name.push("_Data");

    let mut steam_api_dll = exec_dir.join(data_dir_name);
    steam_api_dll.push("Plugins");
    steam_api_dll.push("x86_64");
    steam_api_dll.push("steam_api64.dll");

    steam_api_dll.metadata().is_ok_and(|m| m.is_file())
}