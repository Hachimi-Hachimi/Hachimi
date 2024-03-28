use crate::{core::{ext::StringExt, Hachimi}, il2cpp::{symbols::get_method_addr, types::*}};

type SetTextFn = extern "C" fn(this: *mut Il2CppObject, text: *mut Il2CppString);
extern "C" fn SetText(this: *mut Il2CppObject, text_: *mut Il2CppString) {
    let text = unsafe { (*text_).to_utf16str() };

    // doesn't run through TextGenerator, remove filters
    let new_text = if text.as_slice().contains(&36) { // 36 = dollar sign ($)
        Hachimi::instance().template_parser
            .remove_filters(&text.to_string())
            .to_il2cpp_string()
    }
    else {
        text_
    };
    
    get_orig_fn!(SetText, SetTextFn)(this, new_text)
}

pub fn init(Plugins: *const Il2CppImage) {
    get_class_or_return!(Plugins, AnimateToUnity, AnText);

    let SetText_addr = get_method_addr(AnText, cstr!("SetText"), 1);

    new_hook!(SetText_addr, SetText);
}