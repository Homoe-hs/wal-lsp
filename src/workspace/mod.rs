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
}