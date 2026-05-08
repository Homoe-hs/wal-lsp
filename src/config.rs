use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indentation_spaces: Option<u32>,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            indentation_spaces: Some(2),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspConfig {
    #[serde(default)]
    pub rules: HashMap<String, RuleConfig>,
    #[serde(default)]
    pub format: FormatConfig,
}

impl Default for LspConfig {
    fn default() -> Self {
        Self {
            rules: HashMap::new(),
            format: FormatConfig::default(),
        }
    }
}

impl LspConfig {
    pub fn is_rule_enabled(&self, rule_id: &str, default: bool) -> bool {
        self.rules
            .get(rule_id)
            .and_then(|r| r.enabled)
            .unwrap_or(default)
    }

    pub fn rule_severity(&self, rule_id: &str) -> Option<lsp_types::DiagnosticSeverity> {
        self.rules.get(rule_id).and_then(|r| {
            r.severity.as_deref().and_then(|s| match s {
                "error" => Some(lsp_types::DiagnosticSeverity::ERROR),
                "warning" => Some(lsp_types::DiagnosticSeverity::WARNING),
                "information" => Some(lsp_types::DiagnosticSeverity::INFORMATION),
                "hint" => Some(lsp_types::DiagnosticSeverity::HINT),
                _ => None,
            })
        })
    }

    pub fn apply_to_diagnostics(&self, diagnostics: &mut [lsp_types::Diagnostic]) {
        for diag in diagnostics.iter_mut() {
            let rule_id = diag
                .source
                .as_deref()
                .and_then(|s| s.strip_prefix("wal-lsp:"));
            if let Some(rule_id) = rule_id {
                if let Some(severity) = self.rule_severity(rule_id) {
                    diag.severity = Some(severity);
                }
            }
        }
    }
}

pub static CONFIG: std::sync::LazyLock<RwLock<LspConfig>> =
    std::sync::LazyLock::new(|| RwLock::new(LspConfig::default()));
