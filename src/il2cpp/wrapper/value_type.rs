use mlua::UserData;

use super::{NativePointer, Type};

#[derive(Debug, Clone)]
pub struct ValueType {
    ptr: NativePointer,
    type_: Type
}

impl ValueType {
    pub fn new(ptr: impl Into<NativePointer>, type_: Type) -> Option<ValueType> {
        let ptr = ptr.into();
        if ptr.get().is_null() { None } else { Some(Self { ptr, type_ }) }
    }

    pub unsafe fn new_unchecked(ptr: impl Into<NativePointer>, type_: Type) -> ValueType {
        Self { ptr: ptr.into(), type_ }
    }

    pub fn ptr(&self) -> &NativePointer {
        &self.ptr
    }

    pub fn type_(&self) -> Type {
        self.type_
    }
}

impl UserData for ValueType {
    // TODO
}