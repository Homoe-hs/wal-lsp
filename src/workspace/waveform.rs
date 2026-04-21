use crate::wal::waveform::parse_waveform_header;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct WaveformManager {
    pub signals: Vec<String>,
    pub virtual_signals: Vec<String>,
    pub loaded_files: Vec<PathBuf>,
}

impl WaveformManager {
    pub fn new() -> Self {
        Self {
            signals: Vec::new(),
            virtual_signals: Vec::new(),
            loaded_files: Vec::new(),
        }
    }

    pub fn load_vcd(&mut self, path: &Path) -> Result<(), String> {
        if self.loaded_files.contains(&path.to_path_buf()) {
            return Ok(());
        }

        let info = parse_waveform_header(path)?;
        self.signals.extend(info.signals);
        self.loaded_files.push(path.to_path_buf());
        Ok(())
    }

    pub fn add_virtual_signal(&mut self, name: String) {
        if !self.virtual_signals.contains(&name) {
            self.virtual_signals.push(name);
        }
    }

    pub fn clear(&mut self) {
        self.signals.clear();
        self.virtual_signals.clear();
        self.loaded_files.clear();
    }

    pub fn get_all_signals(&self) -> Vec<String> {
        let mut all = self.signals.clone();
        all.extend(self.virtual_signals.clone());
        all
    }

    pub fn find_by_prefix(&self, prefix: &str) -> Vec<String> {
        if prefix.is_empty() {
            return self.get_all_signals();
        }

        self.get_all_signals()
            .into_iter()
            .filter(|s| s.starts_with(prefix))
            .collect()
    }

    pub fn find_fuzzy(&self, query: &str) -> Vec<String> {
        if query.is_empty() {
            return self.get_all_signals();
        }

        let query_lower = query.to_lowercase();
        let mut scored: Vec<(String, usize)> = self
            .get_all_signals()
            .into_iter()
            .filter_map(|s| {
                let idx = s.to_lowercase().find(&query_lower)?;
                Some((s, idx))
            })
            .collect();

        scored.sort_by_key(|(_, idx)| *idx);
        scored.into_iter().map(|(s, _)| s).collect()
    }

    pub fn find(&self, query: &str) -> Vec<String> {
        if query.is_empty() {
            return self.get_all_signals();
        }

        let prefix_results = self.find_by_prefix(query);
        if !prefix_results.is_empty() {
            return prefix_results;
        }

        self.find_fuzzy(query)
    }

    pub fn extract_load_calls(source: &str) -> Vec<String> {
        let mut paths = Vec::new();
        let mut in_string = false;
        let mut current_string = String::new();
        let mut paren_depth = 0;

        let bytes = source.as_bytes();

        let mut i = 0;
        while i < bytes.len() {
            let c = bytes[i] as char;

            if !in_string && c == '(' {
                paren_depth += 1;
            } else if !in_string && c == ')' && paren_depth > 0 {
                paren_depth -= 1;
            }

            if c == '"' {
                if in_string {
                    paths.push(current_string.clone());
                    current_string.clear();
                }
                in_string = !in_string;
            } else if in_string {
                current_string.push(c);
            }

            i += 1;
        }

        paths.retain(|p| {
            let ext = Path::new(p)
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            matches!(ext.to_lowercase().as_str(), "vcd" | "csv" | "fst")
        });

        paths
    }

    pub fn extract_defsig_signals(source: &str) -> Vec<String> {
        let mut signals = Vec::new();
        let chars: Vec<char> = source.chars().collect();
        let bytes = source.as_bytes();

        let mut i = 0;
        while i < bytes.len().saturating_sub(7) {
            if bytes[i] == b'(' {
                if i + 7 <= bytes.len() && &source[i+1..i+7] == "defsig" {
                    let mut j = i + 7;
                    while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                        j += 1;
                    }

                    let start = j;
                    while j < bytes.len()
                        && !bytes[j].is_ascii_whitespace()
                        && bytes[j] != b')'
                        && bytes[j] != b'['
                    {
                        j += 1;
                    }

                    if j > start {
                        let name: String = chars[start..j].iter().collect();
                        if !name.is_empty() && name.chars().next().map(|c| c.is_alphanumeric()).unwrap_or(false) {
                            signals.push(name);
                        }
                    }
                }
            }
            i += 1;
        }

        signals
    }
}

impl Default for WaveformManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_by_prefix() {
        let mut wm = WaveformManager::new();
        wm.signals = vec![
            "tb.clk".to_string(),
            "tb.data".to_string(),
            "tb.rst".to_string(),
            "tb.sub.signal".to_string(),
        ];

        let results = wm.find_by_prefix("tb.");
        assert_eq!(results.len(), 4);
    }

    #[test]
    fn test_find_fuzzy() {
        let mut wm = WaveformManager::new();
        wm.signals = vec![
            "tb.clk".to_string(),
            "tb.data".to_string(),
            "tb.rst".to_string(),
        ];

        let results = wm.find_fuzzy("lk");
        assert!(results.contains(&"tb.clk".to_string()));
    }

    #[test]
    fn test_find_prefers_prefix() {
        let mut wm = WaveformManager::new();
        wm.signals = vec![
            "tb.clk".to_string(),
            "tb.clock_enable".to_string(),
        ];

        let results = wm.find("clk");
        assert_eq!(results[0], "tb.clk".to_string());
    }

    #[test]
    fn test_extract_load_calls() {
        let source = r#"
(load "signals.vcd")
(define x 42)
(load "counter.csv")
(step)
"#;
        let paths = WaveformManager::extract_load_calls(source);
        assert!(paths.contains(&"signals.vcd".to_string()));
        assert!(paths.contains(&"counter.csv".to_string()));
    }

    #[test]
    fn test_extract_defsig_signals() {
        let source = r#"
(defsig clock-gen (posedge clk))
(defsig data-valid (= data 1))
(define x 42)
"#;
        let signals = WaveformManager::extract_defsig_signals(source);
        assert!(signals.contains(&"clock-gen".to_string()));
        assert!(signals.contains(&"data-valid".to_string()));
    }

    #[test]
    fn test_virtual_signals() {
        let mut wm = WaveformManager::new();
        wm.add_virtual_signal("virtual1".to_string());
        wm.add_virtual_signal("virtual1".to_string());

        assert_eq!(wm.virtual_signals.len(), 1);
        assert!(wm.get_all_signals().contains(&"virtual1".to_string()));
    }

    #[test]
    fn test_load_vcd_twice() {
        use std::env::temp_dir;
        use std::fs::{self, File};
        use std::io::Write;

        let content = r#"
$timescale 1ns $end
$scope module tb $end
$var wire 1 ! clk $end
$upscope $end
$enddefinitions $end
"#;

        let mut path = temp_dir();
        path.push("test.vcd");

        let mut file = File::create(&path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        drop(file);

        let mut wm = WaveformManager::new();
        wm.load_vcd(&path).unwrap();
        wm.load_vcd(&path).unwrap();

        assert_eq!(wm.signals.len(), 1);

        fs::remove_file(&path).ok();
    }
}