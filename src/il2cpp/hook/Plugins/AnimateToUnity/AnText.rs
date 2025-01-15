use std::ptr::null_mut;

use crate::{
    core::Hachimi,
    il2cpp::{
        ext::{Il2CppStringExt, StringExt}, hook::UnityEngine_TextRenderingModule::TextGenerator::IgnoreTGFiltersContext, symbols::{get_field_from_name, get_field_object_value, get_method_addr, set_field_object_value}, types::*
    }
};

static mut TEXT_FIELD: *mut FieldInfo = null_mut();
fn get__text(this: *mut Il2CppObject) -> *mut Il2CppString {
    get_field_object_value(this, unsafe { TEXT_FIELD })
}

fn set__text(this: *mut Il2CppObject, value: *mut Il2CppString) {
    set_field_object_value(this, unsafe { TEXT_FIELD }, value);
}

type _UpdateTextFn = extern "C" fn(this: *mut Il2CppObject);
extern "C" fn _UpdateText(this: *mut Il2CppObject) {
    let text_ptr = get__text(this);
    if text_ptr.is_null() {
        return get_orig_fn!(_UpdateText, _UpdateTextFn)(this);
    }

    let text = unsafe { (*text_ptr).as_utf16str() };

    // doesn't run through TextGenerator, ignore its filters
    if text.as_slice().contains(&36) { // 36 = dollar sign ($)
        set__text(this, Hachimi::instance().template_parser
            .eval_with_context(&text.to_string(), &mut IgnoreTGFiltersContext())
            .to_il2cpp_string());
    }
    
    get_orig_fn!(_UpdateText, _UpdateTextFn)(this);
}

pub fn init(Plugins: *const Il2CppImage) {
    get_class_or_return!(Plugins, AnimateToUnity, AnText);

    let _UpdateText_addr = get_method_addr(AnText, c"_UpdateText", 0);

    new_hook!(_UpdateText_addr, _UpdateText);

    unsafe {
        TEXT_FIELD = get_field_from_name(AnText, c"_text");
    }
}