use crate::il2cpp::{symbols::{get_field_from_name, set_field_object_value}, types::*};

static mut _TEXT_FIELD: *mut FieldInfo = 0 as _;
pub fn set__text(this: *mut Il2CppObject, value: *mut Il2CppString) {
    set_field_object_value(this, unsafe { _TEXT_FIELD }, value);
}

pub fn init(Plugins: *const Il2CppImage) {
    get_class_or_return!(Plugins, AnimateToUnity, AnTextParameter);

    unsafe {
        _TEXT_FIELD = get_field_from_name(AnTextParameter, c"_text");
    }
}