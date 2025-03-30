use mlua::{UserData, UserDataFields};

use crate::il2cpp::{api::il2cpp_class_from_type, types::*};

use super::Class;

wrapper_struct!(Type, *const Il2CppType);

impl Type {
    pub fn type_enum(&self) -> Il2CppTypeEnum {
        unsafe { (*self.0).type_() }
    }

    pub fn class(&self) -> Class {
        unsafe { Class::new_unchecked(il2cpp_class_from_type(self.0)) }
    }
}

impl UserData for Type {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        Self::add_raw_field(fields);
    }
}