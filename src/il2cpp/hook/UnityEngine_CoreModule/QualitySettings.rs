use std::sync::atomic;

use crate::{core::Hachimi, il2cpp::{api::il2cpp_resolve_icall, types::Il2CppImage}};

type SetVSyncCountFn = extern "C" fn(value: i32);
pub extern "C" fn set_vSyncCount(mut value: i32) {
    let vsync_count = Hachimi::instance().vsync_count.load(atomic::Ordering::Relaxed);
    if vsync_count != -1 {
        value = vsync_count;
    }
    get_orig_fn!(set_vSyncCount, SetVSyncCountFn)(value);
}

pub fn init(_UnityEngine_CoreModule: *const Il2CppImage) {
    let set_vSyncCount_addr = il2cpp_resolve_icall(
        cstr!("UnityEngine.QualitySettings::set_vSyncCount(System.Int32)").as_ptr()
    );

    new_hook!(set_vSyncCount_addr, set_vSyncCount);
}