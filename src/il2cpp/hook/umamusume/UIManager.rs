use crate::{core::{ext::StringExt, Hachimi}, il2cpp::{symbols::get_method_overload_addr, types::*}};

type SetHeaderTitleTextFn = extern "C" fn(this: *mut Il2CppObject, text: *mut Il2CppString, guide_id: i32);
extern "C" fn SetHeaderTitleText(this: *mut Il2CppObject, text_: *mut Il2CppString, guide_id: i32) {
    let text = unsafe { (*text_).to_utf16str() };

    // The title text (aka the purple ribbon on the top left of the screen) doesn't run
    // through TextGenerator, so we have to evaluate templates here (by emptying any filter exprs)
    let new_text = if text.as_slice().contains(&36) { // 36 = dollar sign ($)
        Hachimi::instance().template_parser
            .remove_filters(&text.to_string())
            .to_il2cpp_string()
    }
    else {
        text_
    };

    get_orig_fn!(SetHeaderTitleText, SetHeaderTitleTextFn)(this, new_text, guide_id)
}

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, UIManager);

    let SetHeaderTitleText_addr = get_method_overload_addr(UIManager, "SetHeaderTitleText",
        &[Il2CppTypeEnum_IL2CPP_TYPE_STRING, Il2CppTypeEnum_IL2CPP_TYPE_VALUETYPE]);
    
    new_hook!(SetHeaderTitleText_addr, SetHeaderTitleText);
}