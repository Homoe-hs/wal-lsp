use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct WaveformInfo {
    pub signals: Vec<String>,
    pub scopes: Vec<String>,
    pub timescale: Option<String>,
}

#[allow(dead_code)]
pub fn parse_vcd_header(path: &Path) -> Result<WaveformInfo, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
    let reader = BufReader::new(file);

    let mut signals = Vec::new();
    let mut scopes = Vec::new();
    let mut timescale = None;
    let mut current_scope = Vec::new();

    for line in reader.lines() {
        let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
        let trimmed = line.trim();

        if trimmed == "$enddefinitions" {
            break;
        }

        if trimmed.starts_with("$scope") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 3 {
                current_scope.push(parts[2].to_string());
                scopes.push(current_scope.join("."));
            }
        } else if trimmed.starts_with("$upscope") {
            current_scope.pop();
        } else if trimmed.starts_with("$var") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 5 {
                let var_name = parts[4].to_string();
                let full_name = if current_scope.is_empty() {
                    var_name
                } else {
                    format!("{}.{}", current_scope.join("."), var_name)
                };
                signals.push(full_name);
            }
        } else if trimmed.starts_with("$timescale") {
            timescale = Some(
                trimmed
                    .replace("$timescale", "")
                    .replace("$end", "")
                    .trim()
                    .to_string(),
            );
        }
    }

    Ok(WaveformInfo {
        signals,
        scopes,
        timescale,
    })
}

#[allow(dead_code)]
pub fn parse_csv_header(path: &Path) -> Result<WaveformInfo, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
    let mut reader = BufReader::new(file);

    let mut first_line = String::new();
    reader
        .read_line(&mut first_line)
        .map_err(|e| format!("Failed to read line: {}", e))?;

    let headers: Vec<String> = first_line
        .trim()
        .split(',')
        .map(|s| s.replace("Time [s]", "").trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    Ok(WaveformInfo {
        signals: headers,
        scopes: vec![],
        timescale: None,
    })
}

#[allow(dead_code)]
pub fn parse_waveform_header(path: &Path) -> Result<WaveformInfo, String> {
    let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");

    match extension.to_lowercase().as_str() {
        "vcd" => parse_vcd_header(path),
        "csv" => parse_csv_header(path),
        _ => Err(format!("Unsupported waveform format: {}", extension)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_vcd_header() {
        let mut file = NamedTempFile::new().unwrap();
        let content = r#"
$timescale 1ns $end
$scope module tb $end
$var wire 1 ! clk $end
$var wire 8 " data $end
$upscope $end
$enddefinitions $end
#0
b0 !
"#;
        file.write_all(content.as_bytes()).unwrap();
        file.flush().unwrap();

        let info = parse_vcd_header(file.path()).unwrap();
        assert!(
            info.signals.contains(&"tb.clk".to_string()),
            "Expected tb.clk in signals, got: {:?}",
            info.signals
        );
        assert!(info.signals.contains(&"tb.data".to_string()));
    }
}
