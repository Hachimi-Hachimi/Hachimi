use std::sync::Mutex;

use crate::il2cpp::{symbols::{get_method_addr, GCHandle}, types::*};

static mut GET_ISFINISHED_ADDR: usize = 0;
impl_addr_wrapper_fn!(get_IsFinished, GET_ISFINISHED_ADDR, bool, this: *mut Il2CppObject);

static mut GET_TIMELINEDATA_ADDR: usize = 0;
impl_addr_wrapper_fn!(get_TimelineData, GET_TIMELINEDATA_ADDR, *mut Il2CppObject, this: *mut Il2CppObject);

pub static CURRENT: Mutex<Option<GCHandle>> = Mutex::new(None);

type GotoBlockFn = extern "C" fn(this: *mut Il2CppObject, block_id: i32, weaken_cy_spring: bool, is_update: bool, is_choice: bool);
extern "C" fn GotoBlock(this: *mut Il2CppObject, block_id: i32, weaken_cy_spring: bool, is_update: bool, is_choice: bool) {
    *CURRENT.lock().unwrap() = Some(GCHandle::new_weak_ref(this, false));
    get_orig_fn!(GotoBlock, GotoBlockFn)(this, block_id, weaken_cy_spring, is_update, is_choice);
}

pub fn GotoBlock_orig(this: *mut Il2CppObject, block_id: i32, weaken_cy_spring: bool, is_update: bool, is_choice: bool) {
    get_orig_fn!(GotoBlock, GotoBlockFn)(this, block_id, weaken_cy_spring, is_update, is_choice);
}

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, StoryTimelineController);

    let GotoBlock_addr = get_method_addr(StoryTimelineController, c"GotoBlock", 4);

    new_hook!(GotoBlock_addr, GotoBlock);

    unsafe {
        GET_ISFINISHED_ADDR = get_method_addr(StoryTimelineController, c"get_IsFinished", 0);
        GET_TIMELINEDATA_ADDR = get_method_addr(StoryTimelineController, c"get_TimelineData", 0);
    }
}