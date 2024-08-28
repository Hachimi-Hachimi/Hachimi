use crate::il2cpp::{api::il2cpp_resolve_icall, symbols::{get_method_addr, Array}, types::*};

static mut DESTROY_ADDR: usize = 0;
impl_addr_wrapper_fn!(Destroy, DESTROY_ADDR, (), obj: *mut Il2CppObject);

static mut SET_HIDEFLAGS_ADDR: usize = 0;
impl_addr_wrapper_fn!(set_hideFlags, SET_HIDEFLAGS_ADDR, (), this: *mut Il2CppObject, value: i32);

static mut ISNATIVEOBJECTALIVE_ADDR: usize = 0;
impl_addr_wrapper_fn!(IsNativeObjectAlive, ISNATIVEOBJECTALIVE_ADDR, bool, obj: *mut Il2CppObject);

static mut GET_NAME_ADDR: usize = 0;
impl_addr_wrapper_fn!(get_name, GET_NAME_ADDR, *mut Il2CppString, this: *mut Il2CppObject);

static mut FINDOBJECTSOFTYPE_ADDR: usize = 0;
impl_addr_wrapper_fn!(
    FindObjectsOfType, FINDOBJECTSOFTYPE_ADDR, Array<*mut Il2CppObject>, type_: *mut Il2CppObject, include_inactive: bool
);

pub fn init(UnityEngine_CoreModule: *const Il2CppImage) {
    get_class_or_return!(UnityEngine_CoreModule, UnityEngine, Object);

    unsafe {
        DESTROY_ADDR = get_method_addr(Object, c"Destroy", 1);
        SET_HIDEFLAGS_ADDR = get_method_addr(Object, c"set_hideFlags", 1);
        ISNATIVEOBJECTALIVE_ADDR = get_method_addr(Object, c"IsNativeObjectAlive", 1);
        GET_NAME_ADDR = get_method_addr(Object, c"get_name", 0);
        FINDOBJECTSOFTYPE_ADDR = il2cpp_resolve_icall(
            c"UnityEngine.Object::FindObjectsOfType(System.Type,System.Boolean)".as_ptr()
        );
    }
}