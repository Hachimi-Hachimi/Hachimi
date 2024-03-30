#![allow(non_snake_case)]

use std::num::NonZeroU32;
use std::os::raw::c_char;
use std::os::raw::{c_uint, c_void};
use std::sync::Arc;
use glow::HasContext;
use once_cell::unsync::OnceCell;

use crate::core::{Error, Gui, Hachimi};
use crate::il2cpp::hook::UnityEngine_CoreModule::Screen;

type EGLBoolean = c_uint;
type EGLDisplay = *mut c_void;
type EGLSurface = *mut c_void;

fn get_binding_parameter<T>(gl: &Arc<glow::Context>, parameter: u32, create_wrapper: fn(NonZeroU32) -> T) -> Option<T> {
    let v = unsafe { gl.get_parameter_i32(parameter) };
    if let Some(value) = NonZeroU32::new(v as u32) {
        return Some(create_wrapper(value));
    }

    None
}

// Performance critical, store the trampoline addr directly
static mut EGLSWAPBUFFERS_ADDR: usize = 0;
type EGLSwapBuffersFn = extern "C" fn(display: EGLDisplay, surface: EGLSurface) -> EGLBoolean;
extern "C" fn eglSwapBuffers(display: EGLDisplay, surface: EGLSurface) -> EGLBoolean {
    let orig_fn: EGLSwapBuffersFn = unsafe { std::mem::transmute(EGLSWAPBUFFERS_ADDR) };
    let mut gui = Gui::instance_or_init("Vol Up + Vol Down").lock().unwrap();
    // Big fat state destroyer, initialize it as soon as possible
    let painter = match init_painter() {
        Ok(v) => v,
        Err(e) => {
            error!("{}", e);
            info!("Unhooking eglSwapBuffers");

            let res = orig_fn(display, surface);
            Hachimi::instance().interceptor.unhook(eglSwapBuffers as usize);
            return res;
        }
    };
    // Skip if its empty, or it's still too early
    if gui.is_empty() || gui.start_time.elapsed().as_secs_f32() < 1.0 {
        return orig_fn(display, surface);
    }

    let resolution = Screen::get_resolution();
    gui.set_screen_size(resolution.width(), resolution.height());
    let output = gui.run();

    let clipped_primitives = gui.context.tessellate(output.shapes, output.pixels_per_point);
    let dimensions: [u32; 2] = [resolution.width() as u32, resolution.height() as u32];

    // Save VBO and VAO since Unity doesn't rebind them unless it needs to
    // (might be slow...? could always hook the bind functions directly if its noticeably slow)
    let gl = painter.gl().clone();
    let prev_vbo = get_binding_parameter(&gl, glow::ARRAY_BUFFER_BINDING, glow::NativeBuffer);
    let prev_vao = get_binding_parameter(&gl, glow::VERTEX_ARRAY_BINDING, glow::NativeVertexArray);

    painter.paint_and_update_textures(dimensions, output.pixels_per_point, &clipped_primitives, &output.textures_delta);

    // Restore state
    unsafe {
        gl.enable(glow::DEPTH_TEST);
        if prev_vbo.is_some() {
            gl.bind_buffer(glow::ARRAY_BUFFER, prev_vbo);
        }
        if prev_vao.is_some() {
            gl.bind_vertex_array(prev_vao);
        }
    }

    orig_fn(display, surface)
}

static mut PAINTER: OnceCell<egui_glow::Painter> = OnceCell::new();

fn init_painter() -> Result<&'static mut egui_glow::Painter, Error> {
    if let Some(painter) = unsafe { PAINTER.get_mut() } {
        return Ok(painter);
    }

    let gl = init_gl();
    let painter = egui_glow::Painter::new(Arc::new(gl), "", None)?;
    unsafe {
        PAINTER.set(painter).unwrap_unchecked();
    }

    info!("Painter initialized");
    Ok(unsafe { PAINTER.get_mut().unwrap_unchecked() })
}

impl From<egui_glow::PainterError> for Error {
    fn from(e: egui_glow::PainterError) -> Self {
        Error::GuiRendererInitError(e.to_string())
    }
}

type EGLGetProcAddressFn = extern "C" fn(proc_name: *const c_char) -> *mut c_void;
static mut EGLGETPROCADDRESS_ADDR: usize = 0;
fn init_gl() -> glow::Context {
    let egl_get_proc_address: EGLGetProcAddressFn = unsafe { std::mem::transmute(EGLGETPROCADDRESS_ADDR) };
    unsafe {
        glow::Context::from_loader_function_cstr(|s| egl_get_proc_address(s.as_ptr()))
    }
}

fn init_internal() -> Result<(), Error> {
    info!("Hooking eglSwapBuffers");
    let egl_handle = unsafe { libc::dlopen(cstr!("libEGL.so").as_ptr(), libc::RTLD_LAZY) };
    let eglSwapBuffers_addr = unsafe { libc::dlsym(egl_handle, cstr!("eglSwapBuffers").as_ptr()) };

    unsafe {
        EGLSWAPBUFFERS_ADDR = Hachimi::instance().interceptor.hook(eglSwapBuffers_addr as usize, eglSwapBuffers as usize)?;
        EGLGETPROCADDRESS_ADDR = libc::dlsym(egl_handle, cstr!("eglGetProcAddress").as_ptr()) as usize
    }

    Ok(())
}

pub fn init() {
    init_internal().unwrap_or_else(|e| {
        error!("Init failed: {}", e);
    });
}