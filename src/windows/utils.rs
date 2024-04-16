use std::ffi::CStr;

use widestring::Utf16String;
use windows::{
    core::PCSTR,
    Win32::{
        Foundation::{HMODULE, MAX_PATH},
        System::{LibraryLoader::GetProcAddress, SystemInformation::GetSystemDirectoryW}
    }
};

pub fn get_system_directory() -> Utf16String {
    let mut buffer = [0u16; MAX_PATH as usize];
    let length = unsafe { GetSystemDirectoryW(Some(&mut buffer)) };
    unsafe { Utf16String::from_vec_unchecked(buffer[..length as usize].to_vec()) }
}

pub fn get_proc_address(hmodule: HMODULE, name: &CStr) -> usize {
    let res = unsafe { GetProcAddress(hmodule, PCSTR(name.as_ptr() as *const u8)) };
    if let Some(proc) = res {
        proc as usize
    }
    else {
        0
    }
}