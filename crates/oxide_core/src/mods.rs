//! Mounts `.fnfm` packages as the `mod://` asset source.
//!
//! `asset_server.load("mod://com.example/foo.png")` resolves `com.example` to
//! `mods/com.example.fnfm` (or any package whose file stem matches) and reads
//! `foo.png` out of it. Only `store` and `zstd` blocks are decoded here;
//! chunked blocks go through the same path for now.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use bevy::asset::io::{AssetReader, AssetReaderError, PathStream, Reader, VecReader};
use bevy::prelude::*;

use oesdk::pack::reader::Pack;

/// Everything mounted under `mod://`.
#[derive(Resource, Clone, Default)]
pub struct ModRegistry {
    packs: Arc<Vec<MountedPack>>,
}

#[derive(Clone)]
struct MountedPack {
    id: String,
    bytes: Arc<Vec<u8>>,
}

impl ModRegistry {
    /// Scans `dir` for `*.fnfm`, reading each package's TOC once to validate it.
    pub fn scan(dir: &Path) -> Self {
        let mut packs = Vec::new();
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
                            info!(
                                "mod source: mounted {} ({} entries)",
                                id,
                                pack.entries().len()
                            );
                            packs.push(MountedPack { id, bytes });
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
        let parsed = Pack::parse(&pack.bytes).ok()?;
        parsed.read(rest).ok()
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
    std::env::var_os("FNF_OXIDE_MODS")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("mods"))
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
