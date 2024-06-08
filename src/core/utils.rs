use std::{borrow::Cow, io::Write, path::Path};

use serde::Serialize;

use crate::il2cpp::types::Il2CppString;

use super::{ext::StringExt, Error, Hachimi};

pub fn concat_unix_path(left: &str, right: &str) -> String {
    let mut str = String::with_capacity(left.len() + 1 + right.len());
    str.push_str(left);
    str.push_str("/");
    str.push_str(right);
    str
}

pub fn print_json_entry(key: &str, value: &str) {
    info!("{}: {},", serde_json::to_string(key).unwrap(), serde_json::to_string(value).unwrap());
}

pub fn wrap_text(string: &str, base_line_width: i32) -> Option<Vec<Cow<'_, str>>> {
    let config = &Hachimi::instance().localized_data.load().config;
    if !config.use_text_wrapper() {
        return None;
    }

    let line_width = (base_line_width as f32 * config.line_width_multiplier).round() as usize;
    let options = textwrap::Options::new(line_width);
    return Some(textwrap::wrap(string, &options));
}

pub fn wrap_text_il2cpp(string: *mut Il2CppString, base_line_width: i32) -> Option<*mut Il2CppString> {
    if Hachimi::instance().localized_data.load().config.use_text_wrapper() {
        if let Some(result) = wrap_text(unsafe { &(*string).to_utf16str().to_string() }, base_line_width) {
            return Some(result.join("\n").to_il2cpp_string());
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
    let mult = Hachimi::instance().localized_data.load().config.line_width_multiplier;
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
    if let Some(result) = fit_text(unsafe { &(*string).to_utf16str().to_string() },
        base_line_width, base_font_size
    ) {
        return Some(result.to_il2cpp_string());
    }

    None
}

// WRAP IT TILL IT FITS GRAHHH BRUTE FORCE GRAHHH
pub fn wrap_fit_text(string: &str, base_line_width: i32, mut max_line_count: i32, base_font_size: i32) -> Option<String> {
    if !Hachimi::instance().localized_data.load().config.use_text_wrapper() {
        return None;
    }

    // don't wanna mess with different sizes
    if string.contains("<size=") {
        return None;
    }

    let mut line_width = base_line_width as f32;
    let mut font_size = base_font_size as f32;
    loop {
        let Some(wrapped) = wrap_text(string, line_width.round() as i32) else {
            return None;
        };

        if wrapped.len() as i32 <= max_line_count {
            return Some(add_size_tag(&wrapped.join("\n"), font_size.round() as i32));
        }

        let prev_max_line_count = max_line_count;
        max_line_count += 1;

        let scale = prev_max_line_count as f32 / max_line_count as f32;
        font_size = font_size as f32 * scale;
        line_width = line_width as f32 / scale;
    }
}

pub fn wrap_fit_text_il2cpp(string: *mut Il2CppString, base_line_width: i32, max_line_count: i32, base_font_size: i32) -> Option<*mut Il2CppString> {
    if Hachimi::instance().localized_data.load().config.use_text_wrapper() {
        if let Some(result) = wrap_fit_text(unsafe { &(*string).to_utf16str().to_string() },
            base_line_width, max_line_count, base_font_size
        ) {
            return Some(result.to_il2cpp_string());
        }
    }
    
    None
}

pub fn write_json_file<T: Serialize, P: AsRef<Path>>(data: &T, path: P) -> Result<(), Error> {
    let file = std::fs::File::create(path)?;
    let mut writer = std::io::BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, data)?;
    writer.flush()?;
    Ok(())
}

// Checks for both \n and \\n
pub fn game_str_has_newline(string: *mut Il2CppString) -> bool {
    let mut got_backslash = false;
    for c in unsafe { (*string).to_utf16str().as_slice().iter() } {
        if got_backslash {
            if *c == 0x6E { // n
                return true;
            }
            got_backslash = false;
        }

        if *c == 0x0A { // newline
            return true;
        }
        else if *c == 0x5C { // backslash
            got_backslash = true; //
        }
    }

    false
}