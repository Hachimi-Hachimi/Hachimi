use std::{ffi::CStr, path::{Path, PathBuf}};

use widestring::{Utf16Str, Utf16String};
use windows::{
    core::PCSTR,
    Win32::{
        Foundation::{CloseHandle, HMODULE, MAX_PATH},
        System::{
            Diagnostics::ToolHelp::{CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPALL},
            LibraryLoader::{GetModuleFileNameW, GetProcAddress},
            SystemInformation::GetSystemDirectoryW,
            Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE}
        }
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

pub fn get_game_dir() -> Option<PathBuf> {
    let mut slice = [0u16; MAX_PATH as usize];
    let length = unsafe { GetModuleFileNameW(HMODULE::default(), &mut slice) } as usize;
    let exec_path_str = unsafe { Utf16Str::from_slice_unchecked(&slice[..length]) }.to_string();
    let exec_path = Path::new(&exec_path_str);
    let parent = exec_path.parent()?;

    Some(parent.to_owned())
}

pub fn get_game_dir_str() -> Option<String> {
    let mut slice = [0u16; MAX_PATH as usize];
    let length = unsafe { GetModuleFileNameW(HMODULE::default(), &mut slice) } as usize;
    let exec_path_str = unsafe { Utf16Str::from_slice_unchecked(&slice[..length]) }.to_string();
    let exec_path = Path::new(&exec_path_str);
    let parent = exec_path.parent()?;

    Some(parent.display().to_string())
}

pub unsafe fn kill_process_by_name(target_name: &CStr) -> Result<(), windows::core::Error> {
    let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPALL, 0)?;
    let mut entry = PROCESSENTRY32::default();
    entry.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;
    let mut res = Process32First(snapshot, &mut entry);

    while res.is_ok() {
        let process_name = CStr::from_ptr(entry.szExeFile.as_ptr());
        if process_name == target_name {
            if let Ok(process) = OpenProcess(PROCESS_TERMINATE, false, entry.th32ProcessID) {
                TerminateProcess(process, 0)?;
                CloseHandle(process)?;
            }
        }

        res = Process32Next(snapshot, &mut entry);
    }

    Ok(())
}

pub fn get_tmp_installer_path() -> PathBuf {
    let mut installer_path = std::env::temp_dir();
    installer_path.push("hachimi_installer.exe");
    installer_path
}