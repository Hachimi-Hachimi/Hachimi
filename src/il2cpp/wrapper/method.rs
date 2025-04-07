use std::{os::raw::c_void, sync::{Arc, Weak}};

use libffi::{middle::{Cif, Type as FfiType}, raw::ffi_cif};
use mlua::{AnyUserData, Lua, MetaMethod, UserData, UserDataFields, UserDataMethods};

use crate::{core::{interceptor::{FfiHookFn, FfiNext, FfiUserData}, Hachimi}, il2cpp::{api::{il2cpp_method_is_generic, il2cpp_method_is_inflated, il2cpp_method_is_instance, il2cpp_runtime_invoke}, types::*, Error}};

use super::{BoundValue, Class, Exception, GetRaw, NativePointer, Type, Value};

pub trait Method: GetRaw<*const MethodInfo> {
    unsafe fn raw_object(&self) -> *mut c_void;

    fn parameters(&self) -> &'static [Type] {
        unsafe { std::slice::from_raw_parts((*self.raw()).parameters as _, (*self.raw()).parameters_count as _) }
    }

    fn return_type(&self) -> Type {
        unsafe { Type::new_unchecked((*self.raw()).return_type) }
    }

    fn class(&self) -> Class {
        unsafe { Class::new_unchecked((*self.raw()).klass) }
    }

    fn invoke(&self, args: &[Value]) -> Result<Value, Error> {
        let obj = unsafe { self.raw_object() };
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

    fn hook(&self, hook_addr: usize) -> Result<usize, Error> {
        Ok(Hachimi::instance().interceptor().hook(self.method_pointer(), hook_addr)?)
    }

    fn hook_ffi<T: 'static + Send + Sync>(&self, hook_fn: FfiHookFn, userdata: T) -> Result<usize, Error> {
        Ok(Hachimi::instance().interceptor().hook_ffi(self.method_pointer(), self.cif()?, hook_fn, userdata)?)
    }

    fn cif(&self) -> Result<Cif, Error> {
        let raw_params = self.parameters();
        let mut params: Vec<FfiType>;
        if self.is_static() {
            params = Vec::with_capacity(raw_params.len());
        }
        else {
            // extra "this" parameter
            params = Vec::with_capacity(raw_params.len() + 1);
            params.push(
                self.class().type_().to_ffi_type()
                    .ok_or_else(|| Error::UnknownType(self.class().type_().type_enum()))?
            );
        }
        let conv_iter = raw_params
            .iter()
            .map(|p|
                p.to_ffi_type()
                    .ok_or_else(||
                        Error::UnknownType(p.type_enum())
                    )
            );
        for res in conv_iter {
            params.push(res?);
        }

        Ok(Cif::new(
            params,
            self.return_type()
                .to_ffi_type()
                .ok_or_else(||
                    Error::UnknownType(self.return_type().type_enum())
                )?
        ))
    }

    fn method_pointer(&self) -> usize {
        unsafe { (*self.raw()).methodPointer }
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
    fn lua_invoke(&self, args: mlua::Variadic<mlua::Value>) -> mlua::Result<Value> {
        Ok(self.invoke(&self.lua_args_to_values(&args)?)?)
    }

    fn lua_args_to_values(&self, args: &[mlua::Value]) -> mlua::Result<Vec<Value>> {
        let params = self.parameters();
        if params.len() != args.len() {
            Err(Error::InvalidArgumentCount {
                expected: params.len() as _,
                got: args.len() as _
            })?;
        }

        Ok(params.iter()
            .zip(args.into_iter())
            .enumerate()
            .map(|(n, (type_, arg))| {
                Ok(Value::from_lua(arg, *type_).ok_or_else(|| Error::invalid_argument(n as _, None))?)
            })
            .collect::<Result<Vec<Value>, mlua::Error>>()?
        )
    }

    fn ffi_hook_callback(
        cif: &ffi_cif,
        result: &mut c_void,
        args: *const *const c_void,
        next: &FfiNext,
        userdata: FfiUserData,
        id: usize
    ) {
        match Self::ffi_hook_callback_internal(cif, result, args, next, userdata, id) {
            Err(e) => error!("{e}"),
            _ => ()
        }
    }

    fn ffi_hook_callback_internal(
        cif: &ffi_cif,
        result: &mut c_void,
        args: *const *const c_void,
        next: &FfiNext,
        userdata: FfiUserData,
        id: usize
    ) -> mlua::Result<()> {
        let hook_data: &LuaHookUserData = userdata.downcast_ref().unwrap();
        let Some(lua) = hook_data.lua.upgrade() else {
            Hachimi::instance().interceptor().unhook_ffi(hook_data.orig_addr, id);
            next.call(cif, result, args);
            return Ok(());
        };

        let next_wrapper_ud = lua.create_userdata(FfiNextWrapper {
            method: hook_data.method,
            cif,
            result,
            next,
            called: false,
            invalidated: false
        }).inspect_err(|_|
            next.call(cif, result, args)
        )?;

        fn do_call(
            hook_data: &LuaHookUserData,
            args: *const *const c_void,
            lua: Arc<Lua>,
            next_wrapper_ud: AnyUserData
        ) -> mlua::Result<mlua::Value> {
            let mut p = args;
            let mut lua_args = mlua::MultiValue::new();
            if !hook_data.method.is_static() {
                // "this" parameter
                lua_args.push_back(unsafe {
                    lua.pack(Value::from_ffi_value(*args as _, hook_data.method.class().type_())?)?
                });
                unsafe { p = args.add(1) }
            }

            // actual parameters
            let params_iter = hook_data.method.parameters()
                .iter()
                .enumerate();

            for (i, type_) in params_iter {
                let v = unsafe { Value::from_ffi_value(*p.add(i) as _, *type_)? };
                lua_args.push_back(lua.pack(v)?);
            }

            // "next" method
            lua_args.push_back(lua.pack(next_wrapper_ud)?);

            hook_data.callback.call(lua_args)
        }

        let call_res = do_call(hook_data, args, lua.clone(), next_wrapper_ud.clone());
        let mut next_wrapper = next_wrapper_ud.borrow_mut::<FfiNextWrapper>()?;
        next_wrapper.invalidated = true;

        let lua_ret_val = match call_res {
            Ok(v) => v,
            Err(e) => {
                if !next_wrapper.called {
                    warn!("Implicitly calling next function due to error in hook");
                    next.call(cif, result, args);
                }
                return Err(e);
            }
        };
        let ret_type = hook_data.method.return_type();
        let ret_type_enum = ret_type.type_enum();

        if let Some(ret_val) = Value::from_lua(&lua_ret_val, ret_type) {
            if ret_type_enum == Il2CppTypeEnum_IL2CPP_TYPE_VOID {
                // Do nothing, we're just here for the type check
                return Ok(());
            }

            if let Some(invoker_param) = unsafe { ret_val.to_invoker_param(ret_type) } {
                let ptr = invoker_param.get();

                #[allow(non_upper_case_globals)]
                match ret_type_enum {
                    // these types are passed by reference or a raw pointer value
                    Il2CppTypeEnum_IL2CPP_TYPE_STRING |
                    Il2CppTypeEnum_IL2CPP_TYPE_PTR |
                    Il2CppTypeEnum_IL2CPP_TYPE_BYREF |
                    Il2CppTypeEnum_IL2CPP_TYPE_CLASS |
                    Il2CppTypeEnum_IL2CPP_TYPE_ARRAY |
                    Il2CppTypeEnum_IL2CPP_TYPE_GENERICINST |
                    Il2CppTypeEnum_IL2CPP_TYPE_FNPTR |
                    Il2CppTypeEnum_IL2CPP_TYPE_OBJECT |
                    Il2CppTypeEnum_IL2CPP_TYPE_SZARRAY => {
                        unsafe { *(result as *mut _ as *mut *const _) = ptr; }
                    },
                    // everything else is passed by value
                    _ => {
                        let size = ret_type.class().instance_size();
                        unsafe { std::ptr::copy_nonoverlapping(ptr, result, size.try_into().unwrap()); }
                    }
                }

                return Ok(());
            }
        }

        warn!("Unable to convert Lua return value");
        if !next_wrapper.called {
            warn!("Implicitly calling next function");
            next.call(cif, result, args);
        }

        Ok(())
    }

    fn add_method_methods<M: UserDataMethods<Self>>(methods: &mut M) where Self: Sized {
        methods.add_method("invoke", |_, this, args| this.lua_invoke(args));
        methods.add_meta_method(MetaMethod::Call, |_, this, args| this.lua_invoke(args));

        methods.add_method("hook", |lua, this, callback: mlua::Function| {
            this.hook_ffi(Self::ffi_hook_callback, LuaHookUserData {
                method: UnboundMethod(this.raw()),
                lua: lua.app_data_ref::<Weak<Lua>>()
                    .ok_or_else(|| mlua::Error::external("Failed to obtain weak reference to Lua"))?
                    .clone(),
                callback,
                orig_addr: this.method_pointer()
            })?;
            Ok(())
        });
    }

    fn add_method_fields<F: UserDataFields<Self>>(_fields: &mut F) where Self: Sized {}
}

struct LuaHookUserData {
    method: UnboundMethod,
    lua: Weak<Lua>,
    callback: mlua::Function,
    orig_addr: usize
}

unsafe impl Send for LuaHookUserData {}
unsafe impl Sync for LuaHookUserData {}

struct LuaHookHandle {
    orig_addr: usize,
    id: usize
}

impl LuaHookHandle {
    fn unhook(&self) -> bool {
        Hachimi::instance().interceptor().unhook_ffi(self.orig_addr, self.id).is_some()
    }
}

impl UserData for LuaHookHandle {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("unhook", |_, this, ()| Ok(this.unhook()));
    }
}

struct FfiNextWrapper {
    method: UnboundMethod,
    cif: *const ffi_cif,
    result: *mut c_void,
    next: *const FfiNext,
    called: bool,
    invalidated: bool
}

impl UserData for FfiNextWrapper {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method_mut(MetaMethod::Call, |_, this, args: mlua::Variadic<mlua::Value>| {
            if this.invalidated {
                return Err(mlua::Error::external("Attempted to call invalidated next method"));
            }

            let params = this.method.parameters();
            let arg_values: Vec<Value>;
            let this_value: Value;
            let mut ffi_args = if this.method.is_static() {
                arg_values = this.method.lua_args_to_values(&args)?;
                Vec::with_capacity(params.len())
            }
            else {
                arg_values = this.method.lua_args_to_values(
                    args.get(1..).ok_or_else(|| Error::InvalidArgumentCount {
                        expected: params.len() as _,
                        got: args.len() as _
                    })?
                )?;

                let this_type = this.method.class().type_();
                this_value = Value::from_lua(&args[0], this_type)
                    .ok_or_else(|| Error::invalid_argument(0 as _, "invalid \"this\" object type".to_owned()))?;

                let mut vec = Vec::with_capacity(params.len() + 1);
                vec.push(unsafe { this_value.to_ffi_arg(this_type).unwrap() });
                vec
            };

            let conv_iter = arg_values.iter()
                .zip(this.method.parameters())
                .enumerate()
                .map(|(i, (v, type_))| unsafe {
                    v.to_ffi_arg(*type_)
                        .ok_or(Error::invalid_argument(i as _, "incompatible value type".to_owned()))
                });

            for res in conv_iter {
                ffi_args.push(res?);
            }

            unsafe { (*this.next).call(&*this.cif, &mut *this.result, ffi_args.as_ptr() as _); }
            this.called = true;
            Ok(unsafe { Value::from_ffi_value(this.result, this.method.return_type())? })
        });
    }
}

wrapper_struct!(UnboundMethod, *const MethodInfo);

impl UnboundMethod {
    pub fn bind(&self, value: impl Into<BoundValue>) -> BoundMethod {
        BoundMethod(self.0, value.into())
    }
}

impl Method for UnboundMethod {
    unsafe fn raw_object(&self) -> *mut c_void {
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


#[derive(Debug, Clone)]
pub struct BoundMethod(*const MethodInfo, BoundValue);

impl BoundMethod {
    pub fn new(p: *const MethodInfo, value: BoundValue) -> Option<Self> {
        if p.is_null() { None } else { Some(Self(p, value)) }
    }

    pub unsafe fn new_unchecked(p: *const MethodInfo, value: BoundValue) -> Self {
        Self(p, value)
    }

    fn add_raw_field<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("raw", |_, v| Ok(NativePointer::Raw(v.0 as _)));
    }

    pub fn bound_value(&self) -> &BoundValue {
        &self.1
    }
}

impl GetRaw<*const MethodInfo> for BoundMethod {
    fn raw(&self) -> *const MethodInfo {
        self.0
    }
}

impl Method for BoundMethod {
    unsafe fn raw_object(&self) -> *mut c_void {
        match &self.1 {
            BoundValue::Object(object) => object.raw() as _,
            BoundValue::ValueType(value_type) => value_type.ptr().get(),
        }
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