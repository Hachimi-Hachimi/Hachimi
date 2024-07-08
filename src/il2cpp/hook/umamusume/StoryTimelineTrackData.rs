use std::ptr::null_mut;

use crate::il2cpp::{symbols::{get_field_from_name, get_field_object_value}, types::*};

// List<StoryTimelineClipData>
static mut CLIPLIST_FIELD: *mut FieldInfo = null_mut();
pub fn get_ClipList(this: *mut Il2CppObject) -> *mut Il2CppObject {
    get_field_object_value(this, unsafe { CLIPLIST_FIELD })
}

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, StoryTimelineTrackData);

    unsafe {
        CLIPLIST_FIELD = get_field_from_name(StoryTimelineTrackData, c"ClipList");
    }
}