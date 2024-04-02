use crate::{core::{ext::StringExt, Hachimi}, il2cpp::{symbols::get_method_addr, types::*}};

type GetCaptionTextFn = extern "C" fn(this: *mut Il2CppObject, info: *mut Il2CppObject) -> *mut Il2CppString;
extern "C" fn GetCaptionText(this: *mut Il2CppObject, info: *mut Il2CppObject) -> *mut Il2CppString {
    let text = get_orig_fn!(GetCaptionText, GetCaptionTextFn)(this, info);
    let text_utf16 = unsafe { (*text).to_utf16str() };

    // doesn't run through TextGenerator, remove filters
    if text_utf16.as_slice().contains(&36) { // 36 = dollar sign ($)
        Hachimi::instance().template_parser
            .remove_filters(&text_utf16.to_string())
            .to_il2cpp_string()
    }
    else {
        text
    }
}

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, TrainingParamChangeA2U);

    let GetCaptionText_addr = get_method_addr(TrainingParamChangeA2U, cstr!("GetCaptionText"), 1);

    new_hook!(GetCaptionText_addr, GetCaptionText);
}