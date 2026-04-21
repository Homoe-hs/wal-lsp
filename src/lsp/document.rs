use lsp_types::Uri;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

pub static DOCUMENT_CACHE: Lazy<Mutex<HashMap<Uri, String>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub fn get_document(uri: &Uri) -> Option<String> {
    DOCUMENT_CACHE.lock().unwrap().get(uri).cloned()
}

pub fn set_document(uri: Uri, text: String) {
    DOCUMENT_CACHE.lock().unwrap().insert(uri, text);
}

#[allow(dead_code)]
pub fn remove_document(uri: &Uri) {
    DOCUMENT_CACHE.lock().unwrap().remove(uri);
}
