use std::{path::Path, process};

use widestring::Utf16Str;
use windows::Win32::{Foundation::{HMODULE, MAX_PATH}, System::LibraryLoader::GetModuleFileNameW};

use crate::core::game::Region;

pub fn get_package_name() -> String {
    "".to_owned()
}

pub fn get_region(_package_name: &str) -> Region {
    Region::Japan
}

pub fn get_data_dir(_package_name: &str) -> String {
    let mut slice = [0u16; MAX_PATH as usize];
    let length = unsafe { GetModuleFileNameW(HMODULE::default(), &mut slice) } as usize;
    let exec_path_str = unsafe { Utf16Str::from_slice_unchecked(&slice[..length]) }.to_string();
    let exec_path = Path::new(&exec_path_str);

    if let Some(exec_dir_path) = exec_path.parent() {
        exec_dir_path.join("hachimi")
            .to_str()
            .expect("valid utf-8 path")
            .to_string()
    }
    else {
        error!("FATAL: Failed to get the directory of the current executable");
        process::exit(1);
    }
}