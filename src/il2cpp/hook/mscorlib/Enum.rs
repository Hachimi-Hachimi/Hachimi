use crate::il2cpp::{symbols::{get_method_addr, get_method_overload_addr, IEnumerable}, types::*};

static mut TOOBJECT_ADDR: usize = 0;
impl_addr_wrapper_fn!(ToObject, TOOBJECT_ADDR, *mut Il2CppObject, enum_type: *mut Il2CppObject, value: i32);

static mut TOSTRING_ADDR: usize = 0;
impl_addr_wrapper_fn!(ToString, TOSTRING_ADDR, *mut Il2CppString, this: *mut Il2CppObject);

static mut TOUINT64_ADDR: usize = 0;
impl_addr_wrapper_fn!(ToUInt64, TOUINT64_ADDR, u64, object: *mut Il2CppObject);

static mut PARSE_ADDR: usize = 0;
impl_addr_wrapper_fn!(Parse, PARSE_ADDR, *mut Il2CppObject, enum_type: *mut Il2CppObject, value: *mut Il2CppString);

static mut GETVALUES_ADDR: usize = 0;
impl_addr_wrapper_fn!(GetValues, GETVALUES_ADDR, IEnumerable, enum_type: *mut Il2CppObject);

pub fn init(mscorlib: *const Il2CppImage) {
    get_class_or_return!(mscorlib, System, Enum);

    unsafe {
        TOOBJECT_ADDR = get_method_overload_addr(Enum, "ToObject", &[Il2CppTypeEnum_IL2CPP_TYPE_CLASS, Il2CppTypeEnum_IL2CPP_TYPE_I4]);
        TOSTRING_ADDR = get_method_addr(Enum, c"ToString", 0);
        // ToInt32 would make more sense here; but for some reason it doesn't exist!
        TOUINT64_ADDR = get_method_addr(Enum, c"ToUInt64", 1);
        PARSE_ADDR = get_method_addr(Enum, c"Parse", 2);
        GETVALUES_ADDR = get_method_addr(Enum, c"GetValues", 1);
    }
}