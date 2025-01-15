use widestring::{Utf16Str, Utf16String};

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

/// A convenience trait that can be used together with the type aliases defined to
/// get access to the `new()` and `with_capacity()` methods for the HashMap type alias.
pub trait HashMapExt {
    /// Constructs a new HashMap
    fn new() -> Self;
    /// Constructs a new HashMap with a given initial capacity
    fn with_capacity(capacity: usize) -> Self;
}

impl<K, V, S> HashMapExt for std::collections::HashMap<K, V, S>
where
    S: std::hash::BuildHasher + Default,
{
    fn new() -> Self {
        std::collections::HashMap::with_hasher(S::default())
    }

    fn with_capacity(capacity: usize) -> Self {
        std::collections::HashMap::with_capacity_and_hasher(capacity, S::default())
    }
}