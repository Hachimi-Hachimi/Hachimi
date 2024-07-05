use crate::il2cpp::{symbols::{get_field_from_name, get_field_object_value}, types::*};

// List<AnTextParameter>
static mut _TEXTPARAMLIST_FIELD: *mut FieldInfo = 0 as _;
pub fn get__textParamList(this: *mut Il2CppObject) -> *mut Il2CppObject {
    get_field_object_value(this, unsafe { _TEXTPARAMLIST_FIELD })
}

pub fn init(Plugins: *const Il2CppImage) {
    get_class_or_return!(Plugins, AnimateToUnity, AnMotionParameter);

    unsafe {
        _TEXTPARAMLIST_FIELD = get_field_from_name(AnMotionParameter, cstr!("_textParamList"));
    }
}