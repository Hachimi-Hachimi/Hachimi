pub mod ChoiceData;
pub mod ColorTextInfo;

use std::ptr::null_mut;

use crate::il2cpp::{symbols::{get_field_from_name, get_field_object_value, get_field_value, set_field_object_value}, types::*};

pub const FontSize_Default: i32 = 0;
pub const FontSize_Large: i32 = 1;
pub const FontSize_Small: i32 = 2;
pub const FontSize_BoldCaption: i32 = 3;

static mut CLASS: *mut Il2CppClass = null_mut();
pub fn class() -> *mut Il2CppClass {
    unsafe { CLASS }
}

static mut NAME_FIELD: *mut FieldInfo = null_mut();
pub fn set_Name(this: *mut Il2CppObject, value: *mut Il2CppString) {
    set_field_object_value(this, unsafe { NAME_FIELD }, value);
}

pub fn get_Name(this: *mut Il2CppObject) -> *mut Il2CppString {
    get_field_object_value(this, unsafe { NAME_FIELD })
}

static mut TEXT_FIELD: *mut FieldInfo = null_mut();
pub fn get_Text(this: *mut Il2CppObject) -> *mut Il2CppString {
    get_field_object_value(this, unsafe { TEXT_FIELD })
}

pub fn set_Text(this: *mut Il2CppObject, value: *mut Il2CppString) {
    set_field_object_value(this, unsafe { TEXT_FIELD }, value);
}

static mut SIZE_FIELD: *mut FieldInfo = null_mut();
/// StoryTimelineTextClipData.FontSize
pub fn get_Size(this: *mut Il2CppObject) -> i32 {
    get_field_value(this, unsafe { SIZE_FIELD })
}

static mut CHOICEDDATALIST_FIELD: *mut FieldInfo = null_mut();
pub fn get_ChoiceDataList(this: *mut Il2CppObject) -> *mut Il2CppObject {
    get_field_object_value(this, unsafe { CHOICEDDATALIST_FIELD })
}

static mut COLORTEXTINFOLIST_FIELD: *mut FieldInfo = null_mut();
pub fn get_ColorTextInfoList(this: *mut Il2CppObject) -> *mut Il2CppObject {
    get_field_object_value(this, unsafe { COLORTEXTINFOLIST_FIELD })
}

static mut WAITFRAME_FIELD: *mut FieldInfo = null_mut();
pub fn get_WaitFrame(this: *mut Il2CppObject) -> i32 {
    get_field_value(this, unsafe { WAITFRAME_FIELD })
}

static mut VOICELENGTH_FIELD: *mut FieldInfo = null_mut();
pub fn get_VoiceLength(this: *mut Il2CppObject) -> i32 {
    get_field_value(this, unsafe { VOICELENGTH_FIELD })
}

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, StoryTimelineTextClipData);

    unsafe {
        CLASS = StoryTimelineTextClipData;
        NAME_FIELD = get_field_from_name(StoryTimelineTextClipData, c"Name");
        TEXT_FIELD = get_field_from_name(StoryTimelineTextClipData, c"Text");
        SIZE_FIELD = get_field_from_name(StoryTimelineTextClipData, c"Size");
        CHOICEDDATALIST_FIELD = get_field_from_name(StoryTimelineTextClipData, c"ChoiceDataList");
        COLORTEXTINFOLIST_FIELD = get_field_from_name(StoryTimelineTextClipData, c"ColorTextInfoList");
        WAITFRAME_FIELD = get_field_from_name(StoryTimelineTextClipData, c"WaitFrame");
        VOICELENGTH_FIELD = get_field_from_name(StoryTimelineTextClipData, c"VoiceLength");
    }

    ChoiceData::init(StoryTimelineTextClipData);
    ColorTextInfo::init(StoryTimelineTextClipData);
}