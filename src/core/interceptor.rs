use std::{any::Any, collections::{hash_map, BTreeMap}, ops::Deref, os::raw::c_void, sync::Arc};

use fnv::FnvHashMap;
use libffi::{middle::{Cif, Closure}, raw::{ffi_call, ffi_cif}};

use crate::interceptor_impl;

use super::{Error, Hachimi};

#[derive(Default)]
pub struct Interceptor {
    hooks: Vec<HookHandle>,
    hooks_by_orig_addr: FnvHashMap<usize, usize>,
    hooks_by_hook_addr: FnvHashMap<usize, usize>,
    ffi_hooks: FnvHashMap<usize, FfiHookHandle>
}

pub struct HookHandle {
    pub orig_addr: usize,
    pub hook_addr: usize,
    pub trampoline_addr: usize,
    pub hook_type: HookType,
    pub is_ffi_root_hook: bool,
    pub orig_hook_addr: Option<usize> // Set when a normal hook is already done and an ffi hook takes over
}

pub type FfiUserData = Arc<dyn Any + Send + Sync>;

struct FfiHookHandle {
    pub root_hook: usize,
    #[allow(dead_code)] // Kept around so the closure doesnt get deallocated
    pub closure: ClosureWrapper,
    pub children: BTreeMap<usize, (FfiHookFn, FfiUserData)> // (hook, userdata ptr) -> userdata
}

struct ClosureWrapper(Closure<'static>);

unsafe impl Send for ClosureWrapper {}

impl Deref for ClosureWrapper {
    type Target = Closure<'static>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl HookHandle {
    unsafe fn unhook(&self) -> Result<(), Error> {
        match self.hook_type {
            HookType::Function => interceptor_impl::unhook(self),
            HookType::Vtable => interceptor_impl::unhook_vtable(self)
        }
    }
}

pub enum HookType {
    Function,
    Vtable
}

pub enum HookOrig {
    Function(usize),
    Vtable {
        vtable: *mut usize,
        index: usize
    }
}

impl HookOrig {
    pub fn addr(&self) -> usize {
        match self {
            HookOrig::Function(addr) => *addr,
            HookOrig::Vtable { vtable, index } => unsafe {
                interceptor_impl::get_vtable_entry(*vtable, *index)
            },
        }
    }
}

impl From<usize> for HookOrig {
    fn from(value: usize) -> Self {
        HookOrig::Function(value)
    }
}

impl From<(*mut usize, usize)> for HookOrig {
    fn from((vtable, index): (*mut usize, usize)) -> Self {
        Self::Vtable { vtable, index }
    }
}

pub type FfiHookFn = fn(
    cif: &ffi_cif,
    result: &mut c_void,
    args: *const *const c_void,
    next: &FfiNext,
    userdata: FfiUserData,
    id: usize
);

pub enum FfiNext {
    Hook {
        f: FfiHookFn,
        next: Box<FfiNext>,
        userdata: FfiUserData,
        id: usize
    },
    Root(usize)
}

impl FfiNext {
    pub fn call(&self,
        cif: &ffi_cif,
        result: &mut c_void,
        args: *const *const c_void
    ) {
        match self {
            FfiNext::Hook { f, next, userdata, id } => f(cif, result, args, &next, userdata.clone(), *id),
            FfiNext::Root(addr) => unsafe { ffi_call(
                cif as *const _ as *mut _,
                Some(std::mem::transmute(*addr)),
                result,
                args as _
            )}
        }
    }
}

impl Interceptor {
    pub fn hook(&mut self, orig: impl Into<HookOrig>, hook_addr: usize) -> Result<usize, Error> {
        match self.hooks_by_hook_addr.entry(hook_addr) {
            hash_map::Entry::Occupied(e) => {
                let hook = &mut self.hooks[*e.get()];
                if hook.is_ffi_root_hook {
                    if let Some(addr) = hook.orig_hook_addr {
                        return if addr == hook_addr {
                            Ok(hook.trampoline_addr)
                        }
                        else {
                            Err(Error::AlreadyHooked)
                        }
                    }
                    else {
                        hook.orig_hook_addr = Some(hook_addr);
                    }
                }
                Ok(hook.trampoline_addr)
            },
            hash_map::Entry::Vacant(e) => {
                let orig = orig.into();
                let by_orig_addr_entry = match self.hooks_by_orig_addr.entry(orig.addr()) {
                    hash_map::Entry::Occupied(_) => return Err(Error::AlreadyHooked),
                    hash_map::Entry::Vacant(e) => e
                };

                let hook_handle = match orig {
                    HookOrig::Function(orig_addr) => unsafe {
                        interceptor_impl::hook(orig_addr, hook_addr)?
                    },
                    HookOrig::Vtable { vtable, index } => unsafe {
                        interceptor_impl::hook_vtable(vtable, index, hook_addr)?
                    },
                };
                let trampoline_addr = hook_handle.trampoline_addr;

                let index = self.hooks.len();
                self.hooks.push(hook_handle);
                e.insert(index);
                by_orig_addr_entry.insert(index);

                Ok(trampoline_addr)
            }
        }
    }

    /// Returns an id (determined by the userdata arc) to unhook it
    pub fn hook_ffi<T: 'static + Send + Sync>(
        &mut self,
        orig: impl Into<HookOrig>,
        cif: Cif,
        hook_fn: FfiHookFn,
        userdata: T
    ) -> Result<usize, Error> {
        let orig = orig.into();
        let orig_addr = match orig {
            HookOrig::Function(addr) => addr,
            HookOrig::Vtable { vtable, index } => unsafe {
                interceptor_impl::get_vtable_entry(vtable, index)
            },
        };
        match self.ffi_hooks.entry(orig_addr) {
            hash_map::Entry::Occupied(mut e) => {
                let hook = e.get_mut();
                let userdata = Arc::new(userdata);
                let id = hook.children.last_entry().map(|e| *e.key() + 1).unwrap_or(0);
                hook.children.insert(id, (hook_fn, userdata));
                Ok(id)
            },

            hash_map::Entry::Vacant(e) => {
                match self.hooks_by_orig_addr.entry(orig_addr) {
                    // Hook already exists, but not an ffi hook yet
                    hash_map::Entry::Occupied(e2) => {
                        let index = *e2.get();
                        let hook = &mut self.hooks[index];

                        // Unhook current hook (without removing it)
                        unsafe { hook.unhook()?; }

                        // Create root ffi hook
                        let (new_hook, closure) = Self::create_ffi_root_hook(&orig, cif)?;

                        // Modify the existing hook
                        let orig_hook_addr = hook.hook_addr;
                        hook.is_ffi_root_hook = true;
                        hook.orig_hook_addr = Some(orig_hook_addr);
                        hook.hook_addr = new_hook.hook_addr;
                        hook.trampoline_addr = new_hook.trampoline_addr;

                        // It would make sense to do this, but:
                        // - the original hook still needs to reference itself by its own address
                        // - the ffi hook doesn't need its hook address mapped
                        // so we're keeping it the same
                        /*
                        self.hooks_by_hook_addr.remove(&orig_hook_addr);
                        self.hooks_by_hook_addr.insert(new_hook.hook_addr, index);
                        */

                        let userdata = Arc::new(userdata);
                        e.insert(FfiHookHandle {
                            root_hook: index,
                            closure,
                            children: [(0, (hook_fn, userdata as FfiUserData))].into_iter().collect()
                        });

                        Ok(0)
                    },

                    hash_map::Entry::Vacant(e2) => {
                        let (hook, closure) = Self::create_ffi_root_hook(&orig, cif)?;
                        let index = self.hooks.len();
                        self.hooks.push(hook);
                        e2.insert(index); // Used to prevent multiple hooks to the same orig function
                        // No need to insert it into the hook addr map, we don't use it

                        let userdata = Arc::new(userdata);
                        e.insert(FfiHookHandle {
                            root_hook: index,
                            closure,
                            children: [(0, (hook_fn, userdata as FfiUserData))].into_iter().collect()
                        });

                        Ok(0)
                    }
                }
            }
        }
    }

    fn create_ffi_root_hook(orig: &HookOrig, cif: Cif) -> Result<(HookHandle, ClosureWrapper), Error> {
        let closure = Closure::new(cif, Self::ffi_hook_callback, unsafe { std::mem::transmute(orig.addr()) });
        let hook_addr = *closure.code_ptr() as usize;
        Ok((
            match orig {
                HookOrig::Function(orig_addr) => unsafe {
                    interceptor_impl::hook(*orig_addr, hook_addr)?
                },
                HookOrig::Vtable { vtable, index } => unsafe {
                    interceptor_impl::hook_vtable(*vtable, *index, hook_addr)?
                },
            },
            ClosureWrapper(closure)
        ))
    }

    unsafe extern "C" fn ffi_hook_callback(
        cif: &ffi_cif,
        result: &mut c_void,
        args: *const *const c_void,
        userdata: &usize // dummy ref type, pls ignore
    ) {
        let orig_addr = std::mem::transmute(userdata);

        // Get hook handle
        let hachimi = Hachimi::instance();
        let mut next: Box<FfiNext>;
        {
            let interceptor = hachimi.interceptor();
            let ffi_hook = interceptor.ffi_hooks.get(&orig_addr).unwrap();
            let hook = &interceptor.hooks[ffi_hook.root_hook];

            // Prepare calls
            next = Box::new(FfiNext::Root(hook.orig_hook_addr.unwrap_or_else(|| hook.trampoline_addr)));

            // Yup, that's a Linked List:tm:
            // Last hook will be called first, and then it's up to its mercy to call the other ones
            for (id, (child_hook, userdata)) in ffi_hook.children.iter() {
                next = Box::new(FfiNext::Hook {
                    f: *child_hook,
                    next,
                    userdata: userdata.clone(),
                    id: *id
                });
            }
        } // interceptor drops here

        next.call(cif, result, args);
    }

    pub fn get_trampoline_addr(&self, hook_addr: usize) -> usize {
        if let Some(index) = self.hooks_by_hook_addr.get(&hook_addr) {
            self.hooks[*index].trampoline_addr
        }
        else {
            warn!("Attempted to get invalid hook: {}", hook_addr);
            0
        }
    }

    pub fn unhook(&mut self, hook_addr: usize) -> Option<usize> {
        let Some(index) = self.hooks_by_hook_addr.remove(&hook_addr) else {
            return None;
        };

        if self.hooks[index].is_ffi_root_hook {
            // FFI-replaced hook is still mapped by its original hook address
            let hook = &mut self.hooks[index];
            let removed = hook.orig_hook_addr.take().is_some();
            if removed {
                // Check if the FFI hook is completely empty
                if self.ffi_hooks.get(&hook.orig_addr)
                    .is_some_and(|h| !h.children.is_empty())
                {
                    return Some(hook.orig_addr);
                }
                match self.ffi_hooks.entry(hook.orig_addr) {
                    hash_map::Entry::Occupied(e) => {
                        if !e.get().children.is_empty() {
                            return Some(hook.orig_addr);
                        }
                        else {
                            // If it is empty, then continue to unhook the root ffi hook
                            e.remove();
                        }
                    },
                    hash_map::Entry::Vacant(_) => (),
                }
            }
            else {
                return None;
            }
        }

        let hook = self.hooks.swap_remove(index);
        self.hooks_by_orig_addr.remove(&hook.orig_addr);

        if let Err(e) = unsafe { hook.unhook() } {
            error!("Failed to unhook {}: {}", hook.orig_addr, e);
        }

        self.update_swapped_hook_indexes(index);
        
        Some(hook.orig_addr)
    }

    fn update_swapped_hook_indexes(&mut self, swapped_index: usize) {
        let Some(hook) = self.hooks.get(swapped_index) else {
            return;
        };
        if let Some(index) = self.hooks_by_hook_addr.get_mut(&hook.hook_addr) {
            *index = swapped_index;
        }
        if let Some(index) = self.hooks_by_orig_addr.get_mut(&hook.orig_addr) {
            *index = swapped_index;
        }
        if let Some(hook) = self.ffi_hooks.get_mut(&hook.orig_addr) {
            hook.root_hook = swapped_index;
        }
    }

    pub fn unhook_ffi(&mut self, orig: impl Into<HookOrig>, id: usize) -> Option<(FfiHookFn, Arc<dyn Any + Send + Sync>)> {
        let orig_addr = orig.into().addr();
        match self.ffi_hooks.entry(orig_addr) {
            hash_map::Entry::Occupied(mut e) => {
                let hook = e.get_mut();
                let removed = hook.children.remove(&id);
                if hook.children.is_empty() {
                    let index = hook.root_hook;
                    let root_hook = &self.hooks[index];
                    if root_hook.orig_hook_addr.is_none() {
                        // Unhook the root hook if it's completely empty
                        if let Err(e) = unsafe { root_hook.unhook() } {
                            error!("Failed to unhook {}: {}", root_hook.orig_addr, e);
                        }

                        self.hooks_by_orig_addr.remove(&root_hook.orig_addr);
                        self.hooks.swap_remove(index);
                        e.remove();
                        self.update_swapped_hook_indexes(index);
                    }
                }
                removed
            },
            hash_map::Entry::Vacant(_) => None
        }
    }

    pub fn unhook_all(&mut self) {
        for hook in self.hooks.drain(..) {
            if let Err(e) = unsafe { hook.unhook() } {
                error!("Failed to unhook {}: {}", hook.orig_addr, e);
            }
        }
        self.hooks_by_hook_addr.clear();
        self.hooks_by_orig_addr.clear();
        self.ffi_hooks.clear();
    }

    pub fn get_vtable_from_instance(instance_addr: usize) -> *mut usize {
        unsafe { interceptor_impl::get_vtable_from_instance(instance_addr) }
    }

    pub fn find_symbol_by_name(module: &str, symbol: &str) -> Result<usize, Error> {
        unsafe { interceptor_impl::find_symbol_by_name(module, symbol) }
    }
}

macro_rules! get_orig_fn {
    ($hook:ident, $type:tt) => (
        unsafe {
            let hachimi = crate::core::Hachimi::instance();
            let interceptor = hachimi.interceptor();
            let res: $type = std::mem::transmute(interceptor.get_trampoline_addr($hook as usize));
            std::mem::drop(interceptor);
            res
        }
    )
}