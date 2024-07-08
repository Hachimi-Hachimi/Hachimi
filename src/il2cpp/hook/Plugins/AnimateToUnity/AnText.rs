use std::ptr::null_mut;

use crate::{
    core::{ext::StringExt, Hachimi},
    il2cpp::{
        hook::{
            Plugins::AnimateToUnity::AnTextParameter,
            UnityEngine_TextRenderingModule::TextGenerator::IgnoreTGFiltersContext
        },
        symbols::{get_field_from_name, get_field_object_value, get_method_addr, set_field_object_value},
        types::*
    }
};

use super::AnRoot;

static mut TEXT_FIELD: *mut FieldInfo = null_mut();
fn get__text(this: *mut Il2CppObject) -> *mut Il2CppString {
    get_field_object_value(this, unsafe { TEXT_FIELD })
}

fn set__text(this: *mut Il2CppObject, value: *mut Il2CppString) {
    set_field_object_value(this, unsafe { TEXT_FIELD }, value);
}

static mut TEXTPARAM_FIELD: *mut FieldInfo = null_mut();
fn get__textParam(this: *mut Il2CppObject) -> *mut Il2CppObject {
    get_field_object_value(this, unsafe { TEXTPARAM_FIELD })
}

type _UpdateTextFn = extern "C" fn(this: *mut Il2CppObject);
extern "C" fn _UpdateText(this: *mut Il2CppObject) { // _UpdateText
    let text_param = get__textParam(this);
    if !text_param.is_null() && AnRoot::is_text_param_overridden(text_param) {
        set__text(this, AnTextParameter::get__text(text_param));
        return get_orig_fn!(_UpdateText, _UpdateTextFn)(this);
    }

    let text_ptr = get__text(this);
    if text_ptr.is_null() {
        return get_orig_fn!(_UpdateText, _UpdateTextFn)(this);
    }

    let text = unsafe { (*text_ptr).to_utf16str() };

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
        TEXTPARAM_FIELD = get_field_from_name(AnText, c"_textParam");
    }
}