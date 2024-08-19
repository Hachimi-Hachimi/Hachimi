use crate::il2cpp::types::*;

use super::symbols::{get_assembly_image, get_class, get_method_addr_cached};

#[allow(dead_code)]
pub fn print_stack_trace() {
    let mscorlib = get_assembly_image(c"mscorlib.dll").expect("mscorlib");
    let environment_class = get_class(mscorlib, c"System", c"Environment").expect("System.Environment");
    let get_fn_addr = get_method_addr_cached(environment_class, c"get_StackTrace", 0);
    let get_fn: extern "C" fn() -> *mut Il2CppString = unsafe { std::mem::transmute(get_fn_addr) };
    debug!("{}", unsafe { (*get_fn()).to_utf16str() });
}