use std::os::raw::c_void;

use mlua::UserData;

use super::Type;

#[derive(Debug, Copy, Clone)]
pub struct Pointer {
    ptr: *mut c_void,
    type_: Type
}

impl Pointer {
    pub fn new(ptr: *mut c_void, type_: Type) -> Pointer {
        Self { ptr, type_ }
    }

    pub fn ptr(&self) -> *mut c_void {
        self.ptr
    }

    pub fn type_(&self) -> Type {
        self.type_
    }
}

impl UserData for Pointer {
    // TODO
}