use bitflags::bitflags;
use mlua::{UserData, UserDataFields, UserDataMethods};

use std::{ffi::CStr, mem::MaybeUninit, os::raw::c_void};

use crate::il2cpp::{api::{il2cpp_field_get_flags, il2cpp_field_get_value, il2cpp_field_static_get_value}, types::*, Error};

use super::{Array, Class, GetRaw, NativePointer, Object, Pointer, Reference, Type, Value, ValueType};

pub trait Field: GetRaw<*mut FieldInfo> {
    unsafe fn name(&self) -> &'static CStr {
        CStr::from_ptr((*self.raw()).name)
    }

    fn type_(&self) -> Type {
        unsafe { Type::new_unchecked((*self.raw()).type_) }
    }

    fn parent(&self) -> Class {
        unsafe { Class::new_unchecked((*self.raw()).parent) }
    }

    fn offset(&self) -> i32 {
        unsafe { (*self.raw()).offset }
    }

    fn token(&self) -> u32 {
        unsafe { (*self.raw()).token }
    }

    fn flags(&self) -> FieldAttribute {
        FieldAttribute::from_bits_retain(il2cpp_field_get_flags(self.raw()))
    }

    fn is_static(&self) -> bool {
        self.flags().contains(FieldAttribute::STATIC)
    }

    unsafe fn raw_value_to_ptr(&self, ptr: *mut c_void) -> Result<(), Error>;

    /// EXTREMELY UNSAFE: UB when the wrong type is specified.
    unsafe fn raw_value<T: Clone>(&self) -> Result<T, Error> {
        let mut value = MaybeUninit::uninit();
        self.raw_value_to_ptr(value.as_mut_ptr() as _)?;
        Ok(value.assume_init())
    }

    fn value(&self) -> Result<Value, Error> {
        Ok(unsafe {
            #[allow(non_upper_case_globals)]
            match self.type_().type_enum() {
                Il2CppTypeEnum_IL2CPP_TYPE_VOID => Value::Void,
                Il2CppTypeEnum_IL2CPP_TYPE_BOOLEAN => Value::Boolean(self.raw_value()?),
                Il2CppTypeEnum_IL2CPP_TYPE_CHAR => Value::Char(self.raw_value()?),
                Il2CppTypeEnum_IL2CPP_TYPE_I1 => Value::I1(self.raw_value()?),
                Il2CppTypeEnum_IL2CPP_TYPE_U1 => Value::U1(self.raw_value()?),
                Il2CppTypeEnum_IL2CPP_TYPE_I2 => Value::I2(self.raw_value()?),
                Il2CppTypeEnum_IL2CPP_TYPE_U2 => Value::U2(self.raw_value()?),
                Il2CppTypeEnum_IL2CPP_TYPE_I4 => Value::I4(self.raw_value()?),
                Il2CppTypeEnum_IL2CPP_TYPE_U4 => Value::U4(self.raw_value()?),
                Il2CppTypeEnum_IL2CPP_TYPE_I8 => Value::I8(self.raw_value()?),
                Il2CppTypeEnum_IL2CPP_TYPE_U8 => Value::U8(self.raw_value()?),
                Il2CppTypeEnum_IL2CPP_TYPE_R4 => Value::R4(self.raw_value()?),
                Il2CppTypeEnum_IL2CPP_TYPE_R8 => Value::R8(self.raw_value()?),
                Il2CppTypeEnum_IL2CPP_TYPE_STRING => {
                    match super::String::new(self.raw_value()?) {
                        Some(s) => Value::String(s),
                        None => Value::NULL,
                    }
                },
                Il2CppTypeEnum_IL2CPP_TYPE_PTR | Il2CppTypeEnum_IL2CPP_TYPE_FNPTR => Value::Pointer(
                    Pointer::new(self.raw_value()?, self.type_())
                ),
                Il2CppTypeEnum_IL2CPP_TYPE_BYREF => {
                    match Reference::new(self.raw_value()?, self.type_()) {
                        Some(r) => Value::Reference(r),
                        None => Value::NULL,
                    }
                },
                Il2CppTypeEnum_IL2CPP_TYPE_VALUETYPE => {
                    let size = self.type_().class().value_size();
                    let ptr = NativePointer::alloc(size.try_into().unwrap());
                    self.raw_value_to_ptr(ptr.get())?;
                    Value::ValueType(ValueType::new_unchecked(ptr, self.type_()))
                },
                Il2CppTypeEnum_IL2CPP_TYPE_CLASS => {
                    match Object::new(self.raw_value()?) {
                        Some(o) => Value::Class(o),
                        None => Value::NULL,
                    }
                },
                Il2CppTypeEnum_IL2CPP_TYPE_ARRAY => {
                    match Array::new(self.raw_value()?) {
                        Some(a) => Value::Array(a),
                        None => Value::NULL,
                    }
                },
                Il2CppTypeEnum_IL2CPP_TYPE_GENERICINST => {
                    match Object::new(self.raw_value()?) {
                        Some(o) => Value::GenericInstance(o),
                        None => Value::NULL,
                    }
                },
                Il2CppTypeEnum_IL2CPP_TYPE_OBJECT => {
                    match Object::new(self.raw_value()?) {
                        Some(o) => Value::Object(o),
                        None => Value::NULL,
                    }
                },
                Il2CppTypeEnum_IL2CPP_TYPE_SZARRAY => {
                    match Array::new(self.raw_value()?) {
                        Some(a) => Value::SzArray(a),
                        None => Value::NULL,
                    }
                },
                _ => Value::NULL
            }
        })
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct FieldAttribute: i32 {
        const PRIVATE              = 0x0001;
        const FAM_AND_ASSEM        = 0x0002;
        const ASSEMBLY             = 0x0003;
        const FAMILY               = 0x0004;
        const FAM_OR_ASSEM         = 0x0005;
        const PUBLIC               = 0x0006;

        const STATIC               = 0x0010;
        const INIT_ONLY            = 0x0020;
        const LITERAL              = 0x0040;
        const NOT_SERIALIZED       = 0x0080;
        const SPECIAL_NAME         = 0x0200;
        const PINVOKE_IMPL         = 0x2000;

        const RESERVED_MASK        = 0x9500;
        const RT_SPECIAL_NAME      = 0x0400;
        const HAS_FIELD_MARSHAL    = 0x1000;
        const HAS_DEFAULT          = 0x8000;
        const HAS_FIELD_RVA        = 0x0100;
    }
}

trait FieldUserData: Field {
    fn add_field_methods<M: UserDataMethods<Self>>(_methods: &mut M) where Self: Sized {}

    fn add_field_fields<F: UserDataFields<Self>>(fields: &mut F) where Self: Sized {
        fields.add_field_method_get("value", |_, this| Ok(this.value()?));
    }
}


wrapper_struct!(UnboundField, *mut FieldInfo);

impl UnboundField {
    pub fn bind(&self, obj: Object) -> BoundField {
        BoundField(self.0, obj)
    }
}

impl Field for UnboundField {
    unsafe fn raw_value_to_ptr(&self, ptr: *mut c_void) -> Result<(), Error> {
        if !self.is_static() {
            return Err(Error::AccessUnboundInstanceField);
        }

        il2cpp_field_static_get_value(self.0, ptr);
        Ok(())
    }
}

impl FieldUserData for UnboundField {}

impl UserData for UnboundField {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        Self::add_field_methods(methods);
    }

    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        Self::add_raw_field(fields);
        Self::add_field_fields(fields);
    }
}


#[derive(Debug, Copy, Clone)]
pub struct BoundField(*mut FieldInfo, Object); // TODO: Allow valuetype to be bound

impl BoundField {
    pub fn new(p: *mut FieldInfo, obj: Object) -> Option<Self> {
        if p.is_null() { None } else { Some(Self(p, obj)) }
    }

    pub unsafe fn new_unchecked(p: *mut FieldInfo, obj: Object) -> Self {
        Self(p, obj)
    }

    fn add_raw_field<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("raw", |_, v| Ok(NativePointer::Raw(v.0 as _)));
    }

    pub fn object(&self) -> Object {
        self.1
    }
}

impl GetRaw<*mut FieldInfo> for BoundField {
    fn raw(&self) -> *mut FieldInfo {
        self.0
    }
}

impl Field for BoundField {
    unsafe fn raw_value_to_ptr(&self, ptr: *mut c_void) -> Result<(), Error> {
        if self.is_static() {
            il2cpp_field_static_get_value(self.0, ptr);
        }
        else {
            il2cpp_field_get_value(self.1.raw(), self.0, ptr);
        }
        Ok(())
    }
}

impl FieldUserData for BoundField {}

impl UserData for BoundField {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        Self::add_field_methods(methods);
    }

    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        Self::add_raw_field(fields);
        Self::add_field_fields(fields);
    }
}