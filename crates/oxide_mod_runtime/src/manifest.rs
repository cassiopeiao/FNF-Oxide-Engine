//! Package manifest (`mod.toml` inside the `.fnfm` container).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    /// Mod ABI version, e.g. `"0.1.0"`.
    #[serde(default)]
    pub api: String,
    /// Exported constructor symbol.
    #[serde(default)]
    pub entry: String,
    #[serde(default)]
    pub capabilities: Vec<String>,
    /// Callbacks the host should dispatch to this mod.
    #[serde(default)]
    pub callbacks: Vec<String>,
}

impl ModManifest {
    /// Minimal sanity check; signature/format validation happens at load time.
    pub fn validate(&self, abi: &str) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("manifest: missing `id`".into());
        }
        if self.api != abi {
            return Err(format!(
                "manifest: api `{}` does not match runtime ABI `{abi}`",
                self.api
            ));
        }
        for cb in &self.callbacks {
            if !oesdk::abi::is_known_callback(cb) {
                return Err(format!("manifest: unknown callback `{cb}`"));
            }
        }
        Ok(())
    }
}
