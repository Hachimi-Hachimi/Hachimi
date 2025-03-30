use std::os::raw::c_void;

use mlua::UserData;

use super::Type;

#[derive(Debug, Copy, Clone)]
pub struct Reference {
    ptr: *mut c_void,
    type_: Type
}

impl Reference {
    pub fn new(ptr: *mut c_void, type_: Type) -> Option<Reference> {
        if ptr.is_null() { None } else { Some(Self { ptr, type_ }) }
    }

    pub unsafe fn new_unchecked(ptr: *mut c_void, type_: Type) -> Reference {
        Self { ptr, type_ }
    }

    pub fn ptr(&self) -> *mut c_void {
        self.ptr
    }

    pub fn type_(&self) -> Type {
        self.type_
    }
}

impl UserData for Reference {
    //TODO
}