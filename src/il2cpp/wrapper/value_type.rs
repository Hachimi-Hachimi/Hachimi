use mlua::{UserData, UserDataFields, UserDataMethods};

use super::{BindableValue, Class, NativePointer, Type};

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

    fn add_raw_field<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("raw", |_, v| Ok(v.clone()));
    }

    pub fn ptr(&self) -> &NativePointer {
        &self.ptr
    }

    pub fn type_(&self) -> Type {
        self.type_
    }
}

impl BindableValue for ValueType {
    fn class(&self) -> Class {
        self.type_().class()
    }
}

impl UserData for ValueType {
    // TODO
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        Self::add_bindable_value_methods(methods);
    }

    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        Self::add_raw_field(fields);
    }
}