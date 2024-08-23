use crate::il2cpp::{symbols::{get_field_from_name, get_field_ptr, get_method_addr}, types::*};

static mut M_REFERENCERESOLUTION_FIELD: *mut FieldInfo = 0 as _;
pub fn get_m_ReferenceResolution(this: *mut Il2CppObject) -> *mut Vector2_t {
    get_field_ptr(this, unsafe { M_REFERENCERESOLUTION_FIELD })
}

static mut SET_SCALEFACTOR_ADDR: usize = 0;
impl_addr_wrapper_fn!(set_scaleFactor, SET_SCALEFACTOR_ADDR, (), this: *mut Il2CppObject, value: f32);

pub fn init(UnityEngine_UI: *const Il2CppImage) {
    get_class_or_return!(UnityEngine_UI, "UnityEngine.UI", CanvasScaler);
    
    unsafe {
        M_REFERENCERESOLUTION_FIELD = get_field_from_name(CanvasScaler, c"m_ReferenceResolution");
        SET_SCALEFACTOR_ADDR = get_method_addr(CanvasScaler, c"set_scaleFactor", 1);
    }
}