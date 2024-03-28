use std::{os::raw::c_void, process, sync::RwLock};

use fnv::FnvHashMap;

use super::Error;

#[derive(Default)]
pub struct Interceptor {
    hook_map: RwLock<FnvHashMap<usize, HookHandle>>
}

struct HookHandle {
    orig_addr: usize,
    trampoline_addr: usize
}

impl Interceptor {
    pub fn hook(&self, orig_addr: usize, hook_addr: usize) -> Result<usize, Error> {
        let trampoline_addr = unsafe {
            dobby_rs::hook(orig_addr as *mut c_void, hook_addr as *mut c_void)? as usize
        };

        self.hook_map.write().unwrap().insert(
            hook_addr,
            HookHandle {
                orig_addr,
                trampoline_addr
            }
        );
        Ok(trampoline_addr)
    }

    pub fn get_trampoline_addr(&self, hook_addr: usize) -> usize {
        self.hook_map.read().unwrap().get(&hook_addr).unwrap_or_else(|| {
            error!("FATAL: Attempted to get invalid hook");
            process::exit(1);
        }).trampoline_addr
    }

    pub fn unhook(&self, hook_addr: usize) {
        if let Some(hook) = self.hook_map.write().unwrap().remove(&hook_addr) {
            if let Err(e) = unsafe { dobby_rs::unhook(hook.orig_addr as *mut c_void) } {
                error!("Failed to unhook {}: {}", hook.orig_addr, e);
            }
        }
    }
}

macro_rules! get_orig_fn {
    ($hook:ident, $type:tt) => (
        unsafe { std::mem::transmute::<usize, $type>(crate::core::Hachimi::instance().interceptor.get_trampoline_addr($hook as usize)) }
    )
}