use crate::{
    core::Hachimi,
    il2cpp::{
        hook::UnityEngine_UI::Text,
        symbols::{get_field_from_name, get_field_object_value, get_method_addr},
        types::*
    }
};

use super::TextFrame;

static mut _TEXTFRAME_FIELD: *mut FieldInfo = 0 as _;
fn get__textFrame(this: *mut Il2CppObject) -> *mut Il2CppObject {
    get_field_object_value(this, unsafe { _TEXTFRAME_FIELD })
}

type SetFontSizeFn = extern "C" fn(this: *mut Il2CppObject, font_size: i32);
extern "C" fn SetFontSize(this: *mut Il2CppObject, font_size: i32) {
    get_orig_fn!(SetFontSize, SetFontSizeFn)(this, font_size);

    if let Some(mult) = Hachimi::instance().localized_data.load().config.text_frame_font_size_multiplier {
        let text_frame = get__textFrame(this);
        let text_label = TextFrame::get_TextLabel(text_frame);
        let font_size = Text::get_fontSize(text_label);
        Text::set_fontSize(text_label, (font_size as f32 * mult).round() as i32);
    }
}

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, StoryViewTextControllerLandscape);

    let SetFontSize_addr = get_method_addr(StoryViewTextControllerLandscape, c"SetFontSize", 1);

    new_hook!(SetFontSize_addr, SetFontSize);

    unsafe {
        _TEXTFRAME_FIELD = get_field_from_name(StoryViewTextControllerLandscape, c"_textFrame");
    }
}