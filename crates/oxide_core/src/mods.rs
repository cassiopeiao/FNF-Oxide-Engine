//! Mounts `.fnfm` packages as the `mod://` asset source.
//!
//! `asset_server.load("mod://com.example/foo.png")` resolves `com.example` to
//! `mods/com.example.fnfm` (or any package whose file stem matches) and reads
//! `foo.png` out of it. Only `store` and `zstd` blocks are decoded here;
//! chunked blocks go through the same path for now.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use bevy::asset::io::{AssetReader, AssetReaderError, PathStream, Reader, VecReader};
use bevy::prelude::*;

use oesdk::pack::reader::{Entry, Pack, decode_block};

/// Everything mounted under `mod://`.
#[derive(Resource, Clone, Default)]
pub struct ModRegistry {
    packs: Arc<Vec<MountedPack>>,
}

#[derive(Clone)]
struct MountedPack {
    id: String,
    bytes: Arc<Vec<u8>>,
    /// TOC entries, parsed once: path -> block descriptor.
    blocks: Arc<HashMap<String, Entry>>,
}

/// `mods/trusted.pub`: one hex Ed25519 public key (comments and whitespace ok).
fn load_trusted_key(dir: &Path) -> Option<ed25519_dalek::VerifyingKey> {
    let text = std::fs::read_to_string(dir.join("trusted.pub")).ok()?;
    let hex: String = text
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .flat_map(|line| line.chars())
        .filter(|ch| ch.is_ascii_hexdigit())
        .collect();
    if hex.len() != 64 {
        warn!("mod source: trusted.pub must hold a 32-byte hex key");
        return None;
    }
    let mut bytes = [0u8; 32];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16).ok()?;
    }
    ed25519_dalek::VerifyingKey::from_bytes(&bytes).ok()
}

/// Policy: unsigned packages are refused unless `FNF_OXIDE_ALLOW_UNSIGNED=1`.
fn verify(
    header: &oesdk::pack::Header,
    _bytes: &[u8],
    trusted: &Option<ed25519_dalek::VerifyingKey>,
) -> Result<(), String> {
    let allow_unsigned = std::env::var_os("FNF_OXIDE_ALLOW_UNSIGNED").is_some();
    let signed = header.flags & oesdk::pack::FLAG_SIGNED != 0;
    if !signed {
        return if allow_unsigned {
            Ok(())
        } else {
            Err("package is not signed (set FNF_OXIDE_ALLOW_UNSIGNED=1 for development)".into())
        };
    }
    let Some(key) = trusted else {
        return if allow_unsigned {
            Ok(())
        } else {
            Err("signed package, but mods/trusted.pub is missing".into())
        };
    };
    // Re-parse for the full walk; verification needs every block anyway.
    match Pack::parse(_bytes) {
        Ok(pack) => pack
            .verify_signature(key)
            .map_err(|error| format!("signature check failed: {error}")),
        Err(error) => Err(error),
    }
}

impl ModRegistry {
    /// Scans `dir` for `*.fnfm`, reading each package's TOC once to validate it.
    pub fn scan(dir: &Path) -> Self {
        let mut packs = Vec::new();
        let trusted = load_trusted_key(dir);
        let Ok(entries) = std::fs::read_dir(dir) else {
            return Self::default();
        };
        let mut paths: Vec<PathBuf> = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "fnfm"))
            .collect();
        paths.sort();
        for path in paths {
            match std::fs::read(&path) {
                Ok(bytes) => {
                    let bytes = Arc::new(bytes);
                    match Pack::parse(&bytes) {
                        Ok(pack) => {
                            let id = path
                                .file_stem()
                                .map(|stem| stem.to_string_lossy().into_owned())
                                .unwrap_or_default();
                            if let Err(reason) = verify(pack.header(), &bytes, &trusted) {
                                warn!("mod source: skipping {id}: {reason}");
                                continue;
                            }
                            let blocks: HashMap<String, Entry> = pack
                                .entries()
                                .iter()
                                .map(|entry| (entry.path.clone(), entry.clone()))
                                .collect();
                            info!("mod source: mounted {} ({} entries)", id, blocks.len());
                            packs.push(MountedPack {
                                id,
                                bytes,
                                blocks: Arc::new(blocks),
                            });
                        }
                        Err(error) => warn!("mod source: skipping {}: {error}", path.display()),
                    }
                }
                Err(error) => warn!("mod source: cannot read {}: {error}", path.display()),
            }
        }
        Self {
            packs: Arc::new(packs),
        }
    }

    pub fn len(&self) -> usize {
        self.packs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.packs.is_empty()
    }

    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.packs.iter().map(|pack| pack.id.as_str())
    }

    fn read(&self, path: &Path) -> Option<Vec<u8>> {
        let text = path.to_string_lossy();
        let (id, rest) = text.split_once('/')?;
        let pack = self.packs.iter().find(|pack| pack.id == id)?;
        let entry = pack.blocks.get(rest)?;
        if entry.algo == 2 {
            // chunked: keep the slow path until the texture fast path exists
            let parsed = Pack::parse(&pack.bytes).ok()?;
            return parsed.read(rest).ok();
        }
        decode_block(&pack.bytes, entry).ok()
    }
}

pub struct ModReader {
    registry: ModRegistry,
}

impl ModReader {
    pub fn new(registry: ModRegistry) -> Self {
        Self { registry }
    }
}

impl AssetReader for ModReader {
    async fn read<'a>(&'a self, path: &'a Path) -> Result<impl Reader + 'a, AssetReaderError> {
        self.registry
            .read(path)
            .map(VecReader::new)
            .ok_or_else(|| AssetReaderError::NotFound(path.to_path_buf()))
    }

    async fn read_meta<'a>(&'a self, path: &'a Path) -> Result<impl Reader + 'a, AssetReaderError> {
        Err::<VecReader, _>(AssetReaderError::NotFound(path.to_path_buf()))
    }

    async fn read_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> Result<Box<PathStream>, AssetReaderError> {
        Err(AssetReaderError::NotFound(path.to_path_buf()))
    }

    async fn is_directory<'a>(&'a self, _path: &'a Path) -> Result<bool, AssetReaderError> {
        Ok(false)
    }
}

/// Where packages are looked up: `FNF_OXIDE_MODS`, else `./mods`.
pub fn mods_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("FNF_OXIDE_MODS") {
        return PathBuf::from(dir);
    }
    crate::save::SaveStore::new().dir.join("mods")
}

/// Optional smoke test: `FNF_OXIDE_MOD_ASSET=mod://<id>/<path>`.
#[derive(Resource)]
struct PendingModAsset(Handle<Image>);

pub struct ModsPlugin {
    pub registry: ModRegistry,
}

impl Plugin for ModsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.registry.clone());
        if let Some(spec) = std::env::var_os("FNF_OXIDE_MOD_ASSET") {
            let spec = spec.to_string_lossy().into_owned();
            let server = app.world().resource::<AssetServer>().clone();
            let handle = server.load::<Image>(spec.clone());
            info!("mod source: smoke-loading {spec}");
            app.insert_resource(PendingModAsset(handle));
            app.add_systems(Update, log_pending_mod_asset);
        }
    }
}

fn log_pending_mod_asset(
    pending: Option<Res<PendingModAsset>>,
    images: Res<Assets<Image>>,
    mut logged: Local<bool>,
    mut commands: Commands,
) {
    let Some(pending) = pending else { return };
    if *logged {
        return;
    }
    if let Some(image) = images.get(&pending.0) {
        info!(
            "mod source: loaded {}x{} image from a package",
            image.width(),
            image.height()
        );
        *logged = true;
        commands.remove_resource::<PendingModAsset>();
    }
}
