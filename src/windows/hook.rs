#![allow(non_snake_case)]

use windows::{core::{w, PCWSTR}, Win32::Foundation::HMODULE};

use crate::{core::{Error, Hachimi}, windows::hachimi_impl};

use super::{ffi, proxy, utils};

type LoadLibraryWFn = extern "C" fn(filename: PCWSTR) -> HMODULE;
extern "C" fn LoadLibraryW(filename: PCWSTR) -> HMODULE {
    let hachimi = Hachimi::instance();
    let orig_fn: LoadLibraryWFn = unsafe {
        std::mem::transmute(hachimi.interceptor.get_trampoline_addr(LoadLibraryW as usize))
    };

    let handle = orig_fn(filename);
    let filename_str = unsafe { filename.to_string().expect("valid utf-16 filename") };

    if hachimi_impl::is_criware_lib(&filename_str) {
        // Manually trigger a GameAssembly.dll load anyways since hachimi might have been loaded later
        hachimi.on_dlopen("GameAssembly.dll", orig_fn(w!("GameAssembly.dll")).0 as usize);
    }

    if hachimi.on_dlopen(&filename_str, handle.0 as usize) {
        hachimi.interceptor.unhook(LoadLibraryW as usize);
    }
    handle
}

fn init_internal() -> Result<(), Error> {
    let system_dir = utils::get_system_directory();

    info!("Init dxgi.dll proxy");
    proxy::dxgi::init(&system_dir)?;

    info!("Init version.dll proxy");
    proxy::version::init(&system_dir);

    info!("Init winhttp.dll proxy");
    proxy::winhttp::init(&system_dir);

    info!("Hooking LoadLibraryW");
    let hachimi = Hachimi::instance();
    hachimi.interceptor.hook(ffi::LoadLibraryW as usize, LoadLibraryW as usize)?;    
    Ok(())
}

pub fn init() {
    init_internal().unwrap_or_else(|e| {
        error!("Init failed: {}", e);
    });
}