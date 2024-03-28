use std::io::Write;

use serde::Serialize;

use crate::il2cpp::types::Il2CppString;

use super::{ext::StringExt, Error, Hachimi};

pub fn concat_path(left: &str, right: &str) -> String {
    let mut str = String::with_capacity(left.len() + 1 + right.len());
    str.push_str(left);
    str.push_str("/");
    str.push_str(right);
    str
}

pub fn print_json_entry(key: &str, value: &str) {
    info!("{}: {},", serde_json::to_string(key).unwrap(), serde_json::to_string(value).unwrap());
}

pub fn wrap_text(string: &str, base_line_width: i32) -> Option<String> {
    let config = &Hachimi::instance().localized_data.load().config;
    if let Some(text_wrapper) = config.text_wrapper {
        if let Some(mult) = config.line_width_multiplier {
            let line_width = (base_line_width as f32 * mult).round() as usize;
            match text_wrapper {
                0 => return Some(bwrap::wrap_maybrk!(string, line_width)),
                1 => return Some(bwrap::wrap_nobrk!(string, line_width)),
                _ => warn!("Invalid text_wrapper value")
            };
        }
    }

    None
}

pub fn wrap_text_il2cpp(string: *mut Il2CppString, base_line_width: i32) -> Option<*mut Il2CppString> {
    if Hachimi::instance().localized_data.load().config.text_wrapper.is_some() {
        if let Some(result) = wrap_text(unsafe { &(*string).to_utf16str().to_string() }, base_line_width) {
            return Some(result.to_il2cpp_string());
        }
    }
    
    None
}

pub fn add_size_tag(string: &str, size: i32) -> String {
    // <size=xx>...</size>
    let mut new_str = String::with_capacity(9 + string.len() + 7);
    new_str.push_str("<size=");
    new_str.push_str(&size.to_string());
    new_str.push_str(">");
    new_str.push_str(string);
    new_str.push_str("</size>");
    new_str
}

pub fn fit_text(string: &str, base_line_width: i32, base_font_size: i32) -> Option<String> {
    let Some(mult) = Hachimi::instance().localized_data.load().config.line_width_multiplier else {
        return None;
    };
    let line_width = base_line_width as f32 * mult;

    let count = string.chars().count() as f32;
    if line_width < count {
        Some(add_size_tag(string, (base_font_size as f32 * (line_width / count)) as i32))
    }
    else {
        None
    }
}

pub fn fit_text_il2cpp(string: *mut Il2CppString, base_line_width: i32, base_font_size: i32) -> Option<*mut Il2CppString> {
    if Hachimi::instance().localized_data.load().config.line_width_multiplier.is_some() {
        if let Some(result) = fit_text(unsafe { &(*string).to_utf16str().to_string() },
            base_line_width, base_font_size
        ) {
            return Some(result.to_il2cpp_string());
        }
    }
    
    None
}

// WRAP IT TILL IT FITS GRAHHH BRUTE FORCE GRAHHH
pub fn wrap_fit_text(string: &str, base_line_width: i32, mut max_line_count: i32, base_font_size: i32) -> Option<String> {
    let mut line_width = base_line_width as f32;
    let mut font_size = base_font_size as f32;
    loop {
        let Some(wrapped) = wrap_text(string, line_width.round() as i32) else {
            return None;
        };

        let mut line_count = 1;
        for (i, c) in wrapped.bytes().enumerate() {
            if c == b'\n' {
                line_count += 1;
            }
            else if c == b'<' {
                // using starts_with to prevent slicing anything outside the char boundary
                if wrapped[i..].starts_with("<size=") {
                    // we aren't interested in fighting already sized text rn...
                    // but at least give em the wrapped text so it wouldnt run off the screen
                    return Some(wrapped);
                }
            }
        }

        if line_count <= max_line_count {
            return Some(add_size_tag(&wrapped, font_size.round() as i32));
        }

        let prev_max_line_count = max_line_count;
        max_line_count += 1;

        let scale = prev_max_line_count as f32 / max_line_count as f32;
        font_size = font_size as f32 * scale;
        line_width = line_width as f32 / scale;
    }
}

pub fn wrap_fit_text_il2cpp(string: *mut Il2CppString, base_line_width: i32, max_line_count: i32, base_font_size: i32) -> Option<*mut Il2CppString> {
    if Hachimi::instance().localized_data.load().config.text_wrapper.is_some() {
        if let Some(result) = wrap_fit_text(unsafe { &(*string).to_utf16str().to_string() },
            base_line_width, max_line_count, base_font_size
        ) {
            return Some(result.to_il2cpp_string());
        }
    }
    
    None
}

pub fn write_json_file<T: Serialize>(data: &T, path: &str) -> Result<(), Error> {
    let file = std::fs::File::create(path)?;
    let mut writer = std::io::BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, data)?;
    writer.flush()?;
    Ok(())
}