use crate::il2cpp::{symbols::{get_field_from_name, get_field_object_value}, types::*};

// List<AnMeshParameter>
static mut _MESHPARAMETERLIST_FIELD: *mut FieldInfo = 0 as _;
pub fn get__meshParameterList(this: *mut Il2CppObject) -> *mut Il2CppObject {
    get_field_object_value(this, unsafe { _MESHPARAMETERLIST_FIELD })
}

pub fn init(Plugins: *const Il2CppImage) {
    get_class_or_return!(Plugins, AnimateToUnity, AnMeshParameterGroup);

    unsafe {
        _MESHPARAMETERLIST_FIELD = get_field_from_name(AnMeshParameterGroup, c"_meshParameterList");
    }
}