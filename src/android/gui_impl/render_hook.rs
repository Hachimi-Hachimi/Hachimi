#![allow(non_snake_case)]

use std::num::NonZeroU32;
use std::os::raw::c_char;
use std::os::raw::{c_uint, c_void};
use std::sync::Arc;
use glow::HasContext;
use once_cell::unsync::OnceCell;

use crate::core::{Error, Gui, Hachimi};

type EGLBoolean = c_uint;
type EGLDisplay = *mut c_void;
type EGLSurface = *mut c_void;
type EGLint = i32;

const EGL_WIDTH: EGLint = 0x3057;
const EGL_HEIGHT: EGLint = 0x3056;

fn get_binding_parameter<T>(gl: &Arc<glow::Context>, parameter: u32, create_wrapper: fn(NonZeroU32) -> T) -> Option<T> {
    let v = unsafe { gl.get_parameter_i32(parameter) };
    if let Some(value) = NonZeroU32::new(v as u32) {
        return Some(create_wrapper(value));
    }

    None
}

static mut EGLQUERYSURFACE_ADDR: usize = 0;
type EGLQuerySurfaceFn = extern "C" fn(display: EGLDisplay, surface: EGLSurface, attribute: EGLint, value: *mut EGLint) -> EGLBoolean;
fn eglQuerySurface(display: EGLDisplay, surface: EGLSurface, attribute: EGLint, value: *mut EGLint) -> EGLBoolean {
    let orig_fn: EGLQuerySurfaceFn = unsafe { std::mem::transmute(EGLQUERYSURFACE_ADDR) };
    orig_fn(display, surface, attribute, value)
}

// Performance critical, store the trampoline addr directly
static mut EGLSWAPBUFFERS_ADDR: usize = 0;
type EGLSwapBuffersFn = extern "C" fn(display: EGLDisplay, surface: EGLSurface) -> EGLBoolean;
extern "C" fn eglSwapBuffers(display: EGLDisplay, surface: EGLSurface) -> EGLBoolean {
    let orig_fn: EGLSwapBuffersFn = unsafe { std::mem::transmute(EGLSWAPBUFFERS_ADDR) };
    let mut gui = Gui::instance_or_init("android.menu_open_key").lock().unwrap();
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

    // these queries are actually relatively fast
    let mut width = 0;
    let mut height = 0;
    eglQuerySurface(display, surface, EGL_WIDTH, &mut width);
    eglQuerySurface(display, surface, EGL_HEIGHT, &mut height);

    gui.set_screen_size(width, height);
    let output = gui.run();

    let clipped_primitives = gui.context.tessellate(output.shapes, output.pixels_per_point);
    let dimensions: [u32; 2] = [width as u32, height as u32];

    // Backup state
    let gl = painter.gl().clone();
    let prev_vbo = get_binding_parameter(&gl, glow::ARRAY_BUFFER_BINDING, glow::NativeBuffer);
    let prev_vao = get_binding_parameter(&gl, glow::VERTEX_ARRAY_BINDING, glow::NativeVertexArray);
    let prev_enable_scissor_test = unsafe { gl.is_enabled(glow::SCISSOR_TEST) };
    let prev_enable_cull_face = unsafe { gl.is_enabled(glow::CULL_FACE) };
    let prev_enable_depth_test = unsafe { gl.is_enabled(glow::DEPTH_TEST) };
    let prev_enable_blend = unsafe { gl.is_enabled(glow::BLEND) };
    let prev_blend_equation_rgb = unsafe { gl.get_parameter_i32(glow::BLEND_EQUATION_RGB) as _ };
    let prev_blend_equation_alpha = unsafe { gl.get_parameter_i32(glow::BLEND_EQUATION_ALPHA) as _ };
    let prev_blend_src_rgb = unsafe { gl.get_parameter_i32(glow::BLEND_SRC_RGB) as _ };
    let prev_blend_dst_rgb = unsafe { gl.get_parameter_i32(glow::BLEND_DST_RGB) as _ };
    let prev_blend_src_alpha = unsafe { gl.get_parameter_i32(glow::BLEND_SRC_ALPHA) as _ };
    let prev_blend_dst_alpha = unsafe { gl.get_parameter_i32(glow::BLEND_DST_ALPHA) as _ };
    let prev_program = get_binding_parameter(&gl, glow::CURRENT_PROGRAM, glow::NativeProgram);
    let prev_texture = get_binding_parameter(&gl, glow::TEXTURE_BINDING_2D, glow::NativeTexture);
    let prev_active_texture = unsafe { gl.get_parameter_i32(glow::ACTIVE_TEXTURE) as _ };

    painter.paint_and_update_textures(dimensions, output.pixels_per_point, &clipped_primitives, &output.textures_delta);

    // Restore state
    unsafe {
        gl.bind_buffer(glow::ARRAY_BUFFER, prev_vbo);
        gl.bind_vertex_array(prev_vao);
        if prev_enable_scissor_test { gl.enable(glow::SCISSOR_TEST) } else { gl.disable(glow::SCISSOR_TEST) }
        if prev_enable_cull_face    { gl.enable(glow::CULL_FACE) }    else { gl.disable(glow::CULL_FACE) }
        if prev_enable_depth_test   { gl.enable(glow::DEPTH_TEST) }   else { gl.disable(glow::DEPTH_TEST) }
        if prev_enable_blend        { gl.enable(glow::BLEND) }        else { gl.disable(glow::BLEND) }
        gl.blend_equation_separate(prev_blend_equation_rgb, prev_blend_equation_alpha);
        gl.blend_func_separate(prev_blend_src_rgb, prev_blend_dst_rgb, prev_blend_src_alpha, prev_blend_dst_alpha);
        if prev_program.is_none() || gl.is_program(prev_program.unwrap()) {
            gl.use_program(prev_program);
        }
        gl.bind_texture(glow::TEXTURE_2D, prev_texture);
        gl.active_texture(prev_active_texture);
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
    let egl_handle = unsafe { libc::dlopen(c"libEGL.so".as_ptr(), libc::RTLD_LAZY) };
    let eglSwapBuffers_addr = unsafe { libc::dlsym(egl_handle, c"eglSwapBuffers".as_ptr()) };

    unsafe {
        EGLSWAPBUFFERS_ADDR = Hachimi::instance().interceptor.hook(eglSwapBuffers_addr as usize, eglSwapBuffers as usize)?;
        EGLGETPROCADDRESS_ADDR = libc::dlsym(egl_handle, c"eglGetProcAddress".as_ptr()) as usize;
        EGLQUERYSURFACE_ADDR = libc::dlsym(egl_handle, c"eglQuerySurface".as_ptr()) as usize
    }

    Ok(())
}

pub fn init() {
    init_internal().unwrap_or_else(|e| {
        error!("Init failed: {}", e);
    });
}