use std::ptr::null_mut;

use crate::il2cpp::{symbols::{get_field_from_name, get_field_object_value, get_method_addr}, types::*};

static mut GET_TEXTURESETNAME_ADDR: usize = 0;
impl_addr_wrapper_fn!(get_TextureSetName, GET_TEXTURESETNAME_ADDR, *mut Il2CppString, this: *mut Il2CppObject);

static mut _TEXTURESETCOLOR_FIELD: *mut FieldInfo = null_mut();
pub fn get__textureSetColor(this: *mut Il2CppObject) -> *mut Il2CppObject {
    get_field_object_value(this, unsafe { _TEXTURESETCOLOR_FIELD })
}

pub fn init(Plugins: *const Il2CppImage) {
    get_class_or_return!(Plugins, AnimateToUnity, AnMeshInfoParameterGroup);

    unsafe {
        GET_TEXTURESETNAME_ADDR = get_method_addr(AnMeshInfoParameterGroup, cstr!("get_TextureSetName"), 0);
        _TEXTURESETCOLOR_FIELD = get_field_from_name(AnMeshInfoParameterGroup, cstr!("_textureSetColor"));
    }
}