#![allow(non_camel_case_types)]

use tree_sitter::Language;

extern "C" {
    pub fn tree_sitter_wal() -> *const tree_sitter::ffi::TSLanguage;
}

pub unsafe fn language() -> Language {
    Language::from_raw(tree_sitter_wal())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_accessor() {
        unsafe {
            let lang = tree_sitter_wal();
            assert!(!lang.is_null());
            let lang2 = language();
            assert!(lang2.into_raw() != std::ptr::null());
        }
    }
}
