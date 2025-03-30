use std::ffi::CString;

use mlua::{UserData, UserDataFields, UserDataMethods};

use crate::il2cpp::{api::*, types::*};

use super::Class;

wrapper_struct!(Image, *const Il2CppImage);

impl UserData for Image {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("class", |_, image, (namespace, name): (CString, CString)|
            Ok(Class::new(il2cpp_class_from_name(image.0, namespace.as_ptr(), name.as_ptr())))
        );
    }

    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        Self::add_raw_field(fields);
    }
}