#![allow(non_snake_case)]
use std::{os::raw::{c_uint, c_void}, sync::Mutex};

use once_cell::sync::OnceCell;
use windows::{
    core::{Interface, HRESULT},
    Win32::{
        Foundation::RECT,
        Graphics::Dxgi::{Common::DXGI_FORMAT, IDXGISwapChain},
        UI::WindowsAndMessaging::{GetClientRect, IsIconic}
    }
};

use crate::{core::{Error, Gui, Hachimi}, windows::proxy::dxgi};

use super::d3d11_painter::D3D11Painter;

static mut PRESENT_ADDR: usize = 0; 
type PresentFn = extern "C" fn(this: *mut c_void, sync_interval: c_uint, flags: c_uint) -> HRESULT;
extern "C" fn IDXGISwapChain_Present(this: *mut c_void, sync_interval: c_uint, flags: c_uint) -> HRESULT {
    let orig_fn: PresentFn = unsafe { std::mem::transmute(PRESENT_ADDR) };
    let mut gui = Gui::instance_or_init("Right arrow").lock().unwrap();
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
    let hwnd = dxgi::get_swap_chain_hwnd();
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

    let painter_mutex = match init_painter(this) {
        Ok(v) => v,
        Err(e) => {
            error!("{}", e);
            info!("Unhooking IDXGISwapChain hooks");

            let interceptor = &Hachimi::instance().interceptor;
            interceptor.unhook(IDXGISwapChain_Present as usize);
            interceptor.unhook(IDXGISwapChain_ResizeBuffers as usize);
            return orig_fn(
                this, buffer_count, width, height, new_format, swap_chain_flags
            );
        }
    };
    let mut painter = painter_mutex.lock().unwrap();
    if this != painter.swap_chain().as_raw() {
        return orig_fn(
            this, buffer_count, width, height, new_format, swap_chain_flags
        );
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

fn init_internal() -> Result<(), Error> {
    let swap_chain_vtable = dxgi::get_swap_chain_vtable();
    if swap_chain_vtable.is_null() {
        return Err(Error::HookingError("Swap chain vtable is null".to_owned()));
    }

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
    init_internal().unwrap_or_else(|e| {
        error!("Init failed: {}", e);
    });
}