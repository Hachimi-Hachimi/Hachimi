use std::{borrow::Cow, collections::hash_map, ffi::{CStr, CString}, os::raw::c_void, sync::{Arc, Mutex}};

use fnv::FnvHashMap;
use mlua::{UserData, UserDataFields, UserDataMethods, Value as LuaValue, FromLua};
use once_cell::sync::Lazy;

use crate::il2cpp::{api::*, types::*};

use super::{Field, UnboundField, UnboundMethod};

static CLASS_CACHES: Lazy<Mutex<FnvHashMap<usize, ClassCache>>> = Lazy::new(|| Mutex::default());

#[derive(Default)]
struct ClassCache<'a> {
    method_by_name: FnvHashMap<String, usize>,
    method_by_name_param_count: FnvHashMap<(Cow<'a, str>, u8), usize>,
    method_by_name_param_list: FnvHashMap<(Cow<'a, str>, Cow<'a, [Il2CppTypeEnum]>), usize>
}


wrapper_struct!(Class, *mut Il2CppClass);

pub enum MethodFindMode<'a> {
    NameOnly,
    WithParamCount(u8),
    WithParamList(&'a [Il2CppTypeEnum])
}

impl Class {
    pub fn method(&self, name: &str, mode: MethodFindMode) -> Option<UnboundMethod> {
        let mut caches = CLASS_CACHES.lock().unwrap();
        let cache_entry = caches.entry(self.0 as usize);
        let cache_opt = match &cache_entry {
            hash_map::Entry::Occupied(e) => Some(e.get()),
            hash_map::Entry::Vacant(_) => None
        };
        match mode {
            MethodFindMode::NameOnly => {
                if let Some(p) = cache_opt
                    .map(|c| c.method_by_name.get(name))
                    .flatten()
                {
                    return UnboundMethod::new(*p as _);
                }
            },

            MethodFindMode::WithParamCount(count) => {
                if let Some(p) = cache_opt
                    .map(|c| c.method_by_name_param_count.get(&(name.into(), count)))
                    .flatten()
                {
                    return UnboundMethod::new(*p as _);
                }
            },

            MethodFindMode::WithParamList(list) => {
                if let Some(p) = cache_opt
                    .map(|c| c.method_by_name_param_list.get(&(name.into(), list.into())))
                    .flatten()
                {
                    return UnboundMethod::new(*p as _);
                }
            }
        };

        let mut iter: *mut c_void = 0 as _;
        loop {
            let method = il2cpp_class_get_methods(self.0, &mut iter);
            if method.is_null() {
                break;
            }

            // Check name
            let method_name = unsafe { CStr::from_ptr((*method).name) };
            if method_name.to_str().unwrap() != name {
                continue;
            }

            match mode {
                MethodFindMode::NameOnly => {
                    cache_entry.or_insert_with(|| Default::default())
                        .method_by_name
                        .insert(name.to_owned(), method as _);
                    return UnboundMethod::new(method);
                },

                MethodFindMode::WithParamCount(count) => {
                    let param_count = unsafe { (*method).parameters_count };
                    if param_count == count {
                        cache_entry.or_insert_with(|| Default::default())
                            .method_by_name_param_count
                            .insert((name.to_owned().into(), param_count), method as _);
                        return UnboundMethod::new(method);
                    }
                },

                MethodFindMode::WithParamList(params) => {
                    let param_count = unsafe { (*method).parameters_count } as usize;
                    if param_count != params.len() {
                        continue;
                    }

                    let mut ok = true;
                    for i in 0..param_count {
                        let param = unsafe { *(*method).parameters.add(i) };
                        if unsafe { (*param).type_() } != params[i] {
                            ok = false;
                            break;
                        }
                    }

                    if ok {
                        cache_entry.or_insert_with(|| Default::default())
                            .method_by_name_param_list
                            .insert((name.to_owned().into(), params.to_vec().into()), method as _);
                        return UnboundMethod::new(method);
                    }
                }
            }
        }

        None
    }

    pub fn method_by_name(&self, name: &str) -> Option<UnboundMethod> {
        self.method(name, MethodFindMode::NameOnly)
    }

    pub fn method_with_param_count(&self, name: &str, param_count: u8) -> Option<UnboundMethod> {
        self.method(name, MethodFindMode::WithParamCount(param_count))
    }

    pub fn method_with_param_list(&self, name: &str, param_list: &[Il2CppTypeEnum]) -> Option<UnboundMethod> {
        self.method(name, MethodFindMode::WithParamList(param_list))
    }

    pub fn nested(&self, name: &CStr) -> Option<Class> {
        let mut iter: *mut c_void = 0 as _;
        loop {
            let nested_class = il2cpp_class_get_nested_types(self.0, &mut iter);
            if nested_class.is_null() { break; }

            let class_name = unsafe { CStr::from_ptr((*nested_class).name) };
            if class_name == name {
                return Class::new(nested_class);
            }
        }

        None
    }

    pub fn is_enum(&self) -> bool {
        il2cpp_class_is_enum(self.0)
    }

    pub fn is_enum_with_type(&self, type_enum: Il2CppTypeEnum) -> bool {
        self.is_enum() && 
        self.field(c"value__")
            .is_some_and(|f| f.type_().type_enum() == type_enum)
    }

    pub fn field(&self, name: &CStr) -> Option<UnboundField> {
        UnboundField::new(il2cpp_class_get_field_from_name(self.0, name.as_ptr()))
    }

    pub fn instance_size(&self) -> i32 {
        unsafe { (*self.0).instance_size as _ }
    }

    pub fn value_size(&self) -> i32 {
        self.instance_size() - std::mem::size_of::<Il2CppObject>() as i32
    }
}

impl UserData for Class {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("method", |lua, class, (name, arg_count_or_list): (String, LuaValue)| {
            let list: Vec<Il2CppTypeEnum>;
            let mode = match arg_count_or_list {
                LuaValue::Nil => MethodFindMode::NameOnly,

                LuaValue::Integer(_) => MethodFindMode::WithParamCount(
                    u8::from_lua(arg_count_or_list, lua)?
                ),

                LuaValue::Table(_) => {
                    list = Vec::from_lua(arg_count_or_list, lua)?;
                    MethodFindMode::WithParamList(&list)
                },

                _ => return Err(mlua::Error::BadArgument {
                    to: Some("method".to_owned()),
                    pos: 3,
                    name: Some("argCountOrList".to_owned()),
                    cause: Arc::new(mlua::Error::external("Invalid type")),
                })
            };

            Ok(class.method(&name, mode))
        });

        methods.add_method("nested", |_, class, name: CString|
            Ok(class.nested(&name))
        );

        methods.add_method("field", |_, class, name: CString|
            Ok(class.field(&name))
        );
    }

    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        Self::add_raw_field(fields);
    }
}