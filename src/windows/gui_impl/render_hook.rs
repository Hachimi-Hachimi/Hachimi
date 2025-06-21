#![allow(non_snake_case)]
use std::{os::raw::{c_uint, c_void}, sync::Mutex};

use once_cell::sync::OnceCell;
use windows::{
    core::{w, Interface, HRESULT},
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM}, Graphics::{
            Direct3D::{D3D_DRIVER_TYPE_HARDWARE, D3D_FEATURE_LEVEL, D3D_FEATURE_LEVEL_11_0},
            Direct3D11::{D3D11CreateDeviceAndSwapChain, ID3D11Device, D3D11_CREATE_DEVICE_FLAG, D3D11_SDK_VERSION},
            Dxgi::{
                Common::{DXGI_FORMAT, DXGI_FORMAT_R8G8B8A8_UNORM},
                IDXGISwapChain, DXGI_SWAP_CHAIN_DESC,
                DXGI_USAGE_RENDER_TARGET_OUTPUT
            }
        },
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DestroyWindow, GetClientRect, IsIconic,
            RegisterClassExW, UnregisterClassW, WINDOW_EX_STYLE, WNDCLASSEXW, WS_DISABLED
        }
    }
};

use crate::{core::{Error, Gui, Hachimi, Interceptor}, windows::wnd_hook};

use super::d3d11_painter::D3D11Painter;

fn check_hwnd(this: *mut c_void) -> HWND {
    let swap_chain = unsafe { IDXGISwapChain::from_raw(this) };
    let mut desc = DXGI_SWAP_CHAIN_DESC::default();
    unsafe {
        if swap_chain.GetDesc(&mut desc).is_err() {
            return HWND(0);
        }
    }

    let target = wnd_hook::get_target_hwnd();
    if desc.OutputWindow == target {
        target
    }
    else {
        HWND(0)
    }
}

static mut PRESENT_ADDR: usize = 0; 
type PresentFn = extern "C" fn(this: *mut c_void, sync_interval: c_uint, flags: c_uint) -> HRESULT;
extern "C" fn IDXGISwapChain_Present(this: *mut c_void, sync_interval: c_uint, flags: c_uint) -> HRESULT {
    let orig_fn: PresentFn = unsafe { std::mem::transmute(PRESENT_ADDR) };

    let hwnd = check_hwnd(this);
    if hwnd.0 == 0 {
        return orig_fn(this, sync_interval, flags);
    }

    let mut gui = Gui::instance_or_init("windows.menu_open_key").lock().unwrap();
    let painter_mutex = match init_painter(this) {
        Ok(v) => v,
        Err(e) => {
            error!("{}", e);
            info!("Unhooking IDXGISwapChain hooks");

            let res = orig_fn(this, sync_interval, flags);
            let interceptor = &Hachimi::instance().interceptor;
            interceptor.unhook(IDXGISwapChain_Present as usize);
            interceptor.unhook(IDXGISwapChain_ResizeBuffers as usize);
            return res;
        }
    };
    // Skip if the GUI is empty or the window is minimized
    if gui.is_empty() || unsafe { IsIconic(hwnd).into() } {
        return orig_fn(this, sync_interval, flags);
    }
    // Check if this is the right swap chain
    let mut painter = painter_mutex.lock().unwrap();
    if this != painter.swap_chain().as_raw() {
        return orig_fn(this, sync_interval, flags);
    }

    // Get window size
    let mut rect = RECT::default();
    if let Err(e) = unsafe { GetClientRect(hwnd, &mut rect) } {
        error!("Failed to get client rect: {}", e);
        return orig_fn(this, sync_interval, flags);
    }
    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;
    gui.set_screen_size(width, height);

    // Run and render the GUI
    let output = gui.run();
    let renderer_output = egui_directx11::split_output(output).0;
    if let Err(e) = painter.present(&gui.context, renderer_output, 1.0) {
        error!("Failed to render GUI: {}", e);
    }

    orig_fn(this, sync_interval, flags)
}

static mut RESIZEBUFFERS_ADDR: usize = 0; 
type ResizeBuffersFn = extern "C" fn(
    this: *mut c_void, buffer_count: c_uint, width: c_uint, height: c_uint,
    new_format: DXGI_FORMAT, swap_chain_flags: c_uint
) -> HRESULT;
extern "C" fn IDXGISwapChain_ResizeBuffers(
    this: *mut c_void, buffer_count: c_uint, width: c_uint, height: c_uint,
    new_format: DXGI_FORMAT, swap_chain_flags: c_uint
) -> HRESULT {
    let orig_fn: ResizeBuffersFn = unsafe { std::mem::transmute(RESIZEBUFFERS_ADDR) };

    // Make sure that a swap chain has the right HWND first before initing the painter,
    // even if we don't use it here.
    if check_hwnd(this).0 == 0 {
        return orig_fn(this, buffer_count, width, height, new_format, swap_chain_flags);
    }

    let painter_mutex = match init_painter(this) {
        Ok(v) => v,
        Err(e) => {
            error!("{}", e);
            info!("Unhooking IDXGISwapChain hooks");

            let interceptor = &Hachimi::instance().interceptor;
            interceptor.unhook(IDXGISwapChain_Present as usize);
            interceptor.unhook(IDXGISwapChain_ResizeBuffers as usize);
            return orig_fn(this, buffer_count, width, height, new_format, swap_chain_flags);
        }
    };
    let mut painter = painter_mutex.lock().unwrap();
    if this != painter.swap_chain().as_raw() {
        return orig_fn(this, buffer_count, width, height, new_format, swap_chain_flags);
    }

    painter.resize_buffers(|| orig_fn(
        this, buffer_count, width, height, new_format, swap_chain_flags
    ))
}

static PAINTER: OnceCell<Mutex<D3D11Painter>> = OnceCell::new();
fn init_painter(p_swap_chain: *mut c_void) -> Result<&'static Mutex<D3D11Painter>, Error> {
    PAINTER.get_or_try_init(|| {
        let swap_chain = unsafe { IDXGISwapChain::from_raw(p_swap_chain) };
        let painter = D3D11Painter::new(swap_chain)?;
        Ok(Mutex::new(painter))
    })
}

unsafe extern "system" fn dummy_wnd_proc(hwnd: HWND, umsg: c_uint, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    DefWindowProcW(hwnd, umsg, wparam, lparam)
}

fn get_swap_chain_vtable() -> Result<*mut usize, Error> {
    let hmodule = unsafe { GetModuleHandleW(None).unwrap() };

    // Create a fake swap chain to obtain the vtable
    let mut wc = WNDCLASSEXW::default();
    wc.cbSize = std::mem::size_of::<WNDCLASSEXW>() as u32;
    wc.lpfnWndProc = Some(dummy_wnd_proc);
    wc.lpszClassName = w!("Hachimi");

    if unsafe { RegisterClassExW(&wc) } == 0 {
        return Err(Error::RuntimeError("Failed to register dummy window class".to_owned()));
    }

    let hwnd = unsafe {
        CreateWindowExW(WINDOW_EX_STYLE(0), wc.lpszClassName, w!(""), WS_DISABLED, 0, 0, 0, 0, None, None, None, None)
    };

    if hwnd.0 == 0 {
        return Err(Error::RuntimeError("Failed to create dummy window".to_string()));
    }

    let mut swap_chain_desc = DXGI_SWAP_CHAIN_DESC::default();
    swap_chain_desc.BufferCount = 1;
    swap_chain_desc.BufferUsage = DXGI_USAGE_RENDER_TARGET_OUTPUT;
    swap_chain_desc.BufferDesc.Format = DXGI_FORMAT_R8G8B8A8_UNORM;
	swap_chain_desc.OutputWindow = hwnd;
	swap_chain_desc.SampleDesc.Count = 1;
	swap_chain_desc.Windowed = true.into();

    let mut p_swap_chain: Option<IDXGISwapChain> = None;
    let mut p_device: Option<ID3D11Device> = None;
    let mut feature_level = D3D_FEATURE_LEVEL::default();

    unsafe {
        D3D11CreateDeviceAndSwapChain(
            None, D3D_DRIVER_TYPE_HARDWARE, None, D3D11_CREATE_DEVICE_FLAG(0), Some(&[D3D_FEATURE_LEVEL_11_0]),
            D3D11_SDK_VERSION, Some(&swap_chain_desc), Some(&mut p_swap_chain), Some(&mut p_device),
            Some(&mut feature_level), None
        )
    }.map_err(|e| {
        unsafe {
            _ = DestroyWindow(hwnd);
            _ = UnregisterClassW(wc.lpszClassName, hmodule);
        }
        Error::RuntimeError(e.to_string())
    })?;

    let swap_chain_vtable = p_swap_chain.map(|swap_chain|
        Interceptor::get_vtable_from_instance(swap_chain.as_raw() as _)
    );
    std::mem::drop(p_device);

    unsafe {
        _ = DestroyWindow(hwnd);
        _ = UnregisterClassW(wc.lpszClassName, hmodule);
    }

    Ok(swap_chain_vtable.unwrap_or(0 as _))
}

fn init_internal() -> Result<(), Error> {
    let swap_chain_vtable = get_swap_chain_vtable()?;
    let interceptor = &Hachimi::instance().interceptor;

    unsafe {
        info!("Hooking IDXGISwapChain::Present");
        PRESENT_ADDR = interceptor.hook_vtable(swap_chain_vtable, 8, IDXGISwapChain_Present as usize)?;

        info!("Hooking IDXGISwapChain::ResizeBuffers");
        RESIZEBUFFERS_ADDR = interceptor.hook_vtable(swap_chain_vtable, 13, IDXGISwapChain_ResizeBuffers as usize)?;
    }

    Ok(())
}

pub fn init() {
    std::thread::spawn(|| {
        init_internal().unwrap_or_else(|e| {
            error!("Init failed: {}", e);
        });
    });
}