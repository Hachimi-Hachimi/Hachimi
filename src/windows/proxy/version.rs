#![allow(non_snake_case, non_upper_case_globals)]

use widestring::{U16CString, Utf16Str};
use windows::{core::PCWSTR, Win32::System::LibraryLoader::LoadLibraryW};

use crate::windows::utils;

proxy_proc!(GetFileVersionInfoA, GetFileVersionInfoA_orig);
proxy_proc!(GetFileVersionInfoExA, GetFileVersionInfoExA_orig);
proxy_proc!(GetFileVersionInfoExW, GetFileVersionInfoExW_orig);
proxy_proc!(GetFileVersionInfoSizeA, GetFileVersionInfoSizeA_orig);
proxy_proc!(GetFileVersionInfoSizeExA, GetFileVersionInfoSizeExA_orig);
proxy_proc!(GetFileVersionInfoSizeExW, GetFileVersionInfoSizeExW_orig);
proxy_proc!(GetFileVersionInfoSizeW, GetFileVersionInfoSizeW_orig);
proxy_proc!(GetFileVersionInfoW, GetFileVersionInfoW_orig);
proxy_proc!(VerFindFileA, VerFindFileA_orig);
proxy_proc!(VerFindFileW, VerFindFileW_orig);
proxy_proc!(VerInstallFileA, VerInstallFileA_orig);
proxy_proc!(VerInstallFileW, VerInstallFileW_orig);
proxy_proc!(VerLanguageNameA, VerLanguageNameA_orig);
proxy_proc!(VerLanguageNameW, VerLanguageNameW_orig);
proxy_proc!(VerQueryValueA, VerQueryValueA_orig);
proxy_proc!(VerQueryValueW, VerQueryValueW_orig);

pub fn init(system_dir: &Utf16Str) {
    unsafe {
        let dll_path = system_dir.to_owned() + "\\version.dll";
        let dll_path_cstr = U16CString::from_vec(dll_path.into_vec()).unwrap();
        let handle = LoadLibraryW(PCWSTR(dll_path_cstr.as_ptr())).expect("version.dll");

        GetFileVersionInfoA_orig = utils::get_proc_address(handle, c"GetFileVersionInfoA");
        GetFileVersionInfoExA_orig = utils::get_proc_address(handle, c"GetFileVersionInfoExA");
        GetFileVersionInfoExW_orig = utils::get_proc_address(handle, c"GetFileVersionInfoExW");
        GetFileVersionInfoSizeA_orig = utils::get_proc_address(handle, c"GetFileVersionInfoSizeA");
        GetFileVersionInfoSizeExA_orig = utils::get_proc_address(handle, c"GetFileVersionInfoSizeExA");
        GetFileVersionInfoSizeExW_orig = utils::get_proc_address(handle, c"GetFileVersionInfoSizeExW");
        GetFileVersionInfoSizeW_orig = utils::get_proc_address(handle, c"GetFileVersionInfoSizeW");
        GetFileVersionInfoW_orig = utils::get_proc_address(handle, c"GetFileVersionInfoW");
        VerFindFileA_orig = utils::get_proc_address(handle, c"VerFindFileA");
        VerFindFileW_orig = utils::get_proc_address(handle, c"VerFindFileW");
        VerInstallFileA_orig = utils::get_proc_address(handle, c"VerInstallFileA");
        VerInstallFileW_orig = utils::get_proc_address(handle, c"VerInstallFileW");
        VerLanguageNameA_orig = utils::get_proc_address(handle, c"VerLanguageNameA");
        VerLanguageNameW_orig = utils::get_proc_address(handle, c"VerLanguageNameW");
        VerQueryValueA_orig = utils::get_proc_address(handle, c"VerQueryValueA");
        VerQueryValueW_orig = utils::get_proc_address(handle, c"VerQueryValueW");
    }
}