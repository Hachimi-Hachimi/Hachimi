use crate::il2cpp::{symbols::{get_field_from_name, get_field_object_value}, types::*};

// List<AnMeshInfoParameterGroup>
static mut _MESHPARAMETERGROUPLIST_FIELD: *mut FieldInfo = 0 as _;
pub fn get__meshParameterGroupList(this: *mut Il2CppObject) -> *mut Il2CppObject {
    get_field_object_value(this, unsafe { _MESHPARAMETERGROUPLIST_FIELD })
}

pub fn init(Plugins: *const Il2CppImage) {
    get_class_or_return!(Plugins, AnimateToUnity, AnMeshParameter);

    unsafe {
        _MESHPARAMETERGROUPLIST_FIELD = get_field_from_name(AnMeshParameter, c"_meshParameterGroupList");
    }
}