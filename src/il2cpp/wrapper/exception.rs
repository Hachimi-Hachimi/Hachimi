use std::fmt::Display;

use mlua::{MetaMethod, UserData, UserDataFields, UserDataMethods};

use crate::il2cpp::{ext::Il2CppObjectExt, types::*};

use super::{Class, Object};

wrapper_struct!(Exception, *mut Il2CppException);

impl Exception {
    pub fn class(&self) -> Class {
        unsafe { Class::new_unchecked((*self.0).object.klass()) }
    }

    pub fn as_object(&self) -> Object {
        // SAFETY: An exception is also an object
        unsafe { Object::new_unchecked(self.0 as _) }
    }

    pub fn class_name(&self) -> super::String {
        unsafe { super::String::new_unchecked((*self.0).className) }
    }

    pub fn message(&self) -> super::String {
        unsafe { super::String::new_unchecked((*self.0).message) }
    }

    pub fn stack_trace(&self) -> Option<super::String> {
        super::String::new(unsafe { (*self.0).stack_trace })
    }
}

impl Display for Exception {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Exception.Class: Exception message
        //    at SomeClass.Method()
        write!(f,
            "{}: {}\n{}",
            self.class_name(),
            self.message(),
            self.stack_trace()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "   [no stack trace available]".to_string())
        )
    }
}

impl UserData for Exception {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(MetaMethod::ToString, |lua, this, ()|
            lua.create_string(this.to_string())
        );
    }

    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        Self::add_raw_field(fields);
    }
}