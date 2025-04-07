use mlua::{UserData, UserDataFields, UserDataMethods};

use crate::il2cpp::{api::*, ext::Il2CppObjectExt, types::*};

use super::{Array, BindableValue, Class};

wrapper_struct!(Object, *mut Il2CppObject);

impl Object {
    pub unsafe fn unbox<T: Copy>(obj: *mut Il2CppObject) -> T {
        *(il2cpp_object_unbox(obj) as *mut T)
    }

    pub unsafe fn as_string(&self) -> super::String {
        super::String::new_unchecked(self.0 as _)
    }

    pub unsafe fn as_array(&self) -> Array {
        super::Array::new_unchecked(self.0 as _)
    }
}

impl BindableValue for Object {
    fn class(&self) -> Class {
        unsafe { Class::new_unchecked((*self.0).klass()) }
    }
}

impl UserData for Object {
    // TODO
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        Self::add_bindable_value_methods(methods);
    }

    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        Self::add_raw_field(fields);
    }
}