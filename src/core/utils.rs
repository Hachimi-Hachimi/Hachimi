use std::{borrow::Cow, io::Write, path::Path};

use serde::Serialize;
use textwrap::{core::Word, wrap_algorithms, WordSeparator::UnicodeBreakProperties};

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

fn custom_word_separator(line: &str) -> Box<dyn Iterator<Item = Word<'_>> + '_> {
    // Split into sections of tags and other text (e.g. ['test', '<size=16>', 'hello world', '</size>'])
    // Iter returns str slice and whether to separate words in the section
    // We're only breaking the string on ascii chars, so it's safe to use the bytes
    // iterator and split them based on the index.
    let mut line_iter = line.bytes();
    let mut i = 0;
    let mut current_byte = line_iter.next();
    let mut split_iter = std::iter::from_fn(move || {
        if current_byte.is_none() {
            return None;
        }

        let start = i;
        let mut tag_start = 0;
        let mut in_tag = false;
        let mut in_closing_tag = false;
        let mut expecting_tag_name = false;
        while let Some(c) = current_byte {
            if in_tag {
                match c {
                    b'>' | b'=' | b' ' => 'tag_name_end: {
                        if expecting_tag_name {
                            if !in_closing_tag {
                                // Check for a matching closing tag after
                                let tag_name = &line[tag_start+1..i];
                                let mut closing_tag = String::with_capacity(3 + tag_name.len());
                                closing_tag += "</";
                                closing_tag += tag_name;
                                closing_tag += ">";
                                if !line[i..].contains(&closing_tag) {
                                    in_tag = false;
                                    break 'tag_name_end;
                                }
                            }
                            expecting_tag_name = false;
                        }   

                        if c == b'>' {
                            // in_tag = false;
                            loop {
                                i += 1;
                                current_byte = line_iter.next();
                                if let Some(c) = current_byte {
                                    // Capture any whitespace that comes right after it
                                    if char::from(c).is_whitespace() {
                                        continue;
                                    }
                                }
                                break;
                            }
                            return Some((&line[start..i], false));
                        }
                        else if in_closing_tag {
                            // Invalid character
                            in_tag = false;
                        }
                    }
                    b'/' => {
                        if i == tag_start + 1 {
                            in_closing_tag = true;
                        }
                        else if expecting_tag_name {
                            in_tag = false;
                        }
                    }
                    _ => {
                        if expecting_tag_name && !char::from(c).is_ascii_alphabetic() {
                            in_tag = false;
                        }
                    }
                }
            }
            else if c == b'<' {
                if start == i {
                    in_tag = true;
                    expecting_tag_name = true;
                    tag_start = i;
                }
                else {
                    break;
                }
            }

            i += 1;
            current_byte = line_iter.next();
        }

        Some((&line[start..i], true))
    });

    let mut unicode_break_iter: Box<dyn Iterator<Item = Word<'_>> + '_> = Box::new(std::iter::empty());
    Box::new(std::iter::from_fn(move || {
        // Continue breaking current split
        let break_res = unicode_break_iter.next();
        if break_res.is_some() {
            return break_res;
        }

        // Advance to next (non-empty) split
        loop {
            if let Some((next_section, needs_break)) = split_iter.next() {
                if needs_break {
                    let mut iter = UnicodeBreakProperties.find_words(next_section);
                    let break_res = iter.next();
                    if break_res.is_some() {
                        unicode_break_iter = iter;
                        return break_res;
                    }
                }
                else {
                    unicode_break_iter = Box::new(std::iter::empty());
                    return Some(Word::from(next_section));
                }
            }
            else {
                return None;
            }
        }
    }))
}

fn custom_wrap_algorithm<'a, 'b>(words: &'b [Word<'a>], line_widths: &'b [usize]) -> Vec<&'b [Word<'a>]> {
    // Create intermediate buffer that doesn't contain formatting tags
    let mut clean_fragments = Vec::with_capacity(words.len());
    let mut removed_indices = Vec::with_capacity(words.len());
    let mut remove_offset = 0;
    for (i, word) in words.iter().enumerate() {
        if word.starts_with("<") && word.ends_with(">") {
            removed_indices.push(i - remove_offset);
            remove_offset += 1;
            continue;
        }
        clean_fragments.push(words[i]);
    }

    // quick escape!!!11
    let f64_line_widths = line_widths.iter().map(|w| *w as f64).collect::<Vec<_>>();
    if remove_offset == 0 {
        return wrap_algorithms::wrap_optimal_fit(words, &f64_line_widths, &wrap_algorithms::Penalties::new()).unwrap();
    }

    // Wrap without formatting tags
    let wrapped = wrap_algorithms::wrap_optimal_fit(&clean_fragments, &f64_line_widths, &wrap_algorithms::Penalties::new()).unwrap();

    // Create results with formatting tags added back
    // Note: The break word option doesn't really affect the extra long lines since
    // the individual tags are separate words (it breaks words, not lines, duh)
    let mut lines = Vec::with_capacity(wrapped.len());
    let mut start = 0;
    let mut clean_start = 0;
    let mut removed_indices_i = 0;
    for (i, line) in wrapped.iter().enumerate() {
        let mut end: usize;
        if i == wrapped.len() - 1 {
            end = words.len();
        }
        else {
            let clean_end = clean_start + line.len();
            end = start + line.len();
            loop {
                let Some(index) = removed_indices.get(removed_indices_i) else {
                    break;
                };
                if *index >= clean_start {
                    if *index < clean_end {
                        end += 1;
                        removed_indices_i += 1;
                    }
                    else {
                        break;
                    }
                }
            }
            clean_start = clean_end;
        }

        lines.push(&words[start..end]);
        start = end;
    }
    lines
}

pub fn wrap_text(string: &str, base_line_width: i32) -> Option<Vec<Cow<'_, str>>> {
    let config = &Hachimi::instance().localized_data.load().config;
    if !config.use_text_wrapper {
        return None;
    }

    let line_width = (base_line_width as f32 * config.line_width_multiplier).round() as usize;
    let options = textwrap::Options::new(line_width)
        .word_separator(textwrap::WordSeparator::Custom(custom_word_separator))
        .wrap_algorithm(textwrap::WrapAlgorithm::Custom(custom_wrap_algorithm));
    return Some(textwrap::wrap(string, &options));
}

pub fn wrap_text_il2cpp(string: *mut Il2CppString, base_line_width: i32) -> Option<*mut Il2CppString> {
    if Hachimi::instance().localized_data.load().config.use_text_wrapper {
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
    if !Hachimi::instance().localized_data.load().config.use_text_wrapper {
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
    if Hachimi::instance().localized_data.load().config.use_text_wrapper {
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

pub fn scale_to_aspect_ratio(sizes: (i32, i32), aspect_ratio: f32, prefer_larger: bool) -> (i32, i32) {
    let (mut width, mut height) = sizes;
    let orig_aspect_ratio = width as f32 / height as f32;
    // Use original values if possible
    if (aspect_ratio - orig_aspect_ratio).abs() <= 0.001 {
        return sizes;
    }
    else if (aspect_ratio - 1.0/orig_aspect_ratio).abs() <= 0.001 {
        return (height, width);
    }

    let scale_by_height = if prefer_larger { height > width } else { width > height };
    if scale_by_height {
        width = (height as f32 * aspect_ratio).round() as i32;
        height = height;
    }
    else {
        width = width;
        height = (width as f32 / aspect_ratio).round() as i32;
    }

    (width, height)
}