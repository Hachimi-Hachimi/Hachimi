use crate::il2cpp::{symbols::{get_field_from_name, set_field_value}, types::*};

static mut _POSITIONOFFSET_FIELD: *mut FieldInfo = 0 as _;
pub fn set__positionOffset(this: *mut Il2CppObject, value: &Vector3_t) {
    set_field_value(this, unsafe { _POSITIONOFFSET_FIELD }, value);
}

static mut _SCALE_FIELD: *mut FieldInfo = 0 as _;
pub fn set__scale(this: *mut Il2CppObject, value: &Vector3_t) {
    set_field_value(this, unsafe { _SCALE_FIELD }, value);
}                               

pub fn init(Plugins: *const Il2CppImage) {
    get_class_or_return!(Plugins, AnimateToUnity, AnObjectParameterBase);

    unsafe {
        _POSITIONOFFSET_FIELD = get_field_from_name(AnObjectParameterBase, c"_positionOffset");
        _SCALE_FIELD = get_field_from_name(AnObjectParameterBase, c"_scale");
    }
}