use std::ffi::{c_char, c_void, CStr};

use crate::{core::{Hachimi, Interceptor}, il2cpp::{self, types::{il2cpp_array_size_t, FieldInfo, Il2CppArray, Il2CppClass, Il2CppImage, Il2CppObject, Il2CppThread, Il2CppTypeEnum, MethodInfo}}};

const VERSION: i32 = 1;

pub type HachimiInitFn = extern "C" fn(vtable: *const Vtable, version: i32) -> InitResult;

#[repr(i32)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum InitResult {
    Error,
    Ok
}

impl InitResult {
    pub fn is_ok(&self) -> bool {
        match self {
            Self::Ok => true,
            _ => false
        }
    }
}

unsafe extern "C" fn hachimi_instance() -> *const Hachimi {
    Hachimi::instance().as_ref()
}

unsafe extern "C" fn hachimi_get_interceptor(this: *const Hachimi) -> *const Interceptor {
    &(*this).interceptor
}

unsafe extern "C" fn interceptor_hook(
    this: *const Interceptor, orig_addr: *mut c_void, hook_addr: *mut c_void
) -> *mut c_void {
    (*this).hook(orig_addr as _, hook_addr as _)
        .inspect_err(|e| error!("{}", e))
        .unwrap_or(0) as _
}

unsafe extern "C" fn interceptor_hook_vtable(
    this: *const Interceptor, vtable: *mut *mut c_void, vtable_index: usize, hook_addr: *mut c_void
) -> *mut c_void {
    (*this).hook_vtable(vtable as _, vtable_index as _, hook_addr as _)
        .inspect_err(|e| error!("{}", e))
        .unwrap_or(0) as _
}

unsafe extern "C" fn interceptor_get_trampoline_addr(this: *const Interceptor, hook_addr: *mut c_void) -> *mut c_void {
    (*this).get_trampoline_addr(hook_addr as _) as _
}

unsafe extern "C" fn interceptor_unhook(this: *const Interceptor, hook_addr: *mut c_void) -> *mut c_void {
    if let Some(handle) = (*this).unhook(hook_addr as _) {
        handle.orig_addr as _
    }
    else {
        0 as _
    }
}

unsafe extern "C" fn il2cpp_resolve_symbol(name: *const c_char) -> *mut c_void {
    let Ok(name) = CStr::from_ptr(name).to_str() else {
        return 0 as _;
    };
    il2cpp::symbols::dlsym(name) as _
}

unsafe extern "C" fn il2cpp_get_assembly_image(assembly_name: *const c_char) -> *const Il2CppImage {
    il2cpp::symbols::get_assembly_image(CStr::from_ptr(assembly_name))
        .inspect_err(|e| error!("{}", e))
        .unwrap_or(0 as _)
}

unsafe extern "C" fn il2cpp_get_class(
    image: *const Il2CppImage, namespace: *const c_char, class_name: *const c_char
) -> *mut Il2CppClass {
    il2cpp::symbols::get_class(image, CStr::from_ptr(namespace), CStr::from_ptr(class_name))
        .inspect_err(|e| error!("{}", e))
        .unwrap_or(0 as _)
}

unsafe extern "C" fn il2cpp_get_method(
    class: *mut Il2CppClass, name: *const c_char, args_count: i32
) -> *const MethodInfo {
    il2cpp::symbols::get_method(class, CStr::from_ptr(name), args_count)
        .inspect_err(|e| error!("{}", e))
        .unwrap_or(0 as _)
}

unsafe extern "C" fn il2cpp_get_method_overload(
    class: *mut Il2CppClass, name: *const c_char, params: *const Il2CppTypeEnum, param_count: usize
) -> *const MethodInfo {
    let name = CStr::from_ptr(name).to_string_lossy();
    let params = std::slice::from_raw_parts(params, param_count);
    il2cpp::symbols::get_method_overload(class, &name, params)
        .inspect_err(|e| error!("{}", e))
        .unwrap_or(0 as _)
}

unsafe extern "C" fn il2cpp_get_method_addr(
    class: *mut Il2CppClass, name: *const c_char, args_count: i32
) -> *mut c_void {
    il2cpp::symbols::get_method_addr(class, CStr::from_ptr(name), args_count) as _
}

unsafe extern "C" fn il2cpp_get_method_overload_addr(
    class: *mut Il2CppClass, name: *const c_char, params: *const Il2CppTypeEnum, param_count: usize
) -> *mut c_void {
    let name = CStr::from_ptr(name).to_string_lossy();
    let params = std::slice::from_raw_parts(params, param_count);
    il2cpp::symbols::get_method_overload_addr(class, &name, params) as _
}

unsafe extern "C" fn il2cpp_get_method_cached(
    class: *mut Il2CppClass, name: *const c_char, args_count: i32
) -> *const MethodInfo {
    il2cpp::symbols::get_method_cached(class, CStr::from_ptr(name), args_count)
        .inspect_err(|e| error!("{}", e))
        .unwrap_or(0 as _)
}

unsafe extern "C" fn il2cpp_get_method_addr_cached(
    class: *mut Il2CppClass, name: *const c_char, args_count: i32
) -> *mut c_void {
    il2cpp::symbols::get_method_addr_cached(class, CStr::from_ptr(name), args_count) as _
}

unsafe extern "C" fn il2cpp_find_nested_class(
    class: *mut Il2CppClass, name: *const c_char
) -> *mut Il2CppClass {
    il2cpp::symbols::find_nested_class(class, CStr::from_ptr(name))
        .inspect_err(|e| error!("{}", e))
        .unwrap_or(0 as _)
}

unsafe extern "C" fn il2cpp_get_field_from_name(
    class: *mut Il2CppClass, name: *const c_char
) -> *mut FieldInfo {
    il2cpp::api::il2cpp_class_get_field_from_name(class, name)
}

unsafe extern "C" fn il2cpp_get_field_value(
    obj: *mut Il2CppObject, field: *mut FieldInfo, out_value: *mut c_void
) {
    il2cpp::api::il2cpp_field_get_value(obj, field, out_value)
}

unsafe extern "C" fn il2cpp_set_field_value(
    obj: *mut Il2CppObject, field: *mut FieldInfo, value: *const c_void
) {
    il2cpp::api::il2cpp_field_set_value(obj, field, value as _)
}

unsafe extern "C" fn il2cpp_get_static_field_value(
    field: *mut FieldInfo, out_value: *mut c_void
) {
    il2cpp::api::il2cpp_field_static_get_value(field, out_value)
}

unsafe extern "C" fn il2cpp_set_static_field_value(
    field: *mut FieldInfo, value: *const c_void
) {
    il2cpp::api::il2cpp_field_static_set_value(field, value as _)
}

unsafe extern "C" fn il2cpp_unbox(obj: *mut Il2CppObject) -> *mut c_void {
    il2cpp::api::il2cpp_object_unbox(obj)
}

unsafe extern "C" fn il2cpp_get_main_thread() -> *mut Il2CppThread {
    il2cpp::symbols::Thread::main_thread().as_raw()
}

unsafe extern "C" fn il2cpp_get_attached_threads(out_size: *mut usize) -> *mut *mut Il2CppThread {
    il2cpp::api::il2cpp_thread_get_all_attached_threads(out_size)
}

unsafe extern "C" fn il2cpp_schedule_on_thread(thread: *mut Il2CppThread, callback: unsafe extern "C" fn()) {
    il2cpp::symbols::Thread::from_raw(thread).schedule(std::mem::transmute(callback));
}

unsafe extern "C" fn il2cpp_create_array(
    element_type: *mut Il2CppClass, length: il2cpp_array_size_t
) -> *mut Il2CppArray {
    il2cpp::api::il2cpp_array_new(element_type, length)
}

unsafe extern "C" fn il2cpp_get_singleton_like_instance(class: *mut Il2CppClass) -> *mut Il2CppObject {
    il2cpp::symbols::SingletonLike::new(class)
        .map(|s| s.instance())
        .unwrap_or(0 as _)
}

unsafe extern "C" fn log(level: i32, target: *const c_char, message: *const c_char) {
    let target = CStr::from_ptr(target).to_string_lossy();
    let message = CStr::from_ptr(message).to_string_lossy();
    let level = match level {
        1 => log::Level::Error,
        2 => log::Level::Warn,
        3 => log::Level::Info,
        4 => log::Level::Debug,
        5 => log::Level::Trace,

        _ => log::Level::Info
    };
    log!(target: &target, level, "{}", message);
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vtable {
    pub hachimi_instance: unsafe extern "C" fn() -> *const Hachimi,
    pub hachimi_get_interceptor: unsafe extern "C" fn(this: *const Hachimi) -> *const Interceptor,

    pub interceptor_hook: unsafe extern "C" fn(
        this: *const Interceptor, orig_addr: *mut c_void, hook_addr: *mut c_void
    ) -> *mut c_void,
    pub interceptor_hook_vtable: unsafe extern "C" fn(
        this: *const Interceptor, vtable: *mut *mut c_void, vtable_index: usize, hook_addr: *mut c_void
    ) -> *mut c_void,
    pub interceptor_get_trampoline_addr: unsafe extern "C" fn(
        this: *const Interceptor, hook_addr: *mut c_void
    ) -> *mut c_void,
    pub interceptor_unhook: unsafe extern "C" fn(this: *const Interceptor, hook_addr: *mut c_void) -> *mut c_void,

    pub il2cpp_resolve_symbol: unsafe extern "C" fn(name: *const c_char) -> *mut c_void,
    pub il2cpp_get_assembly_image: unsafe extern "C" fn(assembly_name: *const c_char) -> *const Il2CppImage,
    pub il2cpp_get_class: unsafe extern "C" fn(
        image: *const Il2CppImage, namespace: *const c_char, class_name: *const c_char
    ) -> *mut Il2CppClass,
    pub il2cpp_get_method: unsafe extern "C" fn(
        class: *mut Il2CppClass, name: *const c_char, args_count: i32
    ) -> *const MethodInfo,
    pub il2cpp_get_method_overload: unsafe extern "C" fn(
        class: *mut Il2CppClass, name: *const c_char, params: *const Il2CppTypeEnum, param_count: usize
    ) -> *const MethodInfo,
    pub il2cpp_get_method_addr: unsafe extern "C" fn(
        class: *mut Il2CppClass, name: *const c_char, args_count: i32
    ) -> *mut c_void,
    pub il2cpp_get_method_overload_addr: unsafe extern "C" fn(
        class: *mut Il2CppClass, name: *const c_char, params: *const Il2CppTypeEnum, param_count: usize
    ) -> *mut c_void,
        pub il2cpp_get_method_cached: unsafe extern "C" fn(
        class: *mut Il2CppClass, name: *const c_char, args_count: i32
    ) -> *const MethodInfo,
    pub il2cpp_get_method_addr_cached: unsafe extern "C" fn(
        class: *mut Il2CppClass, name: *const c_char, args_count: i32
    ) -> *mut c_void,
    pub il2cpp_find_nested_class: unsafe extern "C" fn(
        class: *mut Il2CppClass, name: *const c_char
    ) -> *mut Il2CppClass,
    pub il2cpp_get_field_from_name: unsafe extern "C" fn(
        class: *mut Il2CppClass, name: *const c_char
    ) -> *mut FieldInfo,
    pub il2cpp_get_field_value: unsafe extern "C" fn(
        obj: *mut Il2CppObject, field: *mut FieldInfo, out_value: *mut c_void
    ),
    pub il2cpp_set_field_value: unsafe extern "C" fn(
        obj: *mut Il2CppObject, field: *mut FieldInfo, value: *const c_void
    ),
    pub il2cpp_get_static_field_value: unsafe extern "C" fn(
        field: *mut FieldInfo, out_value: *mut c_void
    ),
    pub il2cpp_set_static_field_value: unsafe extern "C" fn(
        field: *mut FieldInfo, value: *const c_void
    ),
    pub il2cpp_unbox: unsafe extern "C" fn(obj: *mut Il2CppObject) -> *mut c_void,
    pub il2cpp_get_main_thread: unsafe extern "C" fn() -> *mut Il2CppThread,
    pub il2cpp_get_attached_threads: unsafe extern "C" fn(out_size: *mut usize) -> *mut *mut Il2CppThread,
    pub il2cpp_schedule_on_thread: unsafe extern "C" fn(thread: *mut Il2CppThread, callback: unsafe extern "C" fn()),
    pub il2cpp_create_array: unsafe extern "C" fn(
        element_type: *mut Il2CppClass, length: il2cpp_array_size_t
    ) -> *mut Il2CppArray,
    pub il2cpp_get_singleton_like_instance: unsafe extern "C" fn(class: *mut Il2CppClass) -> *mut Il2CppObject,

    pub log: unsafe extern "C" fn(level: i32, target: *const c_char, message: *const c_char),
}

impl Vtable {
    pub const VALUE: Self = Self {
        hachimi_instance,
        hachimi_get_interceptor,
        interceptor_hook,
        interceptor_hook_vtable,
        interceptor_get_trampoline_addr,
        interceptor_unhook,
        il2cpp_resolve_symbol,
        il2cpp_get_assembly_image,
        il2cpp_get_class,
        il2cpp_get_method,
        il2cpp_get_method_overload,
        il2cpp_get_method_addr,
        il2cpp_get_method_overload_addr,
        il2cpp_get_method_cached,
        il2cpp_get_method_addr_cached,
        il2cpp_find_nested_class,
        il2cpp_get_field_from_name,
        il2cpp_get_field_value,
        il2cpp_set_field_value,
        il2cpp_get_static_field_value,
        il2cpp_set_static_field_value,
        il2cpp_unbox,
        il2cpp_get_main_thread,
        il2cpp_get_attached_threads,
        il2cpp_schedule_on_thread,
        il2cpp_create_array,
        il2cpp_get_singleton_like_instance,
        log,
    };

    pub fn instantiate() -> Self {
        Self::VALUE.clone()
    }
}

pub struct Plugin {
    pub name: String,
    pub init_fn: HachimiInitFn
}

impl Plugin {
    pub fn init(&self) -> InitResult {
        (self.init_fn)(&Vtable::instantiate(), VERSION)
    }
}