//! TOML -> protobuf compilation for mod content.
//!
//! Authors write `content/{stages,characters,songs,charts}/*.toml`; the CLI
//! parses it into typed structures, validates cross-references and stores
//! protobuf in the package. Errors are reported with the TOML line/column.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use prost::Message;
use serde::Deserialize;

#[derive(Clone, PartialEq, Message)]
pub struct Stage {
    #[prost(string, tag = "1")]
    pub id: String,
    #[prost(float, tag = "2")]
    pub camera_zoom: f32,
    #[prost(message, repeated, tag = "3")]
    pub props: Vec<StageProp>,
    #[prost(message, tag = "4")]
    pub characters: Option<StageCast>,
}

#[derive(Clone, PartialEq, Message)]
pub struct StageProp {
    #[prost(string, tag = "1")]
    pub name: String,
    #[prost(string, tag = "2")]
    pub asset: String,
    #[prost(float, repeated, tag = "3")]
    pub position: Vec<f32>,
    #[prost(float, repeated, tag = "4")]
    pub scale: Vec<f32>,
    #[prost(int32, tag = "5")]
    pub z: i32,
}

#[derive(Clone, PartialEq, Message)]
pub struct StageCast {
    #[prost(message, repeated, tag = "1")]
    pub slots: Vec<CastSlot>,
}

#[derive(Clone, PartialEq, Message)]
pub struct CastSlot {
    #[prost(string, tag = "1")]
    pub side: String,
    #[prost(uint32, tag = "2")]
    pub slot: u32,
    #[prost(string, tag = "3")]
    pub character: String,
    #[prost(float, repeated, tag = "4")]
    pub position: Vec<f32>,
}

#[derive(Clone, PartialEq, Message)]
pub struct Character {
    #[prost(string, tag = "1")]
    pub id: String,
    /// sparrow | animate
    #[prost(string, tag = "2")]
    pub render_type: String,
    #[prost(string, tag = "3")]
    pub asset: String,
    #[prost(float, tag = "4")]
    pub scale: f32,
    #[prost(bool, tag = "5")]
    pub flip_x: bool,
    #[prost(string, tag = "6")]
    pub idle: String,
    #[prost(string, repeated, tag = "7")]
    pub sing: Vec<String>,
}

#[derive(Clone, PartialEq, Message)]
pub struct Song {
    #[prost(string, tag = "1")]
    pub id: String,
    #[prost(string, tag = "2")]
    pub song_name: String,
    #[prost(string, tag = "3")]
    pub stage: String,
    #[prost(float, tag = "4")]
    pub bpm: f32,
    #[prost(string, repeated, tag = "5")]
    pub difficulties: Vec<String>,
}

#[derive(Clone, PartialEq, Message)]
pub struct Chart {
    #[prost(string, tag = "1")]
    pub song: String,
    #[prost(string, tag = "2")]
    pub difficulty: String,
    #[prost(message, repeated, tag = "3")]
    pub notes: Vec<ChartNote>,
}

#[derive(Clone, PartialEq, Message)]
pub struct ChartNote {
    #[prost(uint32, tag = "1")]
    pub t: u32,
    #[prost(uint32, tag = "2")]
    pub lane: u32,
    #[prost(uint32, tag = "3")]
    pub kind: u32,
    #[prost(uint32, tag = "4")]
    pub len: u32,
}

#[derive(Debug, Deserialize)]
struct StageFile {
    id: String,
    #[serde(default = "one")]
    camera_zoom: f32,
    #[serde(default)]
    props: Vec<PropFile>,
    #[serde(default)]
    characters: Vec<CastFile>,
}

fn one() -> f32 {
    1.0
}

#[derive(Debug, Deserialize)]
struct PropFile {
    name: String,
    asset: String,
    #[serde(default)]
    position: Vec<f32>,
    #[serde(default)]
    scale: Vec<f32>,
    #[serde(default)]
    z: i32,
}

#[derive(Debug, Deserialize)]
struct CastFile {
    #[serde(default = "player")]
    side: String,
    #[serde(default)]
    slot: u32,
    character: String,
    #[serde(default)]
    position: Vec<f32>,
}

fn player() -> String {
    "player".to_owned()
}

#[derive(Debug, Deserialize)]
struct CharacterFile {
    id: String,
    #[serde(default = "sparrow")]
    render_type: String,
    asset: String,
    #[serde(default = "one")]
    scale: f32,
    #[serde(default)]
    flip_x: bool,
    #[serde(default = "idle")]
    idle: String,
    #[serde(default)]
    sing: Vec<String>,
}

fn sparrow() -> String {
    "sparrow".to_owned()
}

fn idle() -> String {
    "idle".to_owned()
}

#[derive(Debug, Deserialize)]
struct SongFile {
    id: String,
    #[serde(default)]
    song_name: String,
    stage: String,
    bpm: f32,
    #[serde(default)]
    difficulties: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ChartFile {
    song: String,
    difficulty: String,
    #[serde(default)]
    notes: Vec<NoteFile>,
}

#[derive(Debug, Deserialize)]
struct NoteFile {
    t: u32,
    lane: u32,
    #[serde(default)]
    kind: u32,
    #[serde(default)]
    len: u32,
}

/// One compiled output: a package path and its protobuf bytes.
pub struct Compiled {
    pub path: String,
    pub bytes: Vec<u8>,
}

/// Compiles every `*.toml` under `content/`. Ids must be unique per kind.
pub fn compile(content: &Path) -> Result<Vec<Compiled>, String> {
    let mut out = Vec::new();
    let mut ids: BTreeMap<String, String> = BTreeMap::new();

    for (kind, path) in files(content)? {
        let text = std::fs::read_to_string(&path)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        let id_key;
        let bytes = match kind.as_str() {
            "stages" => {
                let file: StageFile = parse(&text, &path)?;
                validate_stage(&file, &path)?;
                id_key = file.id.clone();
                Stage {
                    id: file.id,
                    camera_zoom: file.camera_zoom,
                    props: file
                        .props
                        .into_iter()
                        .map(|prop| StageProp {
                            name: prop.name,
                            asset: prop.asset,
                            position: prop.position,
                            scale: prop.scale,
                            z: prop.z,
                        })
                        .collect(),
                    characters: Some(StageCast {
                        slots: file
                            .characters
                            .into_iter()
                            .map(|cast| CastSlot {
                                side: cast.side,
                                slot: cast.slot,
                                character: cast.character,
                                position: cast.position,
                            })
                            .collect(),
                    }),
                }
                .encode_to_vec()
            }
            "characters" => {
                let file: CharacterFile = parse(&text, &path)?;
                if file.asset.is_empty() {
                    return Err(format!("{}: character needs an asset", path.display()));
                }
                id_key = file.id.clone();
                Character {
                    id: file.id,
                    render_type: file.render_type,
                    asset: file.asset,
                    scale: file.scale,
                    flip_x: file.flip_x,
                    idle: file.idle,
                    sing: file.sing,
                }
                .encode_to_vec()
            }
            "songs" => {
                let file: SongFile = parse(&text, &path)?;
                if file.bpm <= 0.0 {
                    return Err(format!("{}: bpm must be positive", path.display()));
                }
                id_key = file.id.clone();
                Song {
                    id: file.id,
                    song_name: file.song_name,
                    stage: file.stage,
                    bpm: file.bpm,
                    difficulties: file.difficulties,
                }
                .encode_to_vec()
            }
            "charts" => {
                let file: ChartFile = parse(&text, &path)?;
                for note in &file.notes {
                    if note.lane > 7 {
                        return Err(format!(
                            "{}: lane {} out of range 0..7",
                            path.display(),
                            note.lane
                        ));
                    }
                }
                id_key = format!("{}@{}", file.song, file.difficulty);
                Chart {
                    song: file.song,
                    difficulty: file.difficulty,
                    notes: file
                        .notes
                        .into_iter()
                        .map(|note| ChartNote {
                            t: note.t,
                            lane: note.lane,
                            kind: note.kind,
                            len: note.len,
                        })
                        .collect(),
                }
                .encode_to_vec()
            }
            other => return Err(format!("{}: unknown content kind {other}", path.display())),
        };
        if let Some(previous) = ids.insert(id_key.clone(), path.display().to_string()) {
            return Err(format!(
                "duplicate id {id_key:?}: {} and {}",
                previous,
                path.display()
            ));
        }
        let stem = path
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_default();
        out.push(Compiled {
            path: format!("content/{kind}/{stem}.pb"),
            bytes,
        });
    }
    Ok(out)
}

fn parse<T: for<'de> Deserialize<'de>>(text: &str, path: &Path) -> Result<T, String> {
    toml::from_str(text).map_err(|error| {
        let span = error.span();
        format!(
            "{}:{}:{}: {error}",
            path.display(),
            span.map(|span| span.start).unwrap_or(0),
            0
        )
    })
}

fn validate_stage(file: &StageFile, path: &Path) -> Result<(), String> {
    if file.camera_zoom <= 0.0 {
        return Err(format!("{}: camera_zoom must be positive", path.display()));
    }
    for prop in &file.props {
        if prop.asset.is_empty() {
            return Err(format!(
                "{}: prop {:?} has no asset",
                path.display(),
                prop.name
            ));
        }
    }
    for cast in &file.characters {
        if cast.character.is_empty() {
            return Err(format!("{}: cast slot has no character", path.display()));
        }
    }
    Ok(())
}

fn files(content: &Path) -> Result<Vec<(String, PathBuf)>, String> {
    let mut out = Vec::new();
    for kind in ["stages", "characters", "songs", "charts"] {
        let dir = content.join(kind);
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        let mut paths: Vec<PathBuf> = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "toml"))
            .collect();
        paths.sort();
        for path in paths {
            out.push((kind.to_owned(), path));
        }
    }
    Ok(out)
}
