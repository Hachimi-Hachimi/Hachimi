use std::ptr::null_mut;

use crate::il2cpp::{
    ext::Il2CppObjectExt, symbols::{get_field_from_name, get_field_object_value, get_field_value, set_field_value, IList}, types::*
};

use super::{StoryTimelineTextClipData, StoryTimelineTrackData};

// StoryTimelineTextTrackData (derived class of StoryTimelineTrackData)
static mut TEXTTRACK_FIELD: *mut FieldInfo = null_mut();
pub fn get_TextTrack(this: *mut Il2CppObject) -> *mut Il2CppObject {
    get_field_object_value(this, unsafe { TEXTTRACK_FIELD })
}

static mut BLOCKLENGTH_FIELD: *mut FieldInfo = null_mut();
pub fn get_BlockLength(this: *mut Il2CppObject) -> i32 {
    get_field_value(this, unsafe { BLOCKLENGTH_FIELD })
}

pub fn set_BlockLength(this: *mut Il2CppObject, value: i32) {
    set_field_value(this, unsafe { BLOCKLENGTH_FIELD }, &value)
}

// List<StoryTimelineCharaTrackData>
static mut CHARACTERTRACKLIST_FIELD: *mut FieldInfo = null_mut();
pub fn get_CharacterTrackList(this: *mut Il2CppObject) -> *mut Il2CppObject {
    get_field_object_value(this, unsafe { CHARACTERTRACKLIST_FIELD })
}

// List<StoryTimelineScreenEffectTrackData>
static mut SCREENEFFECTTRACKLIST_FIELD: *mut FieldInfo = null_mut();
pub fn get_ScreenEffectTrackList(this: *mut Il2CppObject) -> *mut Il2CppObject {
    get_field_object_value(this, unsafe { SCREENEFFECTTRACKLIST_FIELD })
}

// Specialization
pub fn get_text_clip(this: *mut Il2CppObject) -> Option<*mut Il2CppObject> {
    let text_track = get_TextTrack(this);
    if text_track.is_null() {
        return None;
    }

    let clip_list = <IList>::new(StoryTimelineTrackData::get_ClipList(text_track))?;
    // There should be a single text clip per track
    let clip_data = clip_list.get(0)?;

    let class = unsafe { (*clip_data).klass() };
    if class != StoryTimelineTextClipData::class() {
        return None;
    }

    Some(clip_data)
}

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, StoryTimelineBlockData);

    unsafe {
        TEXTTRACK_FIELD = get_field_from_name(StoryTimelineBlockData, c"TextTrack");
        BLOCKLENGTH_FIELD = get_field_from_name(StoryTimelineBlockData, c"BlockLength");
        CHARACTERTRACKLIST_FIELD = get_field_from_name(StoryTimelineBlockData, c"CharacterTrackList");
        SCREENEFFECTTRACKLIST_FIELD = get_field_from_name(StoryTimelineBlockData, c"ScreenEffectTrackList");
    }
}