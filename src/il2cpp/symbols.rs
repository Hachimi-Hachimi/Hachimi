use std::borrow::Cow;
use std::collections::hash_map;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::os::raw::c_void;
use std::sync::Mutex;

use fnv::FnvHashMap;
use once_cell::sync::Lazy;

use crate::core::Hachimi;
use crate::symbols_impl;
use crate::core::Error;

use super::api::*;
use super::ext::Il2CppObjectExt;
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

pub static METHOD_CACHE: Lazy<
    Mutex<FnvHashMap<usize, FnvHashMap<(Cow<'_, CStr>, i32), usize>>>
> = Lazy::new(|| Mutex::default());

pub fn get_method_cached(class: *mut Il2CppClass, name: &CStr, args_count: i32) -> Result<*const MethodInfo, Error> {
    let mut cache = METHOD_CACHE.lock().unwrap();
    let entries = match cache.entry(class as usize) {
        hash_map::Entry::Occupied(e) => {
            if let Some(addr) = e.get().get(&(name.into(), args_count)) {
                if *addr == 0 {
                    // Only error that get_method returns
                    return Err(Error::MethodNotFound(name.to_str().unwrap().to_owned()));
                }
                else {
                    return Ok(*addr as *const MethodInfo);
                }
            }
            e.into_mut()
        },
        hash_map::Entry::Vacant(e) => e.insert(FnvHashMap::default())
    };
    let res = get_method(class, name, args_count);
    let addr = match res {
        Ok(addr) => addr as usize,
        Err(_) => 0
    };
    entries.insert((name.to_owned().into(), args_count), addr);
    res
}

pub fn get_method_addr_cached(class: *mut Il2CppClass, name: &CStr, args_count: i32) -> usize {
    let res = get_method_cached(class, name, args_count);
    if let Ok(method) = res {
        unsafe { (*method).methodPointer }
    }
    else {
        warn!("get_method_addr_cached: {} = NULL", name.to_str().unwrap());
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

pub fn get_field_value<T>(obj: *mut Il2CppObject, field: *mut FieldInfo) -> T {
    let mut value = MaybeUninit::uninit();
    il2cpp_field_get_value(obj, field, unsafe { std::mem::transmute(&mut value) });
    unsafe { value.assume_init() }
}

pub fn get_field_object_value<T>(obj: *mut Il2CppObject, field: *mut FieldInfo) -> *mut T {
    get_field_value(obj, field)
}

pub fn get_field_ptr<T>(obj: *mut Il2CppObject, field: *mut FieldInfo) -> *mut T {
    unsafe { (obj as usize + (*field).offset as usize) as _ }
}

pub fn set_field_value<T>(obj: *mut Il2CppObject, field: *mut FieldInfo, value: &T) {
    il2cpp_field_set_value(obj, field, std::ptr::from_ref(value) as _);
}

pub fn set_field_object_value<T>(obj: *mut Il2CppObject, field: *mut FieldInfo, value: *const T) {
    il2cpp_field_set_value(obj, field, value as _);
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

#[repr(transparent)]
pub struct IEnumerable<T = *mut Il2CppObject> {
    pub this: *mut Il2CppObject,
    _phantom: PhantomData<T>
}

impl<T> IEnumerable<T> {
    pub fn enumerator(&self) -> Option<IEnumerator> {
        if self.this.is_null() {
            return None;
        }

        let class = unsafe { (*self.this).klass() };
        let get_enumerator_addr = get_method_addr_cached(class, c"GetEnumerator", 0);
        if get_enumerator_addr == 0 {
            return None;
        }
        
        let get_enumerator: extern "C" fn(*mut Il2CppObject) -> *mut Il2CppObject = unsafe {
            std::mem::transmute(get_enumerator_addr)
        };

        Some(IEnumerator::from(get_enumerator(self.this)))
    }
}

impl<T> From<*mut Il2CppObject> for IEnumerable<T> {
    fn from(value: *mut Il2CppObject) -> Self {
        IEnumerable {
            this: value,
            _phantom: PhantomData
        }
    }
}

#[repr(transparent)]
pub struct IEnumerator<T = *mut Il2CppObject> {
    pub this: *mut Il2CppObject,
    _phantom: PhantomData<T>
}

pub type MoveNextFn = extern "C" fn(*mut Il2CppObject) -> bool;

impl<T> IEnumerator<T> {
    pub fn iter(&self) -> Option<IEnumeratorIterator<T>> {
        if self.this.is_null() {
            return None;
        }

        let class = unsafe { (*self.this).klass() };
        // Get addr manually to avoid nullptr warning
        let get_current_method = get_method_cached(class, c"get_Current", 0);
        let get_current_addr = get_current_method.map(|m| unsafe { (*m).methodPointer }).unwrap_or(0);
        let move_next_addr = get_method_addr_cached(class, c"MoveNext", 0);

        if move_next_addr == 0 {
            return None;
        }

        Some(IEnumeratorIterator {
            this: self.this,
            get_Current: unsafe { std::mem::transmute(get_current_addr) },
            MoveNext: unsafe { std::mem::transmute(move_next_addr) }
        })
    }

    pub fn hook_move_next(&self, hook_fn: MoveNextFn) -> Result<usize, Error> {
        let class = unsafe { (*self.this).klass() };
        let move_next_addr = get_method_addr_cached(class, c"MoveNext", 0);

        if move_next_addr == 0 {
            return Err(Error::MethodNotFound("MoveNext".to_owned()));
        }

        Hachimi::instance().interceptor.hook(move_next_addr, hook_fn as usize)
    }
}

impl<T> From<*mut Il2CppObject> for IEnumerator<T> {
    fn from(value: *mut Il2CppObject) -> Self {
        IEnumerator {
            this: value,
            _phantom: PhantomData
        }
    }
}

#[allow(non_snake_case)]
pub struct IEnumeratorIterator<T> {
    this: *mut Il2CppObject,
    get_Current: Option<extern "C" fn(*mut Il2CppObject) -> T>,
    MoveNext: MoveNextFn
}

impl<T> Iterator for IEnumeratorIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        // TODO: properly handle enumerators that returns nothing
        let Some(get_current) = self.get_Current else {
            return None;
        };

        if (self.MoveNext)(self.this) {
            Some(get_current(self.this))
        }
        else {
            None
        }
    }
}

#[allow(non_snake_case)]
pub struct IList<T = *mut Il2CppObject> {
    pub this: *mut Il2CppObject,
    get_Item: extern "C" fn(*mut Il2CppObject, i32) -> T,
    set_Item: extern "C" fn(*mut Il2CppObject, i32, T),
    get_Count: extern "C" fn(*mut Il2CppObject) -> i32
}

impl<T> IList<T> {
    pub fn new(this: *mut Il2CppObject) -> Option<IList<T>> {
        if this.is_null() {
            return None;
        }

        let class = unsafe { (*this).klass() };
        let get_item_addr = get_method_addr_cached(class, c"get_Item", 1);
        let set_item_addr = get_method_addr_cached(class, c"set_Item", 2);
        let get_count_addr = get_method_addr_cached(class, c"get_Count", 0);

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

impl<T> Into<Vec<T>> for IList<T> {
    fn into(self) -> Vec<T> {
        self.iter().collect()
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
    get_Item: extern "C" fn(*mut Il2CppObject, K) -> V,
    set_Item: extern "C" fn(*mut Il2CppObject, K, V),
    Contains: extern "C" fn(*mut Il2CppObject, K) -> bool
}

impl<K, V> IDictionary<K, V> {
    pub fn new(this: *mut Il2CppObject) -> Option<IDictionary<K, V>> {
        if this.is_null() {
            return None;
        }

        let class = unsafe { (*this).klass() };
        let get_item_addr = get_method_addr_cached(class, c"get_Item", 1);
        let set_item_addr = get_method_addr_cached(class, c"set_Item", 2);
        let contains_addr = get_method_addr_cached(class, c"Contains", 1);

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
    pub fn from_raw(ptr: *mut Il2CppThread) -> Self {
        Self(ptr)
    }

    fn sync_ctx(&self) -> *mut Il2CppObject {
        let class = unsafe { (*self.0).obj.klass() };
        let get_exec_ctx_addr = get_method_addr_cached(class, c"GetMutableExecutionContext", 0);
        if get_exec_ctx_addr == 0 {
            return null_mut();
        }

        let get_exec_ctx: extern "C" fn(*mut Il2CppObject) -> *mut Il2CppObject = unsafe {
            std::mem::transmute(get_exec_ctx_addr)
        };
        let exec_ctx = get_exec_ctx(self.0 as *mut Il2CppObject);
        let exec_ctx_class = unsafe { (*exec_ctx).klass() };

        let sync_ctx_field = il2cpp_class_get_field_from_name(exec_ctx_class, c"_syncContext".as_ptr());
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

        let sync_ctx_post: extern "C" fn(*mut Il2CppObject, *mut Il2CppDelegate, *mut Il2CppObject) = unsafe {
            std::mem::transmute(get_method_addr_cached(sync_ctx_class, c"Post", 2))
        };

        let mscorlib = get_assembly_image(c"mscorlib.dll").expect("mscorlib");
        let delegate_class = get_class(mscorlib, c"System.Threading", c"SendOrPostCallback").expect("SendOrPostCallback");
        let delegate = create_delegate(delegate_class, 1, callback).unwrap();

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

    pub fn as_raw(&self) -> *mut Il2CppThread {
        self.0
    }
}

// Delegate creation
pub fn create_delegate(delegate_class: *mut Il2CppClass, args_count: i32, method_ptr: fn()) -> Option<*mut Il2CppDelegate> {
    let delegate_invoke = get_method_cached(delegate_class, c"Invoke", args_count).ok()?;
    
    let delegate_ctor_addr = get_method_addr_cached(delegate_class, c".ctor", 2);
    if delegate_ctor_addr == 0 {
        return None;
    }
    let delegate_ctor: extern "C" fn(*mut Il2CppObject, *mut Il2CppObject, *const MethodInfo) = unsafe {
        std::mem::transmute(delegate_ctor_addr)
    };

    let delegate_obj = il2cpp_object_new(delegate_class);
    delegate_ctor(delegate_obj, delegate_obj, delegate_invoke);
    let delegate = delegate_obj as *mut Il2CppDelegate;
    unsafe {
        (*delegate).method_ptr = method_ptr as _;
        (*delegate).invoke_impl = method_ptr as _;
    }

    Some(delegate)
}

// Singleton-like class wrapper
pub struct SingletonLike {
    instance_field: *mut FieldInfo,
}

impl SingletonLike {
    pub fn new(class: *mut Il2CppClass) -> Option<SingletonLike> {
        let instance_field = get_field_from_name(class, c"_instance");
        if instance_field.is_null() {
            return None;
        }

        Some(SingletonLike {
            instance_field
        })
    }

    pub fn instance(&self) -> *mut Il2CppObject {
        get_static_field_object_value(self.instance_field)
    }
}

// GCHandle wrapper
#[repr(transparent)]
pub struct GCHandle(u32);

impl GCHandle {
    pub fn new(obj: *mut Il2CppObject, pinned: bool) -> GCHandle {
        GCHandle(il2cpp_gchandle_new(obj, pinned))
    }

    pub fn new_weak_ref(obj: *mut Il2CppObject, track_resurrection: bool) -> GCHandle {
        GCHandle(il2cpp_gchandle_new_weakref(obj, track_resurrection))
    }

    pub fn target(&self) -> *mut Il2CppObject {
        il2cpp_gchandle_get_target(self.0)
    }
}

impl Drop for GCHandle {
    fn drop(&mut self) {
        il2cpp_gchandle_free(self.0);
    }
}

// Il2CppArray wrapper
#[repr(transparent)]
pub struct Array<T = *mut Il2CppObject> {
    pub this: *mut Il2CppArray,
    _phantom: PhantomData<T>
}

impl<T> Array<T> {
    pub fn new(element_type: *mut Il2CppClass, length: il2cpp_array_size_t) -> Array<T> {
        Array {
            this: il2cpp_array_new(element_type, length),
            _phantom: PhantomData,
        }
    }

    pub unsafe fn data_ptr(&self) -> *mut T {
        self.this.add(1) as _
    }

    pub unsafe fn as_slice(&self) -> &mut [T] {
        std::slice::from_raw_parts_mut(self.data_ptr(), (*self.this).max_length)
    }

    pub fn len(&self) -> usize {
        unsafe { (*self.this).max_length }
    }
}

impl<T> Into<*mut Il2CppArray> for Array<T> {
    fn into(self) -> *mut Il2CppArray {
        self.this
    }
}

impl<T> From<*mut Il2CppArray> for Array<T> {
    fn from(value: *mut Il2CppArray) -> Self {
        Self {
            this: value,
            _phantom: PhantomData
        }
    }
}

pub struct FieldsIter {
    class: *mut Il2CppClass,
    iter: *mut c_void
}

impl FieldsIter {
    pub fn new(class: *mut Il2CppClass) -> Self {
        Self {
            class,
            iter: null_mut()
        }
    }
}

impl Iterator for FieldsIter {
    type Item = *mut FieldInfo;

    fn next(&mut self) -> Option<Self::Item> {
        let field = il2cpp_class_get_fields(self.class, &mut self.iter);
        if field.is_null() {
            return None;
        }
        Some(field)
    }
}

#[repr(C)]
pub struct Il2CppDictionary {
    pub obj: Il2CppObject,
    pub buckets: *mut Il2CppArray,
    pub entries: *mut Il2CppArray,
    pub count: i32,
    /* STUB */
}

#[repr(C)]
pub struct Il2CppDictionaryEntry<K, V> {
    pub hash_code: i32,
    pub next: i32,
    pub key: K,
    pub value: V
}

// Generic Dictionary wrapper
#[repr(transparent)]
pub struct Dictionary<K, V> {
    pub this: *mut Il2CppDictionary,
    _k: PhantomData<K>,
    _v: PhantomData<V>
}

impl<K, V> Into<*mut Il2CppDictionary> for Dictionary<K, V> {
    fn into(self) -> *mut Il2CppDictionary {
        self.this
    }
}

impl<K, V> From<*mut Il2CppDictionary> for Dictionary<K, V> {
    fn from(value: *mut Il2CppDictionary) -> Self {
        Self {
            this: value,
            _k: PhantomData,
            _v: PhantomData
        }
    }
}

impl<K, V> Dictionary<K, V> {
    pub fn buckets(&self) -> Array<i32> {
        unsafe { (*self.this).buckets.into() }
    }

    pub fn entries(&self) -> Array<Il2CppDictionaryEntry<K, V>> {
        unsafe { (*self.this).entries.into() }
    }

    pub fn count(&self) -> i32 {
        unsafe { (*self.this).count }
    }
}

impl<K: PartialEq, V> Dictionary<K, V> {
    pub fn find_entry(&self, key: &K) -> Option<&'static mut Il2CppDictionaryEntry<K, V>> {
        for entry in unsafe { self.entries().as_slice().iter_mut() } {
            if entry.key == *key {
                // freaky lifetime erasure
                return unsafe { std::ptr::from_mut(entry).as_mut() };
            }
        }

        None
    }
}

impl<K: PartialEq + 'static, V> Dictionary<K, V> {
    pub fn get(&self, key: &K) -> Option<&'static mut V> {
        self.find_entry(&key).map(|e| &mut e.value)
    }
}