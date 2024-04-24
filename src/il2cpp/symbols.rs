use std::ffi::CStr;
use std::os::raw::c_void;

use crate::symbols_impl;
use crate::core::Error;

use super::api::*;
use super::types::*;
use super::types::Il2CppClass;
use std::ptr::null_mut;

static mut HANDLE: *mut c_void = null_mut();
static mut DOMAIN: *mut Il2CppDomain = null_mut();

pub unsafe fn dlsym(name: &str) -> usize {
    symbols_impl::dlsym(HANDLE, name)
}

pub fn set_handle(handle: usize) {
    unsafe { HANDLE = handle as *mut c_void }
}

pub fn init() {
    unsafe { DOMAIN = il2cpp_domain_get() }
}

pub fn get_assembly_image(assembly_name: &CStr) -> Result<*const Il2CppImage, Error> {
    let domain = unsafe { DOMAIN };
    let assembly = il2cpp_domain_assembly_open(domain, assembly_name.as_ptr());
    if assembly.is_null() {
        Err(Error::AssemblyNotFound(assembly_name.to_str().unwrap().to_owned()))
    }
    else {
        Ok(il2cpp_assembly_get_image(assembly))
    }
}

pub fn get_class(image: *const Il2CppImage, namespace: &CStr, class_name: &CStr) -> Result<*mut Il2CppClass, Error> {
    let class = il2cpp_class_from_name(image, namespace.as_ptr(), class_name.as_ptr());
    if class.is_null() {
        Err(Error::ClassNotFound(namespace.to_str().unwrap().to_owned(), class_name.to_str().unwrap().to_owned()))
    }
    else {
        Ok(class)
    }
}

pub fn get_method(class: *mut Il2CppClass, name: &CStr, args_count: i32) -> Result<*const MethodInfo, Error> {
    let method = il2cpp_class_get_method_from_name(class, name.as_ptr(), args_count);
    if method.is_null() {
        Err(Error::MethodNotFound(name.to_str().unwrap().to_owned()))
    }
    else {
        Ok(method)
    }
}

pub fn get_method_overload(class: *mut Il2CppClass, name: &str, params: &[Il2CppTypeEnum]) -> Result<*const MethodInfo, Error> {
    let mut iter: *mut c_void = null_mut();
    
    loop {
        let method = il2cpp_class_get_methods(class, &mut iter);
        if method.is_null() {
            break;
        }

        // Check name
        let method_name = unsafe { CStr::from_ptr((*method).name) };
        if method_name.to_str().unwrap() != name {
            continue;
        }

        // Check params
        let param_count = il2cpp_method_get_param_count(method);
        if param_count != params.len() as u32 {
            continue;
        }

        let mut ok = true;
        for i in 0u32..param_count {
            let param = il2cpp_method_get_param(method, i);
            if unsafe { (*param).type_() } != params[i as usize] {
                ok = false;
                break;
            }
        }

        if ok {
            return Ok(method);
        }
    }
    
    Err(Error::MethodNotFound(name.to_owned()))
}

pub fn get_method_addr(class: *mut Il2CppClass, name: &CStr, args_count: i32) -> usize {
    let res = get_method(class, name, args_count);
    if let Ok(method) = res {
        unsafe { (*method).methodPointer }
    }
    else {
        warn!("get_method_addr: {} = NULL", name.to_str().unwrap());
        0
    }
}

pub fn get_method_overload_addr(class: *mut Il2CppClass, name: &str, params: &[Il2CppTypeEnum]) -> usize {
    let res = get_method_overload(class, name, params);
    if let Ok(method) = res {
        unsafe { (*method).methodPointer }
    }
    else {
        warn!("get_method_overload_addr: {} = NULL", name);
        0
    }
}

pub fn find_nested_class(class: *mut Il2CppClass, name: &CStr) -> Result<*mut Il2CppClass, Error> {
    let mut iter: *mut c_void = null_mut();
    loop {
        let nested_class = il2cpp_class_get_nested_types(class, &mut iter);
        if nested_class.is_null() { break; }

        let class_name = unsafe { CStr::from_ptr((*nested_class).name) };
        if class_name == name {
            return Ok(nested_class);
        }
    }

    let class_name = unsafe { CStr::from_ptr((*class).name).to_str().unwrap() };
    Err(Error::ClassNotFound(class_name.to_owned(), name.to_str().unwrap().to_owned()))
}

pub fn get_field_value<T: Default>(obj: *mut Il2CppObject, field: *mut FieldInfo) -> T {
    let mut value = T::default();
    il2cpp_field_get_value(obj, field, unsafe { std::mem::transmute(&mut value) });
    value
}

pub fn get_field_object_value<T>(obj: *mut Il2CppObject, field: *mut FieldInfo) -> *mut T {
    let mut value = null_mut();
    il2cpp_field_get_value(obj, field, unsafe { std::mem::transmute(&mut value) });
    value
}

pub fn set_field_value<T>(obj: *mut Il2CppObject, field: *mut FieldInfo, value: T) {
    il2cpp_field_set_value(obj, field, unsafe { std::mem::transmute(&value) });
}

pub fn set_field_object_value<T>(obj: *mut Il2CppObject, field: *mut FieldInfo, value: *const T) {
    il2cpp_field_set_value(obj, field, unsafe { std::mem::transmute(value) });
}

pub fn get_field_from_name(class: *mut Il2CppClass, name: &CStr) -> *mut FieldInfo {
    let field = il2cpp_class_get_field_from_name(class, name.as_ptr());
    if field.is_null() {
        warn!("get_field_from_name: {} = NULL", name.to_str().unwrap());
    }

    return field;
}

pub fn get_static_field_value<T: Default>(field: *mut FieldInfo) -> T {
    let mut value = T::default();
    il2cpp_field_static_get_value(field, unsafe { std::mem::transmute(&mut value) });
    value
}

pub fn get_static_field_object_value<T>(field: *mut FieldInfo) -> *mut T {
    let mut value = null_mut();
    il2cpp_field_static_get_value(field, unsafe { std::mem::transmute(&mut value) });
    value
}

pub unsafe fn unbox<T: Copy>(obj: *mut Il2CppObject) -> T {
    *(il2cpp_object_unbox(obj) as *mut T)
}

// IEnumerable wrapper
// (for use with arrays only)
pub struct IEnumerable<T> {
    pub this: *mut Il2CppObject,
    pub enumerator: IEnumerator<T>
}

impl<T> IEnumerable<T> {
    pub fn new(this: *mut Il2CppObject) -> Option<IEnumerable<T>> {
        if this.is_null() {
            return None;
        }

        let class = unsafe { (*this).klass() };
        let get_enumerator_addr = get_method_addr(class, cstr!("GetEnumerator"), 0);
        if get_enumerator_addr == 0 {
            return None;
        }
        
        let get_enumerator: fn(*mut Il2CppObject) -> *mut Il2CppObject = unsafe {
            std::mem::transmute(get_enumerator_addr)
        };
        let Some(enumerator) = IEnumerator::new(get_enumerator(this)) else {
            return None;
        };

        Some(IEnumerable {
            this,
            enumerator
        })
    }
}

// IEnumerator wrapper
#[allow(non_snake_case)]
pub struct IEnumerator<T> {
    pub this: *mut Il2CppObject,
    get_Current: fn(*mut Il2CppObject) -> T,
    MoveNext: fn(*mut Il2CppObject) -> bool
}

impl<T> IEnumerator<T> {
    pub fn new(this: *mut Il2CppObject) -> Option<IEnumerator<T>> {
        if this.is_null() {
            return None;
        }

        let class = unsafe { (*this).klass() };
        let get_current_addr = get_method_addr(class, cstr!("get_Current"), 0);
        let move_next_addr = get_method_addr(class, cstr!("MoveNext"), 0);

        if get_current_addr == 0 || move_next_addr == 0 {
            return None;
        }

        Some(IEnumerator {
            this,
            get_Current: unsafe { std::mem::transmute(get_current_addr) },
            MoveNext: unsafe { std::mem::transmute(move_next_addr) }
        })
    }
}

impl<T> Iterator for IEnumerator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if (self.MoveNext)(self.this) {
            Some((self.get_Current)(self.this))
        }
        else {
            None
        }
    }
}

// IList wrapper
// (because using IEnumerator on List`1[T] causes UB for some reason)
#[allow(non_snake_case)]
pub struct IList<T> {
    pub this: *mut Il2CppObject,
    get_Item: fn(*mut Il2CppObject, i32) -> T,
    set_Item: fn(*mut Il2CppObject, i32, T),
    get_Count: fn(*mut Il2CppObject) -> i32
}

impl<T> IList<T> {
    pub fn new(this: *mut Il2CppObject) -> Option<IList<T>> {
        if this.is_null() {
            return None;
        }

        let class = unsafe { (*this).klass() };
        let get_item_addr = get_method_addr(class, cstr!("get_Item"), 1);
        let set_item_addr = get_method_addr(class, cstr!("set_Item"), 2);
        let get_count_addr = get_method_addr(class, cstr!("get_Count"), 0);

        if get_item_addr == 0 || set_item_addr == 0 || get_count_addr == 0 {
            return None;
        }

        Some(IList {
            this,
            get_Item: unsafe { std::mem::transmute(get_item_addr) },
            set_Item: unsafe { std::mem::transmute(set_item_addr) },
            get_Count: unsafe { std::mem::transmute(get_count_addr) }
        })
    }

    /// Returns `None` if `i` is out of range.
    pub fn get(&self, i: i32) -> Option<T> {
        if i >= 0 && i < self.count() {
            Some((self.get_Item)(self.this, i))
        }
        else {
            None
        }
    }

    /// Returns `false` if `i` is out of range.
    pub fn set(&self, i: i32, value: T) -> bool {
        if i >= 0 && i < self.count() {
            (self.set_Item)(self.this, i, value);
            true
        }
        else {
            false
        }
    }

    pub fn count(&self) -> i32 {
        (self.get_Count)(self.this)
    }

    pub fn iter<'a>(&'a self) -> IListIter<'a, T> {
        IListIter { list: self, i: -1 }
    }
}

impl<'a, T> IntoIterator for &'a IList<T> {
    type Item = T;
    type IntoIter = IListIter<'a, T>;
    
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct IListIter<'a, T> {
    list: &'a IList<T>,
    i: i32
}

impl<'a, T> Iterator for IListIter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.i += 1;
        self.list.get(self.i)
    }
}

// IDictionary wrapper
#[allow(non_snake_case)]
pub struct IDictionary<K, V> {
    pub this: *mut Il2CppObject,
    get_Item: fn(*mut Il2CppObject, K) -> V,
    set_Item: fn(*mut Il2CppObject, K, V),
    Contains: fn(*mut Il2CppObject, K) -> bool
}

impl<K, V> IDictionary<K, V> {
    pub fn new(this: *mut Il2CppObject) -> Option<IDictionary<K, V>> {
        if this.is_null() {
            return None;
        }

        let class = unsafe { (*this).klass() };
        let get_item_addr = get_method_addr(class, cstr!("get_Item"), 1);
        let set_item_addr = get_method_addr(class, cstr!("set_Item"), 2);
        let contains_addr = get_method_addr(class, cstr!("Contains"), 1);

        if get_item_addr == 0 || set_item_addr == 0 || contains_addr == 0 {
            return None;
        }

        Some(IDictionary {
            this,
            get_Item: unsafe { std::mem::transmute(get_item_addr) },
            set_Item: unsafe { std::mem::transmute(set_item_addr) },
            Contains: unsafe { std::mem::transmute(contains_addr) }
        })
    }

    pub fn get(&self, key: K) -> V {
        (self.get_Item)(self.this, key)
    }

    pub fn set(&self, key: K, value: V) {
        (self.set_Item)(self.this, key, value);
    }

    pub fn contains(&self, key: K) -> bool {
        (self.Contains)(self.this, key)
    }
}

// Il2CppThread wrapper
#[repr(transparent)]
#[derive(Clone)]
pub struct Thread(*mut Il2CppThread);

impl Thread {
    fn sync_ctx(&self) -> *mut Il2CppObject {
        let class = unsafe { (*self.0).obj.klass() };
        let get_exec_ctx_addr = get_method_addr(class, cstr!("GetMutableExecutionContext"), 0);
        if get_exec_ctx_addr == 0 {
            return null_mut();
        }

        let get_exec_ctx: fn(*mut Il2CppObject) -> *mut Il2CppObject = unsafe {
            std::mem::transmute(get_exec_ctx_addr)
        };
        let exec_ctx = get_exec_ctx(self.0 as *mut Il2CppObject);
        let exec_ctx_class = unsafe { (*exec_ctx).klass() };

        let sync_ctx_field = il2cpp_class_get_field_from_name(exec_ctx_class, cstr!("_syncContext").as_ptr());
        if sync_ctx_field.is_null() {
            return null_mut();
        }

        get_field_object_value(exec_ctx, sync_ctx_field)
    }

    pub fn schedule(&self, callback: fn()) {
        let sync_ctx = self.sync_ctx();
        if sync_ctx.is_null() {
            error!("synchronization context is null, callback not scheduled");
            return;
        }
        let sync_ctx_class = unsafe { (*sync_ctx).klass() };

        let sync_ctx_post: fn(*mut Il2CppObject, *mut Il2CppDelegate, *mut Il2CppObject) = unsafe {
            std::mem::transmute(get_method_addr(sync_ctx_class, cstr!("Post"), 2))
        };

        let mscorlib = get_assembly_image(cstr!("mscorlib.dll")).expect("mscorlib");
        let delegate_class = get_class(mscorlib, cstr!("System.Threading"), cstr!("SendOrPostCallback")).expect("SendOrPostCallback");
        let delegate_invoke = get_method(delegate_class, cstr!("Invoke"), 1).expect("SendOrPostCallback.Invoke");
        let delegate_ctor: fn(*mut Il2CppObject, *mut Il2CppObject, *const MethodInfo) = unsafe {
            std::mem::transmute(get_method_addr(delegate_class, cstr!(".ctor"), 2))
        };

        let delegate_obj = il2cpp_object_new(delegate_class);
        delegate_ctor(delegate_obj, delegate_obj, delegate_invoke);
        let delegate = delegate_obj as *mut Il2CppDelegate;
        unsafe {
            (*delegate).method_ptr = callback as usize;
        }

        sync_ctx_post(sync_ctx, delegate, null_mut());
    }

    pub fn attached_threads() -> &'static [Thread] {
        let mut size = 0;
        let list_ptr = il2cpp_thread_get_all_attached_threads(&mut size);
        unsafe { std::slice::from_raw_parts(list_ptr as *const Thread, size) }
    }

    pub fn main_thread() -> Thread {
        Self::attached_threads().get(0).expect("main thread must be present").clone()
    }
}

// MonoSingleton wrapper
pub struct MonoSingleton {
    instance_field: *mut FieldInfo,
}

impl MonoSingleton {
    pub fn new(class: *mut Il2CppClass) -> Option<MonoSingleton> {
        let instance_field = get_field_from_name(class, cstr!("_instance"));
        if instance_field.is_null() {
            return None;
        }

        Some(MonoSingleton {
            instance_field
        })
    }

    pub fn instance(&self) -> *mut Il2CppObject {
        get_static_field_object_value(self.instance_field)
    }
}