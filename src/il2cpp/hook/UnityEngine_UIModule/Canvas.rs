use crate::il2cpp::{api::{il2cpp_class_get_type, il2cpp_type_get_object}, types::*};

static mut TYPE_OBJECT: *mut Il2CppObject = 0 as _;
pub fn type_object() -> *mut Il2CppObject {
    unsafe { TYPE_OBJECT }
}

pub fn init(UnityEngine_UIModule: *const Il2CppImage) {
    get_class_or_return!(UnityEngine_UIModule, UnityEngine, Canvas);

    unsafe {
        TYPE_OBJECT = il2cpp_type_get_object(il2cpp_class_get_type(Canvas));
    }
}
