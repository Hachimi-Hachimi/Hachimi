use crate::il2cpp::{symbols::get_method_addr, types::*};

use super::Connection::SELECT_QUERIES;

type GetTextFn = extern "C" fn(this: *mut Il2CppObject, idx: i32) -> *mut Il2CppString;
extern "C" fn GetText(this: *mut Il2CppObject, idx: i32) -> *mut Il2CppString {
    if let Some(query) = SELECT_QUERIES.lock().unwrap().get(&(this as usize)) {
        return query.get_text(this, idx).unwrap_or_else(|| get_orig_fn!(GetText, GetTextFn)(this, idx));
    }
    get_orig_fn!(GetText, GetTextFn)(this, idx)
}

type DisposeFn = extern "C" fn(this: *mut Il2CppObject);
extern "C" fn Dispose(this: *mut Il2CppObject) {
    SELECT_QUERIES.lock().unwrap().remove(&(this as usize));
    get_orig_fn!(Dispose, DisposeFn)(this);
}

static mut GETINT_ADDR: usize = 0;
impl_addr_wrapper_fn!(GetInt, GETINT_ADDR, i32, this: *mut Il2CppObject, index: i32);

pub fn init(LibNative_Runtime: *const Il2CppImage) {
    get_class_or_return!(LibNative_Runtime, "LibNative.Sqlite3", Query);

    let GetText_addr = get_method_addr(Query, c"GetText", 1);
    let Dispose_addr = get_method_addr(Query, c"Dispose", 0);

    new_hook!(GetText_addr, GetText);
    new_hook!(Dispose_addr, Dispose);

    unsafe {
        GETINT_ADDR = get_method_addr(Query, c"GetInt", 1);
    }
}