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

    pub fn auto_load_from_source(&mut self, source: &str) {
        let paths = Self::extract_load_calls(source);
        for p in &paths {
            let path = Path::new(p);
            if path.exists() {
                let _ = self.load_vcd(path);
            }
        }
        let sigs = Self::extract_defsig_signals(source);
        for s in sigs {
            self.add_virtual_signal(s);
        }
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

    #[test]
    fn test_find_with_empty_prefix() {
        let mut wm = WaveformManager::new();
        wm.signals = vec!["tb.clk".to_string(), "tb.rst".to_string()];
        let results = wm.find("");
        assert!(results.iter().all(|s| wm.signals.contains(s)));
    }

    #[test]
    fn test_find_no_match() {
        let mut wm = WaveformManager::new();
        wm.signals = vec!["tb.clk".to_string()];
        let results = wm.find("zzz_no_such_signal");
        assert!(results.is_empty());
    }

    #[test]
    fn test_fuzzy_search_scoring() {
        let mut wm = WaveformManager::new();
        wm.signals = vec![
            "tb.clock".to_string(),
            "tb.clk".to_string(),
            "tb.clock_enable".to_string(),
        ];
        // All should contain "cl" somewhere
        let results = wm.find_fuzzy("cl");
        assert!(!results.is_empty());
        // Fuzzy should match partial substrings
    }

    #[test]
    fn test_find_prefers_exact_match() {
        let mut wm = WaveformManager::new();
        wm.signals = vec![
            "data".to_string(),
            "data_bus".to_string(),
            "ctrl_data".to_string(),
        ];
        let results = wm.find("data");
        // Exact match should be first
        assert_eq!(results[0], "data".to_string());
    }

    #[test]
    fn test_clear_resets_all_state() {
        let mut wm = WaveformManager::new();
        wm.signals = vec!["a".to_string()];
        wm.virtual_signals = vec!["v1".to_string()];
        wm.loaded_files = vec![PathBuf::from("/test.vcd")];

        wm.clear();

        assert!(wm.signals.is_empty());
        assert!(wm.virtual_signals.is_empty());
        assert!(wm.loaded_files.is_empty());
        assert!(wm.get_all_signals().is_empty());
    }

    #[test]
    fn test_add_virtual_signal_dedup() {
        let mut wm = WaveformManager::new();
        wm.add_virtual_signal("v1".to_string());
        wm.add_virtual_signal("v2".to_string());
        wm.add_virtual_signal("v1".to_string());  // duplicate
        assert_eq!(wm.virtual_signals.len(), 2);

        let all = wm.get_all_signals();
        assert!(all.contains(&"v1".to_string()));
        assert!(all.contains(&"v2".to_string()));
    }

    #[test]
    fn test_extract_load_calls_with_ids() {
        let source = r#"
(load "signals.vcd" my-id)
(load "counter.fst")
(load "data.csv" t0)
(define x 42)
"#;
        let paths = WaveformManager::extract_load_calls(source);
        assert_eq!(paths.len(), 3);
        assert!(paths.contains(&"signals.vcd".to_string()));
        assert!(paths.contains(&"counter.fst".to_string()));
        assert!(paths.contains(&"data.csv".to_string()));
    }

    #[test]
    fn test_extract_defsig_complex() {
        let source = r#"
(defsig rising-clk (rising tb.clk))
(defsig handshake (&& #valid #ready))
(defsig overflow (= result 0xFF))
"#;
        let signals = WaveformManager::extract_defsig_signals(source);
        assert_eq!(signals.len(), 3);
        assert!(signals.contains(&"rising-clk".to_string()));
        assert!(signals.contains(&"handshake".to_string()));
        assert!(signals.contains(&"overflow".to_string()));
    }

    #[test]
    fn test_find_by_prefix_partial_hierarchy() {
        let mut wm = WaveformManager::new();
        wm.signals = vec![
            "tb.uart.tx".to_string(),
            "tb.uart.rx".to_string(),
            "tb.spi.mosi".to_string(),
            "tb.spi.miso".to_string(),
        ];
        let uart = wm.find_by_prefix("tb.uart.");
        assert_eq!(uart.len(), 2);
        let spi = wm.find_by_prefix("tb.spi.");
        assert_eq!(spi.len(), 2);
        let tb = wm.find_by_prefix("tb.");
        assert_eq!(tb.len(), 4);
    }

    #[test]
    fn test_bulk_signals_search() {
        let mut wm = WaveformManager::new();
        let mut sigs = Vec::new();
        for i in 0..100 {
            sigs.push(format!("tb.signal_{:03}", i));
        }
        wm.signals = sigs;

        // Prefix search: tb.signal_0 matches signal_000..signal_099 = 100
        assert!(wm.find_by_prefix("tb.signal_0").len() >= 50);
        assert_eq!(wm.find_by_prefix("tb.").len(), 100);
        assert_eq!(wm.find_by_prefix("tb.signal_000").len(), 1);

        // Fuzzy search on bulk
        let results = wm.find_fuzzy("signal_099");
        assert!(results.contains(&"tb.signal_099".to_string()));
    }

    #[test]
    fn test_deep_hierarchical_signals() {
        let mut wm = WaveformManager::new();
        wm.signals = vec![
            "top.cpu.core.alu.op".to_string(),
            "top.cpu.core.alu.result".to_string(),
            "top.cpu.core.reg.file".to_string(),
            "top.cpu.cache.l1.data".to_string(),
            "top.mem.controller.addr".to_string(),
        ];
        let cpu_signals = wm.find_by_prefix("top.cpu.");
        assert_eq!(cpu_signals.len(), 4);
        let alu_signals = wm.find_by_prefix("top.cpu.core.alu.");
        assert_eq!(alu_signals.len(), 2);
        let mem_signals = wm.find_by_prefix("top.mem.");
        assert_eq!(mem_signals.len(), 1);
    }

    #[test]
    fn test_signal_dedup_across_real_and_virtual() {
        let mut wm = WaveformManager::new();
        wm.signals = vec!["tb.clk".to_string()];
        wm.add_virtual_signal("tb.clk".to_string());  // same name, should not dedup in all_signals
        let all = wm.get_all_signals();
        assert!(all.contains(&"tb.clk".to_string()));
    }

    #[test]
    fn test_extract_load_calls_multiline() {
        let source = r#"
;; Config
(load "base.vcd")
;; Additional traces
(load "extra.fst")
(load "data.csv")
;; Done
"#;
        let paths = WaveformManager::extract_load_calls(source);
        assert_eq!(paths.len(), 3);
    }

    #[test]
    fn test_load_vcd_multiple_times_same_file() {
        use std::env::temp_dir;
        use std::fs::{self, File};
        use std::io::Write;

        let content = r#"
$timescale 1ns $end
$scope module top $end
$var wire 1 ! a $end
$var wire 1 " b $end
$upscope $end
$enddefinitions $end
"#;
        let mut path = temp_dir();
        path.push("test_multi_load.vcd");
        let mut file = File::create(&path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        drop(file);

        let mut wm = WaveformManager::new();
        wm.load_vcd(&path).unwrap();
        wm.load_vcd(&path).unwrap();
        wm.load_vcd(&path).unwrap();
        assert_eq!(wm.signals.len(), 2);
        assert_eq!(wm.loaded_files.len(), 1);

        fs::remove_file(&path).ok();
    }
}