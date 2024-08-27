use crate::il2cpp::types::*;

static mut CLASS: *mut Il2CppClass = 0 as _;
pub fn class() -> *mut Il2CppClass {
    unsafe { CLASS }
}

pub fn init(mscorlib: *const Il2CppImage) {
    get_class_or_return!(mscorlib, System, Byte);

    unsafe {
        CLASS = Byte;
    }
}