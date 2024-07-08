use crate::il2cpp::{symbols::{get_field_from_name, get_field_object_value}, types::*};

// List<AnMotionParameter>
static mut _MOTIONPARAMETERLIST_FIELD: *mut FieldInfo = 0 as _;
pub fn get__motionParameterList(this: *mut Il2CppObject) -> *mut Il2CppObject {
    get_field_object_value(this, unsafe { _MOTIONPARAMETERLIST_FIELD })
}

pub fn init(Plugins: *const Il2CppImage) {
    get_class_or_return!(Plugins, AnimateToUnity, AnMotionParameterGroup);

    unsafe {
        _MOTIONPARAMETERLIST_FIELD = get_field_from_name(AnMotionParameterGroup, c"_motionParameterList");
    }
}