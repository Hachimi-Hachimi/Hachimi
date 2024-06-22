use widestring::{Utf16Str, Utf16String};

use crate::il2cpp::{api::il2cpp_string_new_utf16, types::Il2CppString};

pub trait StringExt {
    fn to_il2cpp_string(&self) -> *mut Il2CppString;
}

impl StringExt for str {
    fn to_il2cpp_string(&self) -> *mut Il2CppString {
        let text_utf16 = Utf16String::from_str(self);
        il2cpp_string_new_utf16(text_utf16.as_ptr(), text_utf16.len().try_into().unwrap())
    }
}

impl StringExt for String {
    fn to_il2cpp_string(&self) -> *mut Il2CppString {
        str::to_il2cpp_string(self)
    }
}

pub trait Utf16StringExt {
    fn starts_with(&self, prefix: &str) -> bool;
    fn ends_with(&self, suffix: &str) -> bool;
    fn path_filename(&self) -> &Utf16Str;
    fn path_basename(&self) -> &Utf16Str;
    fn str_eq(&self, other: &str) -> bool;
}

impl Utf16StringExt for Utf16Str {
    fn starts_with(&self, prefix: &str) -> bool {
        let mut prefix_iter = prefix.chars();
        for c in self.chars() {
            if let Some(c2) = prefix_iter.next() {
                if c != c2 {
                    return false;
                }
            }
            else {
                return true;
            }
        }

        // self == prefix || length < prefix_length
        prefix_iter.next().is_none()
    }

    fn ends_with(&self, suffix: &str) -> bool {
        let mut suffix_iter = suffix.chars().rev();
        for c in self.chars().rev() {
            if let Some(c2) = suffix_iter.next() {
                if c != c2 {
                    return false;
                }
            }
            else {
                return true;
            }
        }

        // self == suffix || length < suffix_length
        suffix_iter.next().is_none()
    }

    fn path_filename(&self) -> &Utf16Str {
        let slice = self.as_slice();
        let mut i = slice.len();
        for c in slice.iter().rev() {
            if *c == 47 || *c == 92 { // '/' OR '\\'
                break;
            }
            i -= 1;
        }

        &self[i..]
    }

    fn path_basename(&self) -> &Utf16Str {
        let slice = self.as_slice();
        let mut i = slice.len();
        for c in self.as_slice().iter().rev() {
            i -= 1;
            if *c == 46 { // '.'
                return &self[..i];
            }
        }

        self
    }

    fn str_eq(&self, other: &str) -> bool {
        let mut other_iter = other.chars();
        for c in self.chars() {
            if let Some(c2) = other_iter.next() {
                if c != c2 {
                    return false;
                }
            }
            else {
                return false;
            }
        }
        other_iter.next().is_none()
    }
}

impl Utf16StringExt for Utf16String {
    fn starts_with(&self, prefix: &str) -> bool {
        Utf16Str::starts_with(self, prefix)
    }

    fn ends_with(&self, suffix: &str) -> bool {
        Utf16Str::ends_with(self, suffix)
    }

    fn path_filename(&self) -> &Utf16Str {
        Utf16Str::path_filename(self)
    }

    fn path_basename(&self) -> &Utf16Str {
        Utf16Str::path_basename(self)
    }

    fn str_eq(&self, other: &str) -> bool {
        Utf16Str::str_eq(self, other)
    }
}