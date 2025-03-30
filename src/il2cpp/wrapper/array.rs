use mlua::{UserData, UserDataFields, UserDataMethods};

use crate::il2cpp::{ext::Il2CppObjectExt, types::*};

use super::{Class, Object};

wrapper_struct!(Array, *mut Il2CppArray);

impl Array {
    pub fn class(&self) -> Class {
        unsafe { Class::new_unchecked((*self.0).obj.klass()) }
    }

    pub fn as_object(&self) -> Object {
        // SAFETY: An array is also an object
        unsafe { Object::new_unchecked(self.0 as _) }
    }
}

impl UserData for Array {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {

    }

    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        Self::add_raw_field(fields);
    }
}