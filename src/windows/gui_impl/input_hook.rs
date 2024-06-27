use std::os::raw::c_uint;

use windows::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    UI::WindowsAndMessaging::{DefWindowProcW, SetWindowLongPtrW, GWLP_WNDPROC, WM_KEYDOWN, WM_SYSKEYDOWN, WNDPROC}
};

use crate::{core::{Gui, Hachimi}, windows::proxy::dxgi};

use super::input;

// Safety: only modified once on init
static mut WNDPROC_ORIG: isize = 0;
extern "C" fn wnd_proc(hwnd: HWND, umsg: c_uint, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let Some(orig_fn) = (unsafe { std::mem::transmute::<isize, WNDPROC>(WNDPROC_ORIG) }) else {
        return unsafe { DefWindowProcW(hwnd, umsg, wparam, lparam) };
    };

    // Check for Home key presses
    match umsg {
        WM_KEYDOWN | WM_SYSKEYDOWN => {
            if wparam.0 as u16 == Hachimi::instance().config.load().menu_open_key {
                let Some(mut gui) = Gui::instance().map(|m| m.lock().unwrap()) else {
                    return unsafe { orig_fn(hwnd, umsg, wparam, lparam) };
                };

                gui.toggle_menu();
                return LRESULT(0);
            }
        }
        _ => ()
    }

    // Only capture input if gui needs it
    if !Gui::is_consuming_input_atomic() {
        return unsafe { orig_fn(hwnd, umsg, wparam, lparam) };
    }

    // Check if the input processor handles this message
    if !input::is_handled_msg(umsg) {
        return unsafe { orig_fn(hwnd, umsg, wparam, lparam) };
    }

    // A deadlock would *sometimes* consistently occur if this was done on the current thread
    // (when moving the window, etc.)
    // I assume that SwapChain::Present and WndProc are running on the same thread
    std::thread::spawn(move || {
        let Some(mut gui) = Gui::instance().map(|m| m.lock().unwrap()) else {
            return;
        };

        let zoom_factor = gui.context.zoom_factor();
        input::process(&mut gui.input, zoom_factor, umsg, wparam.0, lparam.0);
    });

    LRESULT(0)
}

pub fn init() {
    info!("Replacing WndProc");
    let hwnd = dxgi::get_swap_chain_hwnd();
    unsafe {
        WNDPROC_ORIG = SetWindowLongPtrW(hwnd, GWLP_WNDPROC, wnd_proc as isize);
    }
}