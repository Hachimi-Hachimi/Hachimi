#![allow(non_snake_case)]

use windows::{core::w, Win32::System::LibraryLoader::LoadLibraryW};

use crate::core::{Error, Hachimi};

use super::proxy;

fn init_internal() -> Result<(), Error> {
    info!("Init cri_mana_vpx.dll proxy");
    proxy::cri_mana_vpx::init();

    let hachimi = Hachimi::instance();
    hachimi.on_dlopen("GameAssembly.dll", unsafe { LoadLibraryW(w!("GameAssembly.dll")).unwrap().0 as usize });
    hachimi.on_hooking_finished();

    Ok(())
}

pub fn init() {
    init_internal().unwrap_or_else(|e| {
        error!("Init failed: {}", e);
    });
}