use std::os::raw::{c_ulong, c_void};

use widestring::U16CString;
use windows::{core::PCWSTR, Win32::{Foundation::{BOOL, HMODULE, TRUE}, System::LibraryLoader::LoadLibraryW}};

use crate::{core::{hachimi::Plugin, Hachimi}, windows::utils};

use super::{hook, wnd_hook};

const DLL_PROCESS_ATTACH: c_ulong = 1;
const DLL_PROCESS_DETACH: c_ulong = 0;

pub fn load_libraries() -> Vec<Plugin> {
    let mut plugins = Vec::new();
    for name in Hachimi::instance().config.load().windows.load_libraries.iter() {
        let Ok(name_cstr) = U16CString::from_str(name) else {
            warn!("Invalid library name: {}", name);
            continue;
        };
        let res = unsafe { LoadLibraryW(PCWSTR(name_cstr.as_ptr())) };

        if let Ok(handle) = res {
            if !handle.is_invalid() {
                info!("Loaded library: {}", name);

                let hachimi_init_addr = utils::get_proc_address(handle, c"hachimi_init");
                if hachimi_init_addr != 0 {
                    plugins.push(Plugin {
                        name: name.clone(),
                        init_fn: unsafe { std::mem::transmute(hachimi_init_addr) }
                    });
                }

                continue;
            }
        }

        warn!("Failed to load library: {}", name);
    }

    plugins
}

pub static mut DLL_HMODULE: HMODULE = HMODULE(0 as _);

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn DllMain(hmodule: HMODULE, call_reason: c_ulong, _reserved: *mut c_void) -> BOOL {
    if call_reason == DLL_PROCESS_ATTACH {
        unsafe { DLL_HMODULE = hmodule; }
        if !Hachimi::init() {
            return TRUE;
        }

        let hachimi = Hachimi::instance();
        *hachimi.plugins.lock().unwrap() = load_libraries();

        hook::init();
        info!("Attach completed");
    }
    else if call_reason == DLL_PROCESS_DETACH && Hachimi::is_initialized() {
        wnd_hook::uninit();

        info!("Unhooking everything");
        Hachimi::instance().interceptor.unhook_all();
    }
    TRUE
}