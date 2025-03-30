use std::fmt::Display;

use mlua::{MetaMethod, UserData, UserDataFields, UserDataMethods};
use widestring::{Utf16Str, Utf16String};

use crate::il2cpp::{api::*, ext::{Il2CppObjectExt, Il2CppStringExt}, types::*};

use super::{Class, Object};

wrapper_struct!(String, *mut Il2CppString);

impl String {
    pub fn class(&self) -> Class {
        unsafe { Class::new_unchecked((*self.0).object.klass()) }
    }

    pub fn as_object(&self) -> Object {
        // SAFETY: A string is also an object
        unsafe { Object::new_unchecked(self.0 as _) }
    }

    pub fn len(&self) -> i32 {
        unsafe { (*self.0).length }
    }

    pub fn chars(&self) -> *const Il2CppChar {
        unsafe { (*self.0).chars_ptr() }
    }

    pub fn as_utf16str(&self) -> &Utf16Str {
        unsafe { (*self.0).as_utf16str() }
    }
}

impl Display for String {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.as_utf16str().to_string())
    }
}

impl UserData for String {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(MetaMethod::ToString, |lua, this, ()|
            lua.create_string(this.to_string())
        );
    }

    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        Self::add_raw_field(fields);
    }
}

impl From<&str> for String {
    fn from(value: &str) -> Self {
        Utf16String::from_str(value).into()
    }
}

impl From<std::string::String> for String {
    fn from(value: std::string::String) -> Self {
        value.as_str().into()
    }
}

impl From<&Utf16Str> for String {
    fn from(value: &Utf16Str) -> Self {
        Self(il2cpp_string_new_utf16(value.as_ptr(), value.len().try_into().unwrap()))
    }
}

impl From<Utf16String> for String {
    fn from(value: Utf16String) -> Self {
        value.as_utfstr().into()
    }
}