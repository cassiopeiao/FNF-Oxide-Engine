//! `mod.toml` — the human-authored half of a mod.
//!
//! Deliberately parsed with a tiny built-in reader so the SDK has **zero
//! dependencies**: a mod author's `cargo build` must stay fast, and the CLI has
//! to run on a bare machine. The real validator lives in `oesdk build`, which
//! parses the same file with `toml` and reports `file:line:col` errors.

use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    /// ABI the mod was written against, e.g. `"0.1"`.
    pub api: String,
    /// Optional entry symbol; defaults to `__oesdk_make_mod`.
    pub entry: String,
    pub capabilities: Vec<String>,
    /// Callbacks the host should actually call, e.g. `["on-note-hit"]`.
    pub callbacks: Vec<String>,
    /// Free-form metadata (author, description, ...).
    pub extra: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestError {
    pub line: usize,
    pub message: String,
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "mod.toml:{}: {}", self.line, self.message)
    }
}

impl std::error::Error for ManifestError {}

impl ModManifest {
    /// Parses the flat subset of TOML used by `mod.toml`.
    ///
    /// Supported: `key = "string"`, `key = ["a", "b"]`, `#` comments and
    /// `[section]` headers (sections are folded into `extra`).
    pub fn parse(src: &str) -> Result<Self, ManifestError> {
        let mut manifest = Self::default();
        let mut section = String::new();

        for (idx, raw) in src.lines().enumerate() {
            let line_no = idx + 1;
            let line = strip_comment(raw).trim();
            if line.is_empty() {
                continue;
            }
            if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
                section = name.trim().to_string();
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                return Err(ManifestError {
                    line: line_no,
                    message: format!("expected `key = value`, found `{line}`"),
                });
            };
            let key = key.trim();
            if key.is_empty() {
                return Err(ManifestError {
                    line: line_no,
                    message: "empty key".into(),
                });
            }
            let value = value.trim();
            let full_key = if section.is_empty() {
                key.to_string()
            } else {
                format!("{section}.{key}")
            };

            match full_key.as_str() {
                "id" => manifest.id = parse_string(value, line_no)?,
                "name" => manifest.name = parse_string(value, line_no)?,
                "version" => manifest.version = parse_string(value, line_no)?,
                "api" => manifest.api = parse_string(value, line_no)?,
                "entry" => manifest.entry = parse_string(value, line_no)?,
                "capabilities" => manifest.capabilities = parse_array(value, line_no)?,
                "callbacks" => manifest.callbacks = parse_array(value, line_no)?,
                _ => {
                    let parsed = if value.starts_with('[') {
                        parse_array(value, line_no)?.join(",")
                    } else {
                        parse_string(value, line_no)?
                    };
                    manifest.extra.insert(full_key, parsed);
                }
            }
        }

        if manifest.id.is_empty() {
            return Err(ManifestError {
                line: 0,
                message: "missing required key `id`".into(),
            });
        }
        if manifest.api.is_empty() {
            manifest.api = crate::abi::ABI_VERSION.to_string();
        }
        if manifest.entry.is_empty() {
            manifest.entry = "__oesdk_make_mod".to_string();
        }
        Ok(manifest)
    }
}

fn strip_comment(line: &str) -> &str {
    let mut in_string = false;
    let mut escaped = false;
    for (i, ch) in line.char_indices() {
        match ch {
            '"' if !escaped => in_string = !in_string,
            '#' if !in_string => return &line[..i],
            '\\' if in_string => escaped = !escaped,
            _ => escaped = false,
        }
    }
    line
}

fn parse_string(value: &str, line: usize) -> Result<String, ManifestError> {
    let trimmed = value.trim();
    let inner = trimmed
        .strip_prefix('"')
        .and_then(|v| v.strip_suffix('"'))
        .ok_or_else(|| ManifestError {
            line,
            message: format!("expected a quoted string, found `{trimmed}`"),
        })?;
    Ok(inner.replace("\\\"", "\"").replace("\\\\", "\\"))
}

fn parse_array(value: &str, line: usize) -> Result<Vec<String>, ManifestError> {
    let inner = value
        .strip_prefix('[')
        .and_then(|v| v.strip_suffix(']'))
        .ok_or_else(|| ManifestError {
            line,
            message: format!("expected an array, found `{value}`"),
        })?;
    if inner.trim().is_empty() {
        return Ok(Vec::new());
    }
    inner
        .split(',')
        .map(|item| parse_string(item.trim(), line))
        .collect()
}
