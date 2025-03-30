use std::ffi::CStr;

use mlua::{UserData, UserDataFields, UserDataMethods};

use crate::il2cpp::{api::*, ext::Il2CppObjectExt, types::*};

use super::{Array, BoundField, Class, GetRaw};

wrapper_struct!(Object, *mut Il2CppObject);

impl Object {
    pub fn class(&self) -> Class {
        unsafe { Class::new_unchecked((*self.0).klass()) }
    }

    pub unsafe fn unbox<T: Copy>(obj: *mut Il2CppObject) -> T {
        *(il2cpp_object_unbox(obj) as *mut T)
    }

    pub unsafe fn as_string(&self) -> super::String {
        super::String::new_unchecked(self.0 as _)
    }

    pub unsafe fn as_array(&self) -> Array {
        super::Array::new_unchecked(self.0 as _)
    }

    pub fn field(&self, name: &CStr) -> Option<BoundField> {
        BoundField::new(il2cpp_class_get_field_from_name(self.class().raw(), name.as_ptr()), *self)
    }
}

impl UserData for Object {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {

    }

    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        Self::add_raw_field(fields);
    }
}