use std::{io::Write, path::{Path, PathBuf}};

use crate::{core::utils::{get_file_modified_time, load_rgba_png_file}, il2cpp::{ext::Il2CppStringExt, types::*}};

use super::{
    hook::{mscorlib, UnityEngine_CoreModule::{Texture, Texture2D},
    UnityEngine_ImageConversionModule::ImageConversion},
    symbols::{get_assembly_image, get_class, get_method_addr_cached, Array}
};

#[allow(dead_code)]
pub fn print_stack_trace() {
    let mscorlib = get_assembly_image(c"mscorlib.dll").expect("mscorlib");
    let environment_class = get_class(mscorlib, c"System", c"Environment").expect("System.Environment");
    let get_fn_addr = get_method_addr_cached(environment_class, c"get_StackTrace", 0);
    let get_fn: extern "C" fn() -> *mut Il2CppString = unsafe { std::mem::transmute(get_fn_addr) };
    debug!("{}", unsafe { (*get_fn()).as_utf16str() });
}

pub fn get_texture_diff_path<P: AsRef<Path>>(path: P) -> PathBuf {
    let mut diff_path = path.as_ref().to_owned();
    diff_path.set_extension("diff.png");
    diff_path
}

pub fn replace_texture_with_diff<P: AsRef<Path>>(texture: *mut Il2CppObject, path: P, mark_non_readable: bool) -> bool {
    replace_texture_with_diff_ex(texture, &path, get_texture_diff_path(&path), mark_non_readable, true)
}

pub fn replace_texture_with_diff_ex<P1: AsRef<Path>, P2: AsRef<Path>>(
    texture: *mut Il2CppObject, path: P1, diff_path: P2, mark_non_readable: bool, allow_fallback: bool
) -> bool {
    let Some(diff_mtime) = get_file_modified_time(&diff_path) else {
        // No diff, try to load image directly
        return if allow_fallback {
            Texture2D::load_image_file(texture, &path, mark_non_readable)
        }
        else {
            false
        }
    };

    if let Some(image_mtime) = get_file_modified_time(&path) {
        if diff_mtime < image_mtime {
            // Try to load image, otherwise generate it
            // SAFETY: Path has been guaranteed to be a file in mtime check
            if unsafe { Texture2D::load_image_file_unsafe(texture, &path, mark_non_readable) } {
                return true;
            }
        }
    }

    let Some((mut pixels, diff_info)) = load_rgba_png_file(&diff_path) else {
        error!("Failed to load texture diff: {}", diff_path.as_ref().display());
        return false;
    };

    let width = Texture::GetDataWidth(texture) as usize;
    let height = Texture::GetDataHeight(texture) as usize;

    if width as u32 != diff_info.width || height as u32 != diff_info.height {
        error!(
            "Texture diff size mismatch (expected {}x{}, got {}x{}): {}",
            width, height, diff_info.width, diff_info.height, diff_path.as_ref().display()
        );
        return false;
    }

    let new_texture = Texture2D::render_to_texture(texture);
    let orig_pixels_array = Texture2D::GetPixels32(new_texture, 0);
    let orig_pixels = unsafe { orig_pixels_array.as_slice() };
    
    // Apply diff (reuse/write directly into diff pixels buffer)
    for y in 0..height {
        for x in 0..width {
            let start = (y * width + x) * 4;
            let end = start + 4;
            let pixel = &mut pixels[start..end];
            if pixel[3] == 0 {
                // Use original pixel if diff pixel is transparent
                // Original image is flipped
                let orig_pixel = &orig_pixels[(height - y - 1) * width + x];
                pixel.copy_from_slice(orig_pixel.as_slice());
            }
            else if pixel == [255, 0, 255, 255] {
                // Make pixel transparent if it's #FF00FF
                pixel.fill(0);
            }
            // else keep the diff pixel
        }
    }

    // 1MiB should be enough for most images
    let mut png_buffer = Vec::with_capacity(std::cmp::min(pixels.len(), 1048576));
    let mut encoder = png::Encoder::new(&mut png_buffer, width as u32, height as u32);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_compression(png::Compression::Fast);

    { // Scope to drop writer and release borrow to png buffer
        let mut writer = match encoder.write_header() {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to write PNG header: {}", e);
                return false;
            }
        };

        if let Err(e) = writer.write_image_data(&pixels) {
            error!("Failed to write PNG image: {}", e);
            return false;
        }
    }

    // Reclaim some memory...
    std::mem::drop(pixels);

    // Create output dir
    let Some(path_dir) = path.as_ref().parent() else {
        return false;
    };
    match std::fs::create_dir_all(path_dir) {
        Ok(_) => (),
        Err(e) => {
            error!("Failed to create directory: {}", e);
            return false;
        }
    }

    // Write to file
    let mut out_file = match std::fs::File::create(&path) {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to create file: {}", e);
            return false;
        }
    };

    if let Err(e) = out_file.write(&png_buffer) {
        error!("Failed to write to file: {}", e);
        return false;
    }

    // And finally load image to texture
    let png_array = Array::<u8>::new(mscorlib::Byte::class(), png_buffer.len());
    unsafe { png_array.as_slice().copy_from_slice(&png_buffer); }
    ImageConversion::LoadImage(texture, png_array.this, mark_non_readable);

    true
}