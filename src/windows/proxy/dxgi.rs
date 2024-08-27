#![allow(non_snake_case)]
// These are meant to be hook functions, but they also act as proxies.

use std::{os::raw::{c_uint, c_void}, ptr::null_mut, sync::atomic::{self, AtomicIsize, AtomicUsize}};

use widestring::{U16CString, Utf16Str};
use windows::{
    core::{IUnknown, Interface, GUID, HRESULT, PCWSTR},
    Win32::{
        Foundation::{E_NOTIMPL, HWND, S_OK},
        Graphics::Dxgi::IDXGIFactory2,
        System::LibraryLoader::LoadLibraryW
    }
};

use crate::{core::{Hachimi, Interceptor}, windows::utils};

#[no_mangle]
pub extern "C" fn CreateDXGIFactory(riid: *mut GUID, pp_factory: *mut *mut c_void) -> HRESULT {
    debug!("CreateDXGIFactory");
    // DXGI 1.1 should always be available so just call it anyways
    CreateDXGIFactory1(riid, pp_factory)
}

type CreateDXGIFactory1Fn = extern "C" fn(riid: *mut GUID, pp_factory: *mut *mut c_void) -> HRESULT;
#[no_mangle]
pub extern "C" fn CreateDXGIFactory1(riid: *mut GUID, pp_factory: *mut *mut c_void) -> HRESULT {
    debug!("CreateDXGIFactory1");
    let res = get_orig_fn!(CreateDXGIFactory1, CreateDXGIFactory1Fn)(riid, pp_factory);
    if res.is_ok() {
        hook_factory(unsafe { *pp_factory });
    }
    else {
        error!("CreateDXGIFactory1: {}", res.message());
    }
    res
}

type CreateDXGIFactory2Fn = extern "C" fn(flags: c_uint, riid: *mut GUID, pp_factory: *mut *mut c_void) -> HRESULT;
#[no_mangle]
pub extern "C" fn CreateDXGIFactory2(flags: c_uint, riid: *mut GUID, pp_factory: *mut *mut c_void) -> HRESULT {
    debug!("CreateDXGIFactory2");
    let res = get_orig_fn!(CreateDXGIFactory2, CreateDXGIFactory2Fn)(flags, riid, pp_factory);
    if res.is_ok() {
        hook_factory(unsafe { *pp_factory });
    }
    else {
        error!("CreateDXGIFactory2: {}", res.message());
    }
    res
}

fn hook_factory(p_factory: *mut c_void) {
    let hachimi = Hachimi::instance();
    // Seems like Unity doesn't use this for d3d11? leaving it here just in case
    /*
    let vtable = Interceptor::get_vtable_from_instance(p_factory as usize);
    if let Err(e) = hachimi.interceptor.hook_vtable(vtable, 10, IDXGIFactory_CreateSwapChain as usize) {
        error!("hook_factory - IDXGIFactory: {}", e);
    }
    */

    let com_obj = unsafe { IUnknown::from_raw(p_factory) };
    let mut factory2 = null_mut();
    if unsafe { com_obj.query(&IDXGIFactory2::IID, &mut factory2).is_ok() } {
        let vtable = Interceptor::get_vtable_from_instance(factory2 as usize);
        if let Err(e) = hachimi.interceptor.hook_vtable(vtable, 15, IDXGIFactory2_CreateSwapChainForHwnd as usize) {
            error!("hook_factory - IDXGIFactory2: {}", e);
        }
    }
}

// Will be used by gui_impl
static SWAP_CHAIN_VTABLE: AtomicUsize = AtomicUsize::new(0);
pub fn get_swap_chain_vtable() -> *mut usize {
    SWAP_CHAIN_VTABLE.load(atomic::Ordering::Relaxed) as *mut usize
}

static SWAP_CHAIN_HWND: AtomicIsize = AtomicIsize::new(0);
pub fn get_swap_chain_hwnd() -> HWND {
    HWND(SWAP_CHAIN_HWND.load(atomic::Ordering::Relaxed) as _)
}

/*
type CreateSwapChainFn = extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *mut *mut c_void) -> HRESULT;
extern "C" fn IDXGIFactory_CreateSwapChain(this: *mut c_void, p_device: *mut c_void, p_desc: *mut c_void, pp_swap_chain: *mut *mut c_void) -> HRESULT {
    debug!("IDXGIFactory_CreateSwapChain");
    let res = get_orig_fn!(IDXGIFactory_CreateSwapChain, CreateSwapChainFn)(this, p_device, p_desc, pp_swap_chain);
    if res.is_ok() {
        let p_swap_chain = unsafe { *pp_swap_chain };
        let vtable = Interceptor::get_vtable_from_instance(p_swap_chain as usize);
        IDXGISWAPCHAIN_VTABLE.store(vtable as usize, atomic::Ordering::Relaxed);

        info!("Got IDXGISwapChain vtable");
        Hachimi::instance().interceptor.unhook(IDXGIFactory_CreateSwapChain as usize);
    }
    else {
        error!("IDXGIFactory_CreateSwapChain: {}", res.message());
    }
    res
}
*/

type CreateSwapChainForHwndFn = extern "C" fn(
    this: *mut c_void, p_device: *mut c_void, hwnd: HWND, p_desc: *mut c_void, p_fullscreen_desc: *mut c_void,
    p_restrict_to_output: *mut c_void, pp_swap_chain: *mut *mut c_void
) -> HRESULT;
extern "C" fn IDXGIFactory2_CreateSwapChainForHwnd(
    this: *mut c_void, p_device: *mut c_void, hwnd: HWND, p_desc: *mut c_void, p_fullscreen_desc: *mut c_void,
    p_restrict_to_output: *mut c_void, pp_swap_chain: *mut *mut c_void
) -> HRESULT {
    debug!("IDXGIFactory2_CreateSwapChainForHwnd");
    let res = get_orig_fn!(IDXGIFactory2_CreateSwapChainForHwnd, CreateSwapChainForHwndFn)(
        this, p_device, hwnd, p_desc, p_fullscreen_desc, p_restrict_to_output, pp_swap_chain
    );
    if res.is_ok() {
        let p_swap_chain = unsafe { *pp_swap_chain };
        let vtable = Interceptor::get_vtable_from_instance(p_swap_chain as usize);
        SWAP_CHAIN_VTABLE.store(vtable as usize, atomic::Ordering::Relaxed);
        SWAP_CHAIN_HWND.store(hwnd.0 as _, atomic::Ordering::Relaxed);

        info!("Got IDXGISwapChain vtable and HWND");
        Hachimi::instance().interceptor.unhook(IDXGIFactory2_CreateSwapChainForHwnd as usize);
    }
    else {
        error!("IDXGIFactory2_CreateSwapChainForHwnd: {}", res.message());
    }
    res
}

type DXGIGetDebugInterface1Fn = extern "C" fn(flags: c_uint, riid: *mut GUID, p_debug: *mut *mut c_void) -> HRESULT;
#[no_mangle]
pub extern "C" fn DXGIGetDebugInterface1(flags: c_uint, riid: *mut GUID, p_debug: *mut *mut c_void) -> HRESULT {
    get_orig_fn!(DXGIGetDebugInterface1, DXGIGetDebugInterface1Fn)(flags, riid, p_debug)
}

// Windows 10 version 1803 and up
type DXGIDeclareAdapterRemovalSupportFn = extern "C" fn() -> HRESULT;
#[no_mangle]
pub extern "C" fn DXGIDeclareAdapterRemovalSupport() -> HRESULT {
    let trampoline_addr = Hachimi::instance().interceptor.get_trampoline_addr(DXGIDeclareAdapterRemovalSupport as usize);

    if trampoline_addr == 0 {
        return S_OK;
    }

    unsafe { std::mem::transmute::<usize, DXGIDeclareAdapterRemovalSupportFn>(trampoline_addr)() }
}

// These are called internally by the Direct3D driver on some versions of Windows (even when using d3d11)
// Bogus but compatible fn typedef, dont mind it
type DXGID3D10CreateDeviceFn = extern "C" fn(a: usize, b: usize, c: usize, d: c_uint, e: usize, f: c_uint, g: usize) -> HRESULT;
#[no_mangle]
pub extern "C" fn DXGID3D10CreateDevice(a: usize, b: usize, c: usize, d: c_uint, e: usize, f: c_uint, g: usize) -> HRESULT {
    let trampoline_addr = Hachimi::instance().interceptor.get_trampoline_addr(DXGID3D10CreateDevice as usize);

    if trampoline_addr == 0 {
        return E_NOTIMPL;
    }

    unsafe { std::mem::transmute::<usize, DXGID3D10CreateDeviceFn>(trampoline_addr)(a, b, c, d, e, f, g) }
}

type DXGID3D10RegisterLayersFn = extern "C" fn(a: usize, b: c_uint) -> HRESULT;
#[no_mangle]
pub extern "C" fn DXGID3D10RegisterLayers(a: usize, b: c_uint) -> HRESULT {
    let trampoline_addr = Hachimi::instance().interceptor.get_trampoline_addr(DXGID3D10RegisterLayers as usize);

    if trampoline_addr == 0 {
        return E_NOTIMPL;
    }

    unsafe { std::mem::transmute::<usize, DXGID3D10RegisterLayersFn>(trampoline_addr)(a, b) }
}

pub fn init(system_dir: &Utf16Str) -> std::result::Result<(), crate::core::Error> {
    let dll_path = system_dir.to_owned() + "\\dxgi.dll";
    let dll_path_cstr = U16CString::from_vec(dll_path.into_vec()).unwrap();
    let handle = unsafe { LoadLibraryW(PCWSTR(dll_path_cstr.as_ptr())).expect("dxgi.dll") };

    let CreateDXGIFactory_addr = utils::get_proc_address(handle, c"CreateDXGIFactory");
    let CreateDXGIFactory1_addr = utils::get_proc_address(handle, c"CreateDXGIFactory1");
    let CreateDXGIFactory2_addr = utils::get_proc_address(handle, c"CreateDXGIFactory2");
    let DXGIGetDebugInterface1_addr = utils::get_proc_address(handle, c"DXGIGetDebugInterface1");
    let DXGIDeclareAdapterRemovalSupport_addr = utils::get_proc_address(handle, c"DXGIDeclareAdapterRemovalSupport");
    let DXGID3D10CreateDevice_addr = utils::get_proc_address(handle, c"DXGID3D10CreateDevice");
    let DXGID3D10RegisterLayers_addr = utils::get_proc_address(handle, c"DXGID3D10RegisterLayers");

    let interceptor = &Hachimi::instance().interceptor;
    interceptor.hook(CreateDXGIFactory_addr, CreateDXGIFactory as usize)?;
    interceptor.hook(CreateDXGIFactory1_addr, CreateDXGIFactory1 as usize)?;
    interceptor.hook(CreateDXGIFactory2_addr, CreateDXGIFactory2 as usize)?;
    interceptor.hook(DXGIGetDebugInterface1_addr, DXGIGetDebugInterface1 as usize)?;
    // There's a chance these hooks won't work, ignore them as they are non-crucial
    if DXGIDeclareAdapterRemovalSupport_addr != 0 {
        interceptor.hook(DXGIDeclareAdapterRemovalSupport_addr, DXGIDeclareAdapterRemovalSupport as usize).ok();
    }
    if DXGID3D10CreateDevice_addr != 0 {
        interceptor.hook(DXGID3D10CreateDevice_addr, DXGID3D10CreateDevice as usize).ok();
    }
    if DXGID3D10RegisterLayers_addr != 0 {
        interceptor.hook(DXGID3D10RegisterLayers_addr, DXGID3D10RegisterLayers as usize).ok();
    }

    Ok(())
}