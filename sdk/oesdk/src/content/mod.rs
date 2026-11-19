//! TOML -> protobuf compilation for mod content.
//!
//! Authors write `content/{stages,characters,songs,charts}/*.toml`; the CLI
//! parses it into typed structures, validates cross-references and stores
//! protobuf in the package. Errors are reported with the TOML line/column.

use std::collections::{BTreeMap, BTreeSet};
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

#[derive(Clone, PartialEq, Message)]
pub struct Level {
    #[prost(string, tag = "1")]
    pub id: String,
    #[prost(string, tag = "2")]
    pub name: String,
    #[prost(string, tag = "3")]
    pub title_asset: String,
    #[prost(string, tag = "4")]
    pub background: String,
    #[prost(string, tag = "5")]
    pub capsule: String,
    #[prost(string, repeated, tag = "6")]
    pub songs: Vec<String>,
    #[prost(message, repeated, tag = "7")]
    pub props: Vec<LevelProp>,
}

#[derive(Clone, PartialEq, Message)]
pub struct LevelProp {
    #[prost(string, tag = "1")]
    pub asset_path: String,
    #[prost(float, tag = "2")]
    pub scale: f32,
    #[prost(float, repeated, tag = "3")]
    pub offsets: Vec<f32>,
    #[prost(message, repeated, tag = "4")]
    pub animations: Vec<PropAnim>,
}

#[derive(Clone, PartialEq, Message)]
pub struct PropAnim {
    #[prost(string, tag = "1")]
    pub name: String,
    #[prost(string, tag = "2")]
    pub prefix: String,
    #[prost(float, tag = "3")]
    pub frame_rate: f32,
    #[prost(int32, repeated, tag = "4")]
    pub frame_indices: Vec<i32>,
}

#[derive(Clone, PartialEq, Message)]
pub struct NoteStyle {
    #[prost(string, tag = "1")]
    pub id: String,
    #[prost(string, tag = "2")]
    pub author: String,
    #[prost(string, tag = "3")]
    pub fallback: String,
    #[prost(map = "string, message", tag = "4")]
    pub assets: std::collections::HashMap<String, AssetSlot>,
}

#[derive(Clone, PartialEq, Message)]
pub struct AssetSlot {
    #[prost(string, tag = "1")]
    pub asset_path: String,
    #[prost(float, tag = "2")]
    pub scale: f32,
    #[prost(float, repeated, tag = "3")]
    pub offsets: Vec<f32>,
    #[prost(string, tag = "4")]
    pub prefix: String,
    #[prost(string, tag = "5")]
    pub audio_path: String,
    #[prost(string, tag = "6")]
    pub animation: String,
}

#[derive(Clone, PartialEq, Message)]
pub struct DialogueBox {
    #[prost(string, tag = "1")]
    pub id: String,
    #[prost(string, tag = "2")]
    pub asset_path: String,
    #[prost(float, tag = "3")]
    pub scale: f32,
    #[prost(bool, tag = "4")]
    pub is_pixel: bool,
    #[prost(float, repeated, tag = "5")]
    pub offsets: Vec<f32>,
}

#[derive(Clone, PartialEq, Message)]
pub struct DialogueSpeaker {
    #[prost(string, tag = "1")]
    pub id: String,
    #[prost(string, tag = "2")]
    pub name: String,
    #[prost(string, tag = "3")]
    pub character: String,
    #[prost(string, tag = "4")]
    pub box_id: String,
}

#[derive(Clone, PartialEq, Message)]
pub struct DialogueConversation {
    #[prost(string, tag = "1")]
    pub id: String,
    #[prost(message, repeated, tag = "2")]
    pub lines: Vec<DialogueLine>,
}

#[derive(Clone, PartialEq, Message)]
pub struct DialogueLine {
    #[prost(string, tag = "1")]
    pub speaker: String,
    #[prost(string, tag = "2")]
    pub text: String,
}

#[derive(Clone, PartialEq, Message)]
pub struct Player {
    #[prost(string, tag = "1")]
    pub id: String,
    #[prost(string, tag = "2")]
    pub name: String,
    #[prost(string, repeated, tag = "3")]
    pub owned_chars: Vec<String>,
    #[prost(string, tag = "4")]
    pub freeplay_style: String,
    #[prost(string, tag = "5")]
    pub sticker_pack: String,
    #[prost(string, tag = "6")]
    pub dj_asset: String,
    #[prost(int32, tag = "7")]
    pub char_select_position: i32,
    #[prost(message, optional, tag = "8")]
    pub fist_pump: Option<FistPump>,
    #[prost(message, repeated, tag = "9")]
    pub dj_animations: Vec<PropAnim>,
    #[prost(map = "string, message", tag = "10")]
    pub results: std::collections::HashMap<String, ResultTier>,
    #[prost(map = "string, string", tag = "11")]
    pub result_music: std::collections::HashMap<String, String>,
}

#[derive(Clone, PartialEq, Message)]
pub struct FistPump {
    #[prost(int32, tag = "1")]
    pub intro_start: i32,
    #[prost(int32, tag = "2")]
    pub intro_end: i32,
    #[prost(int32, tag = "3")]
    pub loop_start: i32,
    #[prost(int32, tag = "4")]
    pub loop_end: i32,
    #[prost(int32, tag = "5")]
    pub intro_bad_start: i32,
    #[prost(int32, tag = "6")]
    pub intro_bad_end: i32,
    #[prost(int32, tag = "7")]
    pub loop_bad_start: i32,
    #[prost(int32, tag = "8")]
    pub loop_bad_end: i32,
}

#[derive(Clone, PartialEq, Message)]
pub struct ResultTier {
    #[prost(message, repeated, tag = "1")]
    pub stickers: Vec<ResultSticker>,
}

#[derive(Clone, PartialEq, Message)]
pub struct ResultSticker {
    #[prost(string, tag = "1")]
    pub asset: String,
    #[prost(string, tag = "2")]
    pub render_type: String,
    #[prost(string, tag = "3")]
    pub start_label: String,
    #[prost(string, tag = "4")]
    pub loop_label: String,
    #[prost(string, tag = "5")]
    pub filter: String,
    #[prost(int32, tag = "6")]
    pub z_index: i32,
    #[prost(float, repeated, tag = "7")]
    pub offsets: Vec<f32>,
}

#[derive(Clone, PartialEq, Message)]
pub struct StickerPack {
    #[prost(string, tag = "1")]
    pub id: String,
    #[prost(string, tag = "2")]
    pub artist: String,
    #[prost(string, repeated, tag = "3")]
    pub stickers: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct LevelFile {
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    title_asset: String,
    #[serde(default)]
    background: String,
    #[serde(default)]
    capsule: Option<CapsuleFile>,
    #[serde(default)]
    songs: Vec<String>,
    #[serde(default)]
    props: Vec<LevelPropFile>,
}

#[derive(Debug, Deserialize)]
struct CapsuleFile {
    #[serde(default)]
    name: String,
}

#[derive(Debug, Deserialize)]
struct LevelPropFile {
    #[serde(default)]
    asset_path: String,
    #[serde(default = "one")]
    scale: f32,
    #[serde(default)]
    offsets: Vec<f32>,
    #[serde(default)]
    animations: Vec<PropAnimFile>,
}

#[derive(Debug, Deserialize)]
struct PropAnimFile {
    name: String,
    #[serde(default)]
    prefix: String,
    #[serde(default = "twenty_four")]
    frame_rate: f32,
    #[serde(default)]
    frame_indices: Vec<i32>,
}

fn twenty_four() -> f32 {
    24.0
}

#[derive(Debug, Deserialize)]
struct NoteStyleFile {
    id: String,
    #[serde(default)]
    author: String,
    #[serde(default)]
    fallback: String,
    #[serde(default)]
    assets: std::collections::HashMap<String, SlotFile>,
}

#[derive(Debug, Deserialize)]
struct SlotFile {
    #[serde(default)]
    asset_path: String,
    #[serde(default = "one")]
    scale: f32,
    #[serde(default)]
    offsets: Vec<f32>,
    #[serde(default)]
    data: SlotDataFile,
}

#[derive(Debug, Default, Deserialize)]
struct SlotDataFile {
    #[serde(default)]
    audio_path: String,
    #[serde(default)]
    animation: String,
    #[serde(flatten)]
    sides: std::collections::HashMap<String, SidePrefix>,
}

#[derive(Debug, Deserialize)]
struct SidePrefix {
    #[serde(default)]
    prefix: String,
}

#[derive(Debug, Deserialize)]
struct BoxFile {
    id: String,
    #[serde(default)]
    asset_path: String,
    #[serde(default = "one")]
    scale: f32,
    #[serde(default)]
    is_pixel: bool,
    #[serde(default)]
    offsets: Vec<f32>,
}

#[derive(Debug, Deserialize)]
struct SpeakerFile {
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    character: String,
    #[serde(default)]
    box_id: String,
}

#[derive(Debug, Deserialize)]
struct ConversationFile {
    id: String,
    #[serde(default)]
    lines: Vec<LineFile>,
}

#[derive(Debug, Deserialize)]
struct LineFile {
    speaker: String,
    #[serde(default)]
    text: String,
}

#[derive(Debug, Deserialize)]
struct PlayerFile {
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    owned_chars: Vec<String>,
    #[serde(default)]
    freeplay_style: String,
    #[serde(default)]
    sticker_pack: String,
    #[serde(default)]
    freeplay_dj: Option<DjFile>,
    #[serde(default)]
    results: Option<ResultsFile>,
    #[serde(default)]
    char_select: Option<SelectFile>,
}

#[derive(Debug, Deserialize)]
struct DjFile {
    #[serde(default)]
    asset_path: String,
    #[serde(default)]
    fist_pump: Option<FistPumpFile>,
    #[serde(default)]
    animations: Vec<PropAnimFile>,
}

#[derive(Debug, Deserialize)]
struct FistPumpFile {
    #[serde(default)]
    intro_start_frame: i32,
    #[serde(default)]
    intro_end_frame: i32,
    #[serde(default)]
    loop_start_frame: i32,
    #[serde(default)]
    loop_end_frame: i32,
    #[serde(default)]
    intro_bad_start_frame: i32,
    #[serde(default)]
    intro_bad_end_frame: i32,
    #[serde(default)]
    loop_bad_start_frame: i32,
    #[serde(default)]
    loop_bad_end_frame: i32,
}

#[derive(Debug, Default, Deserialize)]
struct ResultsFile {
    #[serde(default)]
    music: std::collections::HashMap<String, String>,
    #[serde(default, rename = "perfectGold")]
    perfect_gold: Vec<StickerFile>,
    #[serde(default)]
    perfect: Vec<StickerFile>,
    #[serde(default)]
    excellent: Vec<StickerFile>,
    #[serde(default)]
    great: Vec<StickerFile>,
    #[serde(default)]
    good: Vec<StickerFile>,
    #[serde(default)]
    loss: Vec<StickerFile>,
}

#[derive(Debug, Deserialize)]
struct StickerFile {
    #[serde(default, rename = "renderType")]
    render_type: String,
    #[serde(default)]
    asset_path: String,
    #[serde(default)]
    start_frame_label: String,
    #[serde(default)]
    loop_frame_label: String,
    #[serde(default)]
    filter: String,
    #[serde(default)]
    z_index: i32,
    #[serde(default)]
    offsets: Vec<f32>,
}

#[derive(Debug, Deserialize)]
struct SelectFile {
    #[serde(default)]
    position: i32,
}

#[derive(Debug, Deserialize)]
struct StickerPackFile {
    id: String,
    #[serde(default)]
    artist: String,
    #[serde(default)]
    stickers: Vec<String>,
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
    let mut song_ids = BTreeSet::new();
    let mut box_ids = Vec::new();
    let mut speaker_ids = Vec::new();
    let mut level_songs = Vec::new();
    let mut speaker_boxes = Vec::new();
    let mut convs = Vec::new();

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
                song_ids.insert(file.id.clone());
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
            "levels" => {
                let file: LevelFile = parse(&text, &path)?;
                id_key = file.id.clone();
                level_songs.push((file.id.clone(), file.songs.clone()));
                Level {
                    id: file.id,
                    name: file.name,
                    title_asset: file.title_asset,
                    background: file.background,
                    capsule: file.capsule.map(|c| c.name).unwrap_or_default(),
                    songs: file.songs,
                    props: file
                        .props
                        .into_iter()
                        .map(|prop| LevelProp {
                            asset_path: prop.asset_path,
                            scale: prop.scale,
                            offsets: prop.offsets,
                            animations: prop
                                .animations
                                .into_iter()
                                .map(|anim| PropAnim {
                                    name: anim.name,
                                    prefix: anim.prefix,
                                    frame_rate: anim.frame_rate,
                                    frame_indices: anim.frame_indices,
                                })
                                .collect(),
                        })
                        .collect(),
                }
                .encode_to_vec()
            }
            "notestyles" => {
                let file: NoteStyleFile = parse(&text, &path)?;
                if file.assets.is_empty() {
                    return Err(format!("{}: note style has no asset slots", path.display()));
                }
                id_key = file.id.clone();
                NoteStyle {
                    id: file.id,
                    author: file.author,
                    fallback: file.fallback,
                    assets: file
                        .assets
                        .into_iter()
                        .map(|(slot, value)| {
                            let prefix = value
                                .data
                                .sides
                                .get("left")
                                .map(|side| side.prefix.clone())
                                .unwrap_or_default();
                            (
                                slot,
                                AssetSlot {
                                    asset_path: value.asset_path,
                                    scale: value.scale,
                                    offsets: value.offsets,
                                    prefix,
                                    audio_path: value.data.audio_path,
                                    animation: value.data.animation,
                                },
                            )
                        })
                        .collect(),
                }
                .encode_to_vec()
            }
            "dialogue/boxes" => {
                let file: BoxFile = parse(&text, &path)?;
                id_key = file.id.clone();
                box_ids.push(file.id.clone());
                DialogueBox {
                    id: file.id,
                    asset_path: file.asset_path,
                    scale: file.scale,
                    is_pixel: file.is_pixel,
                    offsets: file.offsets,
                }
                .encode_to_vec()
            }
            "dialogue/speakers" => {
                let file: SpeakerFile = parse(&text, &path)?;
                id_key = file.id.clone();
                speaker_ids.push(file.id.clone());
                speaker_boxes.push((file.id.clone(), file.box_id.clone()));
                DialogueSpeaker {
                    id: file.id,
                    name: file.name,
                    character: file.character,
                    box_id: file.box_id.clone(),
                }
                .encode_to_vec()
            }
            "dialogue/conversations" => {
                let file: ConversationFile = parse(&text, &path)?;
                id_key = file.id.clone();
                convs.push((
                    file.id.clone(),
                    file.lines
                        .iter()
                        .map(|line| line.speaker.clone())
                        .collect::<Vec<_>>(),
                ));
                DialogueConversation {
                    id: file.id,
                    lines: file
                        .lines
                        .into_iter()
                        .map(|line| DialogueLine {
                            speaker: line.speaker,
                            text: line.text,
                        })
                        .collect(),
                }
                .encode_to_vec()
            }
            "players" => {
                let file: PlayerFile = parse(&text, &path)?;
                id_key = file.id.clone();
                let dj = file.freeplay_dj;
                let mut results = std::collections::HashMap::new();
                let mut result_music = std::collections::HashMap::new();
                if let Some(table) = file.results {
                    result_music = table.music;
                    for (tier, stickers) in [
                        ("perfectGold", table.perfect_gold),
                        ("perfect", table.perfect),
                        ("excellent", table.excellent),
                        ("great", table.great),
                        ("good", table.good),
                        ("loss", table.loss),
                    ] {
                        if stickers.is_empty() {
                            continue;
                        }
                        results.insert(
                            tier.to_owned(),
                            ResultTier {
                                stickers: stickers
                                    .into_iter()
                                    .map(|sticker| ResultSticker {
                                        asset: sticker.asset_path,
                                        render_type: sticker.render_type,
                                        start_label: sticker.start_frame_label,
                                        loop_label: sticker.loop_frame_label,
                                        filter: sticker.filter,
                                        z_index: sticker.z_index,
                                        offsets: sticker.offsets,
                                    })
                                    .collect(),
                            },
                        );
                    }
                }
                Player {
                    id: file.id,
                    name: file.name,
                    owned_chars: file.owned_chars,
                    freeplay_style: file.freeplay_style,
                    sticker_pack: file.sticker_pack,
                    dj_asset: dj
                        .as_ref()
                        .map(|dj| dj.asset_path.clone())
                        .unwrap_or_default(),
                    char_select_position: file
                        .char_select
                        .map(|select| select.position)
                        .unwrap_or(0),
                    fist_pump: dj
                        .as_ref()
                        .and_then(|dj| dj.fist_pump.as_ref())
                        .map(|pump| FistPump {
                            intro_start: pump.intro_start_frame,
                            intro_end: pump.intro_end_frame,
                            loop_start: pump.loop_start_frame,
                            loop_end: pump.loop_end_frame,
                            intro_bad_start: pump.intro_bad_start_frame,
                            intro_bad_end: pump.intro_bad_end_frame,
                            loop_bad_start: pump.loop_bad_start_frame,
                            loop_bad_end: pump.loop_bad_end_frame,
                        }),
                    dj_animations: dj
                        .map(|dj| dj.animations)
                        .unwrap_or_default()
                        .into_iter()
                        .map(|anim| PropAnim {
                            name: anim.name,
                            prefix: anim.prefix,
                            frame_rate: anim.frame_rate,
                            frame_indices: anim.frame_indices,
                        })
                        .collect(),
                    results,
                    result_music,
                }
                .encode_to_vec()
            }
            "stickerpacks" => {
                let file: StickerPackFile = parse(&text, &path)?;
                id_key = file.id.clone();
                StickerPack {
                    id: file.id,
                    artist: file.artist,
                    stickers: file.stickers,
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
    for (level, songs) in &level_songs {
        for song in songs {
            if !song_ids.contains(song) {
                return Err(format!("level {level:?} references unknown song {song:?}"));
            }
        }
    }
    for (speaker, box_id) in &speaker_boxes {
        if !box_id.is_empty() && !box_ids.contains(box_id) {
            return Err(format!(
                "speaker {speaker:?} references unknown dialogue box {box_id:?}"
            ));
        }
    }
    for (conv, speakers) in &convs {
        for speaker in speakers {
            if !speaker_ids.contains(speaker) {
                return Err(format!(
                    "conversation {conv:?} references unknown speaker {speaker:?}"
                ));
            }
        }
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
    for kind in [
        "stages",
        "characters",
        "songs",
        "charts",
        "levels",
        "notestyles",
        "dialogue/boxes",
        "dialogue/speakers",
        "dialogue/conversations",
        "players",
        "stickerpacks",
    ] {
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
