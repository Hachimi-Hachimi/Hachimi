use std::{os::raw::c_uint, sync::atomic::{self, AtomicIsize}};

use egui::mutex::Mutex;
use once_cell::sync::Lazy;
use windows::{core::w, Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    System::Threading::GetCurrentThreadId,
    UI::WindowsAndMessaging::{
        CallNextHookEx, DefWindowProcW, FindWindowW, GetWindowLongPtrW, SetWindowsHookExW, UnhookWindowsHookEx,
        GWLP_WNDPROC, HCBT_MINMAX, HHOOK, SW_RESTORE, WH_CBT, WM_CLOSE, WM_KEYDOWN, WM_SYSKEYDOWN, WM_SIZE, WNDPROC
    }
}};

use crate::{core::{Gui, Hachimi}, il2cpp::{hook::{umamusume::SceneManager, UnityEngine_CoreModule}, symbols::Thread}, windows::utils};

use super::gui_impl::input;

struct WndProcCall {
    hwnd: HWND,
    umsg: c_uint,
    wparam: WPARAM,
    lparam: LPARAM
}

static WM_SIZE_BUFFER: Lazy<Mutex<Vec<WndProcCall>>> = Lazy::new(|| Mutex::default());
pub fn drain_wm_size_buffer() {
    let Some(orig_fn) = (unsafe { std::mem::transmute::<isize, WNDPROC>(WNDPROC_ORIG) }) else {
        return;
    };
    for call in WM_SIZE_BUFFER.lock().drain(..) {
        unsafe { orig_fn(call.hwnd, call.umsg, call.wparam, call.lparam); }
    }
}

static TARGET_HWND: AtomicIsize = AtomicIsize::new(0);
pub fn get_target_hwnd() -> HWND {
    HWND(TARGET_HWND.load(atomic::Ordering::Relaxed))
}

// Safety: only modified once on init
static mut WNDPROC_ORIG: isize = 0;
static mut WNDPROC_RECALL: usize = 0;
extern "system" fn wnd_proc(hwnd: HWND, umsg: c_uint, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let Some(orig_fn) = (unsafe { std::mem::transmute::<isize, WNDPROC>(WNDPROC_ORIG) }) else {
        return unsafe { DefWindowProcW(hwnd, umsg, wparam, lparam) };
    };

    match umsg {
        // Check for Home key presses
        WM_KEYDOWN | WM_SYSKEYDOWN => {
            if wparam.0 as u16 == Hachimi::instance().config.load().windows.menu_open_key {
                let Some(mut gui) = Gui::instance().map(|m| m.lock().unwrap()) else {
                    return unsafe { orig_fn(hwnd, umsg, wparam, lparam) };
                };

                gui.toggle_menu();
                return LRESULT(0);
            }
        },
        WM_CLOSE => {
            if let Some(hook) = Hachimi::instance().interceptor.unhook(wnd_proc as _) {
                unsafe { WNDPROC_RECALL = hook.orig_addr; }
                Thread::main_thread().schedule(|| {
                    unsafe {
                        let orig_fn = std::mem::transmute::<usize, WNDPROC>(WNDPROC_RECALL).unwrap();
                        orig_fn(get_target_hwnd(), WM_CLOSE, WPARAM(0), LPARAM(0));
                    }
                });
            }
            return LRESULT(0);
        },
        WM_SIZE => {
            if !SceneManager::is_splash_shown() {
                WM_SIZE_BUFFER.lock().push(WndProcCall {
                    hwnd, umsg, wparam, lparam
                });
                return LRESULT(0);
            }
            else {
                return unsafe { orig_fn(hwnd, umsg, wparam, lparam) };
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

static mut HCBTHOOK: HHOOK = HHOOK(0);
extern "system" fn cbt_proc(ncode: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if ncode == HCBT_MINMAX as i32 &&
        lparam.0 as i32 != SW_RESTORE.0 &&
        Hachimi::instance().config.load().windows.block_minimize_in_full_screen &&
        UnityEngine_CoreModule::Screen::get_fullScreen()
    {
        return LRESULT(1);
    }

    unsafe { CallNextHookEx(HCBTHOOK, ncode, wparam, lparam) }
}

pub fn init() {
    unsafe {
        let hachimi = Hachimi::instance();

        let hwnd = FindWindowW(w!("UnityWndClass"), w!("umamusume"));
        if hwnd.0 == 0 {
            error!("Failed to find game window");
            return;
        }
        TARGET_HWND.store(hwnd.0, atomic::Ordering::Relaxed);

        info!("Hooking WndProc");
        let wnd_proc_addr = GetWindowLongPtrW(hwnd, GWLP_WNDPROC);
        match hachimi.interceptor.hook(wnd_proc_addr as _, wnd_proc as _) {
            Ok(trampoline_addr) => WNDPROC_ORIG = trampoline_addr as _,
            Err(e) => error!("Failed to hook WndProc: {}", e)
        }

        info!("Adding CBT hook");
        if let Ok(hhook) = SetWindowsHookExW(WH_CBT, Some(cbt_proc), None, GetCurrentThreadId()) {
            HCBTHOOK = hhook;
        }

        // Apply always on top
        if hachimi.window_always_on_top.load(atomic::Ordering::Relaxed) {
            _ = utils::set_window_topmost(hwnd, true);
        }
    }
}

pub fn uninit() {
    unsafe {
        if HCBTHOOK.0 != 0 {
            info!("Removing CBT hook");
            if let Err(e) = UnhookWindowsHookEx(HCBTHOOK) {
                error!("Failed to remove CBT hook: {}", e);
            }
            HCBTHOOK = HHOOK(0);
        }
    }
}