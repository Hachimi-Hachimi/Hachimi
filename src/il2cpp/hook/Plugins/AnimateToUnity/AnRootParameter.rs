use crate::il2cpp::{symbols::{get_field_from_name, get_field_object_value}, types::*};

// AnMotionParameterGroup
static mut _MOTIONPARAMETERGROUP_FIELD: *mut FieldInfo = 0 as _;
pub fn get__motionParameterGroup(this: *mut Il2CppObject) -> *mut Il2CppObject {
    get_field_object_value(this, unsafe { _MOTIONPARAMETERGROUP_FIELD })
}

pub fn init(Plugins: *const Il2CppImage) {
    get_class_or_return!(Plugins, AnimateToUnity, AnRootParameter);

    unsafe {
        _MOTIONPARAMETERGROUP_FIELD = get_field_from_name(AnRootParameter, c"_motionParameterGroup");
    }
}