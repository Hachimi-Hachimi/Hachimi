use std::os::raw::c_void;

use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{BOOL, HANDLE, HMODULE},
        Storage::FileSystem::{FINDEX_INFO_LEVELS, FINDEX_SEARCH_OPS, FIND_FIRST_EX_FLAGS, WIN32_FIND_DATAW}
    }
};

#[link(name = "kernel32")]
extern "C" {
    pub fn LoadLibraryW(filename: PCWSTR) -> HMODULE;
    pub fn FindFirstFileExW(
        filename: PCWSTR,
        info_level_id: FINDEX_INFO_LEVELS,
        ffd: *mut WIN32_FIND_DATAW,
        search_op: FINDEX_SEARCH_OPS,
        search_filter: *const c_void,
        additional_flags: FIND_FIRST_EX_FLAGS
    ) -> HANDLE;
    pub fn FindNextFileW(handle: HANDLE, ffd: *mut WIN32_FIND_DATAW) -> BOOL;
}