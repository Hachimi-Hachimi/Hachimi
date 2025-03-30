use std::ffi::{CStr, CString};

use mlua::{UserData, UserDataFields, UserDataMethods};

use crate::il2cpp::{api::*, types::*};

use super::Assembly;

wrapper_struct!(Domain, *mut Il2CppDomain);

impl Domain {
    pub fn get() -> Self {
        Self(il2cpp_domain_get())
    }

    pub fn assembly(&self, name: &CStr) -> Option<Assembly> {
        Assembly::new(il2cpp_domain_assembly_open(self.0, name.as_ptr()))
    }
}

impl UserData for Domain {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("assembly", |_, domain, name: CString|
            Ok(domain.assembly(&name))
        );
    }

    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        Self::add_raw_field(fields);
    }
}