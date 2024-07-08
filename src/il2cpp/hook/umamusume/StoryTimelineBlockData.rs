use std::ptr::null_mut;

use crate::il2cpp::{symbols::{get_field_from_name, get_field_object_value}, types::*};

// StoryTimelineTextTrackData (derived class of StoryTimelineTrackData)
static mut TEXTTRACK_FIELD: *mut FieldInfo = null_mut();
pub fn get_TextTrack(this: *mut Il2CppObject) -> *mut Il2CppObject {
    get_field_object_value(this, unsafe { TEXTTRACK_FIELD })
}

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, StoryTimelineBlockData);

    unsafe {
        TEXTTRACK_FIELD = get_field_from_name(StoryTimelineBlockData, c"TextTrack");
    }
}