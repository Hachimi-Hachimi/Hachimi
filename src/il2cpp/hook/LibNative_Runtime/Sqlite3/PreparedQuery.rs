use crate::il2cpp::{symbols::get_method_addr, types::*};

use super::Connection::SELECT_QUERIES;

/**
 * LibNative.Sqlite3::PreparedQuery::BindInt
 * Binds an int value to a parameter.
 * idx starts from 1
 */
type BindIntFn = extern "C" fn(this: *mut Il2CppObject, idx: i32, value: i32) -> bool;
extern "C" fn BindInt(this: *mut Il2CppObject, idx: i32, value: i32) -> bool {
    if let Some(query) = SELECT_QUERIES.lock().unwrap().get_mut(&(this as usize)) {
        query.bind_int(idx, value);
    }
    get_orig_fn!(BindInt, BindIntFn)(this, idx, value)
}

pub fn init(LibNative_Runtime: *const Il2CppImage) {
    get_class_or_return!(LibNative_Runtime, "LibNative.Sqlite3", PreparedQuery);

    let BindInt_addr = get_method_addr(PreparedQuery, c"BindInt", 2);

    new_hook!(BindInt_addr, BindInt);
}