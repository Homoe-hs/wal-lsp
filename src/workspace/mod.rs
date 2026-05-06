mod waveform;

use crate::wal::symbols::{WalSymbol, extract_symbols};
use lsp_types::{Range, Uri};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct DocumentInfo {
    pub uri: Uri,
    pub text: String,
    pub version: i32,
}

impl DocumentInfo {
    pub fn new(uri: Uri, text: String) -> Self {
        Self { uri, text, version: 1 }
    }
}

#[derive(Debug, Clone)]
pub struct SymbolLocation {
    pub uri: Uri,
    pub range: Range,
    pub name: String,
}

#[derive(Debug)]
pub struct SymbolIndex {
    pub by_name: HashMap<String, Vec<SymbolLocation>>,
}

impl SymbolIndex {
    pub fn new() -> Self {
        Self {
            by_name: HashMap::new(),
        }
    }

    pub fn add_symbol(&mut self, symbol: &WalSymbol, uri: &Uri) {
        let location = SymbolLocation {
            uri: uri.clone(),
            range: symbol.range,
            name: symbol.name.clone(),
        };
        self.by_name
            .entry(symbol.name.clone())
            .or_insert_with(Vec::new)
            .push(location);
    }

    pub fn index_document(&mut self, uri: &Uri, source: &str) {
        let symbols = extract_symbols(source);
        for symbol in &symbols {
            self.add_symbol(symbol, uri);
        }
    }

    pub fn remove_document(&mut self, uri: &Uri) {
        self.by_name.retain(|_, locations| {
            locations.retain(|loc| &loc.uri != uri);
            !locations.is_empty()
        });
    }

    pub fn find(&self, name: &str) -> Vec<SymbolLocation> {
        self.by_name
            .get(name)
            .cloned()
            .unwrap_or_default()
    }
}

impl Default for SymbolIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct Workspace {
    pub documents: HashMap<Uri, DocumentInfo>,
    pub symbol_index: SymbolIndex,
    pub waveform_manager: waveform::WaveformManager,
    pub root_path: Option<PathBuf>,
}

impl Workspace {
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
            symbol_index: SymbolIndex::new(),
            waveform_manager: waveform::WaveformManager::new(),
            root_path: None,
        }
    }

    pub fn set_root_path(&mut self, path: PathBuf) {
        self.root_path = Some(path);
    }

    pub fn open_document(&mut self, uri: Uri, text: String) {
        let doc = DocumentInfo::new(uri.clone(), text.clone());
        self.documents.insert(uri.clone(), doc);
        self.symbol_index.index_document(&uri, &text);
    }

    pub fn update_document(&mut self, uri: &Uri, text: String) {
        if let Some(doc) = self.documents.get_mut(uri) {
            doc.text = text.clone();
            doc.version += 1;
        }
        self.symbol_index.remove_document(uri);
        self.symbol_index.index_document(uri, &text);
    }

    pub fn close_document(&mut self, uri: &Uri) {
        self.documents.remove(uri);
        self.symbol_index.remove_document(uri);
    }

    pub fn get_document(&self, uri: &Uri) -> Option<&DocumentInfo> {
        self.documents.get(uri)
    }

    pub fn get_word_at_position(&self, uri: &Uri, line: u32, character: u32) -> Option<String> {
        let doc = self.documents.get(uri)?;
        let lines: Vec<&str> = doc.text.lines().collect();
        let line_str = lines.get(line as usize)?;

        let mut start = character as usize;
        let mut end = character as usize;

        while start > 0 && !line_str.is_char_boundary(start - 1) {
            start -= 1;
        }
        while end < line_str.len() && !line_str.is_char_boundary(end) {
            end += 1;
        }

        if end <= start {
            end = (start + 1).min(line_str.len());
        }

        let ch = line_str[start..end].chars().next()?;
        if !ch.is_alphanumeric() && ch != '_' && ch != '-' && ch != '.' && ch != '/' && ch != '#' && ch != '~' {
            return None;
        }

        let mut s = start;
        while s > 0 {
            let prev = line_str[s - 1..s].chars().next()?;
            if prev.is_alphanumeric() || prev == '_' || prev == '-' || prev == '.' || prev == '/' || prev == '#' || prev == '~' {
                s -= 1;
            } else {
                break;
            }
        }

        let mut e = end;
        while e < line_str.len() {
            let next = line_str[e..e + 1].chars().next()?;
            if next.is_alphanumeric() || next == '_' || next == '-' || next == '.' || next == '/' || next == '#' || next == '~' {
                e += 1;
            } else {
                break;
            }
        }

        Some(line_str[s..e].to_string())
    }
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new()
    }
}

pub type SharedWorkspace = Arc<RwLock<Workspace>>;

pub fn create_workspace() -> SharedWorkspace {
    Arc::new(RwLock::new(Workspace::new()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_document_open_close() {
        let mut ws = Workspace::new();
        let uri = Uri::from_str("file:///test.wal").unwrap();
        let text = "(define x 42)".to_string();

        ws.open_document(uri.clone(), text);
        assert!(ws.documents.contains_key(&uri));

        ws.close_document(&uri);
        assert!(!ws.documents.contains_key(&uri));
    }

    #[test]
    fn test_symbol_indexing() {
        let mut ws = Workspace::new();
        let uri = Uri::from_str("file:///test.wal").unwrap();
        let text = "(define x 42)\n(fn add [a b] (+ a b))".to_string();

        ws.open_document(uri.clone(), text);

        let locations = ws.symbol_index.find("x");
        assert!(!locations.is_empty());
        assert_eq!(locations[0].name, "x");
    }

    #[test]
    fn test_get_word_at_position() {
        let mut ws = Workspace::new();
        let uri = Uri::from_str("file:///test.wal").unwrap();
        let text = "(define tb.clk 42)".to_string();

        ws.open_document(uri.clone(), text.clone());

        let word = ws.get_word_at_position(&uri, 0, 8);
        assert_eq!(word, Some("tb.clk".to_string()));
    }

    #[test]
    fn test_cross_file_symbols() {
        let mut ws = Workspace::new();
        let uri1 = Uri::from_str("file:///file1.wal").unwrap();
        let uri2 = Uri::from_str("file:///file2.wal").unwrap();

        ws.open_document(uri1.clone(), "(define foo 1)".to_string());
        ws.open_document(uri2.clone(), "(define bar 2)".to_string());

        let foo_locations = ws.symbol_index.find("foo");
        assert_eq!(foo_locations.len(), 1);
        assert_eq!(foo_locations[0].uri, uri1);

        let bar_locations = ws.symbol_index.find("bar");
        assert_eq!(bar_locations.len(), 1);
        assert_eq!(bar_locations[0].uri, uri2);
    }

    #[test]
    fn test_update_document() {
        let mut ws = Workspace::new();
        let uri = Uri::from_str("file:///test.wal").unwrap();

        ws.open_document(uri.clone(), "(define x 1)".to_string());
        ws.update_document(&uri, "(define x 42)".to_string());

        let doc = ws.get_document(&uri).unwrap();
        assert!(doc.text.contains("42"));
        assert_eq!(doc.version, 2);
    }

    #[test]
    fn test_multiple_documents_symbol_index() {
        let mut ws = Workspace::new();
        let uri1 = Uri::from_str("file:///lib.wal").unwrap();
        let uri2 = Uri::from_str("file:///main.wal").unwrap();
        let uri3 = Uri::from_str("file:///test.wal").unwrap();

        ws.open_document(uri1.clone(), "(define library-fn (fn [x] (* x 2)))".to_string());
        ws.open_document(uri2.clone(), "(defun main-func [n] (factorial n))".to_string());
        ws.open_document(uri3.clone(), "(define test-var 99)".to_string());

        // All symbols should be findable
        assert_eq!(ws.symbol_index.find("library-fn").len(), 1);
        assert_eq!(ws.symbol_index.find("main-func").len(), 1);
        assert_eq!(ws.symbol_index.find("test-var").len(), 1);

        // Close one document, its symbols should be gone
        ws.close_document(&uri2);
        assert!(ws.symbol_index.find("main-func").is_empty());

        // Other documents still indexed
        assert_eq!(ws.symbol_index.find("library-fn").len(), 1);
        assert_eq!(ws.symbol_index.find("test-var").len(), 1);

        assert_eq!(ws.documents.len(), 2);
    }

    #[test]
    fn test_update_document_updates_symbol_index() {
        let mut ws = Workspace::new();
        let uri = Uri::from_str("file:///updates.wal").unwrap();

        ws.open_document(uri.clone(), "(define old-var 1)".to_string());
        assert_eq!(ws.symbol_index.find("old-var").len(), 1);
        assert!(ws.symbol_index.find("new-var").is_empty());

        ws.update_document(&uri, "(define new-var 42)".to_string());
        assert!(ws.symbol_index.find("old-var").is_empty(),
            "old-var should be removed after update");
        assert_eq!(ws.symbol_index.find("new-var").len(), 1);
    }

    #[test]
    fn test_get_word_at_position_edge_cases() {
        let mut ws = Workspace::new();
        let uri = Uri::from_str("file:///test.wal").unwrap();
        let text = "(define tb.clk 42)\n(print \"hello\")\n;; comment\n~clk\n#valid\nCG";
        ws.open_document(uri.clone(), text.to_string());

        // Regular symbol
        assert_eq!(ws.get_word_at_position(&uri, 0, 8), Some("tb.clk".to_string()));
        // Scoped symbol
        assert_eq!(ws.get_word_at_position(&uri, 3, 1), Some("~clk".to_string()));
        // Grouped symbol
        assert_eq!(ws.get_word_at_position(&uri, 4, 1), Some("#valid".to_string()));
        // Special var CG
        assert_eq!(ws.get_word_at_position(&uri, 5, 1), Some("CG".to_string()));
        // String literal characters are captured (they're alphabetic)
        assert_eq!(ws.get_word_at_position(&uri, 1, 9), Some("hello".to_string()),
            "String content characters should be captured as word");
        // After the closing quote, no word
        assert!(ws.get_word_at_position(&uri, 1, 14).is_none(),
            "After closing quote should be None");
        // Comment text is still regular text in the document
        assert_eq!(ws.get_word_at_position(&uri, 2, 3), Some("comment".to_string()),
            "Comment body text is captured as regular word");
    }

    #[test]
    fn test_cross_file_complex_scenario() {
        let mut ws = Workspace::new();
        let lib_uri = Uri::from_str("file:///math.wal").unwrap();
        let main_uri = Uri::from_str("file:///program.wal").unwrap();

        ws.open_document(lib_uri.clone(), r#"
(define pi 3.14)
(defun square [x] (* x x))
(defun cube [x] (* x x x))
(defmacro my-if [c t e] `(if ,c ,t ,e))
"#.to_string());

        ws.open_document(main_uri.clone(), r#"
(define result (square 5))
(defun main [n] (cube n))
"#.to_string());

        assert_eq!(ws.symbol_index.find("pi").len(), 1);
        assert_eq!(ws.symbol_index.find("square").len(), 1);
        assert_eq!(ws.symbol_index.find("cube").len(), 1);
        assert_eq!(ws.symbol_index.find("my-if").len(), 1);
        assert_eq!(ws.symbol_index.find("result").len(), 1);
        assert_eq!(ws.symbol_index.find("main").len(), 1);

        // Same symbol name in multiple files
        assert_eq!(ws.symbol_index.find("cube").len(), 1,
            "Only lib.wal defines cube");
    }

    #[test]
    fn test_document_version_increments() {
        let mut ws = Workspace::new();
        let uri = Uri::from_str("file:///versioned.wal").unwrap();

        ws.open_document(uri.clone(), "(define x 1)".to_string());
        assert_eq!(ws.documents.get(&uri).unwrap().version, 1);

        ws.update_document(&uri, "(define x 2)".to_string());
        assert_eq!(ws.documents.get(&uri).unwrap().version, 2);

        ws.update_document(&uri, "(define x 3)".to_string());
        assert_eq!(ws.documents.get(&uri).unwrap().version, 3);
    }

    #[test]
    fn test_workspace_clear_and_rebuild() {
        let mut ws = Workspace::new();
        let uri = Uri::from_str("file:///test.wal").unwrap();

        ws.open_document(uri.clone(), "(define a 1)".to_string());
        assert_eq!(ws.documents.len(), 1);

        ws.close_document(&uri);
        assert!(ws.documents.is_empty());
        assert!(ws.symbol_index.find("a").is_empty());

        ws.open_document(uri.clone(), "(define b 2)".to_string());
        assert_eq!(ws.documents.len(), 1);
        assert_eq!(ws.symbol_index.find("b").len(), 1);
        assert!(ws.symbol_index.find("a").is_empty());
    }

    #[test]
    fn test_bulk_documents_open_close() {
        let mut ws = Workspace::new();
        let n = 20;
        for i in 0..n {
            let uri = Uri::from_str(&format!("file:///doc{}.wal", i)).unwrap();
            ws.open_document(uri, format!("(define var{} {})", i, i));
        }
        assert_eq!(ws.documents.len(), n);

        for i in 0..n {
            assert_eq!(ws.symbol_index.find(&format!("var{}", i)).len(), 1);
        }

        // Close half
        for i in 0..n/2 {
            let uri = Uri::from_str(&format!("file:///doc{}.wal", i)).unwrap();
            ws.close_document(&uri);
        }
        assert_eq!(ws.documents.len(), n - n/2);

        // Remaining symbols still findable
        for i in n/2..n {
            assert_eq!(ws.symbol_index.find(&format!("var{}", i)).len(), 1);
        }
        // Closed symbols gone
        for i in 0..n/2 {
            assert!(ws.symbol_index.find(&format!("var{}", i)).is_empty());
        }
    }

    #[test]
    fn test_same_symbol_name_across_files() {
        let mut ws = Workspace::new();
        let uri1 = Uri::from_str("file:///a.wal").unwrap();
        let uri2 = Uri::from_str("file:///b.wal").unwrap();
        let uri3 = Uri::from_str("file:///c.wal").unwrap();

        ws.open_document(uri1.clone(), "(define common 1)".to_string());
        ws.open_document(uri2.clone(), "(define common 2)".to_string());
        ws.open_document(uri3.clone(), "(define common 3)".to_string());

        let locations = ws.symbol_index.find("common");
        assert_eq!(locations.len(), 3, "Same symbol in 3 files should have 3 locations");

        // Close one, should drop to 2
        ws.close_document(&uri1);
        assert_eq!(ws.symbol_index.find("common").len(), 2);
    }

    #[test]
    fn test_symbol_index_correct_uri_tracking() {
        let mut ws = Workspace::new();
        let lib = Uri::from_str("file:///lib.wal").unwrap();
        let main = Uri::from_str("file:///main.wal").unwrap();

        ws.open_document(lib.clone(), "(defun lib-fn [x] (* x 2))".to_string());
        ws.open_document(main.clone(), "(defun main-fn [x] (lib-fn x))".to_string());

        let lib_locs = ws.symbol_index.find("lib-fn");
        assert_eq!(lib_locs[0].uri, lib);

        let main_locs = ws.symbol_index.find("main-fn");
        assert_eq!(main_locs[0].uri, main);
    }

    #[test]
    fn test_workspace_root_path() {
        let mut ws = Workspace::new();
        assert!(ws.root_path.is_none());
        ws.set_root_path(std::path::PathBuf::from("/my/project"));
        assert_eq!(ws.root_path.as_ref().unwrap().to_str().unwrap(), "/my/project");
    }

    #[test]
    fn test_document_text_preserved_after_update() {
        let mut ws = Workspace::new();
        let uri = Uri::from_str("file:///test.wal").unwrap();
        let long_text = r#"(define x 1)
(define y 2)
(define z 3)
(defun add [a b] (+ a b))
"#;
        ws.open_document(uri.clone(), long_text.to_string());
        let doc = ws.get_document(&uri).unwrap();
        assert_eq!(doc.text, long_text);
    }

    #[test]
    fn test_get_word_at_position_operators() {
        let mut ws = Workspace::new();
        let uri = Uri::from_str("file:///test.wal").unwrap();
        // get_word_at_position captures alphanumeric + _ . - / # ~ chars only
        // Operators like + - * / are NOT captured by this function
        ws.open_document(uri.clone(), "(add 1 2)\n(sub 3 4)\n(mul 5 6)\n(div 7 8)".to_string());

        assert_eq!(ws.get_word_at_position(&uri, 0, 1), Some("add".to_string()));
        assert_eq!(ws.get_word_at_position(&uri, 1, 1), Some("sub".to_string()));
        assert_eq!(ws.get_word_at_position(&uri, 2, 1), Some("mul".to_string()));
        assert_eq!(ws.get_word_at_position(&uri, 3, 1), Some("div".to_string()));
    }
}