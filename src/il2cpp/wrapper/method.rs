use std::os::raw::c_void;

use mlua::{MetaMethod, UserData, UserDataFields, UserDataMethods};

use crate::il2cpp::{api::{il2cpp_method_is_generic, il2cpp_method_is_inflated, il2cpp_method_is_instance, il2cpp_runtime_invoke}, types::*, Error};

use super::{Array, Exception, Field, GetRaw, NativePointer, Object, Pointer, Reference, Type, Value, ValueType};

pub trait Method: GetRaw<*const MethodInfo> {
    fn raw_object(&self) -> *mut c_void;

    fn parameters(&self) -> &'static [Type] {
        unsafe { std::slice::from_raw_parts((*self.raw()).parameters as _, (*self.raw()).parameters_count as _) }
    }

    fn return_type(&self) -> Type {
        unsafe { Type::new_unchecked((*self.raw()).return_type) }
    }

    fn invoke(&self, args: &[Value]) -> Result<Value, Error> {
        let obj = self.raw_object();
        if !self.is_static() && obj.is_null() {
            return Err(Error::InvokeUnboundInstanceMethod);
        }

        let params = self.parameters();
        if params.len() != args.len() {
            return Err(Error::InvalidArgumentCount {
                expected: params.len() as _,
                got: args.len() as _
            });
        }

        let args = args.iter()
            .zip(params)
            .enumerate()
            .map(|(i, (v, type_))| unsafe {
                v.to_invoker_param(*type_)
                    .ok_or(Error::invalid_argument(i as _, "incompatible value type".to_owned()))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let mut exc: *mut Il2CppException = 0 as _;
        let res = il2cpp_runtime_invoke(
            self.raw(),
            obj,
            args.as_ptr() as _, // InvokerParam is repr(transparent)
            &mut exc
        );
        if let Some(exc) = Exception::new(exc) {
            return Err(Error::Exception(exc));
        }

        Ok(unsafe { Value::from_invoker_return(res, self.return_type()) })
    }

    fn is_static(&self) -> bool {
        !il2cpp_method_is_instance(self.raw())
    }

    fn is_generic(&self) -> bool {
        il2cpp_method_is_generic(self.raw())
    }

    fn is_inflated(&self) -> bool {
        il2cpp_method_is_inflated(self.raw())
    }
}

trait MethodUserData: Method {
    fn lua_invoke(&self, args: mlua::MultiValue) -> Result<Value, mlua::Error> {
        let params = self.parameters();
        if params.len() != args.len() {
            Err(Error::InvalidArgumentCount {
                expected: params.len() as _,
                got: args.len() as _
            })?;
        }

        fn arg_try_into<T, U: TryInto<T>>(value: U, n: usize) -> Result<T, mlua::Error>
        where
            U::Error: std::string::ToString
        {
            Ok(value.try_into().map_err(|e| Error::invalid_argument(n as _, e.to_string()))?)
        }

        fn invalid_arg_type_error(arg: &mlua::Value, n: usize) -> Error {
            Error::invalid_argument(n as _, format!("unexpected type: {}", arg.type_name()))
        }

        #[allow(non_upper_case_globals)]
        let args = params.iter()
            .zip(args)
            .enumerate()
            .map(|(n, (type_, arg))| {
                // All value types are allowed to be null
                if arg.is_nil() {
                    return Ok(Value::NULL);
                }
                match type_.type_enum() {
                    // expect null value but not nil (checked above)
                    Il2CppTypeEnum_IL2CPP_TYPE_VOID => Err(invalid_arg_type_error(&arg, n))?,
                    Il2CppTypeEnum_IL2CPP_TYPE_BOOLEAN => {
                        if let mlua::Value::Boolean(b) = arg {
                            Ok(Value::Boolean(b))
                        } else {
                            Err(invalid_arg_type_error(&arg, n))?
                        }
                    }
                    Il2CppTypeEnum_IL2CPP_TYPE_CHAR => {
                        if let mlua::Value::Integer(i) = arg {
                            Ok(Value::Char(arg_try_into(i, n)?))
                        } else {
                            Err(invalid_arg_type_error(&arg, n))?
                        }
                    }
                    Il2CppTypeEnum_IL2CPP_TYPE_I1 => {
                        if let mlua::Value::Integer(i) = arg {
                            Ok(Value::I1(arg_try_into(i, n)?))
                        } else {
                            Err(invalid_arg_type_error(&arg, n))?
                        }
                    }
                    Il2CppTypeEnum_IL2CPP_TYPE_U1 => {
                        if let mlua::Value::Integer(i) = arg {
                            Ok(Value::U1(arg_try_into(i, n)?))
                        } else {
                            Err(invalid_arg_type_error(&arg, n))?
                        }
                    }
                    Il2CppTypeEnum_IL2CPP_TYPE_I2 => {
                        if let mlua::Value::Integer(i) = arg {
                            Ok(Value::I2(arg_try_into(i, n)?))
                        } else {
                            Err(invalid_arg_type_error(&arg, n))?
                        }
                    }
                    Il2CppTypeEnum_IL2CPP_TYPE_U2 => {
                        if let mlua::Value::Integer(i) = arg {
                            Ok(Value::U2(arg_try_into(i, n)?))
                        } else {
                            Err(invalid_arg_type_error(&arg, n))?
                        }
                    }
                    Il2CppTypeEnum_IL2CPP_TYPE_I4 => {
                        if let mlua::Value::Integer(i) = arg {
                            Ok(Value::I4(arg_try_into(i, n)?))
                        } else {
                            Err(invalid_arg_type_error(&arg, n))?
                        }
                    }
                    Il2CppTypeEnum_IL2CPP_TYPE_U4 => {
                        if let mlua::Value::Integer(i) = arg {
                            Ok(Value::U4(arg_try_into(i, n)?))
                        } else {
                            Err(invalid_arg_type_error(&arg, n))?
                        }
                    }
                    Il2CppTypeEnum_IL2CPP_TYPE_I8 => {
                        if let mlua::Value::Integer(i) = arg {
                            Ok(Value::I8(arg_try_into(i, n)?))
                        } else {
                            Err(invalid_arg_type_error(&arg, n))?
                        }
                    }
                    Il2CppTypeEnum_IL2CPP_TYPE_U8 => {
                        if let mlua::Value::Integer(i) = arg {
                            Ok(Value::U8(arg_try_into(i, n)?))
                        } else {
                            Err(invalid_arg_type_error(&arg, n))?
                        }
                    }
                    Il2CppTypeEnum_IL2CPP_TYPE_R4 => {
                        if let mlua::Value::Number(v) = arg {
                            Ok(Value::R4(v as _))
                        } else {
                            Err(invalid_arg_type_error(&arg, n))?
                        }
                    }
                    Il2CppTypeEnum_IL2CPP_TYPE_R8 => {
                        if let mlua::Value::Number(v) = arg {
                            Ok(Value::R8(v))
                        } else {
                            Err(invalid_arg_type_error(&arg, n))?
                        }
                    }
                    Il2CppTypeEnum_IL2CPP_TYPE_STRING => {
                        if let mlua::Value::UserData(ud) = arg {
                            Ok(Value::String(*ud.borrow::<super::String>()?))
                        } else {
                            Err(invalid_arg_type_error(&arg, n))?
                        }
                    }
                    Il2CppTypeEnum_IL2CPP_TYPE_PTR | Il2CppTypeEnum_IL2CPP_TYPE_FNPTR => {
                        if let mlua::Value::UserData(ud) = arg {
                            Ok(Value::Pointer(*ud.borrow::<Pointer>()?))
                        } else {
                            Err(invalid_arg_type_error(&arg, n))?
                        }
                    }
                    Il2CppTypeEnum_IL2CPP_TYPE_BYREF => {
                        if let mlua::Value::UserData(ud) = arg {
                            Ok(Value::Reference(*ud.borrow::<Reference>()?))
                        } else {
                            Err(invalid_arg_type_error(&arg, n))?
                        }
                    }
                    Il2CppTypeEnum_IL2CPP_TYPE_VALUETYPE => {
                        match arg {
                            mlua::Value::UserData(ud) => Ok(Value::ValueType(ud.borrow::<ValueType>()?.clone())),
                            // Allow passing enum values as integers
                            mlua::Value::Integer(i) => {
                                let class = type_.class();
                                if class.is_enum() {
                                    if let Some(field) = class.field(c"value__") {
                                        match field.type_().type_enum() {
                                            Il2CppTypeEnum_IL2CPP_TYPE_I1 => return Ok(Value::I1(arg_try_into(i, n)?)),
                                            Il2CppTypeEnum_IL2CPP_TYPE_U1 => return Ok(Value::U1(arg_try_into(i, n)?)),
                                            Il2CppTypeEnum_IL2CPP_TYPE_I2 => return Ok(Value::I2(arg_try_into(i, n)?)),
                                            Il2CppTypeEnum_IL2CPP_TYPE_U2 => return Ok(Value::U2(arg_try_into(i, n)?)),
                                            Il2CppTypeEnum_IL2CPP_TYPE_I4 => return Ok(Value::I4(arg_try_into(i, n)?)),
                                            Il2CppTypeEnum_IL2CPP_TYPE_U4 => return Ok(Value::U4(arg_try_into(i, n)?)),
                                            Il2CppTypeEnum_IL2CPP_TYPE_I8 => return Ok(Value::I8(arg_try_into(i, n)?)),
                                            Il2CppTypeEnum_IL2CPP_TYPE_U8 => return Ok(Value::U8(arg_try_into(i, n)?)),
                                            Il2CppTypeEnum_IL2CPP_TYPE_CHAR => return Ok(Value::Char(arg_try_into(i, n)?)),
                                            _ => ()
                                        }
                                    }
                                }
                                Err(invalid_arg_type_error(&arg, n))?
                            }
                            _ => Err(invalid_arg_type_error(&arg, n))?
                        }
                    }
                    Il2CppTypeEnum_IL2CPP_TYPE_CLASS => {
                        if let mlua::Value::UserData(ud) = arg {
                            Ok(Value::Class(*ud.borrow::<Object>()?))
                        } else {
                            Err(invalid_arg_type_error(&arg, n))?
                        }
                    }
                    Il2CppTypeEnum_IL2CPP_TYPE_ARRAY => {
                        if let mlua::Value::UserData(ud) = arg {
                            Ok(Value::Array(*ud.borrow::<Array>()?))
                        } else {
                            Err(invalid_arg_type_error(&arg, n))?
                        }
                    }
                    Il2CppTypeEnum_IL2CPP_TYPE_GENERICINST => {
                        if let mlua::Value::UserData(ud) = arg {
                            Ok(Value::GenericInstance(*ud.borrow::<Object>()?))
                        } else {
                            Err(invalid_arg_type_error(&arg, n))?
                        }
                    }
                    Il2CppTypeEnum_IL2CPP_TYPE_OBJECT => {
                        if let mlua::Value::UserData(ud) = arg {
                            Ok(Value::Object(*ud.borrow::<Object>()?))
                        } else {
                            Err(invalid_arg_type_error(&arg, n))?
                        }
                    }
                    Il2CppTypeEnum_IL2CPP_TYPE_SZARRAY => {
                        if let mlua::Value::UserData(ud) = arg {
                            Ok(Value::SzArray(*ud.borrow::<Array>()?))
                        } else {
                            Err(invalid_arg_type_error(&arg, n))?
                        }
                    }
                    _ => Err(invalid_arg_type_error(&arg, n))?,
                }
            })
            .collect::<Result<Vec<Value>, mlua::Error>>()?;

        Ok(self.invoke(&args)?)
    }

    fn add_method_methods<M: UserDataMethods<Self>>(methods: &mut M) where Self: Sized {
        methods.add_method("invoke", |_, this, args| this.lua_invoke(args));
        methods.add_meta_method(MetaMethod::Call, |_, this, args| this.lua_invoke(args));
    }

    fn add_method_fields<F: UserDataFields<Self>>(_fields: &mut F) where Self: Sized {}
}


wrapper_struct!(UnboundMethod, *const MethodInfo);

impl UnboundMethod {
    pub fn bind(&self, obj: Object) -> BoundMethod {
        BoundMethod(self.0, obj)
    }
}

impl Method for UnboundMethod {
    fn raw_object(&self) -> *mut c_void {
        0 as _
    }
}

impl MethodUserData for UnboundMethod {}

impl UserData for UnboundMethod {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        Self::add_method_methods(methods);
    }

    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        Self::add_raw_field(fields);
        Self::add_method_fields(fields);
    }
}


#[derive(Debug, Copy, Clone)]
pub struct BoundMethod(*const MethodInfo, Object); // TODO: Allow valuetype to be bound

impl BoundMethod {
    pub fn new(p: *const MethodInfo, obj: Object) -> Option<Self> {
        if p.is_null() { None } else { Some(Self(p, obj)) }
    }

    pub unsafe fn new_unchecked(p: *const MethodInfo, obj: Object) -> Self {
        Self(p, obj)
    }

    fn add_raw_field<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("raw", |_, v| Ok(NativePointer::Raw(v.0 as _)));
    }

    pub fn object(&self) -> Object {
        self.1
    }
}

impl GetRaw<*const MethodInfo> for BoundMethod {
    fn raw(&self) -> *const MethodInfo {
        self.0
    }
}

impl Method for BoundMethod {
    fn raw_object(&self) -> *mut c_void {
        self.1.raw() as _
    }
}

impl MethodUserData for BoundMethod {}

impl UserData for BoundMethod {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        Self::add_method_methods(methods);
    }

    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        Self::add_raw_field(fields);
        Self::add_method_fields(fields);
    }
}