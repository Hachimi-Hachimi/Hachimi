use std::{ffi::CStr, path::PathBuf};

use widestring::{U16CString, Utf16Str, Utf16String};
use windows::{
    core::{w, PCSTR, PCWSTR},
    Win32::{
        Foundation::{CloseHandle, HMODULE, HWND, MAX_PATH},
        System::{
            Diagnostics::ToolHelp::{CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPALL},
            LibraryLoader::{GetModuleFileNameW, GetProcAddress},
            SystemInformation::GetSystemDirectoryW,
            Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE}
        },
        UI::WindowsAndMessaging::{MessageBoxW, SetWindowPos, HWND_NOTOPMOST, HWND_TOPMOST, MB_ICONERROR, MB_OK, SWP_NOMOVE, SWP_NOSIZE}
    }
};

use crate::core::{utils::scale_to_aspect_ratio, Hachimi};

use super::hachimi_impl::ResolutionScaling;

pub fn _get_system_directory() -> Utf16String {
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

pub fn get_exec_path() -> PathBuf {
    let mut slice = [0u16; MAX_PATH as usize];
    let length = unsafe { GetModuleFileNameW(HMODULE::default(), &mut slice) } as usize;
    let exec_path_str = unsafe { Utf16Str::from_slice_unchecked(&slice[..length]) }.to_string();

    PathBuf::from(exec_path_str)
}

pub fn get_game_dir() -> PathBuf {
    let exec_path = get_exec_path();
    let parent = exec_path.parent().unwrap();
    parent.to_owned()
}

/*
pub fn get_game_dir_str() -> Option<String> {
    let mut slice = [0u16; MAX_PATH as usize];
    let length = unsafe { GetModuleFileNameW(HMODULE::default(), &mut slice) } as usize;
    let exec_path_str = unsafe { Utf16Str::from_slice_unchecked(&slice[..length]) }.to_string();
    let exec_path = Path::new(&exec_path_str);
    let parent = exec_path.parent()?;

    Some(parent.display().to_string())
}
*/

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

pub fn get_scaling_res() -> Option<(i32, i32)> {
    use crate::il2cpp::hook::UnityEngine_CoreModule::Screen as UnityScreen;
    use crate::il2cpp::hook::umamusume::Screen as GallopScreen;

    match Hachimi::instance().config.load().windows.resolution_scaling {
        ResolutionScaling::Default => None,
        ResolutionScaling::ScaleToScreenSize => {
            let res = UnityScreen::get_currentResolution(); // screen res, not game window res
            let aspect_ratio = GallopScreen::get_Width_orig() as f32 / GallopScreen::get_Height_orig() as f32;
            Some(scale_to_aspect_ratio((res.width, res.height), aspect_ratio, true))
        },
        ResolutionScaling::ScaleToWindowSize => {
            let mut width = UnityScreen::get_width();
            let mut height = UnityScreen::get_height();
            if (GallopScreen::get_Width_orig() > GallopScreen::get_Height_orig()) != (width > height) {
                std::mem::swap(&mut width, &mut height);
            }
            Some((width, height))
        },
    }
}

pub unsafe fn set_window_topmost(hwnd: HWND, topmost: bool) -> Result<(), windows::core::Error> {
    let insert_after = if topmost { HWND_TOPMOST } else { HWND_NOTOPMOST };
    SetWindowPos(hwnd, insert_after, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE)
}

pub fn show_error(e: impl AsRef<str>) {
    let s = e.as_ref();
    error!("{}", s);

    let cstr = U16CString::from_str(s).unwrap();
    unsafe { MessageBoxW(None, PCWSTR(cstr.as_ptr()), w!("Hachimi Error"), MB_ICONERROR | MB_OK); }
}